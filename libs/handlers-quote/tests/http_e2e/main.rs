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
    // A draft has no number yet: it is allocated when the quote is sent,
    // not when it is created (see #313).
    assert_eq!(body["reference"], json!(null), "{envelope}");
    let quote_id = body["id"].as_str().expect("an id").to_owned();

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
        .any(|quote| quote["id"] == json!(quote_id));
    assert!(found, "the quote just created is missing from {listed}");

    // Sending the quote is what allocates its number (see #313).
    let sent: serde_json::Value = reqwest::Client::new()
        .patch(format!("{}/api/v1/quotes/{quote_id}/status", app.base_url))
        .bearer_auth(&app.token)
        .json(&json!({ "status": "SENT" }))
        .send()
        .await
        .expect("the api answers the status update call")
        .json()
        .await
        .expect("the status update answer is json");

    let reference = sent["data"]["reference"]
        .as_str()
        .expect("a reference once sent")
        .to_owned();
    assert!(reference.starts_with("DEV-"), "{reference}");

    app.cleanup().await;
}

/// The general edit endpoint (`PATCH /quotes/{id}`) can move a quote to
/// `Sent` in the same call that edits its content — the emission tests in
/// `domain::quote::service` cover that this reports as both a content
/// change and a transition. This end-to-end pass is for a different bug
/// class: the repository's `update` writes a whole row, and a column left
/// out of its `SET` clause is a silent no-op that no mock can catch, only a
/// real database can.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn editing_a_quote_into_sent_persists_the_allocated_reference() {
    let app = harness::start().await;

    let created: serde_json::Value = reqwest::Client::new()
        .post(app.quotes_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "title": "Roof repair",
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "lines": [
                { "label": "Replace tiles", "quantity": "1", "unit": "FLAT_RATE", "unit_price_cents": 50_000, "photo_keys": [] }
            ]
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the create answer is json");
    let quote_id = created["data"]["id"].as_str().expect("an id").to_owned();
    assert_eq!(created["data"]["reference"], json!(null));

    let edited: serde_json::Value = reqwest::Client::new()
        .patch(format!("{}/api/v1/quotes/{quote_id}", app.base_url))
        .bearer_auth(&app.token)
        .json(&json!({
            "title": "Roof repair - revised",
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "status": "SENT",
            "lines": [
                { "label": "Replace tiles", "quantity": "1", "unit": "FLAT_RATE", "unit_price_cents": 50_000, "photo_keys": [] }
            ]
        }))
        .send()
        .await
        .expect("the api answers the update call")
        .json()
        .await
        .expect("the update answer is json");

    assert_eq!(edited["data"]["status"], json!("SENT"), "{edited}");
    let reference = edited["data"]["reference"]
        .as_str()
        .unwrap_or_else(|| {
            panic!("the reference allocated by this edit was not persisted: {edited}")
        })
        .to_owned();
    assert!(reference.starts_with("DEV-"), "{reference}");

    // Reading it back confirms `update`'s `RETURNING` was not the only
    // place the number appeared — a bug where `SET` never wrote the
    // column but `RETURNING` echoed back the input struct would pass the
    // assertion above and fail here.
    let refetched: serde_json::Value = reqwest::Client::new()
        .get(format!("{}/api/v1/quotes/{quote_id}", app.base_url))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the get call")
        .json()
        .await
        .expect("the get answer is json");
    assert_eq!(
        refetched["data"]["reference"],
        json!(reference),
        "{refetched}"
    );

    app.cleanup().await;
}
