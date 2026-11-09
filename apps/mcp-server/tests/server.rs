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
const CONTRACT: &str = "f1111111-1111-4111-8111-111111111111";
const CUSTOMER: &str = "f2222222-2222-4222-8222-222222222222";
const SITE: &str = "f3333333-3333-4333-8333-333333333333";
const OWN_CUSTOMER: &str = "e1111111-1111-4111-8111-111111111111";
const FOREIGN_CUSTOMER: &str = "e2222222-2222-4222-8222-222222222222";
const OWN_CONTACT: &str = "e3333333-3333-4333-8333-333333333333";
const FOREIGN_CONTACT: &str = "e4444444-4444-4444-8444-444444444444";
const OWN_DEAL: &str = "e5555555-5555-4555-8555-555555555555";
const FOREIGN_DEAL: &str = "e6666666-6666-4666-8666-666666666666";
const OWN_QUOTE: &str = "d1111111-1111-4111-8111-111111111111";
const FOREIGN_QUOTE: &str = "d2222222-2222-4222-8222-222222222222";
const TASK: &str = "c1111111-1111-4111-8111-111111111111";
const OWN_SUPPLIER: &str = "b1111111-1111-4111-8111-111111111111";
const OWN_PRODUCT: &str = "b3333333-3333-4333-8333-333333333333";
const LIBRARY_TOTAL: usize = 60;
const FAILING_PHOTO: usize = 59;
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
    /// `cover_photo_key` of the rich DOE as the fake API returns it.
    cover: Arc<Mutex<Option<String>>>,
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

fn photo_id(index: usize) -> String {
    format!("00000000-0000-4000-8000-{index:012}")
}

fn library_photo(index: usize) -> Value {
    json!({
        "id": photo_id(index), "file_key": format!("library/{ORG_A}/{index}.jpg"),
        "thumbnail_key": format!("library/{ORG_A}/{index}.thumb.jpg"), "mime_type": "image/jpeg",
        "width": 2400, "height": 1600, "size_bytes": 812345,
        "caption": "old", "tags": ["a", "b"], "project_id": PROJECT,
        "taken_at": "2026-10-01T09:30:00Z", "created_at": UPDATED_AT, "updated_at": UPDATED_AT
    })
}

fn thumbnail(index: usize) -> Vec<u8> {
    let mut bytes = JPEG.to_vec();
    bytes.push(index as u8);
    bytes
}

fn library_index(path: &str, suffix: &str) -> Option<usize> {
    let prefix = format!("organizations/{ORG_A}/photo-library/photos/");
    let id = path.strip_prefix(&prefix)?.strip_suffix(suffix)?;
    let index = id.strip_prefix("00000000-0000-4000-8000-")?.parse().ok()?;
    (1..=LIBRARY_TOTAL).contains(&index).then_some(index)
}

fn query_value(query: &str, key: &str) -> Option<usize> {
    query
        .split('&')
        .find_map(|pair| pair.strip_prefix(&format!("{key}=")))
        .and_then(|value| value.parse().ok())
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
        (&Method::GET, p) if p == format!("customers/{OWN_CUSTOMER}") => Json(json!({"data": {
            "id": OWN_CUSTOMER, "organization_id": ORG_A, "status": "PROSPECT", "name": "Dupont",
            "registration_number": null, "phone": "0102030405", "email": null, "kind": "INDIVIDUAL",
            "website": null, "vat_number": null, "owner_member_id": null, "lead_source": null,
            "notes": "à rappeler", "payment_terms_days": 30, "billing_contact_id": null,
            "billing_address_id": null, "updated_at": UPDATED_AT
        }}))
        .into_response(),
        (&Method::GET, p) if p == format!("customers/{FOREIGN_CUSTOMER}") => Json(json!({"data": {
            "id": FOREIGN_CUSTOMER, "organization_id": ORG_B, "name": "Autre"
        }}))
        .into_response(),
        (&Method::GET, p) if p == format!("customers/{OWN_CUSTOMER}/contacts") => {
            Json(json!({"data": [{"id": OWN_CONTACT, "first_name": "Ana"}], "meta": {}})).into_response()
        }
        (&Method::PATCH, p) if p == format!("customers/{OWN_CUSTOMER}") => {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            Json(json!({"data": sent})).into_response()
        }
        (&Method::POST, p) if p == format!("customers/{OWN_CUSTOMER}/contacts") => {
            (StatusCode::CREATED, Json(json!({"data": {"id": OWN_CONTACT}}))).into_response()
        }
        (&Method::GET, p) if p == format!("customer-contacts/{OWN_CONTACT}") => Json(json!({"data": {
            "id": OWN_CONTACT, "customer_id": OWN_CUSTOMER, "first_name": "Ana", "last_name": "Roy",
            "role": null, "phone": null, "mobile": null, "email": null, "is_primary": false
        }}))
        .into_response(),
        (&Method::GET, p) if p == format!("customer-contacts/{FOREIGN_CONTACT}") => Json(json!({"data": {
            "id": FOREIGN_CONTACT, "customer_id": FOREIGN_CUSTOMER, "first_name": "Bob"
        }}))
        .into_response(),
        (&Method::PATCH, p) if p == format!("customer-contacts/{OWN_CONTACT}") => {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            Json(json!({"data": sent})).into_response()
        }
        (&Method::POST, p) if p == format!("organizations/{ORG_A}/customers") => {
            let mut sent: Value = serde_json::from_slice(&bytes).unwrap();
            sent["id"] = json!(OWN_CUSTOMER);
            (StatusCode::CREATED, Json(json!({"data": sent}))).into_response()
        }
        (&Method::POST, p) if p == format!("organizations/{ORG_A}/deals") => {
            let mut sent: Value = serde_json::from_slice(&bytes).unwrap();
            sent["id"] = json!(OWN_DEAL);
            (StatusCode::CREATED, Json(json!({"data": sent}))).into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/deals") => {
            Json(json!({"data": [], "meta": {"next_page": null}})).into_response()
        }
        (&Method::GET, p) if p == format!("deals/{OWN_DEAL}") => Json(json!({"data": {
            "id": OWN_DEAL, "organization_id": ORG_A, "customer_id": OWN_CUSTOMER, "title": "Terrasse",
            "amount_cents": 500000, "stage": "NEW", "expected_close_date": null,
            "owner_member_id": null, "source": null, "notes": null
        }}))
        .into_response(),
        (&Method::GET, p) if p == format!("deals/{FOREIGN_DEAL}") => Json(json!({"data": {
            "id": FOREIGN_DEAL, "organization_id": ORG_B, "title": "Autre"
        }}))
        .into_response(),
        (&Method::PATCH, p) if p == format!("deals/{OWN_DEAL}") => {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            Json(json!({"data": sent})).into_response()
        }
        (&Method::POST, p) if p == format!("deals/{OWN_DEAL}/stage") => {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            Json(json!({"data": sent})).into_response()
        }
        (&Method::GET, p) if p == format!("quotes/{OWN_QUOTE}") => Json(json!({"data": {
            "id": OWN_QUOTE, "organization_id": ORG_A, "title": "Terrasse",
            "customer_id": OWN_CUSTOMER, "customer_context_id": SITE, "deal_id": null,
            "status": "DRAFT", "valid_until": null, "deposit_percent": "30",
            "owner_member_id": null, "notes": null, "billing_contact_id": null,
            "billing_address_id": null, "net_cents": 1000, "reference": "D-1",
            "lines": [{
                "id": "d3333333-3333-4333-8333-333333333333", "organization_id": ORG_A,
                "quote_id": OWN_QUOTE, "service_rate_id": null, "product_id": null,
                "label": "Dalles", "quantity": "10", "unit": "m2", "unit_price_cents": 5000,
                "vat_rate_bp": 2000, "discount_percent": "0", "billing_frequency": "ONE_TIME",
                "net_cents": 50000, "notes": null, "photo_keys": []
            }]
        }}))
        .into_response(),
        (&Method::GET, p) if p == format!("quotes/{FOREIGN_QUOTE}") => Json(json!({"data": {
            "id": FOREIGN_QUOTE, "organization_id": ORG_B
        }}))
        .into_response(),
        (&Method::PATCH, p) if p == format!("quotes/{OWN_QUOTE}") || p == format!("quotes/{OWN_QUOTE}/status") => {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            Json(json!({"data": sent})).into_response()
        }
        (&Method::POST, p) if p == format!("quotes/{OWN_QUOTE}/revision") => {
            (StatusCode::CREATED, Json(json!({"data": {"id": FOREIGN_QUOTE}}))).into_response()
        }
        (&Method::POST, p) if p == format!("organizations/{ORG_A}/quotes") => {
            let mut sent: Value = serde_json::from_slice(&bytes).unwrap();
            sent["id"] = json!(OWN_QUOTE);
            (StatusCode::CREATED, Json(json!({"data": sent}))).into_response()
        }
        (&Method::GET, p) if p == format!("quotes/{OWN_QUOTE}/pdf") => (
            [(header::CONTENT_TYPE, "application/pdf")],
            b"%PDF-1.4 devis".to_vec(),
        )
            .into_response(),
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/tasks") => {
            Json(json!({"data": [], "meta": {"next_page": null}})).into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/planning") => {
            Json(json!({"data": {"resources": [], "entries": []}})).into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/projects/{PROJECT}") => {
            Json(json!({"data": {
                "id": PROJECT, "name": "Chantier", "customer_id": OWN_CUSTOMER,
                "customer_context_id": null, "quote_id": null, "archived_at": null
            }}))
            .into_response()
        }
        (&Method::PATCH, p) if p == format!("organizations/{ORG_A}/projects/{PROJECT}")
            || p == format!("organizations/{ORG_A}/tasks/{TASK}") =>
        {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            Json(json!({"data": sent})).into_response()
        }
        (&Method::POST, p) if p == format!("organizations/{ORG_A}/projects")
            || p == format!("organizations/{ORG_A}/tasks")
            || p == format!("organizations/{ORG_A}/tasks/{TASK}/comments") =>
        {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            (StatusCode::CREATED, Json(json!({"data": sent}))).into_response()
        }
        (&Method::GET, p) if p == format!("products/{OWN_PRODUCT}") => Json(json!({"data": {
            "id": OWN_PRODUCT, "organization_id": ORG_A, "name": "Banc", "sku": "B1", "unit": "u",
            "unit_price_cents": 10000, "default_vat_rate_bp": 2000, "description": null,
            "photo_keys": ["p/1.jpg"], "supplier_id": null, "manufacturer": "ACME",
            "technical_specs": null, "datasheet_keys": [], "category": "Mobilier",
            "billing_frequency": "ONE_TIME", "cost_cents": 6000, "archived_at": "2026-01-01T00:00:00Z",
            "usage_count": 3
        }}))
        .into_response(),
        (&Method::PATCH, p) if p == format!("products/{OWN_PRODUCT}") => {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            Json(json!({"data": sent})).into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/suppliers/{OWN_SUPPLIER}") => {
            Json(json!({"data": {
                "id": OWN_SUPPLIER, "organization_id": ORG_A, "name": "Brico", "registration_number": null,
                "vat_number": null, "address": null, "phone": "0400", "email": null, "iban": null,
                "payment_terms_days": 30, "default_category_id": null, "notes": null,
                "archived_at": null,
                "contacts": [{"id": "b2222222-2222-4222-8222-222222222222", "first_name": "Luc",
                              "last_name": "Roy", "role": "", "email": "", "phone": "", "mobile": ""}],
                "addresses": []
            }}))
            .into_response()
        }
        (&Method::PUT, p) if p == format!("organizations/{ORG_A}/suppliers/{OWN_SUPPLIER}") => {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            Json(json!({"data": sent})).into_response()
        }
        (&Method::POST, p) if p == format!("organizations/{ORG_A}/products")
            || p == format!("organizations/{ORG_A}/suppliers") =>
        {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            (StatusCode::CREATED, Json(json!({"data": sent}))).into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/invoices/outstanding")
            || p == format!("organizations/{ORG_A}/reporting/profitability")
            || p == format!("organizations/{ORG_A}/reporting/worked-hours") =>
        {
            Json(json!({"data": []})).into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/suppliers")
            || p == format!("organizations/{ORG_A}/supplier-invoices")
            || p == format!("organizations/{ORG_A}/expenses") =>
        {
            Json(json!({"data": [], "meta": {"next_page": null}})).into_response()
        }
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
            let mut doe = rich_doe();
            doe["cover_photo_key"] = json!(*fake.cover.lock().unwrap());
            Json(json!({"data": doe})).into_response()
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
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/maintenance/recap") => {
            Json(json!({"data": {"year": 2026, "week": 43, "groups": [{
                "town": "Vichy",
                "lines": [{"label": "Tonte", "contract_id": CONTRACT, "due": {"status": "OVERDUE"}}]
            }]}}))
            .into_response()
        }
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/maintenance/contracts/{CONTRACT}") => {
            Json(json!({"data": {
                "id": CONTRACT, "customer_id": CUSTOMER, "customer_context_id": SITE,
                "status": "DRAFT", "lines": [{"label": "Tonte"}]
            }}))
            .into_response()
        }
        (&Method::POST, p) if p == format!("organizations/{ORG_A}/maintenance/contracts") => {
            let mut sent: Value = serde_json::from_slice(&bytes).unwrap();
            sent["id"] = json!(CONTRACT);
            (StatusCode::CREATED, Json(json!({"data": sent}))).into_response()
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
        (&Method::GET, p) if p == format!("organizations/{ORG_A}/photo-library/photos") => {
            let query = parts.uri.query().unwrap_or("");
            let per_page = query_value(query, "per_page").unwrap_or(20);
            let page = query_value(query, "page").unwrap_or(1);
            let first = (page - 1) * per_page;
            let photos: Vec<Value> = (first..(first + per_page).min(LIBRARY_TOTAL))
                .map(|offset| library_photo(offset + 1))
                .collect();
            Json(json!({
                "data": photos,
                "pagination": {"per_page": per_page, "current_page": page, "total": LIBRARY_TOTAL}
            }))
            .into_response()
        }
        (&Method::GET, p) if library_index(p, "/thumbnail").is_some() => {
            let index = library_index(p, "/thumbnail").unwrap();
            ([(header::CONTENT_TYPE, "image/jpeg")], thumbnail(index)).into_response()
        }
        (&Method::GET, p) if library_index(p, "").is_some() => {
            Json(json!({"data": library_photo(library_index(p, "").unwrap())})).into_response()
        }
        (&Method::PATCH, p) if library_index(p, "") == Some(FAILING_PHOTO) => fail(
            StatusCode::UNPROCESSABLE_ENTITY,
            "étiquette invalide",
        ),
        (&Method::PATCH, p) if library_index(p, "").is_some() => {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            let mut photo = library_photo(library_index(p, "").unwrap());
            for (key, value) in sent.as_object().unwrap() {
                photo[key] = value.clone();
            }
            Json(json!({"data": photo})).into_response()
        }
        (&Method::POST, p) if p == format!("organizations/{ORG_A}/does/{DOE}/photos/place") => {
            let sent: Value = serde_json::from_slice(&bytes).unwrap();
            let results: Vec<Value> = sent["placements"]
                .as_array()
                .unwrap()
                .iter()
                .map(|placement| {
                    let id = placement["photo_id"].as_str().unwrap();
                    if id == photo_id(FAILING_PHOTO) {
                        json!({"photo_id": id, "status": "failed", "error": "photo introuvable"})
                    } else {
                        json!({"photo_id": id, "status": "placed", "key": format!("doe/copy-{id}.jpg")})
                    }
                })
                .collect();
            Json(json!({"doe": doe(), "results": results})).into_response()
        }
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

async fn start_with_cover(key: &str) -> (String, Fake) {
    let (mcp, fake) = start().await;
    *fake.cover.lock().unwrap() = Some(key.to_owned());
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
async fn spec_style_8_a_cover_photo_is_set_and_a_pdf_is_refused_for_it() {
    use base64::Engine;
    let (mcp, fake) = start().await;
    let pdf = base64::engine::general_purpose::STANDARD.encode(b"%PDF-1.4 cover");
    let refused = rich_call(
        &mcp,
        "add_doe_file",
        json!({"target": "cover_photo", "content_base64": pdf}),
    )
    .await;
    assert_eq!(refused["isError"], json!(true));
    assert!(fake.seen().iter().all(|s| s.method != "POST"));

    let jpeg = base64::engine::general_purpose::STANDARD.encode(JPEG);
    let result = rich_call(
        &mcp,
        "add_doe_file",
        json!({"target": "cover_photo", "content_base64": jpeg}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert_eq!(last_put(&fake)["cover_photo_key"], "doe/uploaded.bin");
}

#[tokio::test]
async fn spec_style_8_writes_keep_the_cover_photo_and_delete_can_remove_it() {
    let (mcp, fake) = start_with_cover("doe/cover.jpg").await;
    let result = rich_call(&mcp, "update_doe_header", json!({"title": "Nouveau"})).await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert_eq!(last_put(&fake)["cover_photo_key"], "doe/cover.jpg");

    let result = rich_call(&mcp, "delete_doe_file", json!({"key": "doe/cover.jpg"})).await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert_eq!(last_put(&fake)["cover_photo_key"], Value::Null);
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

fn draft_arguments() -> Value {
    json!({
        "customer_id": CUSTOMER,
        "customer_context_id": SITE,
        "reference": "C25029",
        "season": {"year": 2026, "first_week": 12, "last_week": 48},
        "amount_cents": 250000,
        "notes": "Feuille papier du conducteur",
        "lines": [
            {"label": "Tonte avec ramassage", "cadence": {"kind": "ROLLING", "every_days": 14},
             "minutes_per_intervention": 90},
            {"label": "Taille", "cadence": {"kind": "WINDOWS", "windows": [
                {"from_week": 14, "to_week": 17}, {"from_week": 44, "to_week": 47}]}},
            {"label": "Entretien T1", "cadence": {"kind": "ON_DEMAND"}}
        ]
    })
}

fn posts(fake: &Fake) -> Vec<Seen> {
    fake.seen()
        .into_iter()
        .filter(|s| s.method == "POST")
        .collect()
}

#[tokio::test]
async fn the_due_list_asks_the_recap_of_the_week() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "list_maintenance_due",
        json!({"week": "2026-W43"}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let recap: Value = serde_json::from_str(&text(&result)).unwrap();
    assert_eq!(recap["groups"][0]["lines"][0]["label"], "Tonte");
    let asked = fake
        .seen()
        .into_iter()
        .find(|s| s.path.ends_with("/maintenance/recap"))
        .unwrap();
    assert_eq!(asked.method, "GET");
    assert_eq!(asked.query, "week=2026-W43");
    assert_eq!(asked.authorization, "Bearer solo");
}

#[tokio::test]
async fn a_contract_is_read_by_id_and_a_malformed_id_asks_the_api_nothing() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "get_maintenance_contract",
        json!({"contract_id": CONTRACT}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let contract: Value = serde_json::from_str(&text(&result)).unwrap();
    assert_eq!(contract["customer_context_id"], SITE);

    let before = fake.seen().len();
    let refused = call(
        &mcp,
        "solo",
        "get_maintenance_contract",
        json!({"contract_id": "../../maintenance/grid"}),
    )
    .await;
    assert_eq!(refused["isError"], json!(true));
    assert_eq!(fake.seen().len(), before);
}

#[tokio::test]
async fn a_draft_is_posted_with_its_lines_and_the_status_draft() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "create_maintenance_contract_draft",
        draft_arguments(),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");

    let [post] = posts(&fake).try_into().expect("exactly one write");
    assert_eq!(
        post.path,
        format!("/api/v1/organizations/{ORG_A}/maintenance/contracts")
    );
    assert_eq!(post.body["status"], "DRAFT");
    assert_eq!(post.body["customer_id"], CUSTOMER);
    assert_eq!(post.body["customer_context_id"], SITE);
    assert_eq!(post.body["reference"], "C25029");
    assert_eq!(post.body["amount_cents"], 250000);
    assert_eq!(
        post.body["season"],
        json!({"year": 2026, "first_week": 12, "last_week": 48})
    );
    assert_eq!(post.body["lines"], draft_arguments()["lines"]);
    let created: Value = serde_json::from_str(&text(&result)).unwrap();
    assert_eq!(created["status"], "DRAFT");
}

#[tokio::test]
async fn the_assistant_cannot_ask_for_an_active_contract() {
    let (mcp, fake) = start().await;
    let mut arguments = draft_arguments();
    arguments["status"] = json!("ACTIVE");

    let result = call(&mcp, "solo", "create_maintenance_contract_draft", arguments).await;

    assert_ne!(result["isError"], json!(true), "{result}");
    let [post] = posts(&fake).try_into().expect("exactly one write");
    assert_eq!(post.body["status"], "DRAFT");

    let (_, _, listing) = rpc(&mcp, Some("solo"), "tools/list", json!({})).await;
    let tool = listing["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .find(|tool| tool["name"] == "create_maintenance_contract_draft")
        .unwrap();
    let properties = &tool["inputSchema"]["properties"];
    assert!(properties.get("status").is_none(), "{tool}");
    for field in [
        "organization",
        "customer_id",
        "customer_context_id",
        "season",
        "lines",
    ] {
        assert!(properties.get(field).is_some(), "{field}: {tool}");
    }
}

#[tokio::test]
async fn a_draft_with_a_malformed_id_writes_nothing() {
    let (mcp, fake) = start().await;
    for field in ["customer_id", "customer_context_id"] {
        let mut arguments = draft_arguments();
        arguments[field] = json!("Gaume");
        let result = call(&mcp, "solo", "create_maintenance_contract_draft", arguments).await;
        assert_eq!(result["isError"], json!(true), "{field}");
    }
    assert!(posts(&fake).is_empty());
}

#[tokio::test]
async fn the_maintenance_tools_call_three_routes_and_no_other() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "solo",
        "list_maintenance_due",
        json!({"week": "2026-W43"}),
    )
    .await;
    call(
        &mcp,
        "solo",
        "get_maintenance_contract",
        json!({"contract_id": CONTRACT}),
    )
    .await;
    call(
        &mcp,
        "solo",
        "create_maintenance_contract_draft",
        draft_arguments(),
    )
    .await;

    let mut routes: Vec<(String, String)> = fake
        .seen()
        .into_iter()
        .map(|s| (s.method, s.path.trim_start_matches("/api/v1/").to_owned()))
        .filter(|(_, path)| path != "users/@me/organizations")
        .collect();
    routes.sort();
    routes.dedup();

    assert_eq!(
        routes,
        [
            (
                "GET".to_owned(),
                format!("organizations/{ORG_A}/maintenance/contracts/{CONTRACT}")
            ),
            (
                "GET".to_owned(),
                format!("organizations/{ORG_A}/maintenance/recap")
            ),
            (
                "POST".to_owned(),
                format!("organizations/{ORG_A}/maintenance/contracts")
            ),
        ]
    );
}

fn writes(fake: &Fake) -> Vec<Seen> {
    fake.seen()
        .into_iter()
        .filter(|s| ["POST", "PUT", "PATCH"].contains(&s.method.as_str()))
        .collect()
}

#[tokio::test]
async fn update_customer_keeps_the_fields_it_was_not_given() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "t",
        "update_customer",
        json!({"organization": "acme", "customer_id": OWN_CUSTOMER, "status": "CLIENT", "email": "a@b.fr"}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let sent = &writes(&fake)[0];
    assert_eq!(sent.method, "PATCH");
    assert_eq!(sent.path, format!("/api/v1/customers/{OWN_CUSTOMER}"));
    assert_eq!(sent.body["status"], "CLIENT");
    assert_eq!(sent.body["email"], "a@b.fr");
    assert_eq!(sent.body["name"], "Dupont");
    assert_eq!(sent.body["phone"], "0102030405");
    assert_eq!(sent.body["payment_terms_days"], 30);
    assert!(sent.body.get("updated_at").is_none());
    assert!(sent.body.get("organization_id").is_none());
}

#[tokio::test]
async fn a_customer_of_another_organization_is_never_written() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "t",
        "update_customer",
        json!({"organization": "acme", "customer_id": FOREIGN_CUSTOMER, "name": "X"}),
    )
    .await;
    assert_eq!(result["isError"], json!(true));
    assert!(text(&result).contains("introuvable dans cette organisation"));
    assert!(writes(&fake).is_empty());
}

#[tokio::test]
async fn create_customer_posts_the_given_fields_to_the_organization() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "create_customer",
        json!({"organization": "acme", "name": "Martin", "status": "PROSPECT", "lead_source": "SITE_WEB_X"}),
    )
    .await;
    assert!(
        writes(&fake).is_empty(),
        "an unknown lead source is refused"
    );

    let result = call(
        &mcp,
        "t",
        "create_customer",
        json!({"organization": "acme", "name": "Martin", "status": "PROSPECT",
               "kind": "PROFESSIONAL", "lead_source": "site_web"}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let sent = &writes(&fake)[0];
    assert_eq!(
        sent.path,
        format!("/api/v1/organizations/{ORG_A}/customers")
    );
    assert_eq!(
        sent.body,
        json!({"name": "Martin", "status": "PROSPECT", "kind": "PROFESSIONAL", "lead_source": "site_web"})
    );
}

#[tokio::test]
async fn a_contact_is_added_under_a_customer_of_the_organization_only() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "create_customer_contact",
        json!({"organization": "acme", "customer_id": FOREIGN_CUSTOMER, "first_name": "A", "last_name": "B"}),
    )
    .await;
    assert!(writes(&fake).is_empty());

    call(
        &mcp,
        "t",
        "create_customer_contact",
        json!({"organization": "acme", "customer_id": OWN_CUSTOMER, "first_name": "Ana", "last_name": "Roy"}),
    )
    .await;
    let sent = &writes(&fake)[0];
    assert_eq!(
        sent.path,
        format!("/api/v1/customers/{OWN_CUSTOMER}/contacts")
    );
    assert_eq!(
        sent.body,
        json!({"first_name": "Ana", "last_name": "Roy", "is_primary": false})
    );
}

#[tokio::test]
async fn a_contact_is_updated_through_its_customer_ownership() {
    let (mcp, fake) = start().await;
    let refused = call(
        &mcp,
        "t",
        "update_customer_contact",
        json!({"organization": "acme", "contact_id": FOREIGN_CONTACT, "role": "Gérant"}),
    )
    .await;
    assert_eq!(refused["isError"], json!(true));
    assert!(writes(&fake).is_empty());

    call(
        &mcp,
        "t",
        "update_customer_contact",
        json!({"organization": "acme", "contact_id": OWN_CONTACT, "role": "Gérant"}),
    )
    .await;
    let sent = &writes(&fake)[0];
    assert_eq!(
        sent.path,
        format!("/api/v1/customer-contacts/{OWN_CONTACT}")
    );
    assert_eq!(sent.body["role"], "Gérant");
    assert_eq!(sent.body["last_name"], "Roy");
    assert_eq!(sent.body["is_primary"], false);
}

#[tokio::test]
async fn get_customer_returns_the_contacts_and_refuses_a_foreign_customer() {
    let (mcp, _) = start().await;
    let own = call(
        &mcp,
        "t",
        "get_customer",
        json!({"organization": "acme", "customer_id": OWN_CUSTOMER}),
    )
    .await;
    let body: Value = serde_json::from_str(&text(&own)).unwrap();
    assert_eq!(body["contacts"][0]["id"], OWN_CONTACT);

    let foreign = call(
        &mcp,
        "t",
        "get_customer",
        json!({"organization": "acme", "customer_id": FOREIGN_CUSTOMER}),
    )
    .await;
    assert_eq!(foreign["isError"], json!(true));
}

#[tokio::test]
async fn deals_are_listed_filtered_created_updated_and_moved() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "list_deals",
        json!({"organization": "acme", "stage": "QUOTE_SENT", "customer_id": OWN_CUSTOMER, "query": "terrasse"}),
    )
    .await;
    let list = fake.seen().pop().unwrap();
    assert!(list.query.contains("stage=QUOTE_SENT"), "{}", list.query);
    assert!(list.query.contains(&format!("customer_id={OWN_CUSTOMER}")));
    assert!(list.query.contains("q=terrasse"));

    call(
        &mcp,
        "t",
        "create_deal",
        json!({"organization": "acme", "customer_id": OWN_CUSTOMER, "title": "Terrasse", "amount_cents": 500000}),
    )
    .await;
    call(
        &mcp,
        "t",
        "update_deal",
        json!({"organization": "acme", "deal_id": OWN_DEAL, "notes": "visite faite"}),
    )
    .await;
    call(
        &mcp,
        "t",
        "move_deal_stage",
        json!({"organization": "acme", "deal_id": OWN_DEAL, "stage": "LOST", "lost_reason": "trop cher"}),
    )
    .await;
    let sent = writes(&fake);
    assert_eq!(sent[0].path, format!("/api/v1/organizations/{ORG_A}/deals"));
    assert_eq!(sent[1].method, "PATCH");
    assert_eq!(sent[1].body["title"], "Terrasse");
    assert_eq!(sent[1].body["amount_cents"], 500000);
    assert_eq!(sent[1].body["notes"], "visite faite");
    assert_eq!(sent[2].path, format!("/api/v1/deals/{OWN_DEAL}/stage"));
    assert_eq!(
        sent[2].body,
        json!({"stage": "LOST", "lost_reason": "trop cher"})
    );

    let foreign = call(
        &mcp,
        "t",
        "move_deal_stage",
        json!({"organization": "acme", "deal_id": FOREIGN_DEAL, "stage": "WON"}),
    )
    .await;
    assert_eq!(foreign["isError"], json!(true));
    assert_eq!(writes(&fake).len(), 3);
}

#[tokio::test]
async fn a_quote_is_created_with_its_lines() {
    let (mcp, fake) = start().await;
    let empty = call(
        &mcp,
        "t",
        "create_quote",
        json!({"organization": "acme", "title": "T", "customer_id": OWN_CUSTOMER,
               "customer_context_id": SITE, "lines": []}),
    )
    .await;
    assert_eq!(empty["isError"], json!(true));
    assert!(writes(&fake).is_empty());

    call(
        &mcp,
        "t",
        "create_quote",
        json!({"organization": "acme", "title": "Terrasse", "customer_id": OWN_CUSTOMER,
               "customer_context_id": SITE, "deposit_percent": "30",
               "lines": [{"label": "Dalles", "quantity": "10", "unit": "m2", "unit_price_cents": 5000}]}),
    )
    .await;
    let sent = &writes(&fake)[0];
    assert_eq!(sent.path, format!("/api/v1/organizations/{ORG_A}/quotes"));
    assert_eq!(sent.body["deposit_percent"], "30");
    assert_eq!(sent.body["lines"][0]["label"], "Dalles");
    assert_eq!(sent.body["lines"][0]["photo_keys"], json!([]));
    assert!(sent.body["lines"][0].get("vat_rate_bp").is_none());
}

#[tokio::test]
async fn updating_a_quote_keeps_its_lines_unless_given_new_ones() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "update_quote",
        json!({"organization": "acme", "quote_id": OWN_QUOTE, "notes": "à valider"}),
    )
    .await;
    let kept = &writes(&fake)[0];
    assert_eq!(kept.method, "PATCH");
    assert_eq!(kept.body["notes"], "à valider");
    assert_eq!(kept.body["status"], "DRAFT");
    assert_eq!(kept.body["title"], "Terrasse");
    assert_eq!(kept.body["lines"][0]["label"], "Dalles");
    assert_eq!(kept.body["lines"][0]["quantity"], "10");
    assert!(kept.body["lines"][0].get("id").is_none());
    assert!(kept.body["lines"][0].get("net_cents").is_none());

    call(
        &mcp,
        "t",
        "update_quote",
        json!({"organization": "acme", "quote_id": OWN_QUOTE,
               "lines": [{"label": "Gravier", "quantity": "2", "unit": "u", "unit_price_cents": 900}]}),
    )
    .await;
    let replaced = &writes(&fake)[1];
    assert_eq!(replaced.body["lines"].as_array().unwrap().len(), 1);
    assert_eq!(replaced.body["lines"][0]["label"], "Gravier");

    let foreign = call(
        &mcp,
        "t",
        "update_quote",
        json!({"organization": "acme", "quote_id": FOREIGN_QUOTE, "notes": "x"}),
    )
    .await;
    assert_eq!(foreign["isError"], json!(true));
    assert_eq!(writes(&fake).len(), 2);
}

#[tokio::test]
async fn a_quote_status_can_only_be_sent_declined_or_cancelled() {
    let (mcp, fake) = start().await;
    for refused in ["ACCEPTED", "SUPERSEDED", "DRAFT", "WHATEVER"] {
        call(
            &mcp,
            "t",
            "set_quote_status",
            json!({"organization": "acme", "quote_id": OWN_QUOTE, "status": refused}),
        )
        .await;
    }
    assert!(writes(&fake).is_empty());

    call(
        &mcp,
        "t",
        "set_quote_status",
        json!({"organization": "acme", "quote_id": OWN_QUOTE, "status": "DECLINED", "decline_reason": "trop cher"}),
    )
    .await;
    let sent = &writes(&fake)[0];
    assert_eq!(sent.path, format!("/api/v1/quotes/{OWN_QUOTE}/status"));
    assert_eq!(
        sent.body,
        json!({"status": "DECLINED", "decline_reason": "trop cher"})
    );
}

#[tokio::test]
async fn a_revision_and_a_pdf_are_refused_for_a_foreign_quote() {
    let (mcp, fake) = start().await;
    let revision = call(
        &mcp,
        "t",
        "create_quote_revision",
        json!({"organization": "acme", "quote_id": FOREIGN_QUOTE}),
    )
    .await;
    assert_eq!(revision["isError"], json!(true));
    let pdf = call(
        &mcp,
        "t",
        "export_quote_pdf",
        json!({"organization": "acme", "quote_id": FOREIGN_QUOTE}),
    )
    .await;
    assert_eq!(pdf["isError"], json!(true));
    assert!(writes(&fake).is_empty());

    call(
        &mcp,
        "t",
        "create_quote_revision",
        json!({"organization": "acme", "quote_id": OWN_QUOTE}),
    )
    .await;
    assert_eq!(
        writes(&fake)[0].path,
        format!("/api/v1/quotes/{OWN_QUOTE}/revision")
    );

    let own = call(
        &mcp,
        "t",
        "export_quote_pdf",
        json!({"organization": "acme", "quote_id": OWN_QUOTE}),
    )
    .await;
    assert_eq!(
        own["content"][0]["resource"]["mimeType"], "application/pdf",
        "{own}"
    );
}

#[tokio::test]
async fn tasks_are_listed_with_their_filters() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "list_tasks",
        json!({"organization": "acme", "status": ["BACKLOG", "IN_PROGRESS"], "project_id": PROJECT,
               "query": "toiture", "unscheduled": true}),
    )
    .await;
    let query = fake.seen().pop().unwrap().query;
    assert!(query.contains("status=BACKLOG"), "{query}");
    assert!(query.contains("status=IN_PROGRESS"), "{query}");
    assert!(query.contains(&format!("project_id={PROJECT}")));
    assert!(query.contains("q=toiture"));
    assert!(query.contains("unscheduled=true"));
}

#[tokio::test]
async fn a_task_is_created_with_an_explicit_blocks_availability() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "create_task",
        json!({"organization": "acme", "title": "Poser les dalles"}),
    )
    .await;
    assert!(writes(&fake).is_empty(), "blocks_availability is required");

    call(
        &mcp,
        "t",
        "create_task",
        json!({"organization": "acme", "title": "Poser les dalles", "blocks_availability": true,
               "status": "BACKLOG", "project_id": PROJECT}),
    )
    .await;
    let sent = &writes(&fake)[0];
    assert_eq!(sent.path, format!("/api/v1/organizations/{ORG_A}/tasks"));
    assert_eq!(sent.body["blocks_availability"], true);
    assert_eq!(sent.body["status"], "BACKLOG");
    assert_eq!(sent.body["all_day"], false);
}

#[tokio::test]
async fn a_task_patch_carries_only_the_given_fields() {
    let (mcp, fake) = start().await;
    let nothing = call(
        &mcp,
        "t",
        "update_task",
        json!({"organization": "acme", "task_id": TASK}),
    )
    .await;
    assert_eq!(nothing["isError"], json!(true));
    assert!(writes(&fake).is_empty());

    call(
        &mcp,
        "t",
        "update_task",
        json!({"organization": "acme", "task_id": TASK, "status": "DONE",
               "assignee_member_ids": [OWN_CUSTOMER]}),
    )
    .await;
    let sent = &writes(&fake)[0];
    assert_eq!(sent.method, "PATCH");
    assert_eq!(
        sent.body,
        json!({"status": "DONE", "assignees": [{"member_id": OWN_CUSTOMER}]})
    );
}

#[tokio::test]
async fn a_project_update_keeps_the_links_it_was_not_given() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "update_project",
        json!({"organization": "acme", "project_id": PROJECT, "name": "Chantier Vichy"}),
    )
    .await;
    let sent = &writes(&fake)[0];
    assert_eq!(
        sent.path,
        format!("/api/v1/organizations/{ORG_A}/projects/{PROJECT}")
    );
    assert_eq!(
        sent.body,
        json!({"name": "Chantier Vichy", "customer_id": OWN_CUSTOMER,
               "customer_context_id": null, "quote_id": null})
    );
}

#[tokio::test]
async fn a_project_a_comment_and_the_planning_go_to_the_right_paths() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "create_project",
        json!({"organization": "acme", "name": "Interne"}),
    )
    .await;
    call(
        &mcp,
        "t",
        "add_task_comment",
        json!({"organization": "acme", "task_id": TASK, "body": "commande passée"}),
    )
    .await;
    let sent = writes(&fake);
    assert_eq!(sent[0].body, json!({"name": "Interne"}));
    assert_eq!(
        sent[1].path,
        format!("/api/v1/organizations/{ORG_A}/tasks/{TASK}/comments")
    );
    assert_eq!(sent[1].body, json!({"body": "commande passée"}));

    call(
        &mcp,
        "t",
        "get_planning",
        json!({"organization": "acme", "from": "2026-11-09", "to": "2026-11-15"}),
    )
    .await;
    let query = fake.seen().pop().unwrap().query;
    assert!(query.contains("from=2026-11-09") && query.contains("to=2026-11-15"));
}

#[tokio::test]
async fn updating_a_product_keeps_its_fields_and_its_archive_state() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "update_product",
        json!({"organization": "acme", "product_id": OWN_PRODUCT, "unit_price_cents": 12000}),
    )
    .await;
    let sent = &writes(&fake)[0];
    assert_eq!(sent.method, "PATCH");
    assert_eq!(sent.path, format!("/api/v1/products/{OWN_PRODUCT}"));
    assert_eq!(sent.body["unit_price_cents"], 12000);
    assert_eq!(sent.body["name"], "Banc");
    assert_eq!(sent.body["photo_keys"], json!(["p/1.jpg"]));
    assert_eq!(sent.body["cost_cents"], 6000);
    assert_eq!(sent.body["archived"], true);
    assert!(sent.body.get("usage_count").is_none());
}

#[tokio::test]
async fn a_product_and_a_supplier_are_created_in_the_organization() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "create_product",
        json!({"organization": "acme", "name": "Banc", "unit": "u", "unit_price_cents": 10000}),
    )
    .await;
    call(
        &mcp,
        "t",
        "create_supplier",
        json!({"organization": "acme", "name": "Brico", "payment_terms_days": 30}),
    )
    .await;
    let sent = writes(&fake);
    assert_eq!(
        sent[0].path,
        format!("/api/v1/organizations/{ORG_A}/products")
    );
    assert_eq!(sent[0].body["photo_keys"], json!([]));
    assert_eq!(
        sent[1].path,
        format!("/api/v1/organizations/{ORG_A}/suppliers")
    );
    assert_eq!(
        sent[1].body,
        json!({"name": "Brico", "payment_terms_days": 30})
    );
}

#[tokio::test]
async fn updating_a_supplier_replaces_it_without_losing_its_contacts() {
    let (mcp, fake) = start().await;
    call(
        &mcp,
        "t",
        "update_supplier",
        json!({"organization": "acme", "supplier_id": OWN_SUPPLIER, "email": "a@brico.fr"}),
    )
    .await;
    let sent = &writes(&fake)[0];
    assert_eq!(sent.method, "PUT");
    assert_eq!(sent.body["email"], "a@brico.fr");
    assert_eq!(sent.body["name"], "Brico");
    assert_eq!(sent.body["phone"], "0400");
    assert_eq!(sent.body["contacts"][0]["first_name"], "Luc");
    assert_eq!(sent.body["addresses"], json!([]));
    assert!(sent.body.get("archived_at").is_none());
}

#[tokio::test]
async fn the_purchase_expense_and_reporting_reads_hit_their_endpoints() {
    let (mcp, fake) = start().await;
    for (tool, args) in [
        ("list_suppliers", json!({"organization": "acme"})),
        ("list_supplier_invoices", json!({"organization": "acme"})),
        ("list_expenses", json!({"organization": "acme"})),
        ("list_outstanding_invoices", json!({"organization": "acme"})),
        (
            "get_profitability",
            json!({"organization": "acme", "from": "2026-01-01", "to": "2026-12-31"}),
        ),
        (
            "get_worked_hours",
            json!({"organization": "acme", "from": "2026-01-01", "to": "2026-12-31"}),
        ),
    ] {
        let result = call(&mcp, "t", tool, args).await;
        assert_ne!(result["isError"], json!(true), "{tool}: {result}");
    }
    let paths: Vec<String> = fake
        .seen()
        .into_iter()
        .map(|s| s.path)
        .filter(|p| !p.contains("users/@me"))
        .collect();
    for suffix in [
        "suppliers",
        "supplier-invoices",
        "expenses",
        "invoices/outstanding",
        "reporting/profitability",
        "reporting/worked-hours",
    ] {
        assert!(
            paths.iter().any(|p| p.ends_with(suffix)),
            "{suffix}: {paths:?}"
        );
    }
    assert!(writes(&fake).is_empty());
}

fn images(result: &Value) -> Vec<Vec<u8>> {
    use base64::Engine;
    result["content"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|block| block["type"] == "image")
        .map(|block| {
            assert_eq!(block["mimeType"], "image/jpeg");
            base64::engine::general_purpose::STANDARD
                .decode(block["data"].as_str().unwrap())
                .unwrap()
        })
        .collect()
}

fn library_requests(fake: &Fake, suffix: &str) -> Vec<Seen> {
    fake.seen()
        .into_iter()
        .filter(|s| s.path.contains("/photo-library/") && s.path.ends_with(suffix))
        .collect()
}

#[tokio::test]
async fn searching_the_library_returns_the_text_then_one_thumbnail_per_photo_in_order() {
    let (mcp, fake) = start().await;
    let result = call(&mcp, "solo", "search_photo_library", json!({"limit": 3})).await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let content = result["content"].as_array().unwrap();
    assert_eq!(content.len(), 4);
    assert_eq!(content[0]["type"], "text");
    let listing: Value = serde_json::from_str(&text(&result)).unwrap();
    assert_eq!(listing["page"], 1);
    assert_eq!(listing["total"], LIBRARY_TOTAL);
    let photos = listing["photos"].as_array().unwrap();
    assert_eq!(photos.len(), 3);
    assert_eq!(
        photos[0],
        json!({
            "id": photo_id(1), "caption": "old", "tags": ["a", "b"], "project_id": PROJECT,
            "taken_at": "2026-10-01T09:30:00Z", "width": 2400, "height": 1600
        })
    );
    assert!(photos[0].get("file_key").is_none());
    let thumbnails = images(&result);
    assert_eq!(thumbnails, vec![thumbnail(1), thumbnail(2), thumbnail(3)]);
    let asked: Vec<String> = library_requests(&fake, "/thumbnail")
        .into_iter()
        .map(|s| s.path)
        .collect();
    assert_eq!(asked.len(), 3);
    assert!(asked[2].contains(&photo_id(3)));
}

#[tokio::test]
async fn searching_without_thumbnails_returns_no_image() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "search_photo_library",
        json!({"include_thumbnails": false, "limit": 5}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    assert_eq!(result["content"].as_array().unwrap().len(), 1);
    let listing: Value = serde_json::from_str(&text(&result)).unwrap();
    assert_eq!(listing["photos"].as_array().unwrap().len(), 5);
    assert!(library_requests(&fake, "/thumbnail").is_empty());
}

#[tokio::test]
async fn the_search_limit_defaults_to_eight_and_is_capped_by_the_thumbnails() {
    let (mcp, fake) = start().await;
    let by_default = call(&mcp, "solo", "search_photo_library", json!({})).await;
    assert_eq!(images(&by_default).len(), 8);

    let capped = call(&mcp, "solo", "search_photo_library", json!({"limit": 40})).await;
    assert_eq!(images(&capped).len(), 12);

    let without = call(
        &mcp,
        "solo",
        "search_photo_library",
        json!({"limit": 100, "include_thumbnails": false}),
    )
    .await;
    let listing: Value = serde_json::from_str(&text(&without)).unwrap();
    assert_eq!(listing["photos"].as_array().unwrap().len(), 50);

    let sent: Vec<usize> = library_requests(&fake, "/photos")
        .iter()
        .filter_map(|s| query_value(&s.query, "per_page"))
        .collect();
    assert_eq!(sent, vec![8, 12, 50]);
}

#[tokio::test]
async fn the_search_filters_reach_the_api() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "search_photo_library",
        json!({
            "query": "Rosier", "tags": ["Terrasse", "buis"], "project_id": PROJECT,
            "to_sort": true, "page": 2, "limit": 4, "include_thumbnails": false
        }),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let listing: Value = serde_json::from_str(&text(&result)).unwrap();
    assert_eq!(listing["page"], 2);
    let request = library_requests(&fake, "/photos").pop().unwrap();
    assert_eq!(request.method, "GET");
    for expected in [
        "q=Rosier",
        "tag=terrasse",
        "tag=buis",
        &format!("project_id={PROJECT}"),
        "to_sort=true",
        "page=2",
        "per_page=4",
    ] {
        assert!(
            request.query.contains(expected),
            "{expected} in {}",
            request.query
        );
    }
}

#[tokio::test]
async fn tagging_reports_each_photo_and_a_failure_does_not_stop_the_others() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "tag_library_photos",
        json!({"items": [
            {"photo_id": photo_id(1), "add_tags": [" Rosier ", "B"], "remove_tags": ["a"]},
            {"photo_id": photo_id(FAILING_PHOTO), "add_tags": ["x"]},
            {"photo_id": photo_id(3), "caption": ""},
            {"photo_id": "not-a-uuid", "add_tags": ["x"]},
            {"photo_id": photo_id(4)},
            {"photo_id": photo_id(5), "caption": "Allée", "add_tags": ["allée"]}
        ]}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let report: Value = serde_json::from_str(&text(&result)).unwrap();
    let results = report["results"].as_array().unwrap();
    let statuses: Vec<&str> = results
        .iter()
        .map(|r| r["status"].as_str().unwrap())
        .collect();
    assert_eq!(
        statuses,
        [
            "updated", "failed", "updated", "failed", "failed", "updated"
        ]
    );
    assert_eq!(results[0]["photo_id"], photo_id(1));
    assert!(results[0].get("error").is_none());
    assert_eq!(results[1]["error"], "étiquette invalide");
    assert!(results[3]["error"].as_str().unwrap().contains("not-a-uuid"));

    let patches: Vec<Seen> = fake
        .seen()
        .into_iter()
        .filter(|s| s.method == "PATCH")
        .collect();
    assert_eq!(patches.len(), 4);
    assert_eq!(patches[0].body, json!({"tags": ["b", "rosier"]}));
    assert_eq!(patches[2].body, json!({"tags": ["a", "b"], "caption": ""}));
    assert_eq!(
        patches[3].body,
        json!({"tags": ["a", "b", "allée"], "caption": "Allée"})
    );
}

#[tokio::test]
async fn a_twenty_first_tag_fails_that_photo_before_any_patch() {
    let (mcp, fake) = start().await;
    let many: Vec<String> = (0..19).map(|n| format!("t{n}")).collect();
    let result = call(
        &mcp,
        "solo",
        "tag_library_photos",
        json!({"items": [{"photo_id": photo_id(1), "add_tags": many}]}),
    )
    .await;
    let report: Value = serde_json::from_str(&text(&result)).unwrap();
    assert_eq!(report["results"][0]["status"], "failed");
    assert!(fake.seen().iter().all(|s| s.method != "PATCH"));
}

#[tokio::test]
async fn more_than_fifty_items_are_refused_without_touching_the_library() {
    let (mcp, fake) = start().await;
    let items: Vec<Value> = (1..=51)
        .map(|n| json!({"photo_id": photo_id(n), "add_tags": ["x"]}))
        .collect();
    let result = call(&mcp, "solo", "tag_library_photos", json!({"items": items})).await;
    assert_eq!(result["isError"], json!(true));
    assert!(text(&result).contains("50"));
    let placements: Vec<Value> = (1..=51)
        .map(|n| json!({"photo_id": photo_id(n), "target": "cover"}))
        .collect();
    let result = call(
        &mcp,
        "solo",
        "place_doe_files",
        json!({"doe_id": DOE, "placements": placements}),
    )
    .await;
    assert_eq!(result["isError"], json!(true));
    assert!(
        fake.seen()
            .iter()
            .all(|s| !s.path.contains("/photo-library/") && s.method == "GET")
    );
}

#[tokio::test]
async fn placing_reports_each_photo_and_a_failure_does_not_stop_the_others() {
    let (mcp, fake) = start().await;
    let result = call(
        &mcp,
        "solo",
        "place_doe_files",
        json!({"doe_id": DOE, "placements": [
            {"photo_id": photo_id(1), "target": "section", "section_title": "Photos"},
            {"photo_id": photo_id(FAILING_PHOTO), "target": "supply", "supply_id": SUPPLY},
            {"photo_id": photo_id(3), "target": "cover", "caption": "Façade"}
        ]}),
    )
    .await;
    assert_ne!(result["isError"], json!(true), "{result}");
    let outcome: Value = serde_json::from_str(&text(&result)).unwrap();
    assert_eq!(outcome["doe"]["id"], DOE);
    let statuses: Vec<&str> = outcome["results"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["status"].as_str().unwrap())
        .collect();
    assert_eq!(statuses, ["placed", "failed", "placed"]);
    assert_eq!(outcome["results"][1]["error"], "photo introuvable");

    let post = fake
        .seen()
        .into_iter()
        .find(|s| s.path.ends_with("/photos/place"))
        .unwrap();
    assert_eq!(
        post.body,
        json!({"placements": [
            {"photo_id": photo_id(1), "target": "section", "section_title": "Photos"},
            {"photo_id": photo_id(FAILING_PHOTO), "target": "supply", "supply_id": SUPPLY},
            {"photo_id": photo_id(3), "target": "cover", "caption": "Façade"}
        ]})
    );
    assert!(fake.seen().iter().all(|s| s.method != "PUT"));
}

#[tokio::test]
async fn an_unknown_placement_target_is_refused() {
    let (mcp, fake) = start().await;
    let (_, _, body) = rpc(
        &mcp,
        Some("solo"),
        "tools/call",
        json!({"name": "place_doe_files", "arguments": {
            "doe_id": DOE,
            "placements": [{"photo_id": photo_id(1), "target": "plan"}]
        }}),
    )
    .await;
    assert!(
        body["error"].is_object() || body["result"]["isError"] == json!(true),
        "{body}"
    );
    assert!(
        fake.seen()
            .iter()
            .all(|s| !s.path.ends_with("/photos/place"))
    );
}

#[tokio::test]
async fn no_photo_library_tool_uploads_or_deletes() {
    let (mcp, fake) = start().await;
    call(&mcp, "solo", "search_photo_library", json!({"limit": 2})).await;
    call(
        &mcp,
        "solo",
        "tag_library_photos",
        json!({"items": [{"photo_id": photo_id(1), "add_tags": ["x"]}]}),
    )
    .await;
    call(
        &mcp,
        "solo",
        "place_doe_files",
        json!({"doe_id": DOE, "placements": [{"photo_id": photo_id(1), "target": "cover"}]}),
    )
    .await;
    for seen in fake.seen() {
        assert_ne!(seen.method, "DELETE", "{}", seen.path);
        assert!(
            !(seen.method == "POST" && seen.path.ends_with("/photo-library/photos")),
            "{}",
            seen.path
        );
    }
    for tool in Tool::ALL {
        if tool.name().contains("photo") && tool.name().contains("library") {
            assert!(!tool.name().contains("delete"));
        }
    }
}
