use chrono::{NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AddressKind, BillingFrequency, Customer, CustomerContext, CustomerContextId, CustomerId,
    CustomerProfile, CustomerStatus, DepositPercent, DiscountPercent, MemberId, OrganizationId,
    Quote, QuoteId, QuoteLine, QuoteLineId, QuoteStatus, RecipientSnapshot,
    application::test_support::dev_pool,
    domain::{
        customer::ports::CustomerRepository, customer_context::ports::CustomerContextRepository,
        quote::ports::QuoteRepository,
    },
    infrastructure::{
        customer::postgres::PgCustomerRepository,
        customer_context::postgres::PgCustomerContextRepository,
        postgres::{SharedTx, with_tx},
        quote::postgres::PgQuoteRepository,
    },
};

const DOWN: &str =
    include_str!("../../../../../../migrations/20261109000004_quote_lifecycle.down.sql");

async fn seed_organization(tx: &SharedTx<'_>) -> OrganizationId {
    let mut guard = tx.lock().await;
    let user_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("owner-{user_id}@example.com"))
    .bind(format!("owner-{user_id}"))
    .bind("Owner")
    .bind(format!("sub-{user_id}"))
    .execute(&mut ***guard)
    .await
    .unwrap();

    let org_id = generate_uuid_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(org_id)
        .bind("Test Org")
        .bind(format!("test-org-{org_id}"))
        .bind(user_id)
        .execute(&mut ***guard)
        .await
        .unwrap();

    OrganizationId(org_id)
}

async fn seed_member(tx: &SharedTx<'_>, organization_id: OrganizationId) -> MemberId {
    let mut guard = tx.lock().await;
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, $3)",
    )
    .bind(id)
    .bind(organization_id.0)
    .bind("Claire")
    .execute(&mut ***guard)
    .await
    .unwrap();

    MemberId(id)
}

async fn seed_customer(tx: &SharedTx<'_>, organization_id: OrganizationId) -> Customer {
    let now = Utc::now();
    PgCustomerRepository::new(tx)
        .insert(&Customer {
            id: CustomerId(generate_uuid_v7()),
            organization_id,
            status: CustomerStatus::Prospect,
            name: "Boulangerie".to_owned(),
            registration_number: None,
            phone: None,
            email: None,
            profile: CustomerProfile::default(),
            converted_at: None,
            archived_at: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap()
}

async fn seed_context(tx: &SharedTx<'_>, customer_id: CustomerId) -> CustomerContextId {
    let now = Utc::now();
    PgCustomerContextRepository::new(tx)
        .insert(&CustomerContext {
            id: CustomerContextId(generate_uuid_v7()),
            customer_id,
            label: "Siège".to_owned(),
            kind: AddressKind::Billing,
            address_line: Some("12 rue des Lilas".to_owned()),
            address_line2: None,
            postal_code: Some("69003".to_owned()),
            city: Some("Lyon".to_owned()),
            country: "FR".to_owned(),
            photo_key: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap()
        .id
}

async fn seed_contact(tx: &SharedTx<'_>, customer_id: CustomerId) -> Uuid {
    let mut guard = tx.lock().await;
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO customer_contacts (id, customer_id, first_name, last_name) VALUES ($1, $2, 'Anne', 'Duval')",
    )
    .bind(id)
    .bind(customer_id.0)
    .execute(&mut ***guard)
    .await
    .unwrap();

    id
}

struct Seed {
    organization: OrganizationId,
    customer: Customer,
    context: CustomerContextId,
}

async fn seed(tx: &SharedTx<'_>) -> Seed {
    let organization = seed_organization(tx).await;
    let customer = seed_customer(tx, organization).await;
    let context = seed_context(tx, customer.id).await;

    Seed {
        organization,
        customer,
        context,
    }
}

fn line(seed: &Seed, quote_id: QuoteId) -> QuoteLine {
    let now = Utc::now();
    QuoteLine {
        id: QuoteLineId(generate_uuid_v7()),
        organization_id: seed.organization,
        quote_id,
        service_rate_id: None,
        product_id: None,
        label: "Pose".to_owned(),
        quantity: Decimal::new(25, 1),
        unit: "HOUR".to_owned(),
        unit_price_cents: 4500,
        vat_rate_bp: Some(2000),
        discount_percent: DiscountPercent::try_from(Decimal::new(1250, 2)).unwrap(),
        billing_frequency: BillingFrequency::Monthly,
        notes: None,
        photo_keys: vec![],
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn quote(seed: &Seed) -> Quote {
    let id = QuoteId(generate_uuid_v7());
    Quote {
        net_cents: 9844,
        gross_cents: 11813,
        lines: vec![line(seed, id)],
        ..Quote::draft(
            id,
            seed.organization,
            seed.customer.id,
            seed.context,
            "Devis".to_owned(),
            Utc::now(),
        )
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

async fn refused(tx: &SharedTx<'_>, statement: &str) -> bool {
    let mut guard = tx.lock().await;
    sqlx::query("SAVEPOINT attempt")
        .execute(&mut ***guard)
        .await
        .unwrap();
    let outcome = sqlx::query(statement).execute(&mut ***guard).await;
    sqlx::query("ROLLBACK TO SAVEPOINT attempt")
        .execute(&mut ***guard)
        .await
        .unwrap();
    outcome.is_err()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_quote_round_trips_every_lifecycle_column_and_every_new_line_column() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let owner = seed_member(&tx, seed.organization).await;
        let contact = seed_contact(&tx, seed.customer.id).await;
        let mut stored = quote(&seed);
        stored.valid_until = NaiveDate::from_ymd_opt(2026, 12, 1);
        stored.deposit_percent = DepositPercent::try_from(Decimal::new(305, 1)).unwrap();
        stored.owner_member_id = Some(owner);
        stored.notes = Some("Accès par la cour".to_owned());
        stored.billing_contact_id = Some(crate::CustomerContactId(contact));
        stored.billing_address_id = Some(seed.context);
        stored.recipient_snapshot = Some(RecipientSnapshot {
            name: "Boulangerie".to_owned(),
            registration_number: None,
            vat_number: None,
            contact_name: Some("Anne Duval".to_owned()),
            contact_email: None,
            address: Default::default(),
        });
        stored.issuer_snapshot = Some(crate::IssuerSnapshot {
            name: "Acme".to_owned(),
            legal_form: None,
            registration_number: None,
            vat_number: None,
            vat_exemption_basis: None,
            share_capital_cents: None,
            address: Default::default(),
            contact_email: None,
            contact_phone: None,
            insurance_mention: None,
            iban: Some("FR76".to_owned()),
            bic: None,
            late_penalty: "pénalité".to_owned(),
            recovery_fee_cents: 4000,
            invoice_footer: String::new(),
        });
        let mut repo = PgQuoteRepository::new(&tx);

        repo.insert(&stored).await?;
        let found = repo.find_by_id(stored.id).await?.unwrap();

        assert_eq!(found.valid_until, stored.valid_until);
        assert_eq!(found.deposit_percent, stored.deposit_percent);
        assert_eq!(found.owner_member_id, Some(owner));
        assert_eq!(found.notes, stored.notes);
        assert_eq!(found.billing_contact_id, stored.billing_contact_id);
        assert_eq!(found.billing_address_id, Some(seed.context));
        assert_eq!(found.issuer_snapshot, stored.issuer_snapshot);
        assert_eq!(found.recipient_snapshot, stored.recipient_snapshot);
        assert_eq!(
            found.lines[0].discount_percent,
            stored.lines[0].discount_percent
        );
        assert_eq!(found.lines[0].billing_frequency, BillingFrequency::Monthly);
        assert_eq!(found.superseded_by_id, None);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_quote_inserted_with_the_defaults_reads_back_as_before_the_migration() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let id = generate_uuid_v7();
        {
            let mut guard = tx.lock().await;
            sqlx::query(
                "INSERT INTO quotes (id, org_id, customer_id, customer_context_id, status, net_cents, gross_cents, title)
                 VALUES ($1, $2, $3, $4, 'SENT'::quote_status, 100, 100, 'Ancien devis')",
            )
            .bind(id)
            .bind(seed.organization.0)
            .bind(seed.customer.id.0)
            .bind(seed.context.0)
            .execute(&mut ***guard)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO quote_lines (id, org_id, quote_id, label, quantity, unit, unit_price_cents)
                 VALUES ($1, $2, $3, 'Pose', 1, 'HOUR', 100)",
            )
            .bind(generate_uuid_v7())
            .bind(seed.organization.0)
            .bind(id)
            .execute(&mut ***guard)
            .await
            .unwrap();
        }

        let found = PgQuoteRepository::new(&tx)
            .find_by_id(QuoteId(id))
            .await?
            .unwrap();

        assert_eq!(found.status, QuoteStatus::Sent);
        assert_eq!(found.deposit_percent, DepositPercent::ZERO);
        assert_eq!(found.valid_until, None);
        assert_eq!(found.issuer_snapshot, None);
        assert_eq!(found.lines[0].discount_percent, DiscountPercent::ZERO);
        assert_eq!(found.lines[0].billing_frequency, BillingFrequency::OneShot);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn updating_the_lifecycle_keeps_the_lines_and_a_revision_is_found_as_superseding() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let source = quote(&seed);
        let mut repo = PgQuoteRepository::new(&tx);
        repo.insert(&source).await?;
        let revision = Quote {
            revision_of_id: Some(source.id),
            lines: vec![line(&seed, QuoteId(Uuid::nil()))],
            ..Quote::draft(
                QuoteId(generate_uuid_v7()),
                seed.organization,
                seed.customer.id,
                seed.context,
                "Devis révisé".to_owned(),
                Utc::now(),
            )
        };
        let revision = Quote {
            lines: vec![line(&seed, revision.id)],
            ..revision
        };
        repo.insert(&revision).await?;

        let saved = repo
            .update_lifecycle(&Quote {
                status: QuoteStatus::Superseded,
                reference: Some("DEV-2026-0001".to_owned()),
                ..source.clone()
            })
            .await?;

        assert_eq!(saved.status, QuoteStatus::Superseded);
        assert_eq!(saved.lines.len(), 1);
        assert_eq!(saved.lines[0].id, source.lines[0].id);
        assert_eq!(saved.superseded_by_id, Some(revision.id));
        let reloaded = repo.find_by_id(revision.id).await?.unwrap();
        assert_eq!(reloaded.revision_of_id, Some(source.id));
        assert_eq!(reloaded.superseded_by_id, None);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_constraints_refuse_what_the_domain_refuses() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let other = seed_organization(&tx).await;
        let other_member = seed_member(&tx, other).await;
        let other_customer = seed_customer(&tx, other).await;
        let other_context = seed_context(&tx, other_customer.id).await;
        let other_contact = seed_contact(&tx, other_customer.id).await;
        let foreign_quote = quote(&Seed {
            organization: other,
            customer: other_customer.clone(),
            context: other_context,
        });
        let own = quote(&seed);
        {
            let mut repo = PgQuoteRepository::new(&tx);
            repo.insert(&own).await?;
            repo.insert(&foreign_quote).await?;
        }
        let id = own.id.0;
        let line_id = own.lines[0].id.0;

        let update = |set: &str| format!("UPDATE quotes SET {set} WHERE id = '{id}'");
        let update_line =
            |set: &str| format!("UPDATE quote_lines SET {set} WHERE id = '{line_id}'");

        for (name, statement) in [
            ("deposit above 100", update("deposit_percent = 100.5")),
            ("negative deposit", update("deposit_percent = -1")),
            (
                "owner of another organization",
                update(&format!("owner_member_id = '{}'", other_member.0)),
            ),
            (
                "billing contact of another customer",
                update(&format!("billing_contact_id = '{other_contact}'")),
            ),
            (
                "billing address of another customer",
                update(&format!("billing_address_id = '{}'", other_context.0)),
            ),
            (
                "revision of a quote of another organization",
                update(&format!("revision_of_id = '{}'", foreign_quote.id.0)),
            ),
            (
                "revision of itself",
                update(&format!("revision_of_id = '{id}'")),
            ),
            (
                "one snapshot without the other",
                update("issuer_snapshot = '{}'::jsonb"),
            ),
            (
                "blank signatory",
                update("responded_at = now(), responded_by = '  '"),
            ),
            (
                "blank decline reason",
                update("responded_at = now(), decline_reason = ''"),
            ),
            (
                "signatory without a date",
                update("responded_by = 'Mme Duval'"),
            ),
            ("blank notes", update("notes = ' '")),
            (
                "discount above 100",
                update_line("discount_percent = 100.01"),
            ),
            ("negative discount", update_line("discount_percent = -1")),
            (
                "unknown frequency",
                update_line("billing_frequency = 'WEEKLY'"),
            ),
        ] {
            assert!(refused(&tx, &statement).await, "{name} must be refused");
        }

        assert!(!refused(&tx, &update("deposit_percent = 100")).await);
        assert!(!refused(&tx, &update_line("discount_percent = 100")).await);
        assert!(!refused(&tx, &update_line("billing_frequency = 'YEARLY'")).await);
        assert!(
            !refused(
                &tx,
                &update(&format!(
                    "owner_member_id = '{}'",
                    seed_member(&tx, seed.organization).await.0
                ))
            )
            .await
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_line_cannot_reference_a_product_and_a_service_rate_together() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let own = quote(&seed);
        PgQuoteRepository::new(&tx).insert(&own).await?;
        let product = Uuid::new_v4();
        let rate = Uuid::new_v4();
        {
            let mut guard = tx.lock().await;
            sqlx::query(
                "INSERT INTO products (id, org_id, name, unit, unit_price_cents) VALUES ($1, $2, 'Vis', 'UNIT', 10)",
            )
            .bind(product)
            .bind(seed.organization.0)
            .execute(&mut ***guard)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO service_rates (id, org_id, label, unit, rate_cents) VALUES ($1, $2, 'Heure', 'HOUR', 10)",
            )
            .bind(rate)
            .bind(seed.organization.0)
            .execute(&mut ***guard)
            .await
            .unwrap();
        }
        let line_id = own.lines[0].id.0;

        let both = format!(
            "UPDATE quote_lines SET product_id = '{product}', service_rate_id = '{rate}' WHERE id = '{line_id}'"
        );
        let product_only =
            format!("UPDATE quote_lines SET product_id = '{product}' WHERE id = '{line_id}'");

        assert!(refused(&tx, &both).await);
        assert!(!refused(&tx, &product_only).await);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn reverting_the_migration_turns_superseded_quotes_back_into_sent_ones() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let superseded = Quote {
            status: QuoteStatus::Sent,
            reference: Some("DEV-2026-0042".to_owned()),
            ..quote(&seed)
        };
        PgQuoteRepository::new(&tx).insert(&superseded).await?;
        PgQuoteRepository::new(&tx)
            .update_lifecycle(&Quote {
                status: QuoteStatus::Superseded,
                ..superseded.clone()
            })
            .await?;

        let mut guard = tx.lock().await;
        sqlx::raw_sql("ALTER TABLE invoices DROP CONSTRAINT IF EXISTS fk_invoices_quote")
            .execute(&mut ***guard)
            .await
            .unwrap();
        sqlx::raw_sql(DOWN).execute(&mut ***guard).await.unwrap();
        let status: String = sqlx::query_scalar("SELECT status::text FROM quotes WHERE id = $1")
            .bind(superseded.id.0)
            .fetch_one(&mut ***guard)
            .await
            .unwrap();
        let labels: Vec<String> =
            sqlx::query_scalar("SELECT unnest(enum_range(NULL::quote_status))::text ORDER BY 1")
                .fetch_all(&mut ***guard)
                .await
                .unwrap();

        assert_eq!(status, "SENT");
        assert_eq!(
            labels,
            ["ACCEPTED", "CANCELLED", "DECLINED", "DRAFT", "SENT"]
        );
        Ok(())
    })
    .await;
}
