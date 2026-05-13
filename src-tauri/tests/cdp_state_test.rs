// EGrab - CDP State Machine Serialization Tests
// Verifies ConnectionState, ConnectionInfo, TabInfo, CdpEndpoint serialization
// aligns with frontend TypeScript types (src/protocols/data-models.ts).

use egrab::models::{CdpEndpoint, ConnectionInfo, ConnectionState, TabInfo};

// ---------------------------------------------------------------------------
// ConnectionState tagged-enum serialization (tag = "type", PascalCase)
// ---------------------------------------------------------------------------

#[test]
fn cdp_connection_state_disconnected_serializes() {
    let state = ConnectionState::Disconnected;
    let json = serde_json::to_string(&state).expect("should serialize");
    assert_eq!(json, r#"{"type":"Disconnected"}"#);
}

#[test]
fn cdp_connection_state_connecting_serializes() {
    let state = ConnectionState::Connecting;
    let json = serde_json::to_string(&state).expect("should serialize");
    assert_eq!(json, r#"{"type":"Connecting"}"#);
}

#[test]
fn cdp_connection_state_connected_serializes() {
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
fn cdp_connection_state_reconnecting_serializes() {
    let state = ConnectionState::Reconnecting { attempt: 3 };
    let json = serde_json::to_string(&state).expect("should serialize");
    assert_eq!(json, r#"{"type":"Reconnecting","attempt":3}"#);
}

#[test]
fn cdp_connection_state_failed_serializes() {
    let state = ConnectionState::Failed {
        reason: "Connection refused".to_string(),
    };
    let json = serde_json::to_string(&state).expect("should serialize");
    assert_eq!(json, r#"{"type":"Failed","reason":"Connection refused"}"#);
}

#[test]
fn cdp_connection_state_deserializes_all_variants() {
    let disconnected: ConnectionState =
        serde_json::from_str(r#"{"type":"Disconnected"}"#).expect("should deserialize");
    assert!(matches!(disconnected, ConnectionState::Disconnected));

    let connecting: ConnectionState =
        serde_json::from_str(r#"{"type":"Connecting"}"#).expect("should deserialize");
    assert!(matches!(connecting, ConnectionState::Connecting));

    let connected: ConnectionState =
        serde_json::from_str(r#"{"type":"Connected","browser_version":"Chrome/90"}"#)
            .expect("should deserialize");
    assert!(matches!(connected, ConnectionState::Connected { .. }));

    let reconnecting: ConnectionState =
        serde_json::from_str(r#"{"type":"Reconnecting","attempt":2}"#).expect("should deserialize");
    assert!(matches!(reconnecting, ConnectionState::Reconnecting { .. }));

    let failed: ConnectionState = serde_json::from_str(r#"{"type":"Failed","reason":"timeout"}"#)
        .expect("should deserialize");
    assert!(matches!(failed, ConnectionState::Failed { .. }));
}

#[test]
fn cdp_connection_state_round_trip_all_variants() {
    let states = vec![
        ConnectionState::Disconnected,
        ConnectionState::Connecting,
        ConnectionState::Connected {
            browser_version: "Chrome/90".to_string(),
        },
        ConnectionState::Reconnecting { attempt: 1 },
        ConnectionState::Reconnecting { attempt: 3 },
        ConnectionState::Failed {
            reason: "timeout".to_string(),
        },
    ];
    for original in states {
        let json = serde_json::to_string(&original).expect("should serialize");
        let deserialized: ConnectionState =
            serde_json::from_str(&json).expect("should deserialize");
        let json2 = serde_json::to_string(&deserialized).expect("should re-serialize");
        assert_eq!(
            json, json2,
            "ConnectionState round-trip failed. Original JSON: {}",
            json
        );
    }
}

// ---------------------------------------------------------------------------
// ConnectionInfo serialization
// ---------------------------------------------------------------------------

#[test]
fn cdp_connection_info_serializes_correctly() {
    let info = ConnectionInfo {
        port: 9222,
        endpoint: "ws://127.0.0.1:9222/devtools/browser/abc".to_string(),
        browser_version: "Chrome/90.0.0.0".to_string(),
        state: ConnectionState::Connected {
            browser_version: "Chrome/90.0.0.0".to_string(),
        },
    };
    let json = serde_json::to_string(&info).expect("ConnectionInfo should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(value["port"].as_u64().expect("port should be number"), 9222);
    assert_eq!(
        value["endpoint"]
            .as_str()
            .expect("endpoint should be string"),
        "ws://127.0.0.1:9222/devtools/browser/abc"
    );
    assert_eq!(
        value["browser_version"]
            .as_str()
            .expect("browser_version should be string"),
        "Chrome/90.0.0.0"
    );
    assert!(value["state"].is_object(), "state should be object");
    assert_eq!(
        value["state"]["type"]
            .as_str()
            .expect("state.type should be string"),
        "Connected"
    );
    assert_eq!(
        value["state"]["browser_version"]
            .as_str()
            .expect("state.browser_version should be string"),
        "Chrome/90.0.0.0"
    );

    let deserialized: ConnectionInfo = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.port, info.port);
    assert_eq!(deserialized.endpoint, info.endpoint);
    assert_eq!(deserialized.browser_version, info.browser_version);
}

// ---------------------------------------------------------------------------
// TabInfo serialization (type field rename)
// ---------------------------------------------------------------------------

#[test]
fn cdp_tab_info_serializes_with_type_field() {
    let tab = TabInfo {
        id: "ABC123".to_string(),
        title: "Test Page".to_string(),
        url: "https://example.com".to_string(),
        tab_type: "page".to_string(),
    };
    let json = serde_json::to_string(&tab).expect("TabInfo should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(value["id"].as_str().expect("id should be string"), "ABC123");
    assert_eq!(
        value["title"].as_str().expect("title should be string"),
        "Test Page"
    );
    assert_eq!(
        value["url"].as_str().expect("url should be string"),
        "https://example.com"
    );
    assert_eq!(
        value["type"].as_str().expect("type should be string"),
        "page"
    );

    // Ensure the Rust field name tab_type does NOT appear in JSON
    assert!(
        value.get("tab_type").is_none(),
        "tab_type should be renamed to type"
    );

    let deserialized: TabInfo = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.tab_type, tab.tab_type);
    assert_eq!(deserialized.id, tab.id);
    assert_eq!(deserialized.title, tab.title);
    assert_eq!(deserialized.url, tab.url);
}

#[test]
fn cdp_tab_info_deserializes_with_type_field() {
    let json = r#"{"id":"TAB1","title":"Home","url":"https://home.com","type":"page"}"#;
    let tab: TabInfo = serde_json::from_str(json).expect("should deserialize");
    assert_eq!(tab.id, "TAB1");
    assert_eq!(tab.title, "Home");
    assert_eq!(tab.url, "https://home.com");
    assert_eq!(tab.tab_type, "page");
}

// ---------------------------------------------------------------------------
// CdpEndpoint serialization
// ---------------------------------------------------------------------------

#[test]
fn cdp_endpoint_serializes_correctly() {
    let endpoint = CdpEndpoint {
        port: 9222,
        endpoint: "ws://127.0.0.1:9222/devtools/browser/abc".to_string(),
        browser_version: Some("Chrome/90.0.0.0".to_string()),
    };
    let json = serde_json::to_string(&endpoint).expect("CdpEndpoint should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert_eq!(value["port"].as_u64().expect("port should be number"), 9222);
    assert_eq!(
        value["endpoint"]
            .as_str()
            .expect("endpoint should be string"),
        "ws://127.0.0.1:9222/devtools/browser/abc"
    );
    assert_eq!(
        value["browser_version"]
            .as_str()
            .expect("browser_version should be string"),
        "Chrome/90.0.0.0"
    );

    let deserialized: CdpEndpoint = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.port, endpoint.port);
    assert_eq!(deserialized.endpoint, endpoint.endpoint);
    assert_eq!(deserialized.browser_version, endpoint.browser_version);
}

#[test]
fn cdp_endpoint_without_browser_version_serializes() {
    let endpoint = CdpEndpoint {
        port: 9223,
        endpoint: "ws://127.0.0.1:9223/devtools/browser/def".to_string(),
        browser_version: None,
    };
    let json = serde_json::to_string(&endpoint).expect("CdpEndpoint should serialize");
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON should parse");

    assert!(
        value["browser_version"].is_null(),
        "browser_version should be null"
    );

    let deserialized: CdpEndpoint = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(deserialized.browser_version, None);
    assert_eq!(deserialized.port, endpoint.port);
}
