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

/// #305: a member with `quote.manage` (the fixture's main caller, seeded a
/// role carrying `Permissions::ALL`) can create a quote — the positive case
/// already covered above by
/// `a_quote_posted_over_http_is_priced_persisted_and_listed`. This is the
/// negative one: a member of the same organization with a membership row
/// but no role assignment at all gets turned away with a 403 on a write,
/// never a 500 and never silently allowed through.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_quote_manage_is_refused_creating_a_quote() {
    let app = harness::start().await;

    let refused = reqwest::Client::new()
        .post(app.quotes_url())
        .bearer_auth(&app.no_role_token)
        .json(&json!({
            "title": "Kitchen renovation",
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "lines": [
                { "label": "Strip out existing", "quantity": "4", "unit": "HOUR", "unit_price_cents": 4500, "photo_keys": [] }
            ]
        }))
        .send()
        .await
        .expect("the api answers the create call");

    assert_eq!(
        refused.status(),
        403,
        "a member without `quote.manage` must be refused, not silently allowed through"
    );

    app.cleanup().await;
}

/// Creates a minimal quote and returns its id. Shared by the two PDF tests
/// below, neither of which cares about pricing — only about whether the
/// export is allowed to happen at all.
async fn create_a_quote(app: &harness::App) -> String {
    let created: serde_json::Value = reqwest::Client::new()
        .post(app.quotes_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "title": "Kitchen renovation",
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "lines": [
                { "label": "Strip out existing", "quantity": "4", "unit": "HOUR", "unit_price_cents": 4500, "photo_keys": [] }
            ]
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the create answer is json");

    created["data"]["id"].as_str().expect("an id").to_owned()
}

/// The seeded test organization (`harness::seed`) carries no legal
/// identity — that is the point of this test. #310 makes completeness a
/// type-level fact (`LegalIdentity::try_from_organization`); this proves
/// the refusal actually reaches an HTTP caller as a 409 naming what is
/// missing, not as a blank PDF or a 500.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn pdf_export_refuses_an_incomplete_legal_identity_naming_the_missing_fields() {
    let app = harness::start().await;
    let quote_id = create_a_quote(&app).await;

    let response = reqwest::Client::new()
        .get(format!("{}/api/v1/quotes/{quote_id}/pdf", app.base_url))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the pdf call");

    assert_eq!(
        response.status(),
        409,
        "an incomplete identity is a 409, not a 200 or a 500"
    );
    let body: serde_json::Value = response.json().await.expect("a json error body");
    let message = body["message"].as_str().unwrap_or_default();
    for field in [
        "legal_name",
        "legal_form",
        "registration_number",
        "insurance_mention",
        "vat_status",
        "address_line1",
        "address_postal_code",
        "address_city",
        "address_country",
    ] {
        assert!(
            message.contains(field),
            "the refusal must name `{field}` as missing: {message}"
        );
    }

    app.cleanup().await;
}

/// Creates a quote with one hourly line and one per-square-meter line, and
/// accepts it — the fixture every #298 test below starts from.
async fn create_and_accept_a_quote(app: &harness::App) -> (String, serde_json::Value) {
    let created: serde_json::Value = reqwest::Client::new()
        .post(app.quotes_url())
        .bearer_auth(&app.token)
        .json(&json!({
            "title": "Terrasse Dupont",
            "customer_id": app.customer_id,
            "customer_context_id": app.customer_context_id,
            "lines": [
                { "label": "Terrassement", "quantity": "3", "unit": "HOUR", "unit_price_cents": 4500, "photo_keys": [] },
                { "label": "Pose de dalles", "quantity": "10", "unit": "M2", "unit_price_cents": 3800, "photo_keys": [] }
            ]
        }))
        .send()
        .await
        .expect("the api answers the create call")
        .json()
        .await
        .expect("the create answer is json");
    let quote_id = created["data"]["id"].as_str().expect("an id").to_owned();

    let accepted = reqwest::Client::new()
        .patch(format!("{}/api/v1/quotes/{quote_id}/status", app.base_url))
        .bearer_auth(&app.token)
        .json(&json!({ "status": "ACCEPTED" }))
        .send()
        .await
        .expect("the api answers the status update call");
    assert!(accepted.status().is_success(), "{}", accepted.status());

    (quote_id, created["data"].clone())
}

/// The proposal names a duration for the hourly line and none for the
/// per-square-meter one, and the confirmed plan turns into a real project
/// carrying the quote, with a real task under it — #298's whole point.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_accepted_quote_proposes_and_then_plans_a_project() {
    let app = harness::start().await;
    let (quote_id, quote) = create_and_accept_a_quote(&app).await;
    let lines = quote["lines"].as_array().expect("the quote carries lines");
    let hourly_line_id = lines[0]["id"].as_str().expect("a line id").to_owned();
    let per_unit_line_id = lines[1]["id"].as_str().expect("a line id").to_owned();

    let proposal: serde_json::Value = reqwest::Client::new()
        .get(app.plan_proposal_url(&quote_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the plan-proposal call")
        .json()
        .await
        .expect("the plan-proposal answer is json");

    let tasks = proposal["data"]["tasks"]
        .as_array()
        .expect("the proposal carries an array of tasks");
    let hourly_proposal = tasks
        .iter()
        .find(|task| task["quote_line_id"] == json!(hourly_line_id))
        .expect("the hourly line has a proposal");
    assert_eq!(
        hourly_proposal["suggested_minutes"],
        json!(180),
        "3 hours must suggest 180 minutes: {proposal}"
    );
    let per_unit_proposal = tasks
        .iter()
        .find(|task| task["quote_line_id"] == json!(per_unit_line_id))
        .expect("the per-unit line has a proposal");
    assert_eq!(
        per_unit_proposal["suggested_minutes"],
        json!(null),
        "a per-unit line must suggest no duration rather than a guess: {proposal}"
    );

    let planned = reqwest::Client::new()
        .post(app.plan_url(&quote_id))
        .bearer_auth(&app.token)
        .json(&json!({
            "name": "Terrasse Dupont",
            "tasks": [
                {
                    "title": "Terrassement",
                    "starts_at": "2026-09-01T08:00:00Z",
                    "ends_at": "2026-09-01T11:00:00Z",
                    "all_day": false,
                    "blocks_availability": true,
                    "quote_line_ids": [hourly_line_id],
                }
            ]
        }))
        .send()
        .await
        .expect("the api answers the plan call");
    let status = planned.status();
    let body: serde_json::Value = planned
        .json()
        .await
        .unwrap_or_else(|e| panic!("the plan answer is json: {e}"));
    assert!(status.is_success(), "plan failed with {status}: {body}");

    assert_eq!(body["data"]["project"]["quote_id"], json!(quote_id));
    assert_eq!(
        body["data"]["project"]["customer_id"],
        json!(app.customer_id)
    );
    let created_tasks = body["data"]["tasks"]
        .as_array()
        .expect("the plan carries an array of tasks");
    assert_eq!(created_tasks.len(), 1);
    assert_eq!(created_tasks[0]["title"], json!("Terrassement"));
    assert_eq!(
        created_tasks[0]["project_id"],
        body["data"]["project"]["id"]
    );

    app.cleanup().await;
}

/// A quote that is not accepted is refused with its own 409, on both the
/// proposal and the plan — never a 500, and never silently ignored.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_quote_that_is_not_accepted_refuses_both_proposal_and_plan() {
    let app = harness::start().await;
    let quote_id = create_a_quote(&app).await; // stays DRAFT

    let proposal = reqwest::Client::new()
        .get(app.plan_proposal_url(&quote_id))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the plan-proposal call");
    assert_eq!(proposal.status(), 409, "a draft quote has no plan-proposal");

    let planned = reqwest::Client::new()
        .post(app.plan_url(&quote_id))
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Terrasse Dupont", "tasks": [] }))
        .send()
        .await
        .expect("the api answers the plan call");
    assert_eq!(planned.status(), 409, "a draft quote cannot be planned");

    app.cleanup().await;
}

/// Two projects on one quote makes the margin ambiguous (#260), so a second
/// plan is refused unless the caller explicitly asks for one.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_second_plan_is_refused_without_an_explicit_force_new() {
    let app = harness::start().await;
    let (quote_id, _quote) = create_and_accept_a_quote(&app).await;

    let first = reqwest::Client::new()
        .post(app.plan_url(&quote_id))
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Terrasse Dupont", "tasks": [] }))
        .send()
        .await
        .expect("the api answers the first plan call");
    assert!(first.status().is_success(), "{}", first.status());

    let second = reqwest::Client::new()
        .post(app.plan_url(&quote_id))
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Terrasse Dupont (bis)", "tasks": [] }))
        .send()
        .await
        .expect("the api answers the second plan call");
    assert_eq!(
        second.status(),
        409,
        "a second project on the same quote must be refused by default"
    );

    let forced = reqwest::Client::new()
        .post(app.plan_url(&quote_id))
        .bearer_auth(&app.token)
        .json(&json!({ "name": "Terrasse Dupont (bis)", "force_new": true, "tasks": [] }))
        .send()
        .await
        .expect("the api answers the forced plan call");
    assert!(
        forced.status().is_success(),
        "force_new must allow a second project: {}",
        forced.status()
    );

    app.cleanup().await;
}

/// The other side of the same rule: once every field
/// `LegalIdentity::try_from_organization` requires is filled in, the export
/// actually produces a PDF. Written straight to the row rather than through
/// the settings endpoint (#311, a different crate) — this suite already
/// seeds fixtures with raw SQL, and the legal-identity columns are exactly
/// the ones #310 added.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn pdf_export_renders_once_the_legal_identity_is_complete() {
    let app = harness::start().await;

    sqlx::query(
        r#"UPDATE organizations SET
            legal_name = $2,
            legal_form = $3,
            registration_number = $4,
            vat_status = $5,
            vat_exemption_basis = $6,
            insurance_mention = $7,
            address_line1 = $8,
            address_postal_code = $9,
            address_city = $10,
            address_country = $11
        WHERE id = $1"#,
    )
    .bind(app.organization_id)
    .bind("Acme SARL")
    .bind("SARL")
    .bind("123 456 789 00012")
    .bind("not_subject")
    .bind("Article 293 B du CGI")
    .bind("RC Pro n. 123456 - MAAF Assurances")
    .bind("12 rue des Artisans")
    .bind("75001")
    .bind("Paris")
    .bind("FR")
    .execute(&app.pool)
    .await
    .expect("seed the legal identity");

    let quote_id = create_a_quote(&app).await;

    let response = reqwest::Client::new()
        .get(format!("{}/api/v1/quotes/{quote_id}/pdf", app.base_url))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the pdf call");

    assert_eq!(
        response.status(),
        200,
        "a complete identity must be allowed to export"
    );
    assert_eq!(
        response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok()),
        Some("application/pdf")
    );
    let bytes = response.bytes().await.expect("the pdf body");
    assert!(
        bytes.starts_with(b"%PDF-1.4"),
        "the response must be an actual PDF, not a placeholder"
    );
    // A test that only asserts the bytes are non-empty proves nothing about
    // a document somebody sends to a customer — check that the mentions
    // this issue names actually made it into the page content stream.
    let text = String::from_utf8_lossy(&bytes);
    for expected in [
        "Acme SARL",
        "SARL",
        "123 456 789 00012",
        "Article 293 B du CGI",
    ] {
        assert!(
            text.contains(expected),
            "the pdf content stream must carry `{expected}`"
        );
    }

    app.cleanup().await;
}
