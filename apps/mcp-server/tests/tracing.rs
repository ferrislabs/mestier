use std::sync::{Arc, Mutex};

use args::{log::LogArgs, observability::ObservabilityArgs};
use axum::{Json, Router, extract::State, http::HeaderMap};
use mcp_server::{ApiClient, router};
use serde_json::{Value, json};

const TRACE_ID: &str = "0af7651916cd43dd8448eb211c80319c";

async fn serve(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

async fn record(State(seen): State<Arc<Mutex<Vec<String>>>>, headers: HeaderMap) -> Json<Value> {
    let traceparent = headers
        .get("traceparent")
        .and_then(|value| value.to_str().ok())
        .unwrap_or_default();
    seen.lock().unwrap().push(traceparent.to_owned());
    Json(json!({"data": []}))
}

/// The exporter connects lazily, so a collector that is not there only costs
/// a background warning; the propagation is what is under test.
fn enable_telemetry() {
    let endpoint = Some("http://127.0.0.1:1".to_owned());
    let observability = ObservabilityArgs {
        active_observability: true,
        otlp_endpoint: endpoint.clone(),
        metrics_endpoint: endpoint,
    };
    telemetry::init(&LogArgs::default(), "mcp-server-test", &observability).unwrap();
}

#[tokio::test]
async fn the_trace_of_a_request_follows_it_to_the_api() {
    enable_telemetry();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let api = serve(Router::new().fallback(record).with_state(seen.clone())).await;
    let mcp = serve(router(ApiClient::new(api))).await;

    reqwest::Client::new()
        .post(format!("{mcp}/mcp"))
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("traceparent", format!("00-{TRACE_ID}-b7ad6b7169203331-01"))
        .bearer_auth("delegated-token")
        .json(&json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": {"name": "list_organizations", "arguments": {}}
        }))
        .send()
        .await
        .unwrap();

    let seen = seen.lock().unwrap().clone();
    assert!(!seen.is_empty(), "the API was never called");
    assert!(
        seen.iter()
            .all(|header| header.starts_with(&format!("00-{TRACE_ID}-"))),
        "the API did not receive the incoming trace: {seen:?}"
    );
}
