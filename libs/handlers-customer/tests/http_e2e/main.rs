//! End-to-end tests for the customer API.
//!
//! Real socket, real database, real auth. #305 enforces `customer.manage` on
//! every write in this bounded context; this suite proves the gate is real
//! over HTTP, not just at the domain-service unit-test level: a member who
//! holds some business permissions (`VIEW_PLANNING`) but not
//! `MANAGE_CUSTOMERS` is refused on a customer write, while the fixture's
//! full-permission caller keeps working exactly as before.
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
