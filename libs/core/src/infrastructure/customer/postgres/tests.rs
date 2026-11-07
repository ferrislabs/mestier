use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AddressKind, Customer, CustomerContact, CustomerContactId, CustomerContext, CustomerContextId,
    CustomerId, CustomerKind, CustomerPipelineStage, CustomerProfile, CustomerStatus, LeadSource,
    MemberId, OrganizationId, PaymentTermsDays,
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
        pipeline_stage: CustomerPipelineStage::New,
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
