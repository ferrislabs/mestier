use chrono::{NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    Customer, CustomerId, CustomerProfile, CustomerStatus, Expense, ExpenseCategory,
    ExpenseCategoryId, ExpenseId, ExpenseStatus, MemberId, OrganizationId, PaymentMethod, Supplier,
    SupplierId, SupplierProfile,
    application::test_support::dev_pool,
    domain::{
        customer::ports::CustomerRepository,
        expense::ports::{ExpenseCategoryRepository, ExpenseRepository},
        supplier_invoice::ports::SupplierRepository,
    },
    infrastructure::{
        customer::postgres::PgCustomerRepository,
        expense::postgres::{PgExpenseCategoryRepository, PgExpenseRepository},
        postgres::{SharedTx, with_tx},
        supplier_invoice::postgres::supplier_repository::PgSupplierRepository,
    },
};

fn day(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, day).unwrap()
}

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

async fn seed_customer(tx: &SharedTx<'_>, organization_id: OrganizationId) -> Customer {
    let now = Utc::now();
    PgCustomerRepository::new(tx)
        .insert(&Customer {
            id: CustomerId(generate_uuid_v7()),
            organization_id,
            status: CustomerStatus::Client,
            name: "Atelier Bois".to_owned(),
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

async fn seed_supplier(tx: &SharedTx<'_>, organization_id: OrganizationId) -> Supplier {
    let now = Utc::now();
    PgSupplierRepository::new(tx)
        .insert(&Supplier {
            id: SupplierId(generate_uuid_v7()),
            organization_id,
            name: "Quincaillerie Bernard".to_owned(),
            registration_number: None,
            vat_number: None,
            address: None,
            phone: None,
            email: None,
            profile: SupplierProfile::default(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap()
}

async fn seed_category(
    tx: &SharedTx<'_>,
    organization_id: OrganizationId,
    name: &str,
) -> ExpenseCategory {
    let now = Utc::now();
    PgExpenseCategoryRepository::new(tx)
        .insert(&ExpenseCategory {
            id: ExpenseCategoryId(generate_uuid_v7()),
            organization_id,
            name: name.to_owned(),
            hue: 60,
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap()
}

fn expense(
    organization_id: OrganizationId,
    category_id: ExpenseCategoryId,
    date: NaiveDate,
) -> Expense {
    let now = Utc::now();
    Expense {
        id: ExpenseId(generate_uuid_v7()),
        organization_id,
        date,
        label: "Visserie".to_owned(),
        supplier_id: None,
        category_id,
        reference: Some("FA-88213".to_owned()),
        amount_ht_cents: 84_200,
        vat_cents: 16_840,
        due_date: Some(day(30)),
        status: ExpenseStatus::ToPay,
        paid_on: None,
        method: None,
        advanced_by_member_id: None,
        reimbursed_on: None,
        customer_id: None,
        proof_file_key: None,
        note: None,
        created_at: now,
        updated_at: now,
        supplier_name: None,
        customer_name: None,
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

async fn refused(tx: &SharedTx<'_>, statement: &str, id: Uuid) -> bool {
    let mut guard = tx.lock().await;
    sqlx::query("SAVEPOINT attempt")
        .execute(&mut ***guard)
        .await
        .unwrap();
    let outcome = sqlx::query(statement).bind(id).execute(&mut ***guard).await;
    sqlx::query("ROLLBACK TO SAVEPOINT attempt")
        .execute(&mut ***guard)
        .await
        .unwrap();
    outcome.is_err()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_expense_round_trips_with_its_supplier_and_customer_names() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let category = seed_category(&tx, org, "Matériaux").await;
        let supplier = seed_supplier(&tx, org).await;
        let customer = seed_customer(&tx, org).await;
        let mut stored = expense(org, category.id, day(5));
        stored.supplier_id = Some(supplier.id);
        stored.customer_id = Some(customer.id);
        stored.proof_file_key = Some("proofs/a.pdf".to_owned());
        stored.note = Some("Chantier Tilleuls".to_owned());
        let mut repo = PgExpenseRepository::new(&tx);

        let inserted = repo.insert(&stored).await?;
        let found = repo.find_by_id(stored.id).await?.unwrap();

        assert_eq!(
            inserted.supplier_name.as_deref(),
            Some("Quincaillerie Bernard")
        );
        assert_eq!(found.customer_name.as_deref(), Some("Atelier Bois"));
        assert_eq!(found.amount_ht_cents, 84_200);
        assert_eq!(found.vat_cents, 16_840);
        assert_eq!(found.total_cents(), 101_040);
        assert_eq!(found.status, ExpenseStatus::ToPay);
        assert_eq!(found.proof_file_key.as_deref(), Some("proofs/a.pdf"));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn expenses_are_listed_by_organization_newest_first() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let category = seed_category(&tx, org, "Matériaux").await;
        let other_category = seed_category(&tx, other, "Matériaux").await;
        let mut repo = PgExpenseRepository::new(&tx);
        let old = repo.insert(&expense(org, category.id, day(2))).await?;
        let recent = repo.insert(&expense(org, category.id, day(9))).await?;
        repo.insert(&expense(other, other_category.id, day(5)))
            .await?;

        let listed = repo.list_by_organization(org).await?;

        assert_eq!(
            listed.iter().map(|e| e.id).collect::<Vec<_>>(),
            vec![recent.id, old.id]
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_expense_is_marked_paid_once_and_only_when_it_is_to_pay_and_not_advanced() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let category = seed_category(&tx, org, "Matériaux").await;
        let mut repo = PgExpenseRepository::new(&tx);
        let to_pay = repo.insert(&expense(org, category.id, day(2))).await?;
        let mut advanced = expense(org, category.id, day(2));
        advanced.status = ExpenseStatus::Paid;
        advanced.paid_on = Some(day(2));
        advanced.method = Some(PaymentMethod::Card);
        advanced.due_date = None;
        advanced.advanced_by_member_id = Some(member);
        let advanced = repo.insert(&advanced).await?;

        let paid = repo
            .mark_paid(to_pay.id, day(9), PaymentMethod::Transfer, Utc::now())
            .await?
            .unwrap();
        let again = repo
            .mark_paid(to_pay.id, day(10), PaymentMethod::Cash, Utc::now())
            .await?;
        let refused = repo
            .mark_paid(advanced.id, day(10), PaymentMethod::Cash, Utc::now())
            .await?;

        assert_eq!(paid.status, ExpenseStatus::Paid);
        assert_eq!(paid.paid_on, Some(day(9)));
        assert_eq!(paid.method, Some(PaymentMethod::Transfer));
        assert!(again.is_none());
        assert!(refused.is_none());
        let stored = repo.find_by_id(to_pay.id).await?.unwrap();
        assert_eq!(stored.paid_on, Some(day(9)));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_advanced_expense_is_reimbursed_once_with_the_first_date_kept() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let member = seed_member(&tx, org).await;
        let category = seed_category(&tx, org, "Outillage").await;
        let mut repo = PgExpenseRepository::new(&tx);
        let mut advanced = expense(org, category.id, day(2));
        advanced.status = ExpenseStatus::Paid;
        advanced.paid_on = Some(day(2));
        advanced.method = Some(PaymentMethod::Card);
        advanced.due_date = None;
        advanced.advanced_by_member_id = Some(member);
        let advanced = repo.insert(&advanced).await?;
        let plain = repo.insert(&expense(org, category.id, day(2))).await?;

        let first = repo
            .reimburse(advanced.id, day(8), Utc::now())
            .await?
            .unwrap();
        let second = repo.reimburse(advanced.id, day(9), Utc::now()).await?;
        let not_advanced = repo.reimburse(plain.id, day(9), Utc::now()).await?;

        assert_eq!(first.reimbursed_on, Some(day(8)));
        assert!(second.is_none());
        assert!(not_advanced.is_none());
        assert_eq!(
            repo.find_by_id(advanced.id).await?.unwrap().reimbursed_on,
            Some(day(8))
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_refuses_an_expense_that_breaks_a_rule() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let category = seed_category(&tx, org, "Matériaux").await;
        let stored = PgExpenseRepository::new(&tx)
            .insert(&expense(org, category.id, day(5)))
            .await?;

        for statement in [
            "UPDATE expenses SET amount_ht_cents = 0 WHERE id = $1",
            "UPDATE expenses SET vat_cents = -1 WHERE id = $1",
            "UPDATE expenses SET due_date = date - 1 WHERE id = $1",
            "UPDATE expenses SET status = 'PAID' WHERE id = $1",
            "UPDATE expenses SET reimbursed_on = date WHERE id = $1",
            "UPDATE expenses SET label = '  ' WHERE id = $1",
        ] {
            assert!(refused(&tx, statement, stored.id.0).await, "{statement}");
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_category_of_another_organization_cannot_be_referenced() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let foreign = seed_category(&tx, other, "Matériaux").await;
        let mut repo = PgExpenseRepository::new(&tx);
        let own = seed_category(&tx, org, "Autres").await;
        let inserted = repo.insert(&expense(org, own.id, day(5))).await?;

        let statement = format!(
            "UPDATE expenses SET category_id = '{}' WHERE id = $1",
            foreign.id.0
        );

        assert!(refused(&tx, &statement, inserted.id.0).await);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_category_is_in_use_when_an_expense_or_a_supplier_points_at_it() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let used = seed_category(&tx, org, "Matériaux").await;
        let unused = seed_category(&tx, org, "Autres").await;
        PgExpenseRepository::new(&tx)
            .insert(&expense(org, used.id, day(5)))
            .await?;
        let mut categories = PgExpenseCategoryRepository::new(&tx);

        assert!(categories.is_used(used.id).await?);
        assert!(!categories.is_used(unused.id).await?);
        categories.delete(unused.id).await?;
        assert!(categories.find_by_id(unused.id).await?.is_none());
        assert!(matches!(
            categories.delete(unused.id).await,
            Err(CoreError::NotFound)
        ));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn two_categories_of_one_organization_cannot_share_a_name_ignoring_case() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        seed_category(&tx, org, "Autres").await;
        seed_category(&tx, other, "Autres").await;
        let now = Utc::now();

        let duplicate = PgExpenseCategoryRepository::new(&tx)
            .insert(&ExpenseCategory {
                id: ExpenseCategoryId(generate_uuid_v7()),
                organization_id: org,
                name: "AUTRES".to_owned(),
                hue: 10,
                created_at: now,
                updated_at: now,
            })
            .await;

        assert!(matches!(duplicate, Err(CoreError::Conflict(_))));
        Ok(())
    })
    .await;
}
