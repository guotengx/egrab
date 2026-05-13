// EGrab - Integration Test: Models Serialization Verification
// Verifies P0-1 through P0-4 fixes for ErrorCode, IpcError, DuplicateTaskConflict, and module visibility

use egrab::models::{
    AppConfig, BrowserLaunchCommand, BrowserOs, BrowserType, ConnectionState, Description,
    DuplicateTaskConflict, ErrorCode, ImageRef, IpcError, MetaJsonDocument, PriceRange,
    ProductData, RawJsonDocument, ScrapeStep, ShopInfo, SkuItem, SpecItem, TaskFilter, TaskStatus,
};
use std::collections::HashMap;

/// P0-1: ErrorCode must serialize to SCREAMING_SNAKE_CASE
#[test]
fn error_code_serializes_to_screaming_snake_case() {
    let json = serde_json::to_string(&ErrorCode::CdpConnectFailed).unwrap();
    assert_eq!(json, "\"CDP_CONNECT_FAILED\"");

    let json = serde_json::to_string(&ErrorCode::DuplicateTask).unwrap();
    assert_eq!(json, "\"DUPLICATE_TASK\"");

    let json = serde_json::to_string(&ErrorCode::UnknownError).unwrap();
    assert_eq!(json, "\"UNKNOWN_ERROR\"");
}

/// P0-1: ErrorCode must deserialize from SCREAMING_SNAKE_CASE
#[test]
fn error_code_deserializes_from_screaming_snake_case() {
    let code: ErrorCode = serde_json::from_str("\"CDP_CONNECT_FAILED\"").unwrap();
    assert!(matches!(code, ErrorCode::CdpConnectFailed));

    let code: ErrorCode = serde_json::from_str("\"DUPLICATE_TASK\"").unwrap();
    assert!(matches!(code, ErrorCode::DuplicateTask));
}

/// P0-4: IpcError.code must be ErrorCode enum (type-safe)
#[test]
fn ipc_error_code_is_error_code_enum() {
    let error = IpcError {
        code: ErrorCode::CdpConnectFailed,
        message: "Connection failed".to_string(),
        recoverable: true,
        step: Some(ScrapeStep::Connecting),
        details: None,
    };

    let json = serde_json::to_string(&error).unwrap();
    assert!(
        json.contains("\"CDP_CONNECT_FAILED\""),
        "IpcError.code should serialize as SCREAMING_SNAKE_CASE ErrorCode"
    );

    // Verify round-trip
    let deserialized: IpcError = serde_json::from_str(&json).unwrap();
    assert!(matches!(deserialized.code, ErrorCode::CdpConnectFailed));
}

/// P0-3: DuplicateTaskConflict.code must be fixed to ErrorCode::DuplicateTask
#[test]
fn duplicate_task_conflict_code_is_fixed_duplicate_task() {
    let conflict = DuplicateTaskConflict::new(
        "task_20260508_000001".to_string(),
        Some("/path/to/folder".to_string()),
    );

    // Verify code is always DuplicateTask
    assert!(matches!(conflict.code, ErrorCode::DuplicateTask));

    // Verify serialization
    let json = serde_json::to_string(&conflict).unwrap();
    assert!(
        json.contains("\"DUPLICATE_TASK\""),
        "DuplicateTaskConflict.code should serialize as DUPLICATE_TASK"
    );
}

/// P0-3: DuplicateTaskConflict round-trip serialization
#[test]
fn duplicate_task_conflict_round_trip() {
    let conflict = DuplicateTaskConflict::new("task_123".to_string(), None);

    let json = serde_json::to_string(&conflict).unwrap();
    let deserialized: DuplicateTaskConflict = serde_json::from_str(&json).unwrap();

    assert!(matches!(deserialized.code, ErrorCode::DuplicateTask));
    assert_eq!(deserialized.existing_task_id, "task_123");
    assert_eq!(deserialized.existing_folder_path, None);
}

/// Verify all ErrorCode variants serialize correctly
#[test]
fn all_error_code_variants_screaming_snake_case() {
    let variants = vec![
        (ErrorCode::CdpConnectFailed, "CDP_CONNECT_FAILED"),
        (ErrorCode::CdpTimeout, "CDP_TIMEOUT"),
        (ErrorCode::UrlInvalid, "URL_INVALID"),
        (ErrorCode::UnsupportedPlatform, "UNSUPPORTED_PLATFORM"),
        (ErrorCode::ItemIdExtractFailed, "ITEM_ID_EXTRACT_FAILED"),
        (ErrorCode::DuplicateTask, "DUPLICATE_TASK"),
        (ErrorCode::TaskAlreadyRunning, "TASK_ALREADY_RUNNING"),
        (ErrorCode::TaskNotFound, "TASK_NOT_FOUND"),
        (ErrorCode::TaskCancelled, "TASK_CANCELLED"),
        (ErrorCode::ParseFailed, "PARSE_FAILED"),
        (ErrorCode::ImageDownloadFailed, "IMAGE_DOWNLOAD_FAILED"),
        (ErrorCode::StorageFailed, "STORAGE_FAILED"),
        (ErrorCode::PathNotAllowed, "PATH_NOT_ALLOWED"),
        (ErrorCode::ConfigInvalid, "CONFIG_INVALID"),
        (ErrorCode::UnknownError, "UNKNOWN_ERROR"),
    ];

    for (variant, expected) in variants {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "ErrorCode::{:?} should serialize as \"{}\"",
            variant,
            expected
        );
    }
}

// ---------------------------------------------------------------------------
// Phase 6 Batch 1: ProductData and related model serialization
// ---------------------------------------------------------------------------

/// Helper: builds a fully populated ProductData for serialization tests.
fn create_test_product() -> ProductData {
    let image = ImageRef {
        original_url: "https://example.com/original.jpg".to_string(),
        thumbnail_url: "https://example.com/thumb.jpg".to_string(),
        local_path: Some("/path/to/image.jpg".to_string()),
    };

    ProductData {
        title: "Test Product".to_string(),
        cover: image.clone(),
        gallery: vec![image.clone()],
        description: Description {
            text: "A great product".to_string(),
            html: Some("<p>A great product</p>".to_string()),
            specs: vec![SpecItem {
                key: "Material".to_string(),
                value: "Cotton".to_string(),
            }],
        },
        detail_images: vec![image.clone()],
        skus: vec![SkuItem {
            name: "Color".to_string(),
            value: "Red".to_string(),
            price: 99.99,
            stock: Some(100),
            image: Some(image.clone()),
        }],
        sku_images: {
            let mut map = HashMap::new();
            map.insert("Red".to_string(), image.clone());
            map
        },
        price: PriceRange {
            min_price: 99.99,
            max_price: 199.99,
            currency: "CNY".to_string(),
        },
        shop: ShopInfo {
            name: "Test Shop".to_string(),
            url: "https://shop.example.com".to_string(),
        },
    }
}

/// ProductData full structure serializes with snake_case field names.
#[test]
fn product_data_full_structure_serializes_to_snake_case() {
    let product = create_test_product();
    let json = serde_json::to_string(&product).expect("ProductData should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert!(value.get("title").is_some(), "missing title");
    assert!(value.get("cover").is_some(), "missing cover");
    assert!(value.get("gallery").is_some(), "missing gallery");
    assert!(value.get("description").is_some(), "missing description");
    assert!(
        value.get("detail_images").is_some(),
        "missing detail_images"
    );
    assert!(value.get("skus").is_some(), "missing skus");
    assert!(value.get("sku_images").is_some(), "missing sku_images");
    assert!(value.get("price").is_some(), "missing price");
    assert!(value.get("shop").is_some(), "missing shop");

    // Ensure no camelCase leakage
    assert!(
        value.get("detailImages").is_none(),
        "camelCase detailImages leaked"
    );
    assert!(
        value.get("skuImages").is_none(),
        "camelCase skuImages leaked"
    );
    assert!(value.get("minPrice").is_none(), "camelCase minPrice leaked");

    // Round-trip field checks (avoid exact JSON string due to HashMap ordering)
    let deserialized: ProductData = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.title, product.title);
    assert_eq!(deserialized.cover.original_url, product.cover.original_url);
    assert_eq!(deserialized.gallery.len(), product.gallery.len());
    assert_eq!(
        deserialized.detail_images.len(),
        product.detail_images.len()
    );
    assert_eq!(deserialized.skus.len(), product.skus.len());
    assert_eq!(deserialized.sku_images.len(), product.sku_images.len());
    assert_eq!(
        deserialized
            .sku_images
            .get("Red")
            .expect("Red SKU image missing")
            .original_url,
        product
            .sku_images
            .get("Red")
            .expect("Red SKU image missing")
            .original_url
    );
    assert_eq!(deserialized.price.min_price, product.price.min_price);
    assert_eq!(deserialized.price.max_price, product.price.max_price);
    assert_eq!(deserialized.price.currency, product.price.currency);
    assert_eq!(deserialized.shop.name, product.shop.name);
    assert_eq!(deserialized.shop.url, product.shop.url);
}

/// ImageRef fields serialize as snake_case.
#[test]
fn image_ref_serializes_correctly() {
    let image = ImageRef {
        original_url: "https://example.com/original.jpg".to_string(),
        thumbnail_url: "https://example.com/thumb.jpg".to_string(),
        local_path: Some("/local/path.jpg".to_string()),
    };
    let json = serde_json::to_string(&image).expect("ImageRef should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["original_url"]
            .as_str()
            .expect("original_url should be string"),
        "https://example.com/original.jpg"
    );
    assert_eq!(
        value["thumbnail_url"]
            .as_str()
            .expect("thumbnail_url should be string"),
        "https://example.com/thumb.jpg"
    );
    assert_eq!(
        value["local_path"]
            .as_str()
            .expect("local_path should be string"),
        "/local/path.jpg"
    );

    let deserialized: ImageRef = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.original_url, image.original_url);
    assert_eq!(deserialized.thumbnail_url, image.thumbnail_url);
    assert_eq!(deserialized.local_path, image.local_path);
}

/// SkuItem fields serialize as snake_case.
#[test]
fn sku_item_serializes_correctly() {
    let image = ImageRef {
        original_url: "https://example.com/sku.jpg".to_string(),
        thumbnail_url: "https://example.com/sku_thumb.jpg".to_string(),
        local_path: None,
    };
    let sku = SkuItem {
        name: "Color".to_string(),
        value: "Blue".to_string(),
        price: 49.99,
        stock: Some(50),
        image: Some(image),
    };
    let json = serde_json::to_string(&sku).expect("SkuItem should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["name"].as_str().expect("name should be string"),
        "Color"
    );
    assert_eq!(
        value["value"].as_str().expect("value should be string"),
        "Blue"
    );
    assert_eq!(
        value["price"].as_f64().expect("price should be number"),
        49.99
    );
    assert_eq!(value["stock"].as_u64().expect("stock should be number"), 50);
    assert!(value["image"].is_object(), "image should be object");

    let deserialized: SkuItem = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.name, sku.name);
    assert_eq!(deserialized.value, sku.value);
    assert_eq!(deserialized.price, sku.price);
    assert_eq!(deserialized.stock, sku.stock);
}

/// PriceRange fields serialize as snake_case.
#[test]
fn price_range_serializes_correctly() {
    let price = PriceRange {
        min_price: 10.0,
        max_price: 20.0,
        currency: "CNY".to_string(),
    };
    let json = serde_json::to_string(&price).expect("PriceRange should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["min_price"]
            .as_f64()
            .expect("min_price should be number"),
        10.0
    );
    assert_eq!(
        value["max_price"]
            .as_f64()
            .expect("max_price should be number"),
        20.0
    );
    assert_eq!(
        value["currency"]
            .as_str()
            .expect("currency should be string"),
        "CNY"
    );

    let deserialized: PriceRange = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.min_price, price.min_price);
    assert_eq!(deserialized.max_price, price.max_price);
    assert_eq!(deserialized.currency, price.currency);
}

/// ShopInfo fields serialize as snake_case.
#[test]
fn shop_info_serializes_correctly() {
    let shop = ShopInfo {
        name: "My Shop".to_string(),
        url: "https://shop.example.com".to_string(),
    };
    let json = serde_json::to_string(&shop).expect("ShopInfo should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["name"].as_str().expect("name should be string"),
        "My Shop"
    );
    assert_eq!(
        value["url"].as_str().expect("url should be string"),
        "https://shop.example.com"
    );

    let deserialized: ShopInfo = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.name, shop.name);
    assert_eq!(deserialized.url, shop.url);
}

/// Description fields serialize as snake_case.
#[test]
fn description_serializes_correctly() {
    let desc = Description {
        text: "Description text".to_string(),
        html: Some("<p>html</p>".to_string()),
        specs: vec![SpecItem {
            key: "Weight".to_string(),
            value: "1kg".to_string(),
        }],
    };
    let json = serde_json::to_string(&desc).expect("Description should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["text"].as_str().expect("text should be string"),
        "Description text"
    );
    assert_eq!(
        value["html"].as_str().expect("html should be string"),
        "<p>html</p>"
    );
    assert!(value["specs"].is_array(), "specs should be array");

    let deserialized: Description = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.text, desc.text);
    assert_eq!(deserialized.html, desc.html);
    assert_eq!(deserialized.specs.len(), desc.specs.len());
}

/// SpecItem fields serialize as snake_case.
#[test]
fn spec_item_serializes_correctly() {
    let spec = SpecItem {
        key: "Color".to_string(),
        value: "Red".to_string(),
    };
    let json = serde_json::to_string(&spec).expect("SpecItem should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["key"].as_str().expect("key should be string"),
        "Color"
    );
    assert_eq!(
        value["value"].as_str().expect("value should be string"),
        "Red"
    );

    let deserialized: SpecItem = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.key, spec.key);
    assert_eq!(deserialized.value, spec.value);
}

// ---------------------------------------------------------------------------
// Phase 6 Batch 1: ConnectionState serialization
// ---------------------------------------------------------------------------

#[test]
fn models_connection_state_disconnected_serializes() {
    let state = ConnectionState::Disconnected;
    let json = serde_json::to_string(&state).expect("should serialize");
    assert_eq!(json, r#"{"type":"Disconnected"}"#);
}

#[test]
fn models_connection_state_connecting_serializes() {
    let state = ConnectionState::Connecting;
    let json = serde_json::to_string(&state).expect("should serialize");
    assert_eq!(json, r#"{"type":"Connecting"}"#);
}

#[test]
fn models_connection_state_connected_serializes() {
    let state = ConnectionState::Connected {
        browser_version: "Chrome/90.0.0.0".to_string(),
    };
    let json = serde_json::to_string(&state).expect("should serialize");
    assert_eq!(
        json,
        r#"{"type":"Connected","browser_version":"Chrome/90.0.0.0"}"#
    );
}

#[test]
fn models_connection_state_reconnecting_serializes() {
    let state = ConnectionState::Reconnecting { attempt: 3 };
    let json = serde_json::to_string(&state).expect("should serialize");
    assert_eq!(json, r#"{"type":"Reconnecting","attempt":3}"#);
}

#[test]
fn models_connection_state_failed_serializes() {
    let state = ConnectionState::Failed {
        reason: "Connection refused".to_string(),
    };
    let json = serde_json::to_string(&state).expect("should serialize");
    assert_eq!(json, r#"{"type":"Failed","reason":"Connection refused"}"#);
}

#[test]
fn models_connection_state_round_trip() {
    let states = vec![
        ConnectionState::Disconnected,
        ConnectionState::Connecting,
        ConnectionState::Connected {
            browser_version: "Chrome/90".to_string(),
        },
        ConnectionState::Reconnecting { attempt: 2 },
        ConnectionState::Failed {
            reason: "timeout".to_string(),
        },
    ];
    for state in states {
        let json = serde_json::to_string(&state).expect("should serialize");
        let deserialized: ConnectionState =
            serde_json::from_str(&json).expect("should deserialize");
        let json2 = serde_json::to_string(&deserialized).expect("should re-serialize");
        assert_eq!(
            json, json2,
            "ConnectionState round-trip failed for {:?}",
            state
        );
    }
}

// ---------------------------------------------------------------------------
// Phase 6 Batch 1: AppConfig and related model serialization
// ---------------------------------------------------------------------------

#[test]
fn app_config_serializes_correctly() {
    let config = AppConfig {
        cdp_port: 9222,
        storage_root: "~/EGrab".to_string(),
        image_concurrency: 3,
        browser_launch_commands: vec![BrowserLaunchCommand {
            os: BrowserOs::Macos,
            browser: BrowserType::Chrome,
            command: r#"open -a "Google Chrome" --args --remote-debugging-port=9222"#.to_string(),
        }],
    };
    let json = serde_json::to_string(&config).expect("AppConfig should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["cdp_port"]
            .as_u64()
            .expect("cdp_port should be number"),
        9222
    );
    assert_eq!(
        value["storage_root"]
            .as_str()
            .expect("storage_root should be string"),
        "~/EGrab"
    );
    assert_eq!(
        value["image_concurrency"]
            .as_u64()
            .expect("image_concurrency should be number"),
        3
    );
    assert!(
        value["browser_launch_commands"].is_array(),
        "browser_launch_commands should be array"
    );

    let deserialized: AppConfig = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.cdp_port, config.cdp_port);
    assert_eq!(deserialized.storage_root, config.storage_root);
    assert_eq!(deserialized.image_concurrency, config.image_concurrency);
    assert_eq!(
        deserialized.browser_launch_commands.len(),
        config.browser_launch_commands.len()
    );
}

#[test]
fn browser_launch_command_serializes_correctly() {
    let cmd = BrowserLaunchCommand {
        os: BrowserOs::Windows,
        browser: BrowserType::Edge,
        command: "msedge.exe --remote-debugging-port=9222".to_string(),
    };
    let json = serde_json::to_string(&cmd).expect("BrowserLaunchCommand should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["os"].as_str().expect("os should be string"),
        "windows"
    );
    assert_eq!(
        value["browser"].as_str().expect("browser should be string"),
        "edge"
    );
    assert_eq!(
        value["command"].as_str().expect("command should be string"),
        "msedge.exe --remote-debugging-port=9222"
    );

    let deserialized: BrowserLaunchCommand =
        serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.command, cmd.command);
}

#[test]
fn meta_json_document_serializes_correctly() {
    let doc = MetaJsonDocument {
        version: "1.0.0".to_string(),
        platform: "taobao".to_string(),
        item_id: "123456".to_string(),
        scraped_at: "2026-05-10T12:00:00Z".to_string(),
        data: create_test_product(),
    };
    let json = serde_json::to_string(&doc).expect("MetaJsonDocument should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["version"].as_str().expect("version should be string"),
        "1.0.0"
    );
    assert_eq!(
        value["platform"]
            .as_str()
            .expect("platform should be string"),
        "taobao"
    );
    assert_eq!(
        value["item_id"].as_str().expect("item_id should be string"),
        "123456"
    );
    assert!(
        value["data"].is_object(),
        "data should be object (ProductData)"
    );

    let deserialized: MetaJsonDocument = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.version, doc.version);
    assert_eq!(deserialized.item_id, doc.item_id);
    assert_eq!(deserialized.platform, doc.platform);
}

#[test]
fn raw_json_document_serializes_correctly() {
    let doc = RawJsonDocument {
        version: "1.0.0".to_string(),
        platform: "jd".to_string(),
        item_id: "789012".to_string(),
        scraped_at: "2026-05-10T12:00:00Z".to_string(),
        url: "https://item.jd.com/789012.html".to_string(),
        raw_data: serde_json::json!({"key": "value"}),
        parser_errors: vec![],
    };
    let json = serde_json::to_string(&doc).expect("RawJsonDocument should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["version"].as_str().expect("version should be string"),
        "1.0.0"
    );
    assert_eq!(
        value["platform"]
            .as_str()
            .expect("platform should be string"),
        "jd"
    );
    assert_eq!(
        value["url"].as_str().expect("url should be string"),
        "https://item.jd.com/789012.html"
    );
    assert!(value["raw_data"].is_object(), "raw_data should be object");

    let deserialized: RawJsonDocument = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.version, doc.version);
    assert_eq!(deserialized.url, doc.url);
}

// ---------------------------------------------------------------------------
// Phase 6 Batch 1: TaskFilter serialization
// ---------------------------------------------------------------------------

#[test]
fn task_filter_with_values_serializes() {
    let filter = TaskFilter {
        platform: Some("taobao".to_string()),
        status: Some(TaskStatus::Success),
        keyword: Some("phone".to_string()),
        item_id: Some("123".to_string()),
        start_time: Some("2026-01-01T00:00:00Z".to_string()),
        end_time: Some("2026-12-31T23:59:59Z".to_string()),
        limit: Some(10),
        offset: Some(0),
    };
    let json = serde_json::to_string(&filter).expect("TaskFilter should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["platform"]
            .as_str()
            .expect("platform should be string"),
        "taobao"
    );
    assert_eq!(
        value["status"].as_str().expect("status should be string"),
        "success"
    );
    assert_eq!(
        value["keyword"].as_str().expect("keyword should be string"),
        "phone"
    );
    assert_eq!(
        value["item_id"].as_str().expect("item_id should be string"),
        "123"
    );
    assert_eq!(
        value["start_time"]
            .as_str()
            .expect("start_time should be string"),
        "2026-01-01T00:00:00Z"
    );
    assert_eq!(
        value["end_time"]
            .as_str()
            .expect("end_time should be string"),
        "2026-12-31T23:59:59Z"
    );
    assert_eq!(value["limit"].as_i64().expect("limit should be number"), 10);
    assert_eq!(
        value["offset"].as_i64().expect("offset should be number"),
        0
    );

    let deserialized: TaskFilter = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.platform, filter.platform);
    assert!(
        matches!(
            (&deserialized.status, &filter.status),
            (Some(TaskStatus::Success), Some(TaskStatus::Success))
        ),
        "status should match"
    );
    assert_eq!(deserialized.limit, filter.limit);
    assert_eq!(deserialized.offset, filter.offset);
}

#[test]
fn task_filter_all_none_serializes() {
    let filter = TaskFilter {
        platform: None,
        status: None,
        keyword: None,
        item_id: None,
        start_time: None,
        end_time: None,
        limit: None,
        offset: None,
    };
    let json = serde_json::to_string(&filter).expect("TaskFilter should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert!(value["platform"].is_null(), "platform should be null");
    assert!(value["status"].is_null(), "status should be null");
    assert!(value["keyword"].is_null(), "keyword should be null");
    assert!(value["item_id"].is_null(), "item_id should be null");
    assert!(value["start_time"].is_null(), "start_time should be null");
    assert!(value["end_time"].is_null(), "end_time should be null");
    assert!(value["limit"].is_null(), "limit should be null");
    assert!(value["offset"].is_null(), "offset should be null");
}
