// EGrab - Commands Parameter Validation Tests
// Tests URL validation logic, ErrorCode+IpcError combos, TaskStatus, ScrapeStep.

use egrab::models::{ErrorCode, IpcError, ScrapeStep, TaskStatus};

// ---------------------------------------------------------------------------
// start_scrape URL empty-string validation (mirrors scrape_commands logic)
// ---------------------------------------------------------------------------

/// Mirrors the URL validation logic in `scrape_commands::start_scrape`.
/// Returns an error if the URL is empty or whitespace-only.
fn validate_scrape_url(url: &str) -> Result<(), IpcError> {
    if url.trim().is_empty() {
        return Err(IpcError {
            code: ErrorCode::UrlInvalid,
            message: "URL must not be empty".to_string(),
            recoverable: true,
            step: Some(ScrapeStep::Connecting),
            details: None,
        });
    }
    Ok(())
}

#[test]
fn start_scrape_rejects_empty_url() {
    let result = validate_scrape_url("");
    assert!(result.is_err(), "empty URL should be rejected");
    let err = result.expect_err("should be error");
    assert!(
        matches!(err.code, ErrorCode::UrlInvalid),
        "error code should be UrlInvalid"
    );
    assert_eq!(err.message, "URL must not be empty");
    assert!(err.recoverable, "error should be recoverable");
    assert!(
        matches!(err.step, Some(ScrapeStep::Connecting)),
        "step should be Connecting"
    );
}

#[test]
fn start_scrape_rejects_whitespace_only_url() {
    let result = validate_scrape_url("   \t\n  ");
    assert!(result.is_err(), "whitespace-only URL should be rejected");
    let err = result.expect_err("should be error");
    assert!(
        matches!(err.code, ErrorCode::UrlInvalid),
        "error code should be UrlInvalid"
    );
    assert_eq!(err.message, "URL must not be empty");
}

#[test]
fn start_scrape_accepts_valid_url() {
    let result = validate_scrape_url("https://item.taobao.com/item.htm?id=123");
    assert!(result.is_ok(), "valid URL should be accepted");
}

// ---------------------------------------------------------------------------
// ErrorCode + IpcError combination validation
// ---------------------------------------------------------------------------

#[test]
fn error_code_ipc_error_combination_serializes() {
    let error = IpcError {
        code: ErrorCode::UrlInvalid,
        message: "URL must not be empty".to_string(),
        recoverable: true,
        step: Some(ScrapeStep::Connecting),
        details: Some(serde_json::json!({"url": ""})),
    };
    let json = serde_json::to_string(&error).expect("IpcError should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(
        value["code"].as_str().expect("code should be string"),
        "URL_INVALID"
    );
    assert_eq!(
        value["message"].as_str().expect("message should be string"),
        "URL must not be empty"
    );
    assert_eq!(
        value["recoverable"]
            .as_bool()
            .expect("recoverable should be boolean"),
        true
    );
    assert_eq!(
        value["step"].as_str().expect("step should be string"),
        "connecting"
    );
    assert!(value["details"].is_object(), "details should be object");

    let deserialized: IpcError = serde_json::from_str(&json).expect("should deserialize");
    assert!(
        matches!(deserialized.code, ErrorCode::UrlInvalid),
        "deserialized code should be UrlInvalid"
    );
    assert_eq!(deserialized.message, error.message);
    assert_eq!(deserialized.recoverable, error.recoverable);
}

// ---------------------------------------------------------------------------
// TaskStatus enum serialization (snake_case)
// ---------------------------------------------------------------------------

#[test]
fn task_status_all_variants_serialize_to_snake_case() {
    let cases = vec![
        (TaskStatus::Pending, "pending"),
        (TaskStatus::Running, "running"),
        (TaskStatus::Success, "success"),
        (TaskStatus::Failed, "failed"),
        (TaskStatus::Partial, "partial"),
        (TaskStatus::Cancelled, "cancelled"),
    ];
    for (variant, expected) in cases {
        let json = serde_json::to_string(&variant).expect("TaskStatus should serialize");
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "TaskStatus::{:?} should serialize as \"{}\"",
            variant,
            expected
        );
        let deserialized: TaskStatus = serde_json::from_str(&json).expect("should deserialize");
        let json2 = serde_json::to_string(&deserialized).expect("should re-serialize");
        assert_eq!(json, json2, "TaskStatus round-trip failed");
    }
}

// ---------------------------------------------------------------------------
// ScrapeStep enum serialization (snake_case)
// ---------------------------------------------------------------------------

#[test]
fn scrape_step_all_variants_serialize_to_snake_case() {
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
        let json = serde_json::to_string(&variant).expect("ScrapeStep should serialize");
        assert_eq!(
            json,
            format!("\"{}\"", expected),
            "ScrapeStep::{:?} should serialize as \"{}\"",
            variant,
            expected
        );
        let deserialized: ScrapeStep = serde_json::from_str(&json).expect("should deserialize");
        let json2 = serde_json::to_string(&deserialized).expect("should re-serialize");
        assert_eq!(json, json2, "ScrapeStep round-trip failed");
    }
}
