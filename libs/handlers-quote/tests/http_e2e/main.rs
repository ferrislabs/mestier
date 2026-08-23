//! End-to-end tests for the quote API.
//!
//! A request enters over a real socket and comes back having crossed the whole
//! stack: typed routing, the rate-limit middleware, the auth middleware with a
//! real RS256 token checked against a real JWKS fetch, the handler, the use
//! case, a Postgres transaction, and the event write that transaction commits.
//! Nothing in that chain is doubled.
//!
//! That is also why they are `#[ignore]`d, like the use-case integration tests
//! already in `libs/core`: they need the compose stack up.
//!
//! ```bash
//! docker compose up -d postgres redis
//! source .env
//! cargo test -p handlers-quote --test http_e2e -- --ignored
//! ```

mod harness;
mod issuer;

use serde_json::json;

/// The whole point in one pass: an unauthenticated call is turned away, an
/// authenticated one is priced and persisted by the server, and a second
/// request sees what the first one wrote.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_quote_posted_over_http_is_priced_persisted_and_listed() {
    let app = harness::start().await;

    let anonymous = reqwest::Client::new()
        .post(app.quotes_url())
        .json(&json!({ "title": "x", "customer_id": app.customer_id, "customer_context_id": app.customer_context_id, "lines": [] }))
        .send()
        .await
        .expect("the api answers an unauthenticated call");
    assert_eq!(
        anonymous.status(),
        401,
        "the auth middleware must be in the chain"
    );

    let created = reqwest::Client::new()
        .post(app.quotes_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "title": "Kitchen renovation",
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "lines": [
                { "label": "Strip out existing", "quantity": "4", "unit": "HOUR", "unit_price_cents": 4500, "photo_keys": [] },
                { "label": "Lay wall tiles", "quantity": "12.5", "unit": "M2", "unit_price_cents": 3800, "photo_keys": [] },
                { "label": "Fit skirting boards", "quantity": "8", "unit": "ML", "unit_price_cents": 2750, "photo_keys": [] }
            ]
        }))
        .send()
        .await
        .expect("the api answers the create call");

    let status = created.status();
    let raw = created.text().await.expect("the create answer has a body");
    assert!(
        status.is_success(),
        "creating a quote failed with {status}: {raw}"
    );
    let envelope: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("the create answer is json: {e}: {raw}"));
    let body = &envelope["data"];

    // The total is the server's, computed from the lines. A client that sent
    // its own would be ignored, which is the rule this asserts. No VAT
    // status is set on the test organization, so gross equals net and the
    // breakdown is empty.
    assert_eq!(body["net_cents"], json!(87_500), "{envelope}");
    assert_eq!(body["gross_cents"], json!(87_500), "{envelope}");
    assert_eq!(body["vat_breakdown"], json!([]), "{envelope}");
    assert_eq!(body["status"], json!("DRAFT"), "{envelope}");
    let reference = body["reference"].as_str().expect("a reference").to_owned();
    assert!(reference.starts_with("DEV-"), "{reference}");

    let listed: serde_json::Value = reqwest::Client::new()
        .get(app.quotes_url())
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the list call")
        .json()
        .await
        .expect("the list answer is json");

    let found = listed["data"]
        .as_array()
        .expect("the list answer carries an array")
        .iter()
        .any(|quote| quote["reference"] == json!(reference));
    assert!(found, "the quote just created is missing from {listed}");

    app.cleanup().await;
}
