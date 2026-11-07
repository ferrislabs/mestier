use chrono::{Days, Utc};
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    AddressKind, Customer, CustomerContext, CustomerContextId, CustomerId, CustomerProfile,
    CustomerStatus, Deal, DealAmount, DealId, DealStage, DealStageName, DealTitle, LostReason,
    MemberId, OrganizationId, Quote, QuoteId,
    application::test_support::dev_pool,
    domain::{
        customer::ports::CustomerRepository,
        customer_context::ports::CustomerContextRepository,
        deal::{commands::DealFilter, ports::DealRepository},
        quote::ports::QuoteRepository,
    },
    infrastructure::{
        customer::postgres::PgCustomerRepository,
        customer_context::postgres::PgCustomerContextRepository,
        deal::postgres::PgDealRepository,
        postgres::{SharedTx, with_tx},
        quote::postgres::PgQuoteRepository,
    },
};

const MIGRATION: &str =
    include_str!("../../../../../../migrations/20261109000003_create_deals.up.sql");

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

async fn seed_customer(tx: &SharedTx<'_>, organization_id: OrganizationId, name: &str) -> Customer {
    let now = Utc::now();
    let customer = Customer {
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
    };
    PgCustomerRepository::new(tx)
        .insert(&customer)
        .await
        .unwrap()
}

fn deal(organization_id: OrganizationId, customer_id: CustomerId, title: &str) -> Deal {
    let now = Utc::now();
    Deal {
        id: DealId(generate_uuid_v7()),
        organization_id,
        customer_id,
        title: DealTitle::try_from(title).unwrap(),
        amount: DealAmount::try_from(250_000).unwrap(),
        stage: DealStage::New,
        expected_close_date: None,
        owner_member_id: None,
        source: None,
        notes: None,
        created_at: now,
        stage_changed_at: now,
        closed_at: None,
        quote_count: 0,
    }
}

async fn seed_quote(tx: &SharedTx<'_>, deal: &Deal) -> Uuid {
    let now = Utc::now();
    let context = PgCustomerContextRepository::new(tx)
        .insert(&CustomerContext {
            id: CustomerContextId(generate_uuid_v7()),
            customer_id: deal.customer_id,
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
    let id = generate_uuid_v7();
    let mut guard = tx.lock().await;
    sqlx::query!(
        r#"
        INSERT INTO quotes (id, org_id, customer_id, customer_context_id, deal_id, status, net_cents, gross_cents, title)
        VALUES ($1, $2, $3, $4, $5, 'DRAFT'::quote_status, 0, 0, 'Devis')
        "#,
        id,
        deal.organization_id.0,
        deal.customer_id.0,
        context.id.0,
        deal.id.0,
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    id
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

async fn refused(tx: &SharedTx<'_>, statement: &str, id: Uuid, org: Uuid) -> bool {
    let mut guard = tx.lock().await;
    sqlx::query("SAVEPOINT attempt")
        .execute(&mut ***guard)
        .await
        .unwrap();
    let outcome = sqlx::query(statement)
        .bind(id)
        .bind(org)
        .execute(&mut ***guard)
        .await;
    sqlx::query("ROLLBACK TO SAVEPOINT attempt")
        .execute(&mut ***guard)
        .await
        .unwrap();
    outcome.is_err()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_deal_round_trips_with_its_stage_and_zero_quotes() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let owner = seed_member(&tx, org).await;
        let customer = seed_customer(&tx, org, "Boulangerie").await;
        let mut stored = deal(org, customer.id, "Fournil");
        stored.owner_member_id = Some(owner);
        stored.source = Some(crate::LeadSource::Website);
        stored.notes = Some("Visite jeudi".to_owned());
        let mut repo = PgDealRepository::new(&tx);

        let inserted = repo.insert(&stored).await?;
        let found = repo.find_by_id(stored.id).await?.unwrap();

        assert_eq!(inserted.id, stored.id);
        assert_eq!(found.title.as_str(), "Fournil");
        assert_eq!(found.stage, DealStage::New);
        assert_eq!(found.owner_member_id, Some(owner));
        assert_eq!(found.quote_count, 0);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_lost_deal_keeps_its_reason_and_close_date() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let customer = seed_customer(&tx, org, "Boulangerie").await;
        let mut stored = deal(org, customer.id, "Fournil");
        let mut repo = PgDealRepository::new(&tx);
        repo.insert(&stored).await?;
        let lost = DealStage::try_from((DealStageName::Lost, Some("Prix trop élevé"))).unwrap();
        stored.move_to(lost, Utc::now());

        let updated = repo.update(&stored).await?;

        assert_eq!(updated.stage.name(), DealStageName::Lost);
        assert_eq!(
            updated.stage.lost_reason().map(LostReason::as_str),
            Some("Prix trop élevé")
        );
        assert!(updated.closed_at.is_some());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_constraints_refuse_what_the_domain_refuses() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let customer = seed_customer(&tx, org, "Boulangerie").await;
        let other_org = seed_organization(&tx).await;
        let stranger = seed_customer(&tx, other_org, "Étranger").await;
        let ok = deal(org, customer.id, "Fournil");
        PgDealRepository::new(&tx).insert(&ok).await?;

        let cases = [
            "UPDATE deals SET stage = 'LOST', closed_at = now() WHERE id = $1 AND org_id = $2",
            "UPDATE deals SET stage = 'LOST', lost_reason = '  ', closed_at = now() WHERE id = $1 AND org_id = $2",
            "UPDATE deals SET stage = 'WON' WHERE id = $1 AND org_id = $2",
            "UPDATE deals SET closed_at = now() WHERE id = $1 AND org_id = $2",
            "UPDATE deals SET lost_reason = 'x' WHERE id = $1 AND org_id = $2",
            "UPDATE deals SET amount_cents = -1 WHERE id = $1 AND org_id = $2",
            "UPDATE deals SET title = '  ' WHERE id = $1 AND org_id = $2",
            "UPDATE deals SET stage = 'CONTACTED' WHERE id = $1 AND org_id = $2",
            "UPDATE deals SET source = 'tv' WHERE id = $1 AND org_id = $2",
        ];
        for statement in cases {
            assert!(refused(&tx, statement, ok.id.0, org.0).await, "{statement}");
        }

        let mut foreign_customer = deal(org, stranger.id, "Intrus");
        foreign_customer.customer_id = stranger.id;
        let result = PgDealRepository::new(&tx).insert(&foreign_customer).await;
        assert!(result.is_err());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_of_another_organization_cannot_own_a_deal() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other_org = seed_organization(&tx).await;
        let foreign_member = seed_member(&tx, other_org).await;
        let customer = seed_customer(&tx, org, "Boulangerie").await;
        let mut stored = deal(org, customer.id, "Fournil");
        stored.owner_member_id = Some(foreign_member);

        let result = PgDealRepository::new(&tx).insert(&stored).await;

        assert!(result.is_err());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_quote_count_ignores_deleted_quotes_and_a_deleted_quote_does_not_block_the_delete() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let customer = seed_customer(&tx, org, "Boulangerie").await;
        let stored = deal(org, customer.id, "Fournil");
        let mut repo = PgDealRepository::new(&tx);
        repo.insert(&stored).await?;
        let quote = seed_quote(&tx, &stored).await;

        let linked = repo.find_by_id(stored.id).await?.unwrap();
        assert_eq!(linked.quote_count, 1);

        {
            let mut guard = tx.lock().await;
            sqlx::query!("UPDATE quotes SET deleted_at = now() WHERE id = $1", quote)
                .execute(&mut ***guard)
                .await
                .unwrap();
        }
        let freed = repo.find_by_id(stored.id).await?.unwrap();
        assert_eq!(freed.quote_count, 0);

        repo.delete(stored.id).await?;

        assert!(repo.find_by_id(stored.id).await?.is_none());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_itself_refuses_to_delete_a_deal_that_has_a_live_quote() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let customer = seed_customer(&tx, org, "Boulangerie").await;
        let stored = deal(org, customer.id, "Fournil");
        PgDealRepository::new(&tx).insert(&stored).await?;
        seed_quote(&tx, &stored).await;

        let refused = refused(
            &tx,
            "DELETE FROM deals WHERE id = $1 AND org_id = $2",
            stored.id.0,
            org.0,
        )
        .await;

        assert!(refused);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn losing_the_open_deals_of_a_customer_leaves_the_closed_ones() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let customer = seed_customer(&tx, org, "Boulangerie").await;
        let other = seed_customer(&tx, org, "Autre").await;
        let mut repo = PgDealRepository::new(&tx);
        let open = deal(org, customer.id, "Ouverte");
        let mut won = deal(org, customer.id, "Gagnée");
        won.move_to(DealStage::Won, Utc::now());
        let untouched = deal(org, other.id, "Autre client");
        for item in [&open, &won, &untouched] {
            repo.insert(item).await?;
        }

        let changed = repo
            .lose_open_of_customer(customer.id, &LostReason::archived_account(), Utc::now())
            .await?;

        assert_eq!(changed, 1);
        let lost = repo.find_by_id(open.id).await?.unwrap();
        assert_eq!(lost.stage.name(), DealStageName::Lost);
        assert_eq!(
            lost.stage.lost_reason().map(LostReason::as_str),
            Some("Compte archivé")
        );
        assert!(lost.closed_at.is_some());
        assert_eq!(
            repo.find_by_id(won.id).await?.unwrap().stage,
            DealStage::Won
        );
        assert_eq!(
            repo.find_by_id(untouched.id).await?.unwrap().stage,
            DealStage::New
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_list_filters_by_customer_stage_and_text() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let lemoine = seed_customer(&tx, org, "Boulangerie Lemoine").await;
        let martin = seed_customer(&tx, org, "Syndic Martin").await;
        let mut repo = PgDealRepository::new(&tx);
        let first = deal(org, lemoine.id, "Fournil 100%");
        let mut second = deal(org, martin.id, "Toiture");
        second.move_to(DealStage::Won, Utc::now());
        repo.insert(&first).await?;
        repo.insert(&second).await?;
        let filter = |customer_id, stage, query: Option<&str>| DealFilter {
            organization_id: org,
            customer_id,
            stage,
            query: query.map(str::to_owned),
        };

        let (all, total) = repo.list(&filter(None, None, None), 10, 0).await?;
        let (by_customer, _) = repo
            .list(&filter(Some(martin.id), None, None), 10, 0)
            .await?;
        let (by_stage, _) = repo
            .list(&filter(None, Some(DealStageName::Won), None), 10, 0)
            .await?;
        let (by_title, _) = repo
            .list(&filter(None, None, Some("fournil")), 10, 0)
            .await?;
        let (by_customer_name, _) = repo
            .list(&filter(None, None, Some("syndic")), 10, 0)
            .await?;
        let (percent, _) = repo.list(&filter(None, None, Some("%")), 10, 0).await?;
        let (page, page_total) = repo.list(&filter(None, None, None), 1, 1).await?;

        assert_eq!((all.len(), total), (2, 2));
        assert_eq!(by_customer.len(), 1);
        assert_eq!(by_stage[0].title.as_str(), "Toiture");
        assert_eq!(by_title[0].title.as_str(), "Fournil 100%");
        assert_eq!(by_customer_name[0].title.as_str(), "Toiture");
        assert_eq!(percent.len(), 1);
        assert_eq!((page.len(), page_total), (1, 2));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_totals_group_by_stage_and_the_closed_ones_by_window() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let customer = seed_customer(&tx, org, "Boulangerie").await;
        let mut repo = PgDealRepository::new(&tx);
        let now = Utc::now();
        let mut recent = deal(org, customer.id, "Récente");
        recent.move_to(DealStage::Won, now);
        let mut old = deal(org, customer.id, "Ancienne");
        old.move_to(
            DealStage::Won,
            now.checked_sub_days(Days::new(120)).unwrap(),
        );
        let open = deal(org, customer.id, "Ouverte");
        for item in [&recent, &old, &open] {
            repo.insert(item).await?;
        }

        let by_stage = repo.stage_totals(org).await?;
        let closed = repo
            .closed_totals_since(org, now.checked_sub_days(Days::new(90)).unwrap())
            .await?;

        let won = by_stage
            .iter()
            .find(|total| total.stage == DealStageName::Won)
            .unwrap();
        assert_eq!((won.count, won.amount_cents), (2, 500_000));
        assert_eq!(closed.len(), 1);
        assert_eq!((closed[0].count, closed[0].amount_cents), (1, 250_000));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn conversions_are_counted_from_the_conversion_date() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut customers = PgCustomerRepository::new(&tx);
        let now = Utc::now();
        for (name, days) in [("Récent", 10_u64), ("Ancien", 120)] {
            let mut customer = seed_customer(&tx, org, name).await;
            customer.change_status(
                CustomerStatus::Client,
                now.checked_sub_days(Days::new(days)).unwrap(),
            );
            customers.update(&customer).await?;
        }

        let count = customers
            .count_converted_since(org, now.checked_sub_days(Days::new(90)).unwrap())
            .await?;

        assert_eq!(count, 1);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_migration_creates_one_deal_per_active_customer_with_the_mapped_stage() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let owner = seed_member(&tx, org).await;
        let seeded_at = Utc::now().checked_sub_days(Days::new(5)).unwrap();
        let cases = [
            ("Nouveau", "PROSPECT", "NEW", false),
            ("Contacté", "PROSPECT", "CONTACTED", false),
            ("Qualifié", "PROSPECT", "QUALIFIED", false),
            ("Devis", "PROSPECT", "QUOTE_SENT", false),
            ("Gagné", "CLIENT", "WON", false),
            ("Perdu", "CLIENT", "LOST", false),
            ("Archivé", "ARCHIVED", "LOST", false),
            ("Supprimé", "CLIENT", "WON", true),
        ];
        {
            let mut guard = tx.lock().await;
            for (name, status, stage, deleted) in cases {
                sqlx::query(
                    r#"
                    INSERT INTO customers (id, org_id, status, pipeline_stage, name, owner_member_id, created_at, updated_at, deleted_at)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $7, CASE WHEN $8 THEN $7 END)
                    "#,
                )
                .bind(generate_uuid_v7())
                .bind(org.0)
                .bind(status)
                .bind(stage)
                .bind(name)
                .bind(owner.0)
                .bind(seeded_at)
                .bind(deleted)
                .execute(&mut ***guard)
                .await
                .unwrap();
            }
            sqlx::query("DELETE FROM deals WHERE org_id <> $1")
                .bind(org.0)
                .execute(&mut ***guard)
                .await
                .unwrap();
            let start = MIGRATION.find("INSERT INTO deals (").unwrap();
            let end = start + MIGRATION[start..].find(';').unwrap();
            sqlx::query(&MIGRATION[start..end])
                .execute(&mut ***guard)
                .await
                .unwrap();
        }

        let (deals, total) = PgDealRepository::new(&tx)
            .list(
                &DealFilter {
                    organization_id: org,
                    customer_id: None,
                    stage: None,
                    query: None,
                },
                50,
                0,
            )
            .await?;

        let mut found: Vec<_> = deals
            .iter()
            .map(|deal| {
                (
                    deal.title.as_str().to_owned(),
                    deal.stage.name(),
                    deal.stage.lost_reason().map(|reason| reason.as_str().to_owned()),
                    deal.closed_at == Some(seeded_at) && deal.stage_changed_at == seeded_at,
                    deal.owner_member_id,
                )
            })
            .collect();
        found.sort_by(|a, b| a.0.cmp(&b.0));
        let reason = Some("Non renseigné".to_owned());
        let mut expected = vec![
            ("Opportunité Contacté".to_owned(), DealStageName::Visit, None, false, Some(owner)),
            ("Opportunité Devis".to_owned(), DealStageName::QuoteSent, None, false, Some(owner)),
            ("Opportunité Gagné".to_owned(), DealStageName::Won, None, true, Some(owner)),
            ("Opportunité Nouveau".to_owned(), DealStageName::New, None, false, Some(owner)),
            ("Opportunité Perdu".to_owned(), DealStageName::Lost, reason, true, Some(owner)),
            ("Opportunité Qualifié".to_owned(), DealStageName::Visit, None, false, Some(owner)),
        ];
        expected.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(total, 6);
        assert_eq!(found, expected);
        assert!(deals.iter().all(|deal| deal.amount == DealAmount::ZERO));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_quote_carries_its_deal_and_the_list_filters_on_it() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let customer = seed_customer(&tx, org, "Boulangerie").await;
        let linked = deal(org, customer.id, "Fournil");
        let other = deal(org, customer.id, "Vitrine");
        let mut deals = PgDealRepository::new(&tx);
        deals.insert(&linked).await?;
        deals.insert(&other).await?;
        let now = Utc::now();
        let context = PgCustomerContextRepository::new(&tx)
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
            .await?;
        let quote = |deal_id: Option<DealId>, title: &str| Quote {
            deal_id,
            ..Quote::draft(
                QuoteId(generate_uuid_v7()),
                org,
                customer.id,
                context.id,
                title.to_owned(),
                now,
            )
        };
        let mut quotes = PgQuoteRepository::new(&tx);
        let with_deal = quotes.insert(&quote(Some(linked.id), "Lié")).await?;
        quotes.insert(&quote(None, "Libre")).await?;

        let (filtered, total) = quotes
            .list_by_organization(org, Some(linked.id), 10, 0)
            .await?;
        let (unfiltered, all) = quotes.list_by_organization(org, None, 10, 0).await?;
        let (none_for_other, _) = quotes
            .list_by_organization(org, Some(other.id), 10, 0)
            .await?;

        assert_eq!(with_deal.deal_id, Some(linked.id));
        assert_eq!((filtered.len(), total), (1, 1));
        assert_eq!(filtered[0].title, "Lié");
        assert_eq!((unfiltered.len(), all), (2, 2));
        assert!(none_for_other.is_empty());
        Ok(())
    })
    .await;
}
