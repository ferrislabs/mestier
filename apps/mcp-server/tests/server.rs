use std::sync::{Arc, Mutex};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use mcp_server::{ApiClient, router};
use mcp_tools::Tool;
use serde_json::{Value, json};

const ORG_A: &str = "11111111-1111-4111-8111-111111111111";
const ORG_B: &str = "22222222-2222-4222-8222-222222222222";
const DOE: &str = "33333333-3333-4333-8333-333333333333";
const BIG_DOE: &str = "44444444-4444-4444-8444-444444444444";
const PRODUCT: &str = "55555555-5555-4555-8555-555555555555";
const SUPPLIES_SECTION: &str = "66666666-6666-4666-8666-666666666666";
const PHOTOS_SECTION: &str = "77777777-7777-4777-8777-777777777777";
const PROJECT: &str = "88888888-8888-4888-8888-888888888888";
const MODEL: &str = "99999999-9999-4999-8999-999999999999";

#[derive(Clone, Debug)]
struct Seen {
    method: String,
    path: String,
    authorization: String,
    body: Value,
}

#[derive(Clone, Default)]
struct Fake(Arc<Mutex<Vec<Seen>>>);

impl Fake {
    fn seen(&self) -> Vec<Seen> {
        self.0.lock().unwrap().clone()
    }
}

fn org(id: &str, slug: &str) -> Value {
    json!({"id": id, "name": slug, "slug": slug})
}

fn doe() -> Value {
    json!({
        "id": DOE, "project_id": PROJECT, "title": "Avenue", "place": null, "lot": "Lot 3",
        "issued_on": "2026-10-07", "author": null, "revision": "A",
        "parties": [{"role": "maitre_ouvrage", "name": "Ville"}],
        "sections": [
            {"id": SUPPLIES_SECTION, "title": "Mobilier", "enabled": true,
             "content": {"kind": "supplies", "maker_label": "Fabricant", "supplies": [{"id": "a0000000-0000-4000-8000-000000000000", "designation": "Banc"}]}},
            {"id": PHOTOS_SECTION, "title": "Photos", "enabled": true,
             "content": {"kind": "photos", "per_page": 4, "photos": []}}
        ]
    })
}

fn fail(status: StatusCode, message: &str) -> Response {
    (
        status,
        Json(json!({"code": "x", "message": message, "status": status.as_u16()})),
    )
        .into_response()
}

async fn fake_api(State(fake): State<Fake>, request: Request) -> Response {
    let (parts, body) = request.into_parts();
    let bytes: Bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    let authorization = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let path = parts.uri.path().to_owned();
    fake.0.lock().unwrap().push(Seen {
        method: parts.method.to_string(),
        path: path.clone(),
        authorization: authorization.clone(),
        body: serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    });
    let token = authorization.trim_start_matches("Bearer ");
    if token == "expired" {
        return fail(StatusCode::UNAUTHORIZED, "token expiré");
    }
    let path = path.trim_start_matches("/api/v1/");
    match (&parts.method, path) {
        (&Method::GET, "users/@me/organizations") => {
            let data = if token == "solo" {
                vec![org(ORG_A, "acme")]
            } else {
                vec![org(ORG_A, "acme"), org(ORG_B, "beta")]
            };
            Json(json!({"data": data})).into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/does") => {
            Json(json!({"data": [{"id": DOE, "title": "Avenue"}], "meta": {"next_page": null}}))
                .into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_B}/projects") => fail(
            StatusCode::FORBIDDEN,
            "l'accès par IA est désactivé pour cette organisation",
        ),
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/does/{DOE}") => {
            Json(json!({"data": doe()})).into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/doe-models") => {
            Json(json!({"data": [{"id": MODEL, "name": "Standard", "archived_at": null}], "meta": {}}))
                .into_response()
        }
        (&Method::POST, p) if p == format!("organizations/{ORG_A}/projects/{PROJECT}/doe") => {
            (StatusCode::CREATED, Json(json!({"data": doe()}))).into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/products") => Json(json!({
            "data": [
                {"id": PRODUCT, "name": "Banc Garden", "sku": null, "manufacturer": null},
                {"id": "b0000000-0000-4000-8000-000000000000", "name": "Poubelle", "sku": null, "manufacturer": null}
            ],
            "meta": {"next_page": null}
        }))
        .into_response(),
        (&Method::GET, p) if p == format!("products/{PRODUCT}") => Json(json!({
            "data": {"id": PRODUCT, "organization_id": ORG_B, "name": "Banc Garden"}
        }))
        .into_response(),
        (&Method::POST, p) if p == format!("organizations/{ORG_A}/does/{DOE}/supplies/draft") => {
            Json(json!({"data": {"id": "c0000000-0000-4000-8000-000000000000", "designation": "Banc Garden", "dpgf_ref": null, "destination": null}}))
                .into_response()
        }
        (&Method::PUT, p) if p == format!("organizations/{ORG_A}/does/{DOE}") => {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            Json(json!({"data": sent})).into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/does/{DOE}/pdf") => (
            [(header::CONTENT_TYPE, "application/pdf")],
            b"%PDF-1.4 fake".to_vec(),
        )
            .into_response(),
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/does/{BIG_DOE}/pdf") => (
            [(header::CONTENT_TYPE, "application/pdf")],
            vec![b'x'; 11 * 1024 * 1024],
        )
            .into_response(),
        _ => fail(StatusCode::NOT_FOUND, "introuvable"),
    }
}

async fn serve(app: Router) -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    format!("http://{addr}")
}

async fn start() -> (String, Fake) {
    let fake = Fake::default();
    let api = serve(Router::new().fallback(fake_api).with_state(fake.clone())).await;
    let mcp = serve(router(ApiClient::new(api))).await;
    (mcp, fake)
}

async fn rpc(
    mcp: &str,
    token: Option<&str>,
    method: &str,
    params: Value,
) -> (StatusCode, HeaderMap, Value) {
    let mut request = reqwest::Client::new()
        .post(format!("{mcp}/mcp"))
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("x-mestier-mcp-client", "test")
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}));
    if let Some(token) = token {
        request = request.bearer_auth(token);
    }
    let response = request.send().await.unwrap();
    let status = response.status();
    let headers = response.headers().clone();
    let body = response.json().await.unwrap_or(Value::Null);
    (status, headers, body)
}

async fn call(mcp: &str, token: &str, tool: &str, arguments: Value) -> Value {
    let (status, _, body) = rpc(
        mcp,
        Some(token),
        "tools/call",
        json!({"name": tool, "arguments": arguments}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    body["result"].clone()
}

fn text(result: &Value) -> String {
    result["content"][0]["text"]
        .as_str()
        .unwrap_or_default()
        .to_owned()
}

#[tokio::test]
async fn the_tool_list_is_the_frozen_catalogue() {
    let (mcp, _) = start().await;
    let (status, _, body) = rpc(&mcp, Some("t"), "tools/list", json!({})).await;
    assert_eq!(status, StatusCode::OK, "{body}");
    let mut served: Vec<String> = body["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|tool| tool["name"].as_str().unwrap().to_owned())
        .collect();
    let mut expected: Vec<String> = Tool::ALL.iter().map(|t| t.name().to_owned()).collect();
    served.sort();
    expected.sort();
    assert_eq!(served, expected);
}

#[tokio::test]
async fn a_request_without_bearer_is_refused() {
    let (mcp, fake) = start().await;
    let (status, headers, _) = rpc(&mcp, None, "tools/list", json!({})).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(headers.contains_key(header::WWW_AUTHENTICATE));
    assert!(fake.seen().is_empty());
}

#[tokio::test]
async fn the_bearer_token_is_forwarded_to_the_api() {
    let (mcp, fake) = start().await;
    let result = call(&mcp, "delegated-token", "list_organizations", json!({})).await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert!(text(&result).contains("acme"));
    let seen = fake.seen();
    assert!(!seen.is_empty());
    assert!(
        seen.iter()
            .all(|s| s.authorization == "Bearer delegated-token")
    );
}

#[tokio::test]
async fn a_single_organization_is_the_default() {
    let (mcp, fake) = start().await;
    let result = call(&mcp, "solo", "list_does", json!({})).await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert!(text(&result).contains(DOE));
    assert!(
        fake.seen()
            .iter()
            .any(|s| s.path.ends_with(&format!("{ORG_A}/does")))
    );
}

#[tokio::test]
async fn several_organizations_without_a_slug_list_the_slugs() {
    let (mcp, _) = start().await;
    let result = call(&mcp, "two", "list_does", json!({})).await;
    assert_eq!(result["isError"], json!(true));
    let message = text(&result);
    assert!(
        message.contains("acme") && message.contains("beta"),
        "{message}"
    );
}

#[tokio::test]
async fn an_unknown_slug_is_an_error_without_data() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "two",
        "list_does",
        json!({"organization": "stranger"}),
    )
    .await;
    assert_eq!(result["isError"], json!(true));
    assert!(text(&result).contains("stranger"));
    assert!(!text(&result).contains(DOE));
    assert!(fake.seen().iter().all(|s| !s.path.contains("/does")));
}

#[tokio::test]
async fn an_api_403_becomes_a_readable_tool_error() {
    let (mcp, _) = start().await;
    let result = call(
        &mcp,
        "two",
        "list_projects",
        json!({"organization": "beta"}),
    )
    .await;
    assert_eq!(result["isError"], json!(true));
    assert_eq!(
        text(&result),
        "l'accès par IA est désactivé pour cette organisation"
    );
}

#[tokio::test]
async fn an_api_401_reads_as_an_expired_session() {
    let (mcp, _) = start().await;
    let result = call(&mcp, "expired", "list_organizations", json!({})).await;
    assert_eq!(result["isError"], json!(true));
    assert_eq!(text(&result), "session expirée");
}

#[tokio::test]
async fn adding_a_supply_appends_to_the_section_and_puts_the_whole_doe() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "add_doe_supply_from_product",
        json!({"doe_id": DOE, "section_title": "mobilier", "product_id": PRODUCT, "dpgf_ref": "1.2", "destination": "Parvis"}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let put = fake.seen().into_iter().find(|s| s.method == "PUT").unwrap();
    assert_eq!(put.body["title"], "Avenue");
    assert_eq!(put.body["parties"], doe()["parties"]);
    let sections = put.body["sections"].as_array().unwrap();
    assert_eq!(sections.len(), 2);
    let supplies = sections[0]["content"]["supplies"].as_array().unwrap();
    assert_eq!(supplies.len(), 2);
    assert_eq!(supplies[1]["designation"], "Banc Garden");
    assert_eq!(supplies[1]["dpgf_ref"], "1.2");
    assert_eq!(supplies[1]["destination"], "Parvis");
    assert_eq!(sections[1]["content"]["photos"], json!([]));
}

#[tokio::test]
async fn adding_a_supply_to_a_photos_section_is_refused() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "add_doe_supply_from_product",
        json!({"doe_id": DOE, "section_id": PHOTOS_SECTION, "product_id": PRODUCT}),
    )
    .await;
    assert_eq!(result["isError"], json!(true));
    assert!(fake.seen().iter().all(|s| s.method != "PUT"));
}

#[tokio::test]
async fn a_supply_typed_in_by_the_user_is_added_without_the_catalogue() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "add_doe_supply",
        json!({
            "doe_id": DOE,
            "section_title": "mobilier",
            "designation": "Banc Athène",
            "dpgf_ref": "MU3",
            "destination": "Voir plan de récolement",
            "supplier_name": "PLAS ECO",
            "supplier_phone": "02 31 26 01 30",
            "maker": "PLAS ECO",
            "specs": "Lo 180 cm\nColoris sable"
        }),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert!(
        fake.seen()
            .iter()
            .all(|s| !s.path.contains("supplies/draft")),
        "no catalogue product is involved"
    );
    let put = fake.seen().into_iter().find(|s| s.method == "PUT").unwrap();
    let supplies = put.body["sections"][0]["content"]["supplies"]
        .as_array()
        .unwrap();
    let added = supplies.last().unwrap();
    assert_eq!(added["designation"], "Banc Athène");
    assert_eq!(added["source_product_id"], Value::Null);
    assert_eq!(added["supplier"]["name"], "PLAS ECO");
    assert_eq!(added["supplier"]["phone"], "02 31 26 01 30");
    assert_eq!(added["specs"], "Lo 180 cm\nColoris sable");
    assert_eq!(added["photos"], json!([]));
    assert!(added["id"].as_str().is_some_and(|id| !id.is_empty()));
}

#[tokio::test]
async fn a_supply_without_designation_is_refused_before_any_write() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "add_doe_supply",
        json!({"doe_id": DOE, "section_title": "mobilier", "designation": "  "}),
    )
    .await;
    assert_eq!(result["isError"], json!(true));
    assert!(fake.seen().iter().all(|s| s.method != "PUT"));
}

#[tokio::test]
async fn updating_the_header_patches_only_the_given_fields() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "update_doe_header",
        json!({"doe_id": DOE, "title": "Nouveau", "lot": "", "revision": "B"}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let put = fake.seen().into_iter().find(|s| s.method == "PUT").unwrap();
    assert_eq!(put.body["title"], "Nouveau");
    assert_eq!(put.body["lot"], Value::Null);
    assert_eq!(put.body["revision"], "B");
    assert_eq!(put.body["sections"], doe()["sections"]);
}

#[tokio::test]
async fn creating_a_doe_defaults_to_the_only_model() {
    let (mcp, fake) = start().await;
    let result = call(&mcp, "solo", "create_doe", json!({"project_id": PROJECT})).await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let post = fake
        .seen()
        .into_iter()
        .find(|s| s.method == "POST")
        .unwrap();
    assert_eq!(post.body, json!({"model_id": MODEL}));
}

#[tokio::test]
async fn product_search_filters_by_name() {
    let (mcp, _) = start().await;
    let result = call(&mcp, "solo", "search_products", json!({"query": "banc"})).await;
    let found: Value = serde_json::from_str(&text(&result)).unwrap();
    assert_eq!(found.as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn a_product_of_another_organization_is_not_returned() {
    let (mcp, _) = start().await;
    let result = call(&mcp, "solo", "get_product", json!({"product_id": PRODUCT})).await;
    assert_eq!(result["isError"], json!(true));
    assert!(!text(&result).contains("Banc Garden"));
}

#[tokio::test]
async fn the_pdf_is_an_embedded_resource() {
    let (mcp, _) = start().await;
    let result = call(&mcp, "solo", "export_doe_pdf", json!({"doe_id": DOE})).await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let resource = &result["content"][0]["resource"];
    assert_eq!(resource["mimeType"], "application/pdf");
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(resource["blob"].as_str().unwrap())
        .unwrap();
    assert!(bytes.starts_with(b"%PDF"));
}

#[tokio::test]
async fn a_pdf_over_ten_megabytes_is_refused() {
    let (mcp, _) = start().await;
    let result = call(&mcp, "solo", "export_doe_pdf", json!({"doe_id": BIG_DOE})).await;
    assert_eq!(result["isError"], json!(true));
    assert!(text(&result).contains("10 Mo"));
}

#[tokio::test]
async fn the_2025_11_25_handshake_is_served() {
    let (mcp, _) = start().await;
    let (status, _, body) = rpc(
        &mcp,
        Some("t"),
        "initialize",
        json!({"protocolVersion": "2025-11-25", "capabilities": {}, "clientInfo": {"name": "t", "version": "1"}}),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{body}");
    assert_eq!(body["result"]["protocolVersion"], "2025-11-25");
    assert!(body["result"]["capabilities"]["tools"].is_object());
}
