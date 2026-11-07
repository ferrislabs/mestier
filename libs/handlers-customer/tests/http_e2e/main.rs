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

async fn create_customer(
    app: &harness::App,
    client: &reqwest::Client,
    body: serde_json::Value,
) -> reqwest::Response {
    client
        .post(app.url(""))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("the api answers the create call")
}

fn profile_payload(name: &str) -> serde_json::Value {
    serde_json::json!({
        "status": "PROSPECT",
        "pipeline_stage": "NEW",
        "name": name,
        "registration_number": "123 456 789 00012",
        "kind": "INDIVIDUAL",
        "website": "https://duval.fr",
        "vat_number": "FR12345678901",
        "lead_source": "salon",
        "notes": "Livraison le matin",
        "payment_terms_days": 45,
    })
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_customer_profile_and_lifecycle_round_trip_over_http() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let created: serde_json::Value = create_customer(&app, &client, profile_payload("Profile"))
        .await
        .json()
        .await
        .expect("the created customer is json");
    let data = &created["data"];
    assert_eq!(data["kind"], "INDIVIDUAL", "{created}");
    assert_eq!(data["registration_number"], "12345678900012", "{created}");
    assert_eq!(data["lead_source"], "salon", "{created}");
    assert_eq!(data["payment_terms_days"], 45, "{created}");
    assert_eq!(data["converted_at"], serde_json::Value::Null, "{created}");
    let id: uuid::Uuid = data["id"].as_str().unwrap().parse().unwrap();

    let mut converted = profile_payload("Profile");
    converted["status"] = "CLIENT".into();
    let patch = |body: serde_json::Value| {
        client
            .patch(app.customer_url(id))
            .bearer_auth(&app.token)
            .json(&body)
            .send()
    };
    let first: serde_json::Value = patch(converted.clone())
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    let first_date = first["data"]["converted_at"].clone();
    assert!(first_date.is_string(), "{first}");

    let again: serde_json::Value = patch(converted.clone())
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(again["data"]["converted_at"], first_date, "{again}");

    let mut archived = converted.clone();
    archived["status"] = "ARCHIVED".into();
    let archived: serde_json::Value = patch(archived).await.unwrap().json().await.unwrap();
    assert!(archived["data"]["archived_at"].is_string(), "{archived}");

    let mut restored = converted;
    restored["status"] = "PROSPECT".into();
    let restored: serde_json::Value = patch(restored).await.unwrap().json().await.unwrap();
    assert_eq!(restored["data"]["status"], "CLIENT", "{restored}");
    assert_eq!(restored["data"]["archived_at"], serde_json::Value::Null);
    assert_eq!(restored["data"]["converted_at"], first_date, "{restored}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_refused_customer_field_answers_422_naming_it() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let mut short = profile_payload("Short");
    short["registration_number"] = "1234567890".into();
    let refused = create_customer(&app, &client, short).await;
    assert_eq!(refused.status(), 422);
    let body: serde_json::Value = refused.json().await.unwrap();
    assert_eq!(body["details"]["field"], "registration_number", "{body}");

    let mut slow = profile_payload("Slow");
    slow["payment_terms_days"] = 61.into();
    let refused = create_customer(&app, &client, slow).await;
    assert_eq!(refused.status(), 422);
    let body: serde_json::Value = refused.json().await.unwrap();
    assert_eq!(body["details"]["field"], "payment_terms_days", "{body}");

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn contacts_addresses_and_billing_designations_over_http() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let contact: serde_json::Value = client
        .post(app.customer_contacts_url(app.customer_id))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "first_name": "Marc",
            "last_name": "Lemoine",
            "phone": "02 40 00 00 00",
            "mobile": "06 12 34 56 78",
            "is_primary": true,
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(contact["data"]["mobile"], "06 12 34 56 78", "{contact}");
    assert_eq!(contact["data"]["phone"], "02 40 00 00 00", "{contact}");

    let address: serde_json::Value = client
        .post(app.customer_contexts_url(app.customer_id))
        .bearer_auth(&app.token)
        .json(&serde_json::json!({
            "label": "Siège",
            "kind": "BILLING",
            "address_line": "1 quai du Port",
            "address_line2": "Bâtiment B",
            "country": "BE",
        }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(address["data"]["kind"], "BILLING", "{address}");
    assert_eq!(address["data"]["country"], "BE", "{address}");
    assert_eq!(address["data"]["address_line2"], "Bâtiment B", "{address}");

    let other: serde_json::Value = create_customer(&app, &client, create_payload())
        .await
        .json()
        .await
        .unwrap();
    let other_id: uuid::Uuid = other["data"]["id"].as_str().unwrap().parse().unwrap();

    let mut foreign_contact = create_payload();
    foreign_contact["billing_contact_id"] = contact["data"]["id"].clone();
    let refused = client
        .patch(app.customer_url(other_id))
        .bearer_auth(&app.token)
        .json(&foreign_contact)
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 422);
    let body: serde_json::Value = refused.json().await.unwrap();
    assert_eq!(body["details"]["field"], "billing_contact_id", "{body}");

    let mut foreign_address = create_payload();
    foreign_address["billing_address_id"] = address["data"]["id"].clone();
    let refused = client
        .patch(app.customer_url(other_id))
        .bearer_auth(&app.token)
        .json(&foreign_address)
        .send()
        .await
        .unwrap();
    assert_eq!(refused.status(), 422);
    let body: serde_json::Value = refused.json().await.unwrap();
    assert_eq!(body["details"]["field"], "billing_address_id", "{body}");

    let mut own = create_payload();
    own["billing_contact_id"] = contact["data"]["id"].clone();
    own["billing_address_id"] = address["data"]["id"].clone();
    let accepted: serde_json::Value = client
        .patch(app.customer_url(app.customer_id))
        .bearer_auth(&app.token)
        .json(&own)
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        accepted["data"]["billing_contact_id"], contact["data"]["id"],
        "{accepted}"
    );

    app.cleanup().await;
}
