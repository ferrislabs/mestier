use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AddressKind, BillingFrequency, CatalogProfile, CatalogUsage, Customer, CustomerContext,
    CustomerContextId, CustomerId, CustomerProfile, CustomerStatus, OrganizationId, Product,
    ProductId, ServiceRate, ServiceRateId,
    application::test_support::dev_pool,
    domain::{
        customer::ports::CustomerRepository, customer_context::ports::CustomerContextRepository,
        product::ports::ProductRepository, service_rate::ports::ServiceRateRepository,
    },
    infrastructure::{
        customer::postgres::PgCustomerRepository,
        customer_context::postgres::PgCustomerContextRepository,
        postgres::{SharedTx, with_tx},
        product::postgres::repository::PgProductRepository,
        service_rate::postgres::PgServiceRateRepository,
    },
};

struct Seed {
    organization_id: OrganizationId,
    customer_id: CustomerId,
    context_id: CustomerContextId,
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
            name: "Lemoine".to_owned(),
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
            kind: AddressKind::Site,
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
        customer_id: customer.id,
        context_id: context.id,
    }
}

fn product(seed: &Seed, name: &str) -> Product {
    let now = Utc::now();
    Product {
        id: ProductId(generate_uuid_v7()),
        organization_id: seed.organization_id,
        name: name.to_owned(),
        sku: None,
        unit: "u".to_owned(),
        unit_price_cents: 1_200,
        default_vat_rate_bp: Some(2_000),
        description: None,
        photo_keys: vec![],
        supplier_id: None,
        manufacturer: None,
        technical_specs: None,
        datasheet_keys: vec![],
        catalog: CatalogProfile {
            category: "Plantes".to_owned(),
            billing_frequency: BillingFrequency::Monthly,
            cost_cents: Some(700),
            archived_at: None,
        },
        usage: CatalogUsage::default(),
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn service_rate(seed: &Seed, label: &str) -> ServiceRate {
    let now = Utc::now();
    ServiceRate {
        id: ServiceRateId(generate_uuid_v7()),
        organization_id: seed.organization_id,
        label: label.to_owned(),
        unit: "HOUR".to_owned(),
        rate_cents: 5_500,
        default_vat_rate_bp: None,
        description: None,
        catalog: CatalogProfile::default(),
        usage: CatalogUsage::default(),
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

async fn quote_line(
    tx: &SharedTx<'_>,
    seed: &Seed,
    product: Option<Uuid>,
    rate: Option<Uuid>,
    deleted_quote: bool,
) {
    let quote_id = generate_uuid_v7();
    let mut guard = tx.lock().await;
    sqlx::query(
        r#"INSERT INTO quotes (id, org_id, customer_id, customer_context_id, status, net_cents, gross_cents, title, deleted_at)
           VALUES ($1, $2, $3, $4, 'DRAFT'::quote_status, 0, 0, 'Devis', $5)"#,
    )
    .bind(quote_id)
    .bind(seed.organization_id.0)
    .bind(seed.customer_id.0)
    .bind(seed.context_id.0)
    .bind(deleted_quote.then(Utc::now))
    .execute(&mut ***guard)
    .await
    .unwrap();
    sqlx::query(
        r#"INSERT INTO quote_lines (id, org_id, quote_id, product_id, service_rate_id, label, quantity, unit, unit_price_cents)
           VALUES ($1, $2, $3, $4, $5, 'Ligne copiée', 2, 'u', 1200)"#,
    )
    .bind(generate_uuid_v7())
    .bind(seed.organization_id.0)
    .bind(quote_id)
    .bind(product)
    .bind(rate)
    .execute(&mut ***guard)
    .await
    .unwrap();
}

async fn invoice_line(tx: &SharedTx<'_>, seed: &Seed, product: Option<Uuid>, rate: Option<Uuid>) {
    let invoice_id = generate_uuid_v7();
    let mut guard = tx.lock().await;
    sqlx::query(
        r#"INSERT INTO invoices (id, org_id, kind, customer_id, customer_context_id, status)
           VALUES ($1, $2, 'STANDARD'::invoice_kind, $3, $4, 'DRAFT'::invoice_status)"#,
    )
    .bind(invoice_id)
    .bind(seed.organization_id.0)
    .bind(seed.customer_id.0)
    .bind(seed.context_id.0)
    .execute(&mut ***guard)
    .await
    .unwrap();
    sqlx::query(
        r#"INSERT INTO invoice_lines (id, org_id, invoice_id, product_id, service_rate_id, label, quantity, unit_price_cents, position)
           VALUES ($1, $2, $3, $4, $5, 'Ligne copiée', 2, 1200, 0)"#,
    )
    .bind(generate_uuid_v7())
    .bind(seed.organization_id.0)
    .bind(invoice_id)
    .bind(product)
    .bind(rate)
    .execute(&mut ***guard)
    .await
    .unwrap();
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
async fn a_product_round_trips_its_catalogue_profile() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let mut repo = PgProductRepository::new(&tx);
        let mut stored = product(&seed, "Rosier");
        stored.catalog.archived_at = Some(Utc::now());

        let inserted = repo.insert(&stored).await?;
        let found = repo.find_by_id(stored.id).await?.unwrap();

        assert_eq!(inserted.catalog.category, "Plantes");
        assert_eq!(found.catalog.billing_frequency, BillingFrequency::Monthly);
        assert_eq!(found.catalog.cost_cents, Some(700));
        assert!(found.catalog.archived_at.is_some());
        assert_eq!(found.usage, CatalogUsage::default());

        let mut changed = found.clone();
        changed.catalog = CatalogProfile::default();
        let updated = repo.update(&changed).await?;
        assert_eq!(updated.catalog, CatalogProfile::default());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_usage_count_adds_the_quote_lines_and_the_invoice_lines_of_live_documents() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let mut products = PgProductRepository::new(&tx);
        let used = products.insert(&product(&seed, "Rosier")).await?;
        let unused = products.insert(&product(&seed, "Lavande")).await?;
        let mut rates = PgServiceRateRepository::new(&tx);
        let rate = rates.insert(&service_rate(&seed, "Taille")).await?;
        quote_line(&tx, &seed, Some(used.id.0), None, false).await;
        quote_line(&tx, &seed, Some(used.id.0), None, false).await;
        quote_line(&tx, &seed, Some(used.id.0), None, true).await;
        invoice_line(&tx, &seed, Some(used.id.0), None).await;
        quote_line(&tx, &seed, None, Some(rate.id.0), false).await;
        invoice_line(&tx, &seed, None, Some(rate.id.0)).await;
        invoice_line(&tx, &seed, None, Some(rate.id.0)).await;
        let mut products = PgProductRepository::new(&tx);
        let mut rates = PgServiceRateRepository::new(&tx);

        let used = products.find_by_id(used.id).await?.unwrap();
        let unused = products.find_by_id(unused.id).await?.unwrap();
        let rate = rates.find_by_id(rate.id).await?.unwrap();
        let (listed, _) = products
            .list_by_organization(seed.organization_id, 10, 0)
            .await?;

        assert_eq!(
            used.usage,
            CatalogUsage {
                quote_lines: 2,
                invoice_lines: 1
            }
        );
        assert_eq!(used.usage.total(), 3);
        assert_eq!(unused.usage, CatalogUsage::default());
        assert_eq!(
            rate.usage,
            CatalogUsage {
                quote_lines: 1,
                invoice_lines: 2
            }
        );
        assert_eq!(
            listed
                .iter()
                .find(|p| p.name == "Rosier")
                .unwrap()
                .usage
                .total(),
            3
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn editing_a_catalogue_item_leaves_the_lines_already_copied_unchanged() {
    run(async |tx| {
        let seed = seed(&tx).await;
        let mut repo = PgProductRepository::new(&tx);
        let stored = repo.insert(&product(&seed, "Rosier")).await?;
        quote_line(&tx, &seed, Some(stored.id.0), None, false).await;
        invoice_line(&tx, &seed, Some(stored.id.0), None).await;
        let mut repo = PgProductRepository::new(&tx);
        let mut edited = stored.clone();
        edited.name = "Rosier grimpant".to_owned();
        edited.unit_price_cents = 9_900;
        edited.unit = "m".to_owned();
        edited.catalog.cost_cents = Some(1);

        repo.update(&edited).await?;

        let mut guard = tx.lock().await;
        let quote: (String, String, i32) = sqlx::query_as(
            "SELECT label, unit, unit_price_cents FROM quote_lines WHERE product_id = $1",
        )
        .bind(stored.id.0)
        .fetch_one(&mut ***guard)
        .await
        .unwrap();
        let invoice: (String, i32) = sqlx::query_as(
            "SELECT label, unit_price_cents FROM invoice_lines WHERE product_id = $1",
        )
        .bind(stored.id.0)
        .fetch_one(&mut ***guard)
        .await
        .unwrap();
        assert_eq!(quote, ("Ligne copiée".to_owned(), "u".to_owned(), 1_200));
        assert_eq!(invoice, ("Ligne copiée".to_owned(), 1_200));
        Ok(())
    })
    .await;
}
