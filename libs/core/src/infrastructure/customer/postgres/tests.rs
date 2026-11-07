use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AddressKind, Customer, CustomerContact, CustomerContactId, CustomerContext, CustomerContextId,
    CustomerId, CustomerKind, CustomerProfile, CustomerStatus, LeadSource, MemberId,
    OrganizationId, PaymentTermsDays,
    application::test_support::dev_pool,
    domain::{
        customer::ports::CustomerRepository, customer_contact::ports::CustomerContactRepository,
        customer_context::ports::CustomerContextRepository,
    },
    infrastructure::{
        customer::postgres::PgCustomerRepository,
        customer_contact::postgres::PgCustomerContactRepository,
        customer_context::postgres::PgCustomerContextRepository,
        postgres::{SharedTx, with_tx},
    },
};

async fn seed_organization(tx: &SharedTx<'_>) -> OrganizationId {
    let mut guard = tx.lock().await;
    let user_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO users (id, email, username, display_name, sub)
           VALUES ($1, $2, $3, $4, $5)"#,
        user_id,
        format!("owner-{user_id}@example.com"),
        format!("owner-{user_id}"),
        "Owner",
        format!("sub-{user_id}"),
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    let org_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)"#,
        org_id,
        "Test Org",
        format!("test-org-{org_id}"),
        user_id,
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    OrganizationId(org_id)
}

async fn seed_member(tx: &SharedTx<'_>, organization_id: OrganizationId) -> MemberId {
    let mut guard = tx.lock().await;
    let id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, $3)"#,
        id,
        organization_id.0,
        "Claire",
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    MemberId(id)
}

fn customer(organization_id: OrganizationId, name: &str) -> Customer {
    let now = Utc::now();
    Customer {
        id: CustomerId(generate_uuid_v7()),
        organization_id,
        status: CustomerStatus::Prospect,
        name: name.to_owned(),
        registration_number: None,
        phone: None,
        email: None,
        profile: CustomerProfile::default(),
        converted_at: None,
        archived_at: None,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn contact(customer_id: CustomerId, first_name: &str, is_primary: bool) -> CustomerContact {
    let now = Utc::now();
    CustomerContact {
        id: CustomerContactId(generate_uuid_v7()),
        customer_id,
        first_name: first_name.to_owned(),
        last_name: "Test".to_owned(),
        role: None,
        phone: None,
        mobile: Some("0612345678".to_owned()),
        email: None,
        is_primary,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn address(customer_id: CustomerId, label: &str, kind: AddressKind) -> CustomerContext {
    let now = Utc::now();
    CustomerContext {
        id: CustomerContextId(generate_uuid_v7()),
        customer_id,
        label: label.to_owned(),
        kind,
        address_line: Some("1 quai du Port".to_owned()),
        address_line2: Some("Bâtiment B".to_owned()),
        postal_code: Some("44000".to_owned()),
        city: Some("Nantes".to_owned()),
        country: "BE".to_owned(),
        photo_key: None,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

async fn run<F>(work: F)
where
    F: AsyncFnOnce(SharedTx<'_>) -> Result<(), CoreError>,
{
    let pool: PgPool = dev_pool().await;
    let outcome = with_tx(&pool, async |tx| {
        work(tx).await?;
        Err::<(), _>(CoreError::Internal("rollback".into()))
    })
    .await;
    assert!(matches!(outcome, Err(CoreError::Internal(message)) if message == "rollback"));
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_customer_profile_round_trips() {
    run(async |tx| {
        let organization_id = seed_organization(&tx).await;
        let owner = seed_member(&tx, organization_id).await;
        let mut customers = PgCustomerRepository::new(&tx);
        let mut contacts = PgCustomerContactRepository::new(&tx);
        let mut contexts = PgCustomerContextRepository::new(&tx);

        let mut stored = customers
            .insert(&customer(organization_id, "Boulangerie Lemoine"))
            .await?;
        let billing_contact = contacts.insert(&contact(stored.id, "Marc", true)).await?;
        let billing_address = contexts
            .insert(&address(stored.id, "Siège", AddressKind::Billing))
            .await?;
        let converted = Utc::now();
        stored.status = CustomerStatus::Client;
        stored.converted_at = Some(converted);
        stored.profile = CustomerProfile {
            kind: CustomerKind::Individual,
            website: Some("https://lemoine.fr".to_owned()),
            vat_number: Some("FR12345678901".to_owned()),
            owner_member_id: Some(owner),
            lead_source: Some(LeadSource::TradeShow),
            notes: Some("Livraison le matin".to_owned()),
            payment_terms: Some(PaymentTermsDays::try_from(45).unwrap()),
            billing_contact_id: Some(billing_contact.id),
            billing_address_id: Some(billing_address.id),
        };
        let updated = customers.update(&stored).await?;

        let found = customers.find_by_id(stored.id).await?.expect("stored");
        assert_eq!(found.profile, stored.profile);
        assert_eq!(found.profile, updated.profile);
        assert_eq!(found.status, CustomerStatus::Client);
        assert!(found.converted_at.is_some());
        assert_eq!(found.archived_at, None);

        let stored_contact = contacts.find_by_id(billing_contact.id).await?.unwrap();
        assert_eq!(stored_contact.mobile.as_deref(), Some("0612345678"));
        let stored_address = contexts.find_by_id(billing_address.id).await?.unwrap();
        assert_eq!(stored_address.kind, AddressKind::Billing);
        assert_eq!(stored_address.country, "BE");
        assert_eq!(stored_address.address_line2.as_deref(), Some("Bâtiment B"));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_ownership_checks_answer_for_the_customer_only() {
    run(async |tx| {
        let organization_id = seed_organization(&tx).await;
        let mut customers = PgCustomerRepository::new(&tx);
        let mut contacts = PgCustomerContactRepository::new(&tx);
        let mut contexts = PgCustomerContextRepository::new(&tx);

        let mine = customers.insert(&customer(organization_id, "Mine")).await?;
        let other = customers
            .insert(&customer(organization_id, "Other"))
            .await?;
        let contact = contacts.insert(&contact(other.id, "Paul", true)).await?;
        let site = contexts
            .insert(&address(other.id, "Atelier", AddressKind::Site))
            .await?;

        assert!(customers.has_contact(other.id, contact.id).await?);
        assert!(!customers.has_contact(mine.id, contact.id).await?);
        assert!(customers.has_address(other.id, site.id).await?);
        assert!(!customers.has_address(mine.id, site.id).await?);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_refuses_a_billing_contact_of_another_customer() {
    run(async |tx| {
        let organization_id = seed_organization(&tx).await;
        let mut customers = PgCustomerRepository::new(&tx);
        let mut contacts = PgCustomerContactRepository::new(&tx);

        let mine = customers.insert(&customer(organization_id, "Mine")).await?;
        let other = customers
            .insert(&customer(organization_id, "Other"))
            .await?;
        let foreign = contacts.insert(&contact(other.id, "Paul", true)).await?;

        let mut tampered = mine;
        tampered.profile.billing_contact_id = Some(foreign.id);
        let refused = customers.update(&tampered).await;

        assert!(refused.is_err());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_refuses_an_owner_from_another_organization() {
    run(async |tx| {
        let organization_id = seed_organization(&tx).await;
        let other_organization = seed_organization(&tx).await;
        let stranger = seed_member(&tx, other_organization).await;
        let mut customers = PgCustomerRepository::new(&tx);

        let mut tampered = customer(organization_id, "Mine");
        tampered.profile.owner_member_id = Some(stranger);
        let refused = customers.insert(&tampered).await;

        assert!(refused.is_err());
        Ok(())
    })
    .await;
}

async fn refused_by_check(tx: &SharedTx<'_>, statement: &str, id: Uuid) -> bool {
    let mut guard = tx.lock().await;
    sqlx::query(statement)
        .bind(id)
        .execute(&mut ***guard)
        .await
        .is_err()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_bounds_payment_terms_kind_and_source() {
    let pool: PgPool = dev_pool().await;
    for statement in [
        "UPDATE customers SET payment_terms_days = 61 WHERE id = $1",
        "UPDATE customers SET payment_terms_days = -1 WHERE id = $1",
        "UPDATE customers SET kind = 'COMPANY' WHERE id = $1",
        "UPDATE customers SET lead_source = 'lead' WHERE id = $1",
    ] {
        let outcome = with_tx(&pool, async |tx| {
            let organization_id = seed_organization(&tx).await;
            let stored = PgCustomerRepository::new(&tx)
                .insert(&customer(organization_id, "Bounds"))
                .await?;
            let refused = refused_by_check(&tx, statement, stored.id.0).await;
            Err::<bool, _>(CoreError::Internal(refused.to_string()))
        })
        .await;
        assert!(
            matches!(outcome, Err(CoreError::Internal(ref flag)) if flag == "true"),
            "{statement}"
        );
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn rows_written_without_the_new_columns_get_the_documented_defaults() {
    run(async |tx| {
        let organization_id = seed_organization(&tx).await;
        let id = generate_uuid_v7();
        {
            let mut guard = tx.lock().await;
            sqlx::query!(
                r#"INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)"#,
                id,
                organization_id.0,
                "Legacy",
            )
            .execute(&mut ***guard)
            .await
            .unwrap();
        }
        let mut customers = PgCustomerRepository::new(&tx);
        let mut contexts = PgCustomerContextRepository::new(&tx);
        let legacy = customers
            .find_by_id(CustomerId(id))
            .await?
            .expect("the legacy row is readable");
        assert_eq!(legacy.profile.kind, CustomerKind::Professional);
        assert_eq!(legacy.profile.payment_terms, None);

        {
            let mut guard = tx.lock().await;
            let context_id = generate_uuid_v7();
            sqlx::query!(
                r#"INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, $3)"#,
                context_id,
                id,
                "Legacy site",
            )
            .execute(&mut ***guard)
            .await
            .unwrap();
            drop(guard);
            let found = contexts
                .find_by_id(CustomerContextId(context_id))
                .await?
                .expect("readable");
            assert_eq!(found.kind, AddressKind::Site);
            assert_eq!(found.country, "FR");
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn deleting_the_billing_contact_falls_back_to_the_next_contact() {
    run(async |tx| {
        let organization_id = seed_organization(&tx).await;
        let mut customers = PgCustomerRepository::new(&tx);
        let mut contacts = PgCustomerContactRepository::new(&tx);

        let mut stored = customers.insert(&customer(organization_id, "Mine")).await?;
        let first = contacts.insert(&contact(stored.id, "Claire", true)).await?;
        let second = contacts.insert(&contact(stored.id, "Marc", false)).await?;
        stored.profile.billing_contact_id = Some(first.id);
        customers.update(&stored).await?;

        contacts.soft_delete(first.id, Utc::now()).await?;
        let after_first = customers.find_by_id(stored.id).await?.unwrap();
        assert_eq!(after_first.profile.billing_contact_id, Some(second.id));

        contacts.soft_delete(second.id, Utc::now()).await?;
        let after_second = customers.find_by_id(stored.id).await?.unwrap();
        assert_eq!(after_second.profile.billing_contact_id, None);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn deleting_the_billing_address_falls_back_to_another_billing_address() {
    run(async |tx| {
        let organization_id = seed_organization(&tx).await;
        let mut customers = PgCustomerRepository::new(&tx);
        let mut contexts = PgCustomerContextRepository::new(&tx);

        let mut stored = customers.insert(&customer(organization_id, "Mine")).await?;
        let first = contexts
            .insert(&address(stored.id, "Siège", AddressKind::Billing))
            .await?;
        let second = contexts
            .insert(&address(stored.id, "Compta", AddressKind::Billing))
            .await?;
        let site = contexts
            .insert(&address(stored.id, "Atelier", AddressKind::Site))
            .await?;
        stored.profile.billing_address_id = Some(first.id);
        customers.update(&stored).await?;

        contexts.soft_delete(first.id, Utc::now()).await?;
        let after_first = customers.find_by_id(stored.id).await?.unwrap();
        assert_eq!(after_first.profile.billing_address_id, Some(second.id));

        contexts.soft_delete(second.id, Utc::now()).await?;
        let after_second = customers.find_by_id(stored.id).await?.unwrap();
        assert_eq!(after_second.profile.billing_address_id, None);
        assert!(customers.has_address(stored.id, site.id).await?);
        Ok(())
    })
    .await;
}

struct SeedInvoice {
    organization_id: OrganizationId,
    customer_id: CustomerId,
    context_id: CustomerContextId,
    kind: &'static str,
    status: &'static str,
    gross_cents: i32,
    issued_on: chrono::NaiveDate,
    due_on: chrono::NaiveDate,
    source: Option<Uuid>,
}

async fn seed_invoice(tx: &SharedTx<'_>, seed: SeedInvoice) -> Uuid {
    let SeedInvoice {
        organization_id,
        customer_id,
        context_id,
        kind,
        status,
        gross_cents,
        issued_on,
        due_on,
        source,
    } = seed;
    let mut guard = tx.lock().await;
    let id = generate_uuid_v7();
    let issued_at = issued_on.and_hms_opt(9, 0, 0).unwrap().and_utc();
    let issued = status != "DRAFT";
    sqlx::query(
        r#"
        INSERT INTO invoices (id, org_id, number, kind, customer_id, customer_context_id, status,
            issued_at, issued_on, due_at, net_cents, gross_cents, issuer_identity, source_invoice_id,
            reason)
        VALUES ($1, $2, $3, $4::invoice_kind, $5, $6, $7::invoice_status, $8, $9, $10, $11, $12, $13,
            $14, $15)
        "#,
    )
    .bind(id)
    .bind(organization_id.0)
    .bind(issued.then(|| format!("N-{id}")))
    .bind(kind)
    .bind(customer_id.0)
    .bind(context_id.0)
    .bind(status)
    .bind(issued.then_some(issued_at))
    .bind(issued.then_some(issued_on))
    .bind(due_on.and_hms_opt(0, 0, 0).unwrap().and_utc())
    .bind(gross_cents * 5 / 6)
    .bind(gross_cents)
    .bind(issued.then(|| serde_json::json!({})))
    .bind(source)
    .bind((kind == "CREDIT_NOTE").then_some("Erreur"))
    .execute(&mut ***guard)
    .await
    .unwrap();
    id
}

async fn seed_payment(
    tx: &SharedTx<'_>,
    organization_id: OrganizationId,
    invoice_id: Uuid,
    amount_cents: i32,
) {
    let mut guard = tx.lock().await;
    let user_id: Uuid = sqlx::query_scalar("SELECT owner_id FROM organizations WHERE id = $1")
        .bind(organization_id.0)
        .fetch_one(&mut ***guard)
        .await
        .unwrap();
    sqlx::query(
        r#"
        INSERT INTO invoice_payments (id, org_id, invoice_id, amount_cents, paid_on, method, recorded_by)
        VALUES ($1, $2, $3, $4, CURRENT_DATE, 'TRANSFER', $5)
        "#,
    )
    .bind(generate_uuid_v7())
    .bind(organization_id.0)
    .bind(invoice_id)
    .bind(amount_cents)
    .bind(user_id)
    .execute(&mut ***guard)
    .await
    .unwrap();
}

async fn seed_quote(
    tx: &SharedTx<'_>,
    organization_id: OrganizationId,
    customer_id: CustomerId,
    context_id: CustomerContextId,
    created_at: chrono::DateTime<Utc>,
    deleted: bool,
) {
    let mut guard = tx.lock().await;
    let id = generate_uuid_v7();
    sqlx::query(
        r#"
        INSERT INTO quotes (id, org_id, customer_id, customer_context_id, status, net_cents,
            gross_cents, title, reference, created_at, deleted_at)
        VALUES ($1, $2, $3, $4, 'DRAFT', 0, 0, 'Devis', $5, $6, $7)
        "#,
    )
    .bind(id)
    .bind(organization_id.0)
    .bind(customer_id.0)
    .bind(context_id.0)
    .bind(format!("Q-{id}"))
    .bind(created_at)
    .bind(deleted.then_some(created_at))
    .execute(&mut ***guard)
    .await
    .unwrap();
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_customer_row_carries_its_city_and_primary_contact() {
    run(async |tx| {
        let organization_id = seed_organization(&tx).await;
        let mut customers = PgCustomerRepository::new(&tx);
        let mut contacts = PgCustomerContactRepository::new(&tx);
        let mut contexts = PgCustomerContextRepository::new(&tx);

        let mut stored = customers.insert(&customer(organization_id, "Mine")).await?;
        let bare = customers.insert(&customer(organization_id, "Bare")).await?;
        let mut site = address(stored.id, "Atelier", AddressKind::Site);
        site.city = Some("Rennes".to_owned());
        contexts.insert(&site).await?;
        let mut billing = address(stored.id, "Siège", AddressKind::Billing);
        billing.city = Some("  Lyon ".to_owned());
        contexts.insert(&billing).await?;
        contacts
            .insert(&contact(stored.id, "Second", false))
            .await?;
        let primary = contact(stored.id, "Marc", true);
        contacts.insert(&primary).await?;

        let entries = customers
            .directory(organization_id, &[stored.id, bare.id])
            .await?;
        let of = |id: CustomerId| {
            entries
                .iter()
                .find(|entry| entry.customer_id == id)
                .unwrap()
        };
        assert_eq!(of(stored.id).city.as_deref(), Some("Lyon"));
        assert_eq!(
            of(stored.id)
                .primary_contact
                .as_ref()
                .map(|c| c.first_name.as_str()),
            Some("Marc")
        );
        assert_eq!(
            of(stored.id)
                .primary_contact
                .as_ref()
                .and_then(|c| c.mobile.as_deref()),
            Some("0612345678")
        );
        assert_eq!(of(bare.id).city, None);
        assert_eq!(of(bare.id).primary_contact, None);

        stored.profile.billing_address_id = None;
        let chosen = contexts
            .insert(&address(stored.id, "Compta", AddressKind::Other))
            .await?;
        stored.profile.billing_address_id = Some(chosen.id);
        customers.update(&stored).await?;
        let entries = customers.directory(organization_id, &[stored.id]).await?;
        assert_eq!(entries[0].city.as_deref(), Some("Nantes"));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_directory_and_the_invoice_facts_stay_inside_the_organization() {
    run(async |tx| {
        let mine = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let mut customers = PgCustomerRepository::new(&tx);
        let mut contexts = PgCustomerContextRepository::new(&tx);
        let stored = customers.insert(&customer(mine, "Mine")).await?;
        let context = contexts
            .insert(&address(stored.id, "Siège", AddressKind::Billing))
            .await?;
        let today = Utc::now().date_naive();
        seed_invoice(
            &tx,
            SeedInvoice {
                organization_id: mine,
                customer_id: stored.id,
                context_id: context.id,
                kind: "STANDARD",
                status: "ISSUED",
                gross_cents: 12_000,
                issued_on: today,
                due_on: today,
                source: None,
            },
        )
        .await;

        assert!(customers.directory(other, &[stored.id]).await?.is_empty());
        assert!(
            customers
                .invoice_facts(other, &[stored.id])
                .await?
                .is_empty()
        );
        assert_eq!(customers.invoice_facts(mine, &[stored.id]).await?.len(), 1);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn invoice_facts_carry_payments_and_credit_notes_and_skip_drafts_and_cancelled() {
    run(async |tx| {
        let organization_id = seed_organization(&tx).await;
        let mut customers = PgCustomerRepository::new(&tx);
        let mut contexts = PgCustomerContextRepository::new(&tx);
        let stored = customers.insert(&customer(organization_id, "Mine")).await?;
        let context = contexts
            .insert(&address(stored.id, "Siège", AddressKind::Billing))
            .await?;
        let today = Utc::now().date_naive();
        let past = today - chrono::Duration::days(40);
        let due = today - chrono::Duration::days(10);
        let id = |kind: &'static str, status: &'static str, gross: i32, source: Option<Uuid>| {
            seed_invoice(
                &tx,
                SeedInvoice {
                    organization_id,
                    customer_id: stored.id,
                    context_id: context.id,
                    kind,
                    status,
                    gross_cents: gross,
                    issued_on: past,
                    due_on: due,
                    source,
                },
            )
        };
        let late = id("STANDARD", "ISSUED", 12_000, None).await;
        id("STANDARD", "DRAFT", 99_000, None).await;
        id("STANDARD", "CANCELLED", 88_000, None).await;
        id("CREDIT_NOTE", "ISSUED", 2_400, Some(late)).await;
        seed_payment(&tx, organization_id, late, 3_000).await;

        let facts = customers
            .invoice_facts(organization_id, &[stored.id])
            .await?;

        assert_eq!(facts.len(), 2);
        let standard = facts
            .iter()
            .find(|fact| fact.kind == crate::InvoiceKind::Standard)
            .unwrap();
        assert_eq!(standard.gross_cents, 12_000);
        assert_eq!(standard.paid_cents, 3_000);
        assert_eq!(standard.credited_cents, 2_400);
        assert_eq!(standard.due_on, Some(due));
        assert_eq!(standard.issued_on, past);

        let indicators = crate::domain::customer::directory::indicators_of(&facts, today);
        assert_eq!(indicators.overdue_cents, 12_000 - 3_000 - 2_400);
        assert_eq!(indicators.revenue_12m_cents, 10_000 - 2_000);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_last_quote_date_ignores_deleted_quotes() {
    run(async |tx| {
        let organization_id = seed_organization(&tx).await;
        let mut customers = PgCustomerRepository::new(&tx);
        let mut contexts = PgCustomerContextRepository::new(&tx);
        let stored = customers.insert(&customer(organization_id, "Mine")).await?;
        let context = contexts
            .insert(&address(stored.id, "Siège", AddressKind::Billing))
            .await?;
        let earlier = Utc::now() - chrono::Duration::days(30);
        let latest = Utc::now() - chrono::Duration::days(3);
        seed_quote(&tx, organization_id, stored.id, context.id, earlier, false).await;
        seed_quote(&tx, organization_id, stored.id, context.id, latest, false).await;
        seed_quote(
            &tx,
            organization_id,
            stored.id,
            context.id,
            Utc::now(),
            true,
        )
        .await;

        let entries = customers.directory(organization_id, &[stored.id]).await?;

        assert_eq!(entries[0].last_quote_at, Some(latest));
        Ok(())
    })
    .await;
}
