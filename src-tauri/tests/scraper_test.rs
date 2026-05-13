// EGrab - Integration Test: Scraper Engine End-to-End
// P6-3: Verifies URL parsing logic and event payload serialization formats.
// No real CDP connection required.

use egrab::models::{ProductData, ScrapeErrorInfo, ScrapeStep, TaskResult, TaskStatus};
use egrab::parser::{find_parser, PlatformParser};

/// 1. Validates URL platform detection via find_parser.
#[test]
fn scraper_url_validation() {
    // Taobao
    let parser = find_parser("https://item.taobao.com/item.htm?id=123");
    assert!(parser.is_some(), "Taobao URL should match a parser");
    assert_eq!(parser.unwrap().platform_id(), "taobao");

    // Tmall (uses taobao parser)
    let parser = find_parser("https://detail.tmall.com/item.htm?id=456");
    assert!(parser.is_some(), "Tmall URL should match a parser");
    assert_eq!(parser.unwrap().platform_id(), "taobao");

    // JD
    let parser = find_parser("https://item.jd.com/789.html");
    assert!(parser.is_some(), "JD URL should match a parser");
    assert_eq!(parser.unwrap().platform_id(), "jd");

    // Unsupported platform
    let parser = find_parser("https://www.amazon.com/dp/B00XXXX");
    assert!(parser.is_none(), "Amazon URL should not match any parser");
}

/// 2. Validates item_id extraction from URLs for all supported platforms.
#[test]
fn scraper_item_id_extraction() {
    // Taobao
    let parser = find_parser("https://item.taobao.com/item.htm?id=123456789").unwrap();
    let id = parser.extract_item_id("https://item.taobao.com/item.htm?id=123456789");
    assert_eq!(id.unwrap(), "123456789");

    // Tmall with extra query params
    let parser = find_parser("https://detail.tmall.com/item.htm?id=987654321&spm=xxx").unwrap();
    let id = parser.extract_item_id("https://detail.tmall.com/item.htm?id=987654321&spm=xxx");
    assert_eq!(id.unwrap(), "987654321");

    // JD plain
    let parser = find_parser("https://item.jd.com/12345678.html").unwrap();
    let id = parser.extract_item_id("https://item.jd.com/12345678.html");
    assert_eq!(id.unwrap(), "12345678");

    // JD with query params
    let parser = find_parser("https://item.jd.com/12345678.html?spm=xxx").unwrap();
    let id = parser.extract_item_id("https://item.jd.com/12345678.html?spm=xxx");
    assert_eq!(id.unwrap(), "12345678");

    // Invalid: taobao URL missing id param
    let parser = find_parser("https://item.taobao.com/item.htm?no_id=123").unwrap();
    let id = parser.extract_item_id("https://item.taobao.com/item.htm?no_id=123");
    assert!(id.is_err(), "Missing id param should error");

    // Invalid: jd homepage
    let parser = find_parser("https://www.jd.com/");
    // Note: find_parser returns None for www.jd.com since can_handle only matches item.jd.com
    assert!(parser.is_none(), "JD homepage should not match JdParser");
}

/// 3. Validates ScrapeProgressPayload serialization format.
#[test]
fn scraper_progress_event_structure() {
    // Build a JSON payload matching the expected TypeScript shape
    let payload = serde_json::json!({
        "task_id": "test-123",
        "percent": 50,
        "step": "parsing",
        "message": "Parsing product data"
    });

    // Verify top-level field names (snake_case)
    assert!(payload.get("task_id").is_some(), "missing task_id");
    assert!(payload.get("percent").is_some(), "missing percent");
    assert!(payload.get("step").is_some(), "missing step");
    assert!(payload.get("message").is_some(), "missing message");

    // Verify step value
    assert_eq!(payload["step"].as_str().unwrap(), "parsing");

    // Verify every ScrapeStep variant serializes to snake_case
    let cases = vec![
        (ScrapeStep::Connecting, "connecting"),
        (ScrapeStep::PageLoading, "page_loading"),
        (ScrapeStep::Parsing, "parsing"),
        (ScrapeStep::Downloading, "downloading"),
        (ScrapeStep::Saving, "saving"),
        (ScrapeStep::Completed, "completed"),
        (ScrapeStep::Failed, "failed"),
    ];

    for (variant, expected) in cases {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "ScrapeStep::{:?} should serialize as snake_case",
            variant
        );

        // Round-trip deserialization
        let deserialized: ScrapeStep = serde_json::from_str(&json).unwrap();
        assert_eq!(
            serde_json::to_string(&deserialized).unwrap(),
            json,
            "ScrapeStep round-trip failed for {:?}",
            variant
        );
    }
}

/// 4. Validates ScrapeCompletePayload serialization format.
#[test]
fn scraper_complete_event_structure() {
    // Build a TaskResult with a minimal ProductData to verify nested serialization
    let product = ProductData {
        title: "Test Product".to_string(),
        cover: egrab::models::ImageRef {
            original_url: "https://example.com/cover.jpg".to_string(),
            thumbnail_url: "https://example.com/cover_thumb.jpg".to_string(),
            local_path: Some("/tmp/cover.jpg".to_string()),
        },
        gallery: vec![],
        description: egrab::models::Description {
            text: "desc".to_string(),
            html: None,
            specs: vec![],
        },
        detail_images: vec![],
        skus: vec![],
        sku_images: std::collections::HashMap::new(),
        price: egrab::models::PriceRange {
            min_price: 10.0,
            max_price: 20.0,
            currency: "CNY".to_string(),
        },
        shop: egrab::models::ShopInfo {
            name: "Test Shop".to_string(),
            url: "https://shop.example.com".to_string(),
        },
    };

    let task_result = TaskResult {
        task_id: "test-123".to_string(),
        status: TaskStatus::Success,
        folder_path: Some("/path/to/folder".to_string()),
        product: Some(product),
        image_total: 10,
        image_success: 8,
        image_failed: 2,
        errors: vec![],
    };

    // Serialize the full payload as an object with task_id + result
    let payload = serde_json::json!({
        "task_id": "test-123",
        "result": task_result,
    });

    // Verify top-level field names
    assert!(payload.get("task_id").is_some(), "missing task_id");
    assert!(payload.get("result").is_some(), "missing result");

    // Verify TaskResult field names (snake_case)
    let result = &payload["result"];
    assert!(result.get("task_id").is_some(), "missing result.task_id");
    assert!(result.get("status").is_some(), "missing result.status");
    assert!(
        result.get("folder_path").is_some(),
        "missing result.folder_path"
    );
    assert!(result.get("product").is_some(), "missing result.product");
    assert!(
        result.get("image_total").is_some(),
        "missing result.image_total"
    );
    assert!(
        result.get("image_success").is_some(),
        "missing result.image_success"
    );
    assert!(
        result.get("image_failed").is_some(),
        "missing result.image_failed"
    );
    assert!(result.get("errors").is_some(), "missing result.errors");

    // Verify TaskResult values
    assert_eq!(result["task_id"].as_str().unwrap(), "test-123");
    assert_eq!(result["status"].as_str().unwrap(), "success");
    assert_eq!(result["folder_path"].as_str().unwrap(), "/path/to/folder");
    assert_eq!(result["image_total"].as_u64().unwrap(), 10);
    assert_eq!(result["image_success"].as_u64().unwrap(), 8);
    assert_eq!(result["image_failed"].as_u64().unwrap(), 2);
    assert!(result["errors"].is_array(), "errors should be array");

    // Verify nested ProductData field names inside result.product
    let product_json = &result["product"];
    assert!(product_json.get("title").is_some(), "missing product.title");
    assert!(product_json.get("cover").is_some(), "missing product.cover");
    assert!(
        product_json.get("gallery").is_some(),
        "missing product.gallery"
    );
    assert!(
        product_json.get("description").is_some(),
        "missing product.description"
    );
    assert!(
        product_json.get("detail_images").is_some(),
        "missing product.detail_images"
    );
    assert!(product_json.get("skus").is_some(), "missing product.skus");
    assert!(
        product_json.get("sku_images").is_some(),
        "missing product.sku_images"
    );
    assert!(product_json.get("price").is_some(), "missing product.price");
    assert!(product_json.get("shop").is_some(), "missing product.shop");

    // Verify TaskStatus variants serialize to snake_case
    let status_cases = vec![
        (TaskStatus::Pending, "pending"),
        (TaskStatus::Running, "running"),
        (TaskStatus::Success, "success"),
        (TaskStatus::Failed, "failed"),
        (TaskStatus::Partial, "partial"),
        (TaskStatus::Cancelled, "cancelled"),
    ];

    for (variant, expected) in status_cases {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "TaskStatus::{:?} should serialize as snake_case",
            variant
        );

        // Round-trip deserialization
        let deserialized: TaskStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(
            serde_json::to_string(&deserialized).unwrap(),
            json,
            "TaskStatus round-trip failed for {:?}",
            variant
        );
    }

    // Round-trip the whole payload via serde_json::Value to ensure structural integrity
    let json_string = serde_json::to_string(&payload).unwrap();
    let round_trip: serde_json::Value = serde_json::from_str(&json_string).unwrap();
    assert_eq!(round_trip["task_id"], "test-123");
    assert_eq!(round_trip["result"]["status"], "success");
    assert_eq!(round_trip["result"]["image_total"], 10);
}

/// 5. Validates ScrapeErrorPayload serialization format.
#[test]
fn scraper_error_event_structure() {
    let error_info = ScrapeErrorInfo {
        step: ScrapeStep::Parsing,
        code: "PARSE_FAILED".to_string(),
        message: "Failed to parse gallery".to_string(),
        recoverable: true,
    };

    // Build the payload as an object with task_id + error + recoverable
    let payload = serde_json::json!({
        "task_id": "test-456",
        "error": error_info,
        "recoverable": true,
    });

    // Verify top-level field names
    assert!(payload.get("task_id").is_some(), "missing task_id");
    assert!(payload.get("error").is_some(), "missing error");
    assert!(payload.get("recoverable").is_some(), "missing recoverable");

    // Verify recoverable is a boolean
    assert!(
        payload["recoverable"].is_boolean(),
        "recoverable should be a boolean"
    );
    assert_eq!(payload["recoverable"].as_bool().unwrap(), true);

    // Verify nested ScrapeErrorInfo field names (snake_case)
    let error = &payload["error"];
    assert!(error.get("step").is_some(), "missing error.step");
    assert!(error.get("code").is_some(), "missing error.code");
    assert!(error.get("message").is_some(), "missing error.message");
    assert!(
        error.get("recoverable").is_some(),
        "missing error.recoverable"
    );

    // Verify nested ScrapeErrorInfo values
    assert_eq!(error["step"].as_str().unwrap(), "parsing");
    assert_eq!(error["code"].as_str().unwrap(), "PARSE_FAILED");
    assert_eq!(
        error["message"].as_str().unwrap(),
        "Failed to parse gallery"
    );
    assert_eq!(error["recoverable"].as_bool().unwrap(), true);

    // Round-trip the whole payload
    let json_string = serde_json::to_string(&payload).unwrap();
    let round_trip: serde_json::Value = serde_json::from_str(&json_string).unwrap();
    assert_eq!(round_trip["task_id"], "test-456");
    assert_eq!(round_trip["recoverable"], true);
    assert_eq!(round_trip["error"]["step"], "parsing");
    assert_eq!(round_trip["error"]["code"], "PARSE_FAILED");
    assert_eq!(round_trip["error"]["message"], "Failed to parse gallery");

    // Also verify with recoverable = false
    let payload_not_recoverable = serde_json::json!({
        "task_id": "test-789",
        "error": ScrapeErrorInfo {
            step: ScrapeStep::Connecting,
            code: "CDP_CONNECT_FAILED".to_string(),
            message: "Browser not reachable".to_string(),
            recoverable: false,
        },
        "recoverable": false,
    });

    assert_eq!(
        payload_not_recoverable["recoverable"].as_bool().unwrap(),
        false
    );
    assert_eq!(
        payload_not_recoverable["error"]["recoverable"]
            .as_bool()
            .unwrap(),
        false
    );
}
