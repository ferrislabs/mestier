use std::sync::{Arc, Mutex};

use axum::{Json, Router};
use mcp_server::{ApiClient, router};
use serde_json::{Value, json};
use tracing::{
    Subscriber,
    field::{Field, Visit},
    span::{Attributes, Id},
};
use tracing_subscriber::{Layer, layer::Context, layer::SubscriberExt, registry::LookupSpan};

#[derive(Debug, Clone)]
struct Seen {
    name: &'static str,
    tool: Option<String>,
    parent: Option<&'static str>,
}

#[derive(Default)]
struct Capture(Arc<Mutex<Vec<Seen>>>);

struct ToolField(Option<String>);

impl Visit for ToolField {
    fn record_str(&mut self, field: &Field, value: &str) {
        if field.name() == "tool" {
            self.0 = Some(value.to_owned());
        }
    }

    fn record_debug(&mut self, _: &Field, _: &dyn std::fmt::Debug) {}
}

impl<S: Subscriber + for<'a> LookupSpan<'a>> Layer<S> for Capture {
    fn on_new_span(&self, attrs: &Attributes<'_>, id: &Id, ctx: Context<'_, S>) {
        let mut tool = ToolField(None);
        attrs.record(&mut tool);
        let parent = ctx
            .span(id)
            .and_then(|span| span.parent())
            .map(|parent| parent.name());
        self.0.lock().unwrap().push(Seen {
            name: attrs.metadata().name(),
            tool: tool.0,
            parent,
        });
    }
}

async fn serve(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

#[tokio::test]
async fn a_tool_call_gets_its_own_span_around_the_api_call() {
    let seen = Arc::new(Mutex::new(Vec::new()));
    tracing::subscriber::set_global_default(
        tracing_subscriber::registry().with(Capture(seen.clone())),
    )
    .unwrap();
    let api = serve(Router::new().fallback(|| async { Json(json!({"data": []})) })).await;
    let mcp = serve(router(ApiClient::new(api))).await;

    reqwest::Client::new()
        .post(format!("{mcp}/mcp"))
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .bearer_auth("delegated-token")
        .json(&json!({
            "jsonrpc": "2.0", "id": 1, "method": "tools/call",
            "params": {"name": "list_organizations", "arguments": {}}
        }))
        .send()
        .await
        .unwrap()
        .json::<Value>()
        .await
        .unwrap();

    let seen = seen.lock().unwrap().clone();
    let tool = seen
        .iter()
        .find(|span| span.name == "mcp.tool")
        .unwrap_or_else(|| panic!("no tool span in {seen:?}"));
    assert_eq!(tool.tool.as_deref(), Some("list_organizations"));
    let api_call = seen
        .iter()
        .find(|span| span.name == "mestier_api.request")
        .unwrap_or_else(|| panic!("no API span in {seen:?}"));
    assert_eq!(api_call.parent, Some("mcp.tool"));
}
