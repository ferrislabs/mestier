use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use mcp_server::{ApiClient, FilePolicy, router, router_with};
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
const RICH: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const PHOTOS2_SECTION: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";
const PLANS_SECTION: &str = "cccccccc-cccc-4ccc-8ccc-cccccccccccc";
const BLANK_SECTION: &str = "dddddddd-dddd-4ddd-8ddd-dddddddddddd";
const SUPPLY: &str = "eeeeeeee-eeee-4eee-8eee-eeeeeeeeeeee";
const UPDATED_AT: &str = "2026-10-10T08:00:00Z";
const JPEG: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0, 0x10, b'J', b'F', b'I', b'F'];

#[derive(Clone, Debug)]
struct Seen {
    method: String,
    path: String,
    query: String,
    content_type: String,
    size: usize,
    authorization: String,
    body: Value,
}

#[derive(Clone, Default)]
struct Fake {
    log: Arc<Mutex<Vec<Seen>>>,
    stale: Arc<AtomicUsize>,
}

impl Fake {
    fn seen(&self) -> Vec<Seen> {
        self.log.lock().unwrap().clone()
    }

    fn puts(&self) -> Vec<Seen> {
        self.seen()
            .into_iter()
            .filter(|s| s.method == "PUT")
            .collect()
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

fn rich_doe() -> Value {
    json!({
        "id": RICH, "project_id": PROJECT, "title": "Avenue", "place": null, "lot": null,
        "issued_on": "2026-10-07", "author": null, "revision": "A",
        "parties": [],
        "updated_at": UPDATED_AT,
        "sections": [
            {"id": SUPPLIES_SECTION, "title": "Mobilier", "enabled": true,
             "content": {"kind": "supplies", "maker_label": "manufacturer", "supplies": [{
                "id": SUPPLY, "source_product_id": null, "title": "Titre", "designation": "Banc",
                "dpgf_ref": "1.1", "destination": "Parvis",
                "supplier": {"name": "ACME", "address": "1 rue", "phone": "01", "email": null},
                "maker": "X", "specs": "s",
                "photos": [{"key": "doe/sp.jpg", "caption": "a"}],
                "datasheet_keys": ["doe/ds.pdf"]
             }]}},
            {"id": PHOTOS_SECTION, "title": "Photos", "enabled": true,
             "content": {"kind": "photos", "per_page": "two", "photos": [{"key": "doe/p1.jpg", "caption": "one"}]}},
            {"id": PHOTOS2_SECTION, "title": "Autres photos", "enabled": true,
             "content": {"kind": "photos", "per_page": "four", "photos": []}},
            {"id": PLANS_SECTION, "title": "Plans", "enabled": true,
             "content": {"kind": "plans", "plans": []}},
            {"id": BLANK_SECTION, "title": "Libre", "enabled": true,
             "content": {"kind": "blank_pages", "count": 2}}
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
    fake.log.lock().unwrap().push(Seen {
        method: parts.method.to_string(),
        path: path.clone(),
        query: parts.uri.query().unwrap_or("").to_owned(),
        content_type: parts
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_owned(),
        size: bytes.len(),
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
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/does/{RICH}") => {
            Json(json!({"data": rich_doe()})).into_response()
        }
        (&Method::PUT, p) if p == format!("organizations/{ORG_A}/does/{RICH}") => {
            let remaining = fake.stale.load(Ordering::SeqCst);
            if remaining > 0 {
                fake.stale.store(remaining - 1, Ordering::SeqCst);
                return fail(
                    StatusCode::CONFLICT,
                    "le DOE a été modifié entre-temps, rechargez-le",
                );
            }
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            Json(json!({"data": sent})).into_response()
        }
        (&Method::POST, "files") => (
            StatusCode::CREATED,
            Json(json!({"data": {"key": "doe/uploaded.bin", "mime_type": "x", "size_bytes": bytes.len()}})),
        )
            .into_response(),
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
    let add_file = body["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "add_doe_file")
        .unwrap();
    assert_eq!(add_file["_meta"]["openai/fileParams"], json!(["file"]));
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

async fn rich_call(mcp: &str, tool: &str, arguments: Value) -> Value {
    let mut arguments = arguments;
    arguments["doe_id"] = json!(RICH);
    call(mcp, "solo", tool, arguments).await
}

fn last_put(fake: &Fake) -> Value {
    fake.puts().last().unwrap().body.clone()
}

fn section_ids(body: &Value) -> Vec<String> {
    body["sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s["id"].as_str().unwrap().to_owned())
        .collect()
}

#[tokio::test]
async fn updating_a_supply_changes_only_the_given_fields() {
    let (mcp, fake) = start().await;
    let result = rich_call(
        &mcp,
        "update_doe_supply",
        json!({"supply_id": SUPPLY, "designation": "Banc 2", "dpgf_ref": "", "supplier_phone": ""}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let put = last_put(&fake);
    assert_eq!(put["expected_updated_at"], UPDATED_AT);
    let supply = &put["sections"][0]["content"]["supplies"][0];
    assert_eq!(supply["designation"], "Banc 2");
    assert_eq!(supply["dpgf_ref"], Value::Null);
    assert_eq!(supply["destination"], "Parvis");
    assert_eq!(supply["supplier"]["name"], "ACME");
    assert_eq!(supply["supplier"]["address"], "1 rue");
    assert_eq!(supply["supplier"]["phone"], Value::Null);
    assert_eq!(
        supply["photos"],
        rich_doe()["sections"][0]["content"]["supplies"][0]["photos"]
    );
}

#[tokio::test]
async fn an_empty_supplier_name_removes_the_supplier() {
    let (mcp, fake) = start().await;
    let result = rich_call(
        &mcp,
        "update_doe_supply",
        json!({"supply_id": SUPPLY, "supplier_name": ""}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert_eq!(
        last_put(&fake)["sections"][0]["content"]["supplies"][0]["supplier"],
        Value::Null
    );
}

#[tokio::test]
async fn a_blank_designation_is_refused_without_a_write() {
    let (mcp, fake) = start().await;
    let result = rich_call(
        &mcp,
        "update_doe_supply",
        json!({"supply_id": SUPPLY, "designation": " "}),
    )
    .await;
    assert_eq!(result["isError"], json!(true));
    assert!(fake.puts().is_empty());
}

#[tokio::test]
async fn deleting_a_supply_removes_it_and_an_unknown_id_writes_nothing() {
    let (mcp, fake) = start().await;
    let unknown = rich_call(
        &mcp,
        "delete_doe_supply",
        json!({"supply_id": "ffffffff-ffff-4fff-8fff-ffffffffffff"}),
    )
    .await;
    assert_eq!(unknown["isError"], json!(true));
    assert!(fake.puts().is_empty());
    let result = rich_call(&mcp, "delete_doe_supply", json!({"supply_id": SUPPLY})).await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert_eq!(
        last_put(&fake)["sections"][0]["content"]["supplies"],
        json!([])
    );
}

#[tokio::test]
async fn a_section_is_added_at_the_requested_position() {
    let (mcp, fake) = start().await;
    let result = rich_call(
        &mcp,
        "add_doe_section",
        json!({"title": "Chantier", "kind": "photos", "position": 2}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let put = last_put(&fake);
    let sections = put["sections"].as_array().unwrap();
    assert_eq!(sections.len(), 6);
    assert_eq!(sections[1]["title"], "Chantier");
    assert_eq!(sections[1]["enabled"], true);
    assert_eq!(
        sections[1]["content"],
        json!({"kind": "photos", "per_page": "two", "photos": []})
    );
    let id = sections[1]["id"].as_str().unwrap();
    assert_eq!(uuid::Uuid::parse_str(id).unwrap().get_version_num(), 7);
}

#[tokio::test]
async fn a_section_is_added_at_the_end_with_its_defaults() {
    let (mcp, fake) = start().await;
    for (kind, expected) in [
        (
            "supplies",
            json!({"kind": "supplies", "maker_label": "manufacturer", "supplies": []}),
        ),
        ("plans", json!({"kind": "plans", "plans": []})),
        ("blank_pages", json!({"kind": "blank_pages", "count": 1})),
    ] {
        let result = rich_call(&mcp, "add_doe_section", json!({"title": "N", "kind": kind})).await;
        assert_ne!(result["isError"], json!(true), "{result}");
        assert_eq!(last_put(&fake)["sections"][5]["content"], expected);
    }
}

#[tokio::test]
async fn a_setting_of_another_kind_is_refused() {
    let (mcp, fake) = start().await;
    let on_add = rich_call(
        &mcp,
        "add_doe_section",
        json!({"title": "N", "kind": "plans", "count": 3}),
    )
    .await;
    assert_eq!(on_add["isError"], json!(true));
    let on_update = rich_call(
        &mcp,
        "update_doe_section",
        json!({"section_id": PLANS_SECTION, "per_page": "four"}),
    )
    .await;
    assert_eq!(on_update["isError"], json!(true));
    let by_count = rich_call(
        &mcp,
        "update_doe_section",
        json!({"section_title": "mobilier", "count": 3}),
    )
    .await;
    assert_eq!(by_count["isError"], json!(true));
    assert!(fake.puts().is_empty());
}

#[tokio::test]
async fn a_section_is_renamed_disabled_and_tuned() {
    let (mcp, fake) = start().await;
    let result = rich_call(
        &mcp,
        "update_doe_section",
        json!({"section_title": "libre", "title": "Pages", "enabled": false, "count": 5}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let section = &last_put(&fake)["sections"][4];
    assert_eq!(section["title"], "Pages");
    assert_eq!(section["enabled"], false);
    assert_eq!(section["content"]["count"], 5);
    let nursery = rich_call(
        &mcp,
        "update_doe_section",
        json!({"section_id": SUPPLIES_SECTION, "maker_label": "nursery"}),
    )
    .await;
    assert_ne!(nursery["isError"], json!(true), "{nursery}");
    assert_eq!(
        last_put(&fake)["sections"][0]["content"]["maker_label"],
        "nursery"
    );
}

#[tokio::test]
async fn only_an_empty_section_can_be_deleted() {
    let (mcp, fake) = start().await;
    for id in [SUPPLIES_SECTION, PHOTOS_SECTION] {
        let result = rich_call(&mcp, "delete_doe_section", json!({"section_id": id})).await;
        assert_eq!(result["isError"], json!(true), "{id}");
        assert!(
            text(&result).contains("n'est pas vide"),
            "{}",
            text(&result)
        );
    }
    assert!(fake.puts().is_empty());
    for id in [PLANS_SECTION, PHOTOS2_SECTION, BLANK_SECTION] {
        let result = rich_call(&mcp, "delete_doe_section", json!({"section_id": id})).await;
        assert_ne!(result["isError"], json!(true), "{result}");
        assert!(!section_ids(&last_put(&fake)).contains(&id.to_owned()));
    }
}

#[tokio::test]
async fn reordering_needs_every_section_exactly_once() {
    let (mcp, fake) = start().await;
    let incomplete = rich_call(
        &mcp,
        "reorder_doe_sections",
        json!({"section_ids": [PHOTOS_SECTION, SUPPLIES_SECTION]}),
    )
    .await;
    assert_eq!(incomplete["isError"], json!(true));
    let duplicate = rich_call(
        &mcp,
        "reorder_doe_sections",
        json!({"section_ids": [PHOTOS_SECTION, PHOTOS_SECTION, PHOTOS2_SECTION, PLANS_SECTION, BLANK_SECTION]}),
    )
    .await;
    assert_eq!(duplicate["isError"], json!(true));
    let unknown = rich_call(
        &mcp,
        "reorder_doe_sections",
        json!({"section_ids": [PHOTOS_SECTION, PHOTOS2_SECTION, PLANS_SECTION, BLANK_SECTION, DOE]}),
    )
    .await;
    assert_eq!(unknown["isError"], json!(true));
    assert!(fake.puts().is_empty());
    let order = [
        BLANK_SECTION,
        PLANS_SECTION,
        PHOTOS2_SECTION,
        PHOTOS_SECTION,
        SUPPLIES_SECTION,
    ];
    let result = rich_call(&mcp, "reorder_doe_sections", json!({"section_ids": order})).await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert_eq!(section_ids(&last_put(&fake)), order);
}

#[tokio::test]
async fn a_stale_write_is_retried_once_after_a_fresh_read() {
    let (mcp, fake) = start().await;
    fake.stale.store(1, Ordering::SeqCst);
    let result = rich_call(&mcp, "delete_doe_supply", json!({"supply_id": SUPPLY})).await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert_eq!(fake.puts().len(), 2);
    let gets = fake
        .seen()
        .iter()
        .filter(|s| s.method == "GET" && s.path.ends_with(RICH))
        .count();
    assert_eq!(gets, 2);
}

#[tokio::test]
async fn a_second_stale_write_is_an_error_with_the_api_message() {
    let (mcp, fake) = start().await;
    fake.stale.store(2, Ordering::SeqCst);
    let result = rich_call(&mcp, "update_doe_header", json!({"title": "Neuf"})).await;
    assert_eq!(result["isError"], json!(true));
    assert!(text(&result).contains("modifié entre-temps"));
    assert_eq!(fake.puts().len(), 2);
}

struct Files {
    mcp: String,
    fake: Fake,
    host: String,
}

async fn download_server() -> String {
    use axum::{response::Redirect, routing::get};
    let app = Router::new()
        .route(
            "/ok.jpg",
            get(|| async { ([(header::CONTENT_TYPE, "image/jpeg")], JPEG.to_vec()) }),
        )
        .route(
            "/fake.jpg",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "image/jpeg")],
                    b"<html>nope</html>".to_vec(),
                )
            }),
        )
        .route(
            "/doc.pdf",
            get(|| async {
                (
                    [(header::CONTENT_TYPE, "application/pdf")],
                    b"%PDF-1.4 x".to_vec(),
                )
            }),
        )
        .route(
            "/redirect",
            get(|| async { Redirect::temporary("/ok.jpg") }),
        )
        .route(
            "/big.jpg",
            get(|| async {
                let mut bytes = JPEG.to_vec();
                bytes.resize(11 * 1024 * 1024, 0);
                ([(header::CONTENT_TYPE, "image/jpeg")], bytes)
            }),
        );
    serve(app).await
}

async fn start_files(policy: FilePolicy) -> Files {
    let fake = Fake::default();
    let api = serve(Router::new().fallback(fake_api).with_state(fake.clone())).await;
    let mcp = serve(router_with(ApiClient::new(api), policy)).await;
    Files {
        mcp,
        fake,
        host: download_server().await,
    }
}

fn local_policy() -> FilePolicy {
    FilePolicy::new(["127.0.0.1".to_owned()]).allow_http_for("127.0.0.1")
}

fn file_arg(host: &str, path: &str) -> Value {
    json!({"download_url": format!("{host}{path}"), "file_id": "file_1"})
}

async fn assert_refused(files: &Files, path_or_url: &str, target: &str, expected: &str) {
    let url = if path_or_url.starts_with("http") {
        path_or_url.to_owned()
    } else {
        format!("{}{path_or_url}", files.host)
    };
    let result = rich_call(
        &files.mcp,
        "add_doe_file",
        json!({"target": target, "section_id": PHOTOS_SECTION, "supply_id": SUPPLY, "file": {"download_url": url, "file_id": "f"}}),
    )
    .await;
    assert_eq!(result["isError"], json!(true), "{url}: {result}");
    assert!(text(&result).contains(expected), "{url}: {}", text(&result));
    let seen = files.fake.seen();
    assert!(
        seen.iter().all(|s| s.method != "POST" && s.method != "PUT"),
        "{url}"
    );
}

#[tokio::test]
async fn a_chatgpt_file_param_is_downloaded_uploaded_and_attached() {
    let files = start_files(local_policy()).await;
    let result = rich_call(
        &files.mcp,
        "add_doe_file",
        json!({
            "target": "photo",
            "section_title": "photos",
            "caption": "Vue du parvis",
            "file": file_arg(&files.host, "/ok.jpg"),
        }),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let upload = files
        .fake
        .seen()
        .into_iter()
        .find(|s| s.method == "POST")
        .unwrap();
    assert_eq!(upload.path, "/api/v1/files");
    assert_eq!(upload.query, "folder=doe");
    assert_eq!(upload.content_type, "image/jpeg");
    assert_eq!(upload.size, JPEG.len());
    let photos = &last_put(&files.fake)["sections"][1]["content"]["photos"];
    assert_eq!(
        photos[1],
        json!({"key": "doe/uploaded.bin", "caption": "Vue du parvis"})
    );
    assert_eq!(photos[0]["key"], "doe/p1.jpg");
}

#[tokio::test]
async fn a_file_that_cannot_be_downloaded_safely_is_refused_before_any_upload() {
    let files = start_files(local_policy()).await;
    assert_refused(
        &files,
        "https://localhost:1/ok.jpg",
        "photo",
        "pas autorisé",
    )
    .await;
    assert_refused(&files, "/redirect", "photo", "redirige").await;
    assert_refused(&files, "/big.jpg", "photo", "trop volumineux").await;
    assert_refused(&files, "/fake.jpg", "photo", "fichier refusé").await;
    assert_refused(&files, "/doc.pdf", "photo", "PDF").await;
    assert_refused(&files, "/doc.pdf", "supply_photo", "PDF").await;
}

#[tokio::test]
async fn plain_http_is_refused_by_default() {
    let files = start_files(FilePolicy::new(["127.0.0.1".to_owned()])).await;
    assert_refused(&files, "/ok.jpg", "photo", "https").await;
}

#[tokio::test]
async fn the_default_allow_list_refuses_foreign_hosts() {
    let files = start_files(FilePolicy::default()).await;
    assert_refused(
        &files,
        "https://files.evil-openai.com/x.jpg",
        "photo",
        "pas autorisé",
    )
    .await;
}

#[tokio::test]
async fn a_call_without_a_file_asks_for_it() {
    let (mcp, fake) = start().await;
    let result = rich_call(&mcp, "add_doe_file", json!({"target": "photo"})).await;
    assert_eq!(result["isError"], json!(true));
    assert!(text(&result).contains("joins le fichier à ta demande"));
    assert!(
        fake.seen()
            .iter()
            .all(|s| s.method != "POST" && s.method != "PUT")
    );
}

#[tokio::test]
async fn a_base64_pdf_becomes_a_supply_datasheet() {
    use base64::Engine;
    let (mcp, fake) = start().await;
    let content = base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.4 fiche");
    let result = rich_call(
        &mcp,
        "add_doe_file",
        json!({"target": "supply_datasheet", "supply_id": SUPPLY, "content_base64": content, "file_name": "fiche.pdf", "mime_type": "text/plain"}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let upload = fake
        .seen()
        .into_iter()
        .find(|s| s.method == "POST")
        .unwrap();
    assert_eq!(upload.content_type, "application/pdf");
    assert_eq!(
        last_put(&fake)["sections"][0]["content"]["supplies"][0]["datasheet_keys"],
        json!(["doe/ds.pdf", "doe/uploaded.bin"])
    );
}

#[tokio::test]
async fn a_base64_over_five_megabytes_is_refused() {
    use base64::Engine;
    let (mcp, fake) = start().await;
    let mut bytes = JPEG.to_vec();
    bytes.resize(5 * 1024 * 1024 + 1, 0);
    let content = base64::engine::general_purpose::STANDARD.encode(bytes);
    let result = rich_call(
        &mcp,
        "add_doe_file",
        json!({"target": "photo", "content_base64": content}),
    )
    .await;
    assert_eq!(result["isError"], json!(true));
    assert!(fake.seen().iter().all(|s| s.method != "POST"));
}

#[tokio::test]
async fn a_plan_goes_to_the_single_plans_section() {
    use base64::Engine;
    let (mcp, fake) = start().await;
    let content = base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.4 plan");
    let result = rich_call(
        &mcp,
        "add_doe_file",
        json!({"target": "plan", "caption": "Récolement", "content_base64": content}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert_eq!(
        last_put(&fake)["sections"][3]["content"]["plans"],
        json!([{"key": "doe/uploaded.bin", "caption": "Récolement"}])
    );
}

#[tokio::test]
async fn a_caption_is_changed_and_a_file_is_moved() {
    let (mcp, fake) = start().await;
    let result = rich_call(
        &mcp,
        "update_doe_file",
        json!({"key": "doe/p1.jpg", "caption": "Nouvelle"}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert_eq!(
        last_put(&fake)["sections"][1]["content"]["photos"][0]["caption"],
        "Nouvelle"
    );
    let moved = rich_call(
        &mcp,
        "update_doe_file",
        json!({"key": "doe/p1.jpg", "section_id": PHOTOS2_SECTION}),
    )
    .await;
    assert_ne!(moved["isError"], json!(true), "{moved}");
    let put = last_put(&fake);
    assert_eq!(put["sections"][1]["content"]["photos"], json!([]));
    assert_eq!(
        put["sections"][2]["content"]["photos"][0]["key"],
        "doe/p1.jpg"
    );
    let wrong_kind = rich_call(
        &mcp,
        "update_doe_file",
        json!({"key": "doe/p1.jpg", "section_id": PLANS_SECTION}),
    )
    .await;
    assert_eq!(wrong_kind["isError"], json!(true));
    let supply_photo = rich_call(
        &mcp,
        "update_doe_file",
        json!({"key": "doe/sp.jpg", "caption": "b"}),
    )
    .await;
    assert_ne!(supply_photo["isError"], json!(true), "{supply_photo}");
    let datasheet = rich_call(
        &mcp,
        "update_doe_file",
        json!({"key": "doe/ds.pdf", "caption": "b"}),
    )
    .await;
    assert_eq!(datasheet["isError"], json!(true));
}

#[tokio::test]
async fn deleting_a_file_removes_its_key_everywhere_and_an_unknown_key_fails() {
    let (mcp, fake) = start().await;
    for key in ["doe/p1.jpg", "doe/sp.jpg", "doe/ds.pdf"] {
        let result = rich_call(&mcp, "delete_doe_file", json!({"key": key})).await;
        assert_ne!(result["isError"], json!(true), "{key}: {result}");
        assert!(!last_put(&fake).to_string().contains(key), "{key}");
    }
    let before = fake.puts().len();
    let unknown = rich_call(&mcp, "delete_doe_file", json!({"key": "doe/none.jpg"})).await;
    assert_eq!(unknown["isError"], json!(true));
    assert_eq!(fake.puts().len(), before);
}
