// EGrab - Platform Parser: JD.com
// Parses product pages from jd.com (item.jd.com).
// Derived from: docs/protocols/parser-interface.md, PRD 3.1.2/3.1.3
//
// Data extraction sources:
//   - pageConfig.product (title, images, price, skus) [legacy]
//   - window.__INITIAL_DATA__ (title, images, price, shop) [modern fallback]
//   - DOM fallback (title, images, price, shop) [final fallback when no JS variables]
//   - page.title() [ultimate title fallback]
//   - DOM extraction for specs and detail images

use crate::models::{
    ImageRef, PriceRange, ScrapeErrorInfo, ScrapeStep, ShopInfo, SkuItem,
};
use crate::parser::{PageHandle, ParseResult};
use crate::parser::utils::{
    build_default_product, clean_jd_image_url, json_array, json_f64, json_object, json_string,
    normalize_image_url, parse_price, parse_price_range,
};
use async_trait::async_trait;


/// JD.com product page parser.
pub struct JdParser;

impl JdParser {
    /// Creates a new JdParser instance.
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl super::PlatformParser for JdParser {
    /// Returns the platform identifier "jd".
    fn platform_id(&self) -> &str {
        "jd"
    }

    /// Checks whether this parser can handle the given URL.
    ///
    /// Matches:
    ///   - item.jd.com
    ///   - item.m.jd.com
    fn can_handle(&self, url: &str) -> bool {
        let parsed = url.to_lowercase();
        parsed.contains("item.jd.com")
    }

    /// Extracts the item ID from a JD.com URL.
    ///
    /// Looks for a numeric ID in the URL path segment like `/{item_id}.html`.
    fn extract_item_id(&self, url: &str) -> anyhow::Result<String> {
        // JD URLs typically look like: https://item.jd.com/12345678.html
        // Extract the numeric part before .html.
        let path = url.split('?').next().unwrap_or(url);

        // Find the last path segment.
        let last_segment = path.rsplit('/').next().unwrap_or("");

        // Try to extract digits before .html.
        if let Some(html_pos) = last_segment.find(".html") {
            let id_part = &last_segment[..html_pos];
            if !id_part.is_empty() && id_part.chars().all(|c| c.is_ascii_digit()) {
                return Ok(id_part.to_string());
            }
        }

        // Fallback: try to find any sequence of 5+ digits in the URL.
        let digits: String = path
            .chars()
            .skip_while(|c| !c.is_ascii_digit())
            .take_while(|c| c.is_ascii_digit())
            .collect();

        if digits.len() >= 5 {
            return Ok(digits);
        }

        anyhow::bail!("Failed to extract item ID from JD URL: {}", url);
    }

    /// Parses a JD.com product page and returns structured data.
    ///
    /// Strategy:
    /// 1. Execute JavaScript to extract pageConfig data
    /// 2. Parse product fields from the extracted JSON
    /// 3. Fall back to DOM extraction for missing fields
    /// 4. Clean image URLs using JD-specific rules
    async fn parse(&self, page: &dyn PageHandle) -> anyhow::Result<ParseResult> {
        let mut errors: Vec<ScrapeErrorInfo> = Vec::new();

        // Step 1: Extract pageConfig data via JavaScript.
        let page_config_js = r#"
            (function() {
                try {
                    if (typeof pageConfig !== 'undefined') {
                        return JSON.parse(JSON.stringify(pageConfig));
                    }
                } catch(e) {}
                return null;
            })()
        "#;

        let page_config = page.evaluate(page_config_js).await.unwrap_or(serde_json::Value::Null);

        // Step 2: Try __INITIAL_DATA__ as a fallback JS variable.
        // Modern JD pages may use this instead of pageConfig.
        let initial_data_js = r#"
            (function() {
                try {
                    if (typeof window.__INITIAL_DATA__ !== 'undefined') {
                        return JSON.parse(JSON.stringify(window.__INITIAL_DATA__));
                    }
                } catch(e) {}
                return null;
            })()
        "#;
        let initial_data = page.evaluate(initial_data_js).await.unwrap_or(serde_json::Value::Null);

        // Step 3: Extract product data from pageConfig first, fall back to __INITIAL_DATA__.
        let product_data = page_config.get("product").or_else(|| initial_data.get("product"));

        // Extract title.
        let title = product_data
            .and_then(|v| json_string(v, "name"))
            .or_else(|| {
                // Try __INITIAL_DATA__ directly for title at root level.
                initial_data.get("product").and_then(|v| json_string(v, "name"))
            })
            .unwrap_or_else(|| "Unknown Title".to_string());

        let mut product = build_default_product(title);

        // Extract cover and gallery images.
        let mut images_found = false;
        if let Some(images_arr) = product_data.and_then(|v| json_array(v, "imageList")) {
            if !images_arr.is_empty() {
                images_found = true;
                // Cover: first image.
                if let Some(first_img) = images_arr[0].as_str() {
                    let cleaned = clean_jd_image_url(first_img);
                    let absolute = normalize_image_url(&cleaned, "item.jd.com");
                    product.cover = ImageRef {
                        original_url: absolute.clone(),
                        thumbnail_url: first_img.to_string(),
                        local_path: None,
                    };
                }

                // Gallery: all images.
                for img_val in images_arr {
                    if let Some(img_url) = img_val.as_str() {
                        let cleaned = clean_jd_image_url(img_url);
                        let absolute = normalize_image_url(&cleaned, "item.jd.com");
                        product.gallery.push(ImageRef {
                            original_url: absolute,
                            thumbnail_url: img_url.to_string(),
                            local_path: None,
                        });
                    }
                }
            }
        }

        // Extract price.
        let mut price_found = false;
        if let Some(product_val) = product_data {
            if let Some(price_str) = json_string(product_val, "price") {
                product.price = parse_price_range(&price_str);
                price_found = true;
            } else if let Some(price_val) = json_f64(product_val, "price") {
                product.price = PriceRange {
                    min_price: price_val,
                    max_price: price_val,
                    currency: "CNY".to_string(),
                };
                price_found = true;
            }
        }

        // Extract shop info from pageConfig or __INITIAL_DATA__.
        let shop_data = page_config
            .get("shop")
            .or_else(|| initial_data.get("shop"));
        if let Some(shop) = shop_data {
            product.shop = ShopInfo {
                name: json_string(shop, "shopName")
                    .or_else(|| json_string(shop, "name"))
                    .unwrap_or_default(),
                url: json_string(shop, "shopUrl")
                    .or_else(|| json_string(shop, "url"))
                    .unwrap_or_default(),
            };
        }

        // Extract SKU data.
        if let Some(product_val) = product_data {
            // Try colorSize first.
            if let Some(color_size) = json_object(product_val, "colorSize") {
                // Convert the Map to a Value reference for json_array.
                let color_size_val = serde_json::Value::Object(color_size.clone());
                // Extract color options.
                if let Some(colors) = json_array(&color_size_val, "color") {
                    for color_val in colors {
                        let color_name = json_string(color_val, "name").unwrap_or_default();
                        let img_url = json_string(color_val, "image").unwrap_or_default();

                        let image = if !img_url.is_empty() {
                            let cleaned = clean_jd_image_url(&img_url);
                            let absolute = normalize_image_url(&cleaned, "item.jd.com");
                            Some(ImageRef {
                                original_url: absolute,
                                thumbnail_url: img_url,
                                local_path: None,
                            })
                        } else {
                            None
                        };

                        if !color_name.is_empty() {
                            product.skus.push(SkuItem {
                                name: "颜色".to_string(),
                                value: color_name.clone(),
                                price: 0.0, // Will be updated from sku list
                                stock: None,
                                image: image.clone(),
                            });

                            if let Some(ref img) = image {
                                product.sku_images.insert(color_name, img.clone());
                            }
                        }
                    }
                }
            }

            // Try skus array for price/stock info.
            if let Some(sku_list) = json_array(product_val, "skus") {
                let mut prices: Vec<f64> = Vec::new();
                for sku_val in sku_list {
                    let price = json_string(sku_val, "price")
                        .map(|p| parse_price(&p))
                        .unwrap_or(0.0);
                    let stock = json_string(sku_val, "stock")
                        .and_then(|s| s.parse::<u32>().ok());

                    if price > 0.0 {
                        prices.push(price);
                    }

                    // Update existing SKU items with price/stock if matched.
                    // For simplicity, we just track the price range.
                    let _ = stock; // Used in future enhancement
                }

                // Update price range from SKU prices.
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

        // Step 4: DOM fallback extraction when JS variables failed to produce useful data.
        // Modern JD pages may not expose any global JS variables at all.
        if product.title == "Unknown Title" || !images_found || !price_found {
            let dom_fallback_js = r#"
                (function() {
                    try {
                        // Title: use .sku-name or any h1-like element
                        var title = document.querySelector('.sku-name')?.textContent?.trim() || '';
                        // Gallery images: use image-carousel-track with deduplication
                        var seen = {};
                        var images = [];
                        document.querySelectorAll('._gallery_116km_1 .image-carousel-track .item img.image').forEach(function(img) {
                            var src = img.getAttribute('src') || '';
                            if (src && !src.startsWith('data:') && !seen[src]) {
                                seen[src] = true;
                                images.push(src);
                            }
                        });
                        // Price: .product-price--value (plain number), fallback to .product-price--main (with ¥)
                        var price = document.querySelector('.product-price--value')?.textContent?.trim() || '';
                        if (!price) {
                            price = document.querySelector('.product-price--main')?.textContent?.trim() || '';
                        }
                        // Shop: .top-name
                        var shop = document.querySelector('.top-name')?.textContent?.trim() || '';
                        return { title: title, images: images, price: price, shop: shop };
                    } catch(e) {
                        return null;
                    }
                })()
            "#;
            let dom_data = page.evaluate(dom_fallback_js).await.unwrap_or(serde_json::Value::Null);

            // Fill title from DOM if still unknown.
            if product.title == "Unknown Title" || product.title.is_empty() {
                if let Some(dom_title) = dom_data.get("title").and_then(|v| v.as_str()) {
                    if !dom_title.is_empty() {
                        product.title = dom_title.to_string();
                    }
                }
            }

            // Fill images from DOM if none found yet.
            if !images_found {
                if let Some(dom_images) = dom_data.get("images").and_then(|v| v.as_array()) {
                    if !dom_images.is_empty() {
                        if let Some(first_img) = dom_images[0].as_str() {
                            let cleaned = clean_jd_image_url(first_img);
                            let absolute = normalize_image_url(&cleaned, "item.jd.com");
                            product.cover = ImageRef {
                                original_url: absolute.clone(),
                                thumbnail_url: first_img.to_string(),
                                local_path: None,
                            };
                        }
                        for img_val in dom_images {
                            if let Some(img_url) = img_val.as_str() {
                                let cleaned = clean_jd_image_url(img_url);
                                let absolute = normalize_image_url(&cleaned, "item.jd.com");
                                product.gallery.push(ImageRef {
                                    original_url: absolute,
                                    thumbnail_url: img_url.to_string(),
                                    local_path: None,
                                });
                            }
                        }
                    }
                }
            }

            // Fill price from DOM if none found yet.
            if !price_found {
                if let Some(dom_price) = dom_data.get("price").and_then(|v| v.as_str()) {
                    if !dom_price.is_empty() {
                        product.price = parse_price_range(dom_price);
                    }
                }
            }

            // Fill shop from DOM if none found yet.
            if product.shop.name.is_empty() {
                if let Some(dom_shop) = dom_data.get("shop").and_then(|v| v.as_str()) {
                    if !dom_shop.is_empty() {
                        product.shop = ShopInfo {
                            name: dom_shop.to_string(),
                            url: String::new(),
                        };
                    }
                }
            }
        }

        // Step 5: Use page.title() as final title fallback.
        // This catches cases where no JS variable or DOM selector could find the title.
        if product.title == "Unknown Title" || product.title.is_empty() {
            if let Ok(page_title) = page.title().await {
                if !page_title.is_empty() && page_title != "京东" {
                    product.title = page_title;
                }
            }
        }

        // Clean JD.com title suffixes like "【行情 报价 价格 评测】-京东" or "-京东".
        product.title = product.title
            .replace("【行情 报价 价格 评测】-京东", "")
            .replace("-京东", "")
            .trim()
            .to_string();

        // Step 6: Extract detail images via JavaScript.
        //
        // Strategy (priority order):
        // 1. #zbViewWeChatMiniImages[value] — comma-separated mobile detail image paths
        // 2. <style> tags — extract background-image:url() references for .ssd-module
        // 3. getComputedStyle() fallback on .ssd-module elements
        let detail_images_js = r#"
            (function() {
                var urls = [];
                var seen = {};
                var debug = {};

                function push(u) {
                    if (!u) return;
                    u = String(u).trim();
                    if (!u) return;
                    u = u.replace(/^["']+|["']+$/g, '');
                    // Normalize URL for dedup: remove protocol prefix differences
                    var canonical = u.replace(/^https?:/, '').replace(/\/\/+$/, '');
                    if (!canonical || seen[canonical]) return;
                    seen[canonical] = true;
                    urls.push(u);
                }
                function toAbs(p) {
                    if (!p) return '';
                    p = String(p).trim().replace(/^["']+|["']+$/g, '');
                    if (!p) return '';
                    if (/^https?:\/\//i.test(p)) return p;
                    if (p.indexOf('//') === 0) return 'https:' + p;
                    if (p.charAt(0) !== '/') p = '/' + p;
                    return 'https://img30.360buyimg.com' + p;
                }

                // Strategy 1: Extract from #zbViewWeChatMiniImages (mobile detail images)
                var zbEl = document.getElementById('zbViewWeChatMiniImages');
                if (zbEl && zbEl.value) {
                    var zbPaths = zbEl.value.split(',');
                    debug.zbPathsCount = zbPaths.length;
                    for (var z = 0; z < zbPaths.length; z++) {
                        var p = zbPaths[z].trim();
                        if (p) {
                            push(toAbs(p));
                        }
                    }
                }
                debug.urlsAfterZb = urls.length;

                // Strategy 2: Parse <style> tags inside detail containers
                // JD detail sections use multiple containers: detail-main, detail-top,
                // detail-header, detail-footer, related-layout-head, related-layout-footer.
                // Scan ALL <style> elements, but only keep those whose ancestor has an
                // ID matching /^(detail-|related-layout-)/ to exclude global page CSS.
                var allStyleNodes = document.querySelectorAll('style');
                debug.allStyleCount = allStyleNodes.length;
                var styleNodes = [];
                for (var s = 0; s < allStyleNodes.length; s++) {
                    var node = allStyleNodes[s];
                    var parent = node.parentElement;
                    var ok = false;
                    while (parent) {
                        var pid = parent.id || '';
                        if (pid.indexOf('detail-') === 0 || pid.indexOf('related-layout-') === 0) {
                            ok = true;
                            break;
                        }
                        parent = parent.parentElement;
                    }
                    if (ok) styleNodes.push(node);
                }
                debug.detailStyleCount = styleNodes.length;

                // Collect styleText for debug
                var styleText = '';
                for (var i = 0; i < styleNodes.length; i++) {
                    styleText += '\n' + (styleNodes[i].textContent || '');
                }
                debug.styleTextLength = styleText.length;
                debug.hasSsdModule = styleText.indexOf('ssd-module') !== -1;
                debug.hasBackgroundImage = styleText.indexOf('background-image') !== -1;

                // Extract all background-image URLs from detail-container <style> tags
                for (var i = 0; i < styleNodes.length; i++) {
                    var text = styleNodes[i].textContent || '';
                    var searchStr = 'background-image:url(';
                    var idx = 0;
                    while (true) {
                        idx = text.indexOf(searchStr, idx);
                        if (idx === -1) break;
                        idx += searchStr.length;
                        var end = text.indexOf(')', idx);
                        if (end === -1) break;
                        var url = text.substring(idx, end).trim();
                        url = url.replace(/^["']+|["']+$/g, '');
                        // Filter: must be on JD CDN domain and have image file extension
                        var isJdCdn = url.indexOf('360buyimg.com') !== -1 || url.indexOf('jd.com') !== -1;
                        var isImageExt = /\.(jpg|jpeg|png|avif|webp|bmp|gif)($|\?|#)/i.test(url);
                        var isNotUtility = url.indexOf('/icon') === -1 && url.indexOf('/tool') === -1 && url.indexOf('/sprite') === -1;
                        if (isJdCdn && isImageExt && isNotUtility) {
                            push(toAbs(url));
                        }
                        idx = end + 1;
                    }
                }
                debug.urlsAfterStyle = urls.length;

                // Strategy 3: getComputedStyle() as backup
                if (urls.length === 0) {
                    var modules = document.querySelectorAll('.ssd-module-wrap .ssd-module, div.ssd-module[data-id]');
                    debug.moduleCount = modules.length;
                    modules.forEach(function(mod) {
                        try {
                            var cs = window.getComputedStyle(mod);
                            var bg = cs && cs.backgroundImage;
                            if (bg && bg !== 'none') {
                                var mm = bg.match(/url\(["']?([^"')]+)["']?\)/);
                                if (mm && mm[1]) push(toAbs(mm[1]));
                            }
                        } catch(e) {}
                    });
                }
                debug.urlsFinal = urls.length;

                // Return both urls and debug info
                return { urls: urls, debug: debug };
            })()
        "#;

        let detail_result = page.evaluate(detail_images_js).await.unwrap_or(serde_json::Value::Null);
        // Log debug info for diagnosis
        if let Some(obj) = detail_result.as_object() {
            if let Some(debug_val) = obj.get("debug") {
                tracing::info!("JD detail images debug: {:?}", debug_val);
            }
            if let Some(arr) = obj.get("urls").and_then(|v| v.as_array()) {
                for url_val in arr {
                    if let Some(url) = url_val.as_str() {
                        let cleaned = clean_jd_image_url(url);
                        let absolute = normalize_image_url(&cleaned, "item.jd.com");
                        product.detail_images.push(ImageRef {
                            original_url: absolute,
                            thumbnail_url: url.to_string(),
                            local_path: None,
                        });
                    }
                }
            }
        }

        // Step 7: Extract specs from DOM.
        let specs_js = r#"
            (function() {
                var specs = [];
                var items = document.querySelectorAll('.attrs .item');
                items.forEach(function(item) {
                    var label = item.querySelector('.label .text') || item.querySelector('.label span.text');
                    var value = item.querySelector('.value .text') || item.querySelector('.value div.text');
                    // Fallback to direct .label and .value if nested text not found
                    if (!label) label = item.querySelector('.label');
                    if (!value) value = item.querySelector('.value');
                    if (label && value) {
                        specs.push({
                            key: label.textContent.trim(),
                            value: value.textContent.trim()
                        });
                    }
                });
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

        // Extract description text from DOM.
        let desc_js = r#"
            (function() {
                var desc = document.querySelector('#detail .detail-content, .detail-content');
                return desc ? desc.textContent.trim().substring(0, 5000) : '';
            })()
        "#;

        let desc_result = page.evaluate(desc_js).await.unwrap_or(serde_json::Value::Null);
        if let Some(text) = desc_result.as_str() {
            if !text.is_empty() && product.description.text.is_empty() {
                product.description.text = text.to_string();
            }
        }

        // Extract SKU images from DOM as fallback.
        // Only runs when JS-variable SKU extraction found no SKU images.
        if product.sku_images.is_empty() {
            let sku_images_js = r#"
                (function() {
                    var skuImages = [];
                    document.querySelectorAll('.specification-item-sku-image').forEach(function(img) {
                        var src = img.getAttribute('src') || '';
                        var alt = img.getAttribute('alt') || '';
                        if (src && !src.startsWith('data:')) {
                            skuImages.push({ url: src, name: alt });
                        }
                    });
                    return skuImages;
                })()
            "#;

            let sku_images_result = page.evaluate(sku_images_js).await.unwrap_or(serde_json::Value::Null);
            if let Some(arr) = sku_images_result.as_array() {
                for sku_img_val in arr {
                    let url = json_string(sku_img_val, "url").unwrap_or_default();
                    let name = json_string(sku_img_val, "name").unwrap_or_default();
                    if !url.is_empty() {
                        let cleaned = clean_jd_image_url(&url);
                        let absolute = normalize_image_url(&cleaned, "item.jd.com");
                        let img_ref = ImageRef {
                            original_url: absolute,
                            thumbnail_url: url,
                            local_path: None,
                        };
                        if !name.is_empty() {
                            product.sku_images.insert(name.clone(), img_ref);
                        }
                        // Also add as a basic SKU entry if we have a name
                        if !name.is_empty() {
                            // Check if we already have this SKU
                            let exists = product.skus.iter().any(|s| s.value == name);
                            if !exists {
                                product.skus.push(SkuItem {
                                    name: String::new(),
                                    value: name,
                                    price: 0.0,
                                    stock: None,
                                    image: None,
                                });
                            }
                        }
                    }
                }
            }
        }

        // Build raw_data for debugging.
        let mut raw_data = serde_json::Map::new();
        raw_data.insert("pageConfig".to_string(), page_config);
        raw_data.insert("__INITIAL_DATA__".to_string(), initial_data);
        raw_data.insert("detail_images_count".to_string(), serde_json::Value::Number(product.detail_images.len().into()));
        raw_data.insert("specs_count".to_string(), serde_json::Value::Number(product.description.specs.len().into()));

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

// ======== Tests ========

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::PlatformParser;

    #[test]
    fn test_can_handle_jd() {
        let parser = JdParser::new();
        assert!(parser.can_handle("https://item.jd.com/12345678.html"));
        // Note: item.m.jd.com is not currently supported by the parser
        // as it uses a different page structure.
    }

    #[test]
    fn test_cannot_handle_taobao() {
        let parser = JdParser::new();
        assert!(!parser.can_handle("https://item.taobao.com/item.htm?id=123456"));
    }

    #[test]
    fn test_extract_item_id_from_jd_url() {
        let parser = JdParser::new();
        let id = parser.extract_item_id("https://item.jd.com/12345678.html").unwrap();
        assert_eq!(id, "12345678");
    }

    #[test]
    fn test_extract_item_id_with_query() {
        let parser = JdParser::new();
        let id = parser.extract_item_id("https://item.jd.com/12345678.html?spm=xxx").unwrap();
        assert_eq!(id, "12345678");
    }

    #[test]
    fn test_extract_item_id_missing() {
        let parser = JdParser::new();
        let result = parser.extract_item_id("https://www.jd.com/");
        assert!(result.is_err());
    }

    #[test]
    fn test_platform_id() {
        let parser = JdParser::new();
        assert_eq!(parser.platform_id(), "jd");
    }
}
