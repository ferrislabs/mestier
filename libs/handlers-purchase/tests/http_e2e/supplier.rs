use serde_json::{Value, json};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{harness, issuer};

struct Outsider {
    token: String,
    user_id: Uuid,
    own_organization_id: Option<Uuid>,
}

/// A user who is either a role-less member of `member_of`, or, when it is
/// `None`, the owner of an organization of their own.
async fn seed_outsider(pool: &PgPool, member_of: Option<Uuid>) -> Outsider {
    let user_id = Uuid::now_v7();
    let sub = format!("sub-supplier-outsider-{user_id}");
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("supplier-outsider-{user_id}@example.com"))
    .bind(format!("supplier-outsider-{user_id}"))
    .bind("Outsider")
    .bind(&sub)
    .execute(pool)
    .await
    .expect("seed the outsider");

    let own_organization_id = match member_of {
        Some(_) => None,
        None => {
            let organization_id = Uuid::now_v7();
            sqlx::query(
                "INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)",
            )
            .bind(organization_id)
            .bind("Outsider Org")
            .bind(format!("supplier-outsider-org-{organization_id}"))
            .bind(user_id)
            .execute(pool)
            .await
            .expect("seed the outsider's organization");
            Some(organization_id)
        }
    };
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(member_of.or(own_organization_id).expect("an organization"))
    .bind(user_id)
    .bind("Outsider")
    .execute(pool)
    .await
    .expect("seed the outsider's membership");

    Outsider {
        token: issuer::mint(&sub),
        user_id,
        own_organization_id,
    }
}

impl Outsider {
    async fn cleanup(&self, pool: &PgPool) {
        sqlx::query("DELETE FROM organization_members WHERE user_id = $1")
            .bind(self.user_id)
            .execute(pool)
            .await
            .expect("clear the outsider's memberships");
        if let Some(organization_id) = self.own_organization_id {
            sqlx::query("DELETE FROM organizations WHERE id = $1")
                .bind(organization_id)
                .execute(pool)
                .await
                .expect("clear the outsider's organization");
        }
        sqlx::query("DELETE FROM users WHERE id = $1")
            .bind(self.user_id)
            .execute(pool)
            .await
            .expect("clear the outsider");
    }
}

fn supplier_url(base_url: &str, organization_id: Uuid, suffix: &str) -> String {
    format!("{base_url}/api/v1/organizations/{organization_id}/suppliers{suffix}")
}

#[tokio::test]
#[ignore = "requires live postgres, redis and object storage"]
async fn a_supplier_stays_inside_its_organization_and_writes_need_reference_manage() {
    let app = harness::start().await;
    let client = reqwest::Client::new();
    let body = json!({
        "name": "DUGOUR POUZZOLANE",
        "address": "Le Cratère\n63530 Volvic",
        "phone": "04 73 33 51 91",
    });

    // The harness owner holds no role, so it cannot write either: seed the
    // supplier directly and check the refusal through the API.
    let refused = client
        .post(supplier_url(&app.base_url, app.organization_id, ""))
        .bearer_auth(&app.token)
        .json(&body)
        .send()
        .await
        .expect("create");
    assert_eq!(refused.status(), 403);

    let supplier_id = Uuid::now_v7().to_string();
    sqlx::query(
        "INSERT INTO suppliers (id, org_id, name, address, phone) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(Uuid::parse_str(&supplier_id).unwrap())
    .bind(app.organization_id)
    .bind("DUGOUR POUZZOLANE")
    .bind("Le Cratère\n63530 Volvic")
    .bind("04 73 33 51 91")
    .execute(&app.pool)
    .await
    .expect("seed the supplier");

    let intruder = seed_outsider(&app.pool, None).await;
    let intruder_org = intruder.own_organization_id.expect("own organization");
    let read = client
        .get(supplier_url(
            &app.base_url,
            intruder_org,
            &format!("/{supplier_id}"),
        ))
        .bearer_auth(&intruder.token)
        .send()
        .await
        .expect("intruder read");
    assert_eq!(read.status(), 404);
    let write = client
        .put(supplier_url(
            &app.base_url,
            intruder_org,
            &format!("/{supplier_id}"),
        ))
        .bearer_auth(&intruder.token)
        .json(&json!({ "name": "Hijacked" }))
        .send()
        .await
        .expect("intruder write");
    assert_eq!(write.status(), 404);

    let member = seed_outsider(&app.pool, Some(app.organization_id)).await;
    let read = client
        .get(supplier_url(
            &app.base_url,
            app.organization_id,
            &format!("/{supplier_id}"),
        ))
        .bearer_auth(&member.token)
        .send()
        .await
        .expect("member read");
    assert_eq!(read.status(), 200);
    let write = client
        .put(supplier_url(
            &app.base_url,
            app.organization_id,
            &format!("/{supplier_id}"),
        ))
        .bearer_auth(&member.token)
        .json(&json!({ "name": "Renamed" }))
        .send()
        .await
        .expect("member write");
    assert_eq!(write.status(), 403);

    let unchanged: Value = client
        .get(supplier_url(
            &app.base_url,
            app.organization_id,
            &format!("/{supplier_id}"),
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("read back")
        .json()
        .await
        .expect("json");
    assert_eq!(unchanged["data"]["name"], "DUGOUR POUZZOLANE");

    intruder.cleanup(&app.pool).await;
    member.cleanup(&app.pool).await;
    sqlx::query("DELETE FROM suppliers WHERE org_id = $1")
        .bind(app.organization_id)
        .execute(&app.pool)
        .await
        .expect("clear suppliers");
    app.cleanup().await;
}
