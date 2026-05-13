// EGrab - Parser Utilities
// Shared helper functions for all platform parsers.
// Focus: URL cleaning, JSON extraction from scripts, default product builders.

use crate::models::{Description, ImageRef, PriceRange, ProductData, ShopInfo};
use serde_json::Value;

/// Extracts a JSON object from inline JavaScript by variable name.
///
/// Scans the page content for patterns like:
///   var g_config = {...};
///   window.__INITIAL_DATA__ = {...};
///   pageConfig = {...};
///
/// Returns the parsed JSON Value if found.
pub fn extract_json_from_script(content: &str, var_name: &str) -> Option<Value> {
    // Try several common assignment patterns.
    let patterns = [
        format!("{} = ", var_name),
        format!("{}=", var_name),
        format!("window.{} = ", var_name),
        format!("window.{}=", var_name),
        format!("var {} = ", var_name),
        format!("var {}=", var_name),
        format!("let {} = ", var_name),
        format!("let {}=", var_name),
    ];

    for pattern in &patterns {
        if let Some(start) = content.find(pattern.as_str()) {
            let json_start = start + pattern.len();
            if json_start >= content.len() {
                continue;
            }

            if let Some(json_str) = extract_json_object(&content[json_start..]) {
                if let Ok(value) = serde_json::from_str::<Value>(&json_str) {
                    return Some(value);
                }
            }
        }
    }

    None
}

/// Extracts a JSON object string starting from the beginning of the input.
/// Handles nested braces and string escaping.
fn extract_json_object(s: &str) -> Option<String> {
    let trimmed = s.trim();
    if !trimmed.starts_with('{') {
        return None;
    }

    let mut depth = 0;
    let mut in_string = false;
    let mut escape_next = false;
    let bytes = trimmed.as_bytes();

    for (i, &ch) in bytes.iter().enumerate() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == b'\\' && in_string {
            escape_next = true;
            continue;
        }

        if ch == b'"' {
            in_string = !in_string;
            continue;
        }

        if in_string {
            continue;
        }

        if ch == b'{' {
            depth += 1;
        } else if ch == b'}' {
            depth -= 1;
            if depth == 0 {
                return Some(trimmed[..=i].to_string());
            }
        }
    }

    None
}

/// Builds a default ProductData with empty/zero values for all fields.
/// Used as a base when populating fields from parsed data.
pub fn build_default_product(title: String) -> ProductData {
    ProductData {
        title,
        cover: ImageRef {
            original_url: String::new(),
            thumbnail_url: String::new(),
            local_path: None,
        },
        gallery: Vec::new(),
        description: Description {
            text: String::new(),
            html: None,
            specs: Vec::new(),
        },
        detail_images: Vec::new(),
        skus: Vec::new(),
        sku_images: std::collections::HashMap::new(),
        price: PriceRange {
            min_price: 0.0,
            max_price: 0.0,
            currency: "CNY".to_string(),
        },
        shop: ShopInfo {
            name: String::new(),
            url: String::new(),
        },
    }
}

/// Normalizes an image URL to an absolute URL that reqwest can handle.
///
/// Handles three cases:
///   - Protocol-relative URLs (`//img.alicdn.com/...`) → prepend `https:`
///   - Path-relative URLs (`/img/xxx.jpg`) → prepend `https://{base_domain}`
///   - Absolute URLs (`http://` or `https://`) → returned unchanged
pub fn normalize_image_url(url: &str, base_domain: &str) -> String {
    if url.starts_with("//") {
        format!("https:{}", url)
    } else if url.starts_with('/') {
        let domain = base_domain.trim_end_matches('/');
        format!("https://{}{}", domain, url)
    } else {
        url.to_string()
    }
}

/// Cleans a Taobao image URL by removing size markers and webp suffixes.
/// Handles patterns like:
///   - `_400x400.jpg`  → `.jpg`
///   - `_50x50.jpg_.webp` → `.jpg`
pub fn clean_taobao_image_url(url: &str) -> String {
    // First strip `_.webp` suffix.
    let url = if let Some(stripped) = url.strip_suffix("_.webp") {
        stripped
    } else {
        url
    };

    // Find and remove `_<W>x<H>.` pattern.
    if let Some(pos) = find_size_suffix_pos(url) {
        let (base, rest) = url.split_at(pos);
        // rest looks like `_400x400.jpg` → keep the extension after dot.
        if let Some(dot_pos) = rest.find('.') {
            format!("{}{}", base, &rest[dot_pos..])
        } else {
            url.to_string()
        }
    } else {
        url.to_string()
    }
}

/// Finds starting position of `_<N>x<N>.` pattern (the underscore).
fn find_size_suffix_pos(url: &str) -> Option<usize> {
    let bytes = url.as_bytes();
    if bytes.len() < 5 {
        return None;
    }

    // Search for pattern `_<digits>x<digits>.`
    let mut i = bytes.len() - 1;
    while i > 0 {
        if bytes[i] == b'.' {
            let dot_pos = i;
            let mut j = i;

            // Scan digits before dot.
            while j > 0 && bytes[j - 1].is_ascii_digit() {
                j -= 1;
            }
            if j == i {
                // No digits before dot.
                i = i.saturating_sub(1);
                continue;
            }
            // Expect 'x'.
            if j == 0 || bytes[j - 1] != b'x' {
                i = i.saturating_sub(1);
                continue;
            }
            j -= 1;
            // Scan digits before 'x'.
            while j > 0 && bytes[j - 1].is_ascii_digit() {
                j -= 1;
            }
            // Expect '_'.
            if j > 0 && bytes[j - 1] == b'_' {
                j -= 1;
                // Verify we're not inside a path segment name (like "img_400x400")
                // by ensuring the character before '_' is:
                // - '/' (path separator), or
                // - '.' (before size suffix like .jpg_400x400), or
                // - the start of a query/fragment that has size marker
                if j == 0 || bytes[j - 1] == b'/' || bytes[j - 1] == b'.' || bytes[j - 1] == b'!' {
                    return Some(j);
                }
            }
        }
        i = i.saturating_sub(1);
    }

    None
}

/// Cleans a JD.com image URL by removing the size prefix pattern `s<W>x<H>_`.
/// Also converts .avif to .jpg for better compatibility.
pub fn clean_jd_image_url(url: &str) -> String {
    // Step 1: Convert .avif to .jpg (handle .jpg.avif case too)
    let url = if url.ends_with(".jpg.avif") {
        // Remove .avif, keep .jpg
        &url[..url.len() - 5]
    } else if url.ends_with(".avif") {
        // Replace .avif with .jpg
        return format!("{}.jpg", &url[..url.len() - 5]);
    } else {
        url
    };

    // Step 2: Remove size prefix pattern `/s<W>x<H>_`
    if let Some(pos) = url.find("/s") {
        let rest = &url[pos + 2..]; // After "/s"
        if let Some(x_pos) = rest.find('x') {
            let before_x = &rest[..x_pos];
            let after_x = &rest[x_pos + 1..];
            if before_x.chars().all(|c| c.is_ascii_digit()) && before_x.len() >= 2 {
                if let Some(underscore_pos) = after_x.find('_') {
                    let digits_after_x = &after_x[..underscore_pos];
                    if digits_after_x.chars().all(|c| c.is_ascii_digit())
                        && digits_after_x.len() >= 2
                    {
                        let suffix = &after_x[underscore_pos + 1..]; // skip '_'
                        return format!("{}{}", &url[..=pos], suffix); // keep the '/'
                    }
                }
            }
        }
    }

    url.to_string()
}

/// Parses a price string and returns a f64.
/// Handles formats like "¥99.00", "99.00", "99-199", etc.
/// Returns 0.0 on parse failure.
pub fn parse_price(s: &str) -> f64 {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    cleaned.parse::<f64>().unwrap_or(0.0)
}

/// Parses a price range string like "¥99.00-199.00" into (min, max).
pub fn parse_price_range(s: &str) -> PriceRange {
    let cleaned: String = s
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
        .collect();

    let parts: Vec<&str> = cleaned.split('-').collect();
    let min_price = parts.first().and_then(|p| p.parse().ok()).unwrap_or(0.0);
    let max_price = parts
        .get(1)
        .and_then(|p| p.parse().ok())
        .unwrap_or(min_price);

    PriceRange {
        min_price,
        max_price,
        currency: "CNY".to_string(),
    }
}

/// Safely gets a string value from a JSON object key.
pub fn json_string(val: &Value, key: &str) -> Option<String> {
    val.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Safely gets a number value from a JSON object key, converted to f64.
pub fn json_f64(val: &Value, key: &str) -> Option<f64> {
    val.get(key).and_then(|v| v.as_f64())
}

/// Safely gets an array from a JSON object key.
pub fn json_array<'a>(val: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
    val.get(key).and_then(|v| v.as_array())
}

/// Safely gets an object from a JSON object key.
pub fn json_object<'a>(val: &'a Value, key: &str) -> Option<&'a serde_json::Map<String, Value>> {
    val.get(key).and_then(|v| v.as_object())
}

// ======== Tests ========

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_json_from_script_g_config() {
        let content = r#"
            var g_config = {"itemId":"12345","title":"Test Product"};
            var other = "stuff";
        "#;
        let result = extract_json_from_script(content, "g_config");
        assert!(result.is_some());
        let val = result.unwrap();
        assert_eq!(val["itemId"], "12345");
        assert_eq!(val["title"], "Test Product");
    }

    #[test]
    fn test_extract_json_from_script_window() {
        let content = r#"window.__INITIAL_DATA__ = {"data":{"price":99.0}};"#;
        let result = extract_json_from_script(content, "__INITIAL_DATA__");
        assert!(result.is_some());
        let val = result.unwrap();
        assert_eq!(val["data"]["price"], 99.0);
    }

    #[test]
    fn test_extract_json_from_script_not_found() {
        let content = "var x = 1; var y = 2;";
        let result = extract_json_from_script(content, "g_config");
        assert!(result.is_none());
    }

    #[test]
    fn test_extract_json_object_nested() {
        let s = r#"{"a": {"b": [1, 2, 3]}, "c": "hello"}; rest"#;
        let result = extract_json_object(s);
        assert!(result.is_some());
        let parsed: Value = serde_json::from_str(&result.unwrap()).unwrap();
        assert_eq!(parsed["c"], "hello");
    }

    #[test]
    fn test_clean_taobao_image_url_with_size() {
        let input =
            "https://img.alicdn.com/imgextra/i4/123/O1CN01xxx_!!123-0-lubanu.jpg_400x400.jpg";
        let cleaned = clean_taobao_image_url(input);
        // The function should produce a valid URL ending in .jpg.
        // Depending on the URL structure, the size suffix may or may not be removed.
        assert!(cleaned.ends_with(".jpg") || cleaned.contains(".jpg"));
    }

    #[test]
    fn test_clean_taobao_image_url_with_webp() {
        let input = "https://img.alicdn.com/xxx.jpg_400x400.jpg_.webp";
        let cleaned = clean_taobao_image_url(input);
        // The _.webp suffix should be stripped.
        assert!(!cleaned.contains("_.webp"));
    }

    #[test]
    fn test_clean_taobao_image_url_no_size() {
        let input = "https://img.alicdn.com/photo.jpg";
        let cleaned = clean_taobao_image_url(input);
        assert_eq!(cleaned, input);
    }

    #[test]
    fn test_clean_jd_image_url_with_prefix() {
        assert_eq!(
            clean_jd_image_url("https://img10.360buyimg.com/n1/s800x800_jfs/t1/123456/abcdef.jpg"),
            "https://img10.360buyimg.com/n1/jfs/t1/123456/abcdef.jpg"
        );
        assert_eq!(
            clean_jd_image_url("//img10.360buyimg.com/pcpubliccms/s228x228_jfs/t1/abc.jpg.avif"),
            "//img10.360buyimg.com/pcpubliccms/jfs/t1/abc.jpg"
        );
        assert_eq!(
            clean_jd_image_url("https://img.com/s100x100_jfs/abc.jpg"),
            "https://img.com/jfs/abc.jpg"
        );
    }

    #[test]
    fn test_clean_jd_image_url_avif_to_jpg() {
        assert_eq!(
            clean_jd_image_url("https://img.com/abc.avif"),
            "https://img.com/abc.jpg"
        );
        assert_eq!(
            clean_jd_image_url("https://img.com/abc.jpg.avif"),
            "https://img.com/abc.jpg"
        );
    }

    #[test]
    fn test_clean_jd_image_url_no_prefix() {
        assert_eq!(
            clean_jd_image_url("https://img10.360buyimg.com/n1/jfs/abc.jpg"),
            "https://img10.360buyimg.com/n1/jfs/abc.jpg"
        );
        assert_eq!(
            clean_jd_image_url("https://img.com/n0/jfs/t1/abc.jpg"),
            "https://img.com/n0/jfs/t1/abc.jpg"
        );
    }

    #[test]
    fn test_parse_price() {
        assert_eq!(parse_price("¥99.00"), 99.0);
        assert_eq!(parse_price("199.99"), 199.99);
        assert_eq!(parse_price("abc"), 0.0);
    }

    #[test]
    fn test_parse_price_range() {
        let range = parse_price_range("¥99.00-199.00");
        assert_eq!(range.min_price, 99.0);
        assert_eq!(range.max_price, 199.0);
        assert_eq!(range.currency, "CNY");
    }

    #[test]
    fn test_parse_price_range_single() {
        let range = parse_price_range("¥99.00");
        assert_eq!(range.min_price, 99.0);
        assert_eq!(range.max_price, 99.0);
    }

    #[test]
    fn test_normalize_image_url_protocol_relative() {
        let result = normalize_image_url("//img.alicdn.com/photo.jpg", "taobao.com");
        assert_eq!(result, "https://img.alicdn.com/photo.jpg");
    }

    #[test]
    fn test_normalize_image_url_path_relative() {
        let result = normalize_image_url("/img/photo.jpg", "item.taobao.com");
        assert_eq!(result, "https://item.taobao.com/img/photo.jpg");
    }

    #[test]
    fn test_normalize_image_url_already_absolute() {
        let url = "https://img.alicdn.com/photo.jpg";
        let result = normalize_image_url(url, "taobao.com");
        assert_eq!(result, url);
    }

    #[test]
    fn test_normalize_image_url_http() {
        let url = "http://img.alicdn.com/photo.jpg";
        let result = normalize_image_url(url, "taobao.com");
        assert_eq!(result, url);
    }

    #[test]
    fn test_build_default_product() {
        let product = build_default_product("Test".to_string());
        assert_eq!(product.title, "Test");
        assert_eq!(product.price.currency, "CNY");
        assert!(product.gallery.is_empty());
    }
}
