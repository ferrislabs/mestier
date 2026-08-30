//! End-to-end tests for the customer API.
//!
//! Real socket, real database, real auth. #305 enforces `customer.manage` on
//! every write in this bounded context; this suite proves the gate is real
//! over HTTP, not just at the domain-service unit-test level: a member who
//! holds some business permissions (`VIEW_PLANNING`) but not
//! `MANAGE_CUSTOMERS` is refused on a customer write, while the fixture's
//! full-permission caller keeps working exactly as before.
//!
//! #395 closes the matching gap on reads: the list/get-one handlers for a
//! customer and its contacts/contexts used to accept plain organization
//! membership. They now require `VIEW_CUSTOMERS` too.
//!
//! ```bash
//! docker compose up -d postgres redis
//! source .env
//! cargo test -p handlers-customer --test http_e2e -- --ignored
//! ```

mod harness;
mod issuer;

fn create_payload() -> serde_json::Value {
    serde_json::json!({
        "status": "PROSPECT",
        "pipeline_stage": "NEW",
        "name": "Duval Masonry",
        "phone": null,
        "email": null,
    })
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_customer_manage_is_refused_on_a_write() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let refused = client
        .post(app.url(""))
        .bearer_auth(&app.restricted_token)
        .json(&create_payload())
        .send()
        .await
        .expect("the api answers the restricted member's create call");
    assert_eq!(
        refused.status(),
        403,
        "a member holding some business permissions but not customer.manage \
         must still be refused on a customer write"
    );

    // The fixture's main caller, who holds every permission, must keep
    // working exactly as it did before #305 enforced anything here.
    let created = client
        .post(app.url(""))
        .bearer_auth(&app.token)
        .json(&create_payload())
        .send()
        .await
        .expect("the api answers the owner's create call")
        .json::<serde_json::Value>()
        .await
        .expect("the created customer is json");
    assert_eq!(
        created["data"]["name"],
        serde_json::json!("Duval Masonry"),
        "{created}"
    );

    app.cleanup().await;
}

/// #395: plain organization membership used to be enough to list customers.
/// A member holding some business permissions (`VIEW_PLANNING`) but not
/// `VIEW_CUSTOMERS` is now refused outright, while a member holding exactly
/// `VIEW_CUSTOMERS` reads the same page the owner does.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn listing_customers_requires_view_customers() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let refused = client
        .get(app.url(""))
        .bearer_auth(&app.restricted_token)
        .send()
        .await
        .expect("the api answers the restricted member's list call");
    assert_eq!(
        refused.status(),
        403,
        "membership alone must not be enough to list customers"
    );

    let allowed = client
        .get(app.url(""))
        .bearer_auth(&app.view_customers_token)
        .send()
        .await
        .expect("the api answers the view-customers member's list call");
    assert_eq!(allowed.status(), 200);
    let body: serde_json::Value = allowed.json().await.expect("the answer is json");
    assert!(
        body["data"]
            .as_array()
            .expect("a page of customers")
            .iter()
            .any(|customer| customer["id"] == serde_json::json!(app.customer_id)),
        "{body}"
    );

    app.cleanup().await;
}

/// Same gate on the single-customer read.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn reading_a_customer_requires_view_customers() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let refused = client
        .get(app.customer_url(app.customer_id))
        .bearer_auth(&app.restricted_token)
        .send()
        .await
        .expect("the api answers the restricted member's get call");
    assert_eq!(refused.status(), 403);

    let allowed = client
        .get(app.customer_url(app.customer_id))
        .bearer_auth(&app.view_customers_token)
        .send()
        .await
        .expect("the api answers the view-customers member's get call");
    assert_eq!(allowed.status(), 200);
    let body: serde_json::Value = allowed.json().await.expect("the answer is json");
    assert_eq!(body["data"]["id"], serde_json::json!(app.customer_id));

    app.cleanup().await;
}

/// Same gate on a customer's contact list.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn listing_customer_contacts_requires_view_customers() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let refused = client
        .get(app.customer_contacts_url(app.customer_id))
        .bearer_auth(&app.restricted_token)
        .send()
        .await
        .expect("the api answers the restricted member's list call");
    assert_eq!(refused.status(), 403);

    let allowed = client
        .get(app.customer_contacts_url(app.customer_id))
        .bearer_auth(&app.view_customers_token)
        .send()
        .await
        .expect("the api answers the view-customers member's list call");
    assert_eq!(allowed.status(), 200);
    let body: serde_json::Value = allowed.json().await.expect("the answer is json");
    assert!(
        body["data"]
            .as_array()
            .expect("a page of contacts")
            .iter()
            .any(|contact| contact["id"] == serde_json::json!(app.customer_contact_id)),
        "{body}"
    );

    app.cleanup().await;
}

/// Same gate on a single contact read, which resolves the contact to its
/// customer's organization before checking the bit.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn reading_a_customer_contact_requires_view_customers() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let refused = client
        .get(app.customer_contact_url(app.customer_contact_id))
        .bearer_auth(&app.restricted_token)
        .send()
        .await
        .expect("the api answers the restricted member's get call");
    assert_eq!(refused.status(), 403);

    let allowed = client
        .get(app.customer_contact_url(app.customer_contact_id))
        .bearer_auth(&app.view_customers_token)
        .send()
        .await
        .expect("the api answers the view-customers member's get call");
    assert_eq!(allowed.status(), 200);
    let body: serde_json::Value = allowed.json().await.expect("the answer is json");
    assert_eq!(
        body["data"]["id"],
        serde_json::json!(app.customer_contact_id)
    );

    app.cleanup().await;
}

/// Same gate on a customer's context list.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn listing_customer_contexts_requires_view_customers() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let refused = client
        .get(app.customer_contexts_url(app.customer_id))
        .bearer_auth(&app.restricted_token)
        .send()
        .await
        .expect("the api answers the restricted member's list call");
    assert_eq!(refused.status(), 403);

    let allowed = client
        .get(app.customer_contexts_url(app.customer_id))
        .bearer_auth(&app.view_customers_token)
        .send()
        .await
        .expect("the api answers the view-customers member's list call");
    assert_eq!(allowed.status(), 200);
    let body: serde_json::Value = allowed.json().await.expect("the answer is json");
    assert!(
        body["data"]
            .as_array()
            .expect("a page of contexts")
            .iter()
            .any(|context| context["id"] == serde_json::json!(app.customer_context_id)),
        "{body}"
    );

    app.cleanup().await;
}

/// Same gate on a single context read, which resolves the context to its
/// customer's organization before checking the bit.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn reading_a_customer_context_requires_view_customers() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let refused = client
        .get(app.customer_context_url(app.customer_context_id))
        .bearer_auth(&app.restricted_token)
        .send()
        .await
        .expect("the api answers the restricted member's get call");
    assert_eq!(refused.status(), 403);

    let allowed = client
        .get(app.customer_context_url(app.customer_context_id))
        .bearer_auth(&app.view_customers_token)
        .send()
        .await
        .expect("the api answers the view-customers member's get call");
    assert_eq!(allowed.status(), 200);
    let body: serde_json::Value = allowed.json().await.expect("the answer is json");
    assert_eq!(
        body["data"]["id"],
        serde_json::json!(app.customer_context_id)
    );

    app.cleanup().await;
}
