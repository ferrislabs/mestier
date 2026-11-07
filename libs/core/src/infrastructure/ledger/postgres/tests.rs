use chrono::{NaiveDate, Utc};
use common::{CoreError, UserId, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    Customer, CustomerContext, CustomerContextId, CustomerId, CustomerProfile, CustomerStatus,
    OrganizationId, PaymentMethod, Receipt, ReceiptId,
    application::test_support::dev_pool,
    domain::{
        customer::ports::CustomerRepository,
        customer_context::ports::CustomerContextRepository,
        ledger::ports::{LedgerRepository, ReceiptRepository},
    },
    infrastructure::{
        customer::postgres::PgCustomerRepository,
        customer_context::postgres::PgCustomerContextRepository,
        ledger::postgres::{PgLedgerRepository, PgReceiptRepository},
        postgres::{SharedTx, with_tx},
    },
};

fn day(month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, month, day).unwrap()
}

struct Seed {
    organization_id: OrganizationId,
    user_id: Uuid,
    customer: Customer,
    context: CustomerContext,
}

async fn seed(tx: &SharedTx<'_>) -> Seed {
    let user_id = generate_uuid_v7();
    let org_id = generate_uuid_v7();
    {
        let mut guard = tx.lock().await;
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
    }
    let organization_id = OrganizationId(org_id);
    let now = Utc::now();
    let customer = PgCustomerRepository::new(tx)
        .insert(&Customer {
            id: CustomerId(generate_uuid_v7()),
            organization_id,
            status: CustomerStatus::Client,
            name: "Boulangerie Lemoine".to_owned(),
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
        .unwrap();
    let context = PgCustomerContextRepository::new(tx)
        .insert(&CustomerContext {
            id: CustomerContextId(generate_uuid_v7()),
            customer_id: customer.id,
            label: "Site".to_owned(),
            kind: crate::AddressKind::Site,
            address_line: None,
            address_line2: None,
            postal_code: None,
            city: None,
            country: "FR".to_owned(),
            photo_key: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap();

    Seed {
        organization_id,
        user_id,
        customer,
        context,
    }
}

async fn seed_invoice(
    tx: &SharedTx<'_>,
    seed: &Seed,
    (kind, status, number): (&str, &str, &str),
    net: i32,
    gross: i32,
    issued_on: NaiveDate,
) -> Uuid {
    let id = generate_uuid_v7();
    let issued = status != "DRAFT";
    let mut guard = tx.lock().await;
    sqlx::query(
        r#"
        INSERT INTO invoices (id, org_id, kind, customer_id, customer_context_id, status, number, issued_at, issued_on, issuer_identity, net_cents, gross_cents, reason)
        VALUES ($1, $2, $3::invoice_kind, $4, $5, $6::invoice_status, $7, $8, $9, $10, $11, $12, $13)
        "#,
    )
    .bind(id)
    .bind(seed.organization_id.0)
    .bind(kind)
    .bind(seed.customer.id.0)
    .bind(seed.context.id.0)
    .bind(status)
    .bind(issued.then_some(number))
    .bind(issued.then(|| issued_on.and_hms_opt(9, 0, 0).unwrap().and_utc()))
    .bind(issued.then_some(issued_on))
    .bind(issued.then(|| serde_json::json!({})))
    .bind(net)
    .bind(gross)
    .bind((kind == "CREDIT_NOTE").then_some("Erreur"))
    .execute(&mut ***guard)
    .await
    .unwrap();

    id
}

async fn seed_payment(
    tx: &SharedTx<'_>,
    seed: &Seed,
    invoice_id: Uuid,
    amount_cents: i32,
    paid_on: NaiveDate,
    deleted: bool,
) {
    let mut guard = tx.lock().await;
    sqlx::query(
        r#"
        INSERT INTO invoice_payments (id, org_id, invoice_id, amount_cents, paid_on, method, recorded_by, deleted_at, deleted_by)
        VALUES ($1, $2, $3, $4, $5, 'TRANSFER', $6, $7, $8)
        "#,
    )
    .bind(generate_uuid_v7())
    .bind(seed.organization_id.0)
    .bind(invoice_id)
    .bind(amount_cents)
    .bind(paid_on)
    .bind(seed.user_id)
    .bind(deleted.then(Utc::now))
    .bind(deleted.then_some(seed.user_id))
    .execute(&mut ***guard)
    .await
    .unwrap();
}

fn receipt(seed: &Seed, amount_cents: i32, paid_on: NaiveDate) -> Receipt {
    Receipt {
        id: ReceiptId(generate_uuid_v7()),
        organization_id: seed.organization_id,
        customer_id: Some(seed.customer.id),
        amount_cents,
        paid_on,
        method: PaymentMethod::Check,
        reference: Some("CHQ 0042871".to_owned()),
        note: Some("Acompte".to_owned()),
        recorded_by: UserId(seed.user_id),
        created_at: Utc::now(),
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
async fn a_receipt_round_trips_and_is_deleted() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let mut repo = PgReceiptRepository::new(&tx);
        let stored = receipt(&seed, 49_000, day(11, 3));

        let inserted = repo.insert(&stored).await?;
        let found = repo.find_by_id(stored.id).await?.unwrap();
        repo.delete(stored.id).await?;

        assert_eq!(inserted.amount_cents, 49_000);
        assert_eq!(found.method, PaymentMethod::Check);
        assert_eq!(found.customer_id, Some(seed.customer.id));
        assert_eq!(found.note.as_deref(), Some("Acompte"));
        assert!(repo.find_by_id(stored.id).await?.is_none());
        assert!(matches!(
            repo.delete(stored.id).await,
            Err(CoreError::NotFound)
        ));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_journal_receipts_join_invoice_payments_refunds_and_unallocated_receipts() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let other = self::seed(&tx).await;
        let invoice = seed_invoice(
            &tx,
            &seed,
            ("STANDARD", "ISSUED", "F-2026-0001"),
            10_000,
            12_000,
            day(11, 1),
        )
        .await;
        seed_payment(&tx, &seed, invoice, 5_000, day(11, 2), false).await;
        seed_payment(&tx, &seed, invoice, -1_000, day(11, 3), false).await;
        seed_payment(&tx, &seed, invoice, 9_999, day(11, 4), true).await;
        PgReceiptRepository::new(&tx)
            .insert(&receipt(&seed, 4_000, day(11, 5)))
            .await?;
        let foreign_invoice = seed_invoice(
            &tx,
            &other,
            ("STANDARD", "ISSUED", "F-2026-0001"),
            1,
            1,
            day(11, 1),
        )
        .await;
        seed_payment(&tx, &other, foreign_invoice, 777, day(11, 2), false).await;
        let mut ledger = PgLedgerRepository::new(&tx);

        let mut rows = ledger.receipts(seed.organization_id).await?;
        rows.sort_by_key(|row| row.date);

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].amount_cents, 5_000);
        assert_eq!(rows[0].invoice_number.as_deref(), Some("F-2026-0001"));
        assert_eq!(
            rows[0].customer_name.as_deref(),
            Some("Boulangerie Lemoine")
        );
        assert_eq!(rows[1].amount_cents, -1_000);
        assert_eq!(rows[2].amount_cents, 4_000);
        assert!(rows[2].invoice_id.is_none());
        assert_eq!(rows[2].note.as_deref(), Some("Acompte"));
        assert_eq!(rows[2].method, PaymentMethod::Check);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn document_vat_counts_issued_invoices_and_credit_notes_but_not_drafts_or_cancelled() {
    run(async |tx| {
        let seed = seed(&tx).await;
        seed_invoice(
            &tx,
            &seed,
            ("STANDARD", "ISSUED", "F-2026-0001"),
            10_000,
            12_000,
            day(11, 1),
        )
        .await;
        seed_invoice(
            &tx,
            &seed,
            ("STANDARD", "PAID", "F-2026-0002"),
            5_000,
            5_500,
            day(10, 12),
        )
        .await;
        seed_invoice(
            &tx,
            &seed,
            ("STANDARD", "DRAFT", "-"),
            9_000,
            10_800,
            day(11, 1),
        )
        .await;
        seed_invoice(
            &tx,
            &seed,
            ("STANDARD", "CANCELLED", "F-2026-0003"),
            8_000,
            9_600,
            day(11, 1),
        )
        .await;
        seed_invoice(
            &tx,
            &seed,
            ("CREDIT_NOTE", "ISSUED", "AV-2026-0001"),
            1_000,
            1_200,
            day(11, 6),
        )
        .await;
        let mut ledger = PgLedgerRepository::new(&tx);

        let mut rows = ledger.document_vat(seed.organization_id).await?;
        rows.sort_by_key(|row| (row.date, row.vat_cents));

        assert_eq!(rows.len(), 3);
        assert_eq!(
            (rows[0].date, rows[0].vat_cents, rows[0].credit_note),
            (day(10, 12), 500, false)
        );
        assert_eq!(
            (rows[1].date, rows[1].vat_cents, rows[1].credit_note),
            (day(11, 1), 2_000, false)
        );
        assert_eq!(
            (rows[2].date, rows[2].vat_cents, rows[2].credit_note),
            (day(11, 6), 200, true)
        );
        Ok(())
    })
    .await;
}
