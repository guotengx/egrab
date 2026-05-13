// EGrab - Platform Parser: Taobao / Tmall
// Parses product pages from taobao.com and tmall.com.
// Derived from: docs/protocols/parser-interface.md, PRD 3.1.2/3.1.3
//
// Data extraction sources:
//   - g_config.idata.item (title, images, price, desc) — traditional Taobao pages
//   - __ICE_APP_CONTEXT__.loaderData/home/data/res/item (title, images) — modern Tmall pages
//   - __ICE_APP_CONTEXT__.loaderData/home/data/res/seller (shop info) — Tmall pages
//   - __ICE_APP_CONTEXT__.loaderData/home/data/res/skuCore/sku2info/0 (price) — Tmall pages
//   - g_config.idata.seller (shop info) — Taobao pages
//   - Hub.config.sku / g_config.idata.sku (SKU data) — Taobao pages
//   - #imageTextInfo-container img (detail images) — Tmall pages
//   - DOM fallback for specs and detail images

use crate::models::{
    Description, ImageRef, PriceRange, ProductData, ScrapeErrorInfo, ScrapeStep, ShopInfo, SkuItem,
};
use crate::parser::{PageHandle, ParseResult};
use crate::parser::utils::{
    build_default_product, clean_taobao_image_url, extract_json_from_script, json_array,
    json_f64, json_object, json_string, normalize_image_url, parse_price, parse_price_range,
};
use async_trait::async_trait;

/// Taobao / Tmall product page parser.
pub struct TaobaoParser;

impl TaobaoParser {
    /// Creates a new TaobaoParser instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl super::PlatformParser for TaobaoParser {
    /// Returns the platform identifier "taobao".
    fn platform_id(&self) -> &str {
        "taobao"
    }

    /// Checks whether this parser can handle the given URL.
    ///
    /// Matches:
    ///   - item.taobao.com
    ///   - detail.tmall.com
    ///   - chaoshi.detail.tmall.com
    fn can_handle(&self, url: &str) -> bool {
        let parsed = url.to_lowercase();
        parsed.contains("item.taobao.com")
            || parsed.contains("detail.tmall.com")
            || parsed.contains("chaoshi.detail.tmall.com")
    }

    /// Extracts the item ID from a Taobao/Tmall URL.
    ///
    /// Looks for the `id` query parameter, which must be a non-empty digit string.
    fn extract_item_id(&self, url: &str) -> anyhow::Result<String> {
        // Parse the URL and extract the "id" query parameter.
        let id = url
            .split('?')
            .nth(1)
            .and_then(|query| {
                query
                    .split('&')
                    .find_map(|pair| {
                        let mut parts = pair.splitn(2, '=');
                        let key = parts.next()?;
                        let value = parts.next()?;
                        if key == "id" {
                            Some(value.to_string())
                        } else {
                            None
                        }
                    })
            })
            .unwrap_or_default();

        // Validate: must be a non-empty digit string.
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_digit()) {
            anyhow::bail!("Failed to extract item ID from URL: {}", url);
        }

        Ok(id)
    }

    /// Parses a Taobao/Tmall product page and returns structured data.
    ///
    /// Strategy:
    /// 1. Execute JavaScript to extract g_config (Taobao) and __ICE_APP_CONTEXT__ (Tmall) data
    /// 2. Parse product fields from the extracted JSON, preferring g_config over ICE
    /// 3. Fall back to DOM extraction for missing fields (detail images, specs)
    /// 4. Clean image URLs using Taobao-specific rules
    async fn parse(&self, page: &dyn PageHandle) -> anyhow::Result<ParseResult> {
        let mut errors: Vec<ScrapeErrorInfo> = Vec::new();

        // Step 1: Extract g_config data (traditional Taobao pages).
        let g_config_js = r#"
            (function() {
                try {
                    if (typeof g_config !== 'undefined') {
                        return JSON.parse(JSON.stringify(g_config));
                    }
                } catch(e) {}
                return null;
            })()
        "#;
        let g_config = page.evaluate(g_config_js).await.unwrap_or(serde_json::Value::Null);

        // Step 2: Extract __ICE_APP_CONTEXT__ data (modern Tmall pages).
        let ice_js = r#"
            (function() {
                try {
                    if (typeof window.__ICE_APP_CONTEXT__ !== 'undefined') {
                        return JSON.parse(JSON.stringify(window.__ICE_APP_CONTEXT__));
                    }
                } catch(e) {}
                return null;
            })()
        "#;
        let ice_data = page.evaluate(ice_js).await.unwrap_or(serde_json::Value::Null);

        // Step 3: Extract data from g_config (Taobao path).
        let item_data = g_config
            .get("idata")
            .and_then(|v| v.get("item"));

        // Step 4: Extract data from ICE (Tmall path).
        // Tmall stores data at loaderData/home/data/res/...
        let ice_item = ice_data
            .pointer("/loaderData/home/data/res/item");
        let ice_seller = ice_data
            .pointer("/loaderData/home/data/res/seller");
        let ice_sku_info = ice_data
            .pointer("/loaderData/home/data/res/skuCore/sku2info/0");

        // Helper: determine if the page is Tmall (for image URL base domain).
        let page_url = page.url().await.unwrap_or_default();
        let is_tmall = page_url.contains("tmall.com");

        // Extract title: g_config first, then ICE, then fallback.
        let title = item_data
            .and_then(|v| json_string(v, "title"))
            .or_else(|| ice_item.and_then(|v| {
                // Tmall ICE: title may be in "title" or "subTitle"
                json_string(v, "title")
                    .or_else(|| json_string(v, "subTitle"))
            }))
            .unwrap_or_else(|| "Unknown Title".to_string());

        let mut product = build_default_product(title);

        // Extract cover and gallery images.
        // Merge from both g_config and ICE sources.
        let images_arr = item_data
            .and_then(|v| json_array(v, "images"));

        let ice_images_arr = ice_item
            .and_then(|v| json_array(v, "images"));

        // Helper closure to process an image array into gallery & cover.
        let process_images = |product: &mut ProductData, arr: &[serde_json::Value]| {
            for (idx, img_val) in arr.iter().enumerate() {
                if let Some(img_url) = img_val.as_str() {
                    let cleaned = clean_taobao_image_url(img_url);
                    let absolute = normalize_image_url(&cleaned, "item.taobao.com");
                    if idx == 0 && product.cover.original_url.is_empty() {
                        product.cover = ImageRef {
                            original_url: absolute.clone(),
                            thumbnail_url: img_url.to_string(),
                            local_path: None,
                        };
                    }
                    product.gallery.push(ImageRef {
                        original_url: absolute,
                        thumbnail_url: img_url.to_string(),
                        local_path: None,
                    });
                }
            }
        };

        // Process g_config images first, then ICE images (if g_config didn't yield any).
        if let Some(arr) = images_arr {
            if !arr.is_empty() {
                process_images(&mut product, arr);
            }
        }
        // Always try ICE images too (may have different/additional images for Tmall).
        if let Some(arr) = ice_images_arr {
            // Only add ICE images if we didn't get any from g_config,
            // or if we're on a Tmall page (ICE is the primary source there).
            let g_config_had_images = images_arr.map(|a| !a.is_empty()).unwrap_or(false);
            if !g_config_had_images {
                process_images(&mut product, arr);
            }
        }

        // Extract price.
        let mut price_found = false;
        if let Some(item) = item_data {
            if let Some(price_str) = json_string(item, "price") {
                product.price = parse_price_range(&price_str);
                price_found = true;
            } else if let Some(price_val) = json_f64(item, "price") {
                product.price = PriceRange {
                    min_price: price_val,
                    max_price: price_val,
                    currency: "CNY".to_string(),
                };
                price_found = true;
            }
        }
        // Fall back to ICE price.
        if !price_found {
            if let Some(pi) = ice_sku_info {
                // Tmall ICE price is typically a string like "279.00"
                if let Some(price_str) = json_string(pi, "priceText")
                    .or_else(|| json_string(pi, "price"))
                {
                    product.price = parse_price_range(&price_str);
                } else if let Some(price_val) = json_f64(pi, "price") {
                    product.price = PriceRange {
                        min_price: price_val,
                        max_price: price_val,
                        currency: "CNY".to_string(),
                    };
                }
            }
            // Also try the top-level price field in ICE item.
            if !price_found {
                if let Some(pi) = ice_item {
                    if let Some(price_str) = json_string(pi, "price") {
                        product.price = parse_price_range(&price_str);
                    } else if let Some(price_val) = json_f64(pi, "price") {
                        product.price = PriceRange {
                            min_price: price_val,
                            max_price: price_val,
                            currency: "CNY".to_string(),
                        };
                    }
                }
            }
        }

        // Extract description.
        if let Some(item) = item_data {
            if let Some(desc) = json_string(item, "desc") {
                let text = strip_html_tags(&desc);
                product.description = Description {
                    text,
                    html: Some(desc),
                    specs: Vec::new(),
                };
            }
        }

        // Extract shop info: g_config first, then ICE.
        let seller_data = g_config.get("idata").and_then(|v| v.get("seller"));
        let mut shop_found = false;
        if let Some(seller) = seller_data {
            let name = json_string(seller, "shopName").unwrap_or_default();
            let url = json_string(seller, "shopUrl").unwrap_or_default();
            if !name.is_empty() {
                product.shop = ShopInfo { name, url };
                shop_found = true;
            }
        }
        if !shop_found {
            if let Some(seller) = ice_seller {
                let name = json_string(seller, "shopName").unwrap_or_default();
                let url = json_string(seller, "shopUrl").unwrap_or_default();
                if !name.is_empty() {
                    product.shop = ShopInfo { name, url };
                }
            }
        }

        // Extract SKU data from g_config.
        let sku_data = g_config
            .get("idata")
            .and_then(|v| v.get("sku"));

        if let Some(sku_val) = sku_data {
            if let Some(sku_list) = json_array(sku_val, "skuList") {
                for sku_item in sku_list {
                    let name = json_string(sku_item, "skuAttr").unwrap_or_default();
                    let price = json_string(sku_item, "price")
                        .map(|p| parse_price(&p))
                        .unwrap_or(0.0);
                    let stock = json_string(sku_item, "stock")
                        .and_then(|s| s.parse::<u32>().ok());

                    let image = json_string(sku_item, "image").map(|img_url| {
                        let cleaned = clean_taobao_image_url(&img_url);
                        let absolute = normalize_image_url(&cleaned, "item.taobao.com");
                        ImageRef {
                            original_url: absolute,
                            thumbnail_url: img_url,
                            local_path: None,
                        }
                    });

                    let value = json_string(sku_item, "value").unwrap_or_default();

                    let sku_key = if !value.is_empty() {
                        value.clone()
                    } else {
                        name.clone()
                    };

                    product.skus.push(SkuItem {
                        name,
                        value,
                        price,
                        stock,
                        image: image.clone(),
                    });

                    if let Some(ref img) = image {
                        product.sku_images.insert(sku_key, img.clone());
                    }
                }
            }

            // Update price range from SKU prices if available.
            if !product.skus.is_empty() {
                let prices: Vec<f64> = product.skus.iter().map(|s| s.price).filter(|p| *p > 0.0).collect();
                if !prices.is_empty() {
                    let min = prices.iter().cloned().fold(f64::INFINITY, f64::min);
                    let max = prices.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                    product.price = PriceRange {
                        min_price: min,
                        max_price: max,
                        currency: "CNY".to_string(),
                    };
                }
            }
        }

        // Step 5: Extract detail images via JavaScript (both Taobao and Tmall selectors).
        let detail_images_js = r#"
            (function() {
                var urls = [];
                var seen = {};
                function addUrl(src) {
                    if (src && !seen[src]) {
                        seen[src] = true;
                        urls.push(src);
                    }
                }
                // Tmall: images in #imageTextInfo-container (stored in <img> tags).
                var tmImgs = document.querySelectorAll('#imageTextInfo-container img, .descV8-singleImage-image');
                tmImgs.forEach(function(img) {
                    var src = img.getAttribute('data-src') || img.getAttribute('src');
                    addUrl(src);
                });
                // Also try lazy-loaded Tmall images with data-src.
                var lazyTmImgs = document.querySelectorAll('.descV8-singleImage-image[data-src]');
                lazyTmImgs.forEach(function(img) {
                    addUrl(img.getAttribute('data-src'));
                });
                // Taobao: images in desc area.
                var tbImgs = document.querySelectorAll('#description img, .desc-detail img, [data-spm="1000983"] img');
                tbImgs.forEach(function(img) {
                    var src = img.getAttribute('data-src') || img.getAttribute('src') || img.getAttribute('data-lazyload-src');
                    addUrl(src);
                });
                return urls;
            })()
        "#;

        let detail_result = page.evaluate(detail_images_js).await.unwrap_or(serde_json::Value::Null);
        if let Some(arr) = detail_result.as_array() {
            for url_val in arr {
                if let Some(url) = url_val.as_str() {
                    let cleaned = clean_taobao_image_url(url);
                    let absolute = normalize_image_url(&cleaned, "item.taobao.com");
                    product.detail_images.push(ImageRef {
                        original_url: absolute,
                        thumbnail_url: url.to_string(),
                        local_path: None,
                    });
                }
            }
        }

        // Step 6: Extract specs from DOM.
        let specs_js = r#"
            (function() {
                var specs = [];
                // Taobao selectors.
                var rows = document.querySelectorAll('.attributes-list li, [data-spm="1000981"] li, .J-attr-list li');
                rows.forEach(function(li) {
                    var spans = li.querySelectorAll('span');
                    if (spans.length >= 2) {
                        specs.push({
                            key: spans[0].textContent.trim(),
                            value: spans[1].textContent.trim()
                        });
                    }
                });
                // Tmall selectors: different DOM structure.
                if (specs.length === 0) {
                    var tmRows = document.querySelectorAll('#J_AttrUL li, .tm-attr-list li');
                    tmRows.forEach(function(li) {
                        var text = li.textContent.trim();
                        var colonIdx = text.indexOf(':');
                        if (colonIdx > 0) {
                            specs.push({
                                key: text.substring(0, colonIdx).trim(),
                                value: text.substring(colonIdx + 1).trim()
                            });
                        }
                    });
                }
                return specs;
            })()
        "#;

        let specs_result = page.evaluate(specs_js).await.unwrap_or(serde_json::Value::Null);
        if let Some(arr) = specs_result.as_array() {
            for spec_val in arr {
                let key = json_string(spec_val, "key").unwrap_or_default();
                let value = json_string(spec_val, "value").unwrap_or_default();
                if !key.is_empty() {
                    product.description.specs.push(crate::models::SpecItem { key, value });
                }
            }
        }

        // Build raw_data for debugging.
        let mut raw_data = serde_json::Map::new();
        raw_data.insert("g_config".to_string(), g_config);
        raw_data.insert("__ICE_APP_CONTEXT__".to_string(), ice_data);
        raw_data.insert("detail_images_count".to_string(), serde_json::Value::Number(product.detail_images.len().into()));
        raw_data.insert("specs_count".to_string(), serde_json::Value::Number(product.description.specs.len().into()));
        raw_data.insert("is_tmall".to_string(), serde_json::Value::Bool(is_tmall));

        // Record partial errors.
        if product.gallery.is_empty() {
            errors.push(ScrapeErrorInfo {
                step: ScrapeStep::Parsing,
                code: "GALLERY_EMPTY".to_string(),
                message: "No gallery images found".to_string(),
                recoverable: true,
            });
        }

        if product.detail_images.is_empty() {
            errors.push(ScrapeErrorInfo {
                step: ScrapeStep::Parsing,
                code: "DETAIL_IMAGES_EMPTY".to_string(),
                message: "No detail images found; lazy-load may not have triggered".to_string(),
                recoverable: true,
            });
        }

        Ok(ParseResult {
            product: Some(product),
            raw_data: serde_json::Value::Object(raw_data),
            errors,
        })
    }
}

/// Strips HTML tags from a string, returning plain text.
fn strip_html_tags(html: &str) -> String {
    // Simple tag stripping: remove anything between < and >.
    let mut result = String::with_capacity(html.len());
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    // Collapse whitespace.
    let collapsed: String = result
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ");
    collapsed
}

// ======== Tests ========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::PlatformParser;

    #[test]
    fn test_can_handle_taobao() {
        let parser = TaobaoParser::new();
        assert!(parser.can_handle("https://item.taobao.com/item.htm?id=123456"));
        assert!(parser.can_handle("https://detail.tmall.com/item.htm?id=789"));
        assert!(parser.can_handle("https://chaoshi.detail.tmall.com/item.htm?id=111"));
    }

    #[test]
    fn test_cannot_handle_jd() {
        let parser = TaobaoParser::new();
        assert!(!parser.can_handle("https://item.jd.com/12345678.html"));
    }

    #[test]
    fn test_extract_item_id_from_taobao_url() {
        let parser = TaobaoParser::new();
        let id = parser.extract_item_id("https://item.taobao.com/item.htm?id=123456789").unwrap();
        assert_eq!(id, "123456789");
    }

    #[test]
    fn test_extract_item_id_from_tmall_url() {
        let parser = TaobaoParser::new();
        let id = parser.extract_item_id("https://detail.tmall.com/item.htm?id=987654321&spm=xxx").unwrap();
        assert_eq!(id, "987654321");
    }

    #[test]
    fn test_extract_item_id_missing() {
        let parser = TaobaoParser::new();
        let result = parser.extract_item_id("https://item.taobao.com/item.htm?no_id=123");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_item_id_non_numeric() {
        let parser = TaobaoParser::new();
        let result = parser.extract_item_id("https://item.taobao.com/item.htm?id=abc123");
        assert!(result.is_err());
    }

    #[test]
    fn test_strip_html_tags() {
        let html = "<p>Hello <b>World</b></p><br/><div>Test</div>";
        let text = strip_html_tags(html);
        // Note: whitespace between adjacent tags may collapse differently.
        assert!(text.contains("Hello"));
        assert!(text.contains("World"));
        assert!(text.contains("Test"));
    }

    #[test]
    fn test_strip_html_tags_empty() {
        assert_eq!(strip_html_tags(""), "");
        assert_eq!(strip_html_tags("<br/>"), "");
    }

    #[test]
    fn test_platform_id() {
        let parser = TaobaoParser::new();
        assert_eq!(parser.platform_id(), "taobao");
    }
}
