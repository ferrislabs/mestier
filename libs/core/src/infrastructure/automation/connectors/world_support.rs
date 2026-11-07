use common::{OrganizationId, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

pub(crate) async fn organization(pool: &PgPool, label: &str) -> OrganizationId {
    let owner_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(owner_id)
    .bind(format!("owner-{owner_id}@example.com"))
    .bind(format!("owner-{owner_id}"))
    .bind("Owner User")
    .bind(format!("sub-owner-{owner_id}"))
    .execute(pool)
    .await
    .unwrap();

    let org_id = generate_uuid_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(org_id)
        .bind(format!("{label} Org"))
        .bind(format!("{label}-{org_id}"))
        .bind(owner_id)
        .execute(pool)
        .await
        .unwrap();

    OrganizationId(org_id)
}

pub(crate) struct Customer {
    pub(crate) id: Uuid,
    pub(crate) context_id: Uuid,
}

pub(crate) async fn customer(pool: &PgPool, org: OrganizationId) -> Customer {
    let id = generate_uuid_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(org.0)
        .bind("Claire Fontaine")
        .execute(pool)
        .await
        .unwrap();
    let context_id = generate_uuid_v7();
    sqlx::query("INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, $3)")
        .bind(context_id)
        .bind(id)
        .bind("Chantier principal")
        .execute(pool)
        .await
        .unwrap();
    Customer { id, context_id }
}

pub(crate) async fn customer_billed_in_nantes(pool: &PgPool, org: OrganizationId) -> Customer {
    let customer = customer(pool, org).await;
    sqlx::query(
        "UPDATE customer_contexts SET address_line = '3 rue des Tilleuls', city = 'Nantes' WHERE id = $1",
    )
    .bind(customer.context_id)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("UPDATE customers SET billing_address_id = $2 WHERE id = $1")
        .bind(customer.id)
        .bind(customer.context_id)
        .execute(pool)
        .await
        .unwrap();
    customer
}

pub(crate) async fn quote(
    pool: &PgPool,
    org: OrganizationId,
    customer: &Customer,
    status: &str,
    title: &str,
    updated_days_ago: i32,
) -> Uuid {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO quotes
             (id, org_id, customer_id, customer_context_id, reference, title, status,
              net_cents, gross_cents, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, CAST($7 AS text)::quote_status, 100000, 120000,
                 now() - make_interval(days => $8))",
    )
    .bind(id)
    .bind(org.0)
    .bind(customer.id)
    .bind(customer.context_id)
    .bind(format!("DEV-{id}"))
    .bind(title)
    .bind(status)
    .bind(updated_days_ago)
    .execute(pool)
    .await
    .unwrap();
    id
}

pub(crate) async fn channel(pool: &PgPool, org: OrganizationId, name: &str) -> Uuid {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO chat.channels (id, org_id, channel_type, name)
         VALUES ($1, $2, CAST('TEXT' AS text)::chat.channel_type, $3)",
    )
    .bind(id)
    .bind(org.0)
    .bind(name)
    .execute(pool)
    .await
    .unwrap();
    id
}

pub(crate) async fn count(pool: &PgPool, sql: &str, id: Uuid) -> i64 {
    sqlx::query_scalar(sql)
        .bind(id)
        .fetch_one(pool)
        .await
        .unwrap()
}
