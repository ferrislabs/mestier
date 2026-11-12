use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    FeatureFlag, PlatformStats,
    application::test_support::dev_pool,
    domain::{
        feature_flag::ports::FeatureFlagRepository, platform::ports::PlatformAdminRepository,
    },
    infrastructure::{
        feature_flag::postgres::PgFeatureFlagRepository,
        platform::postgres::PgPlatformAdminRepository,
        postgres::{SharedTx, with_tx},
    },
};

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

async fn seed_user(
    tx: &SharedTx<'_>,
    display_name: &str,
    email: &str,
    created_at: DateTime<Utc>,
) -> UserId {
    let id = generate_uuid_v7();
    let mut guard = tx.lock().await;
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub, created_at)
         VALUES ($1, $2, $3, $4, $5, $6)",
    )
    .bind(id)
    .bind(email)
    .bind(format!("user-{id}"))
    .bind(display_name)
    .bind(format!("sub-{id}"))
    .bind(created_at)
    .execute(&mut ***guard)
    .await
    .unwrap();
    UserId(id)
}

async fn seed_org(
    tx: &SharedTx<'_>,
    name: &str,
    slug: &str,
    owner: UserId,
    created_at: DateTime<Utc>,
) -> OrganizationId {
    let id = generate_uuid_v7();
    let mut guard = tx.lock().await;
    sqlx::query(
        "INSERT INTO organizations (id, name, slug, owner_id, created_at)
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(name)
    .bind(slug)
    .bind(owner.0)
    .bind(created_at)
    .execute(&mut ***guard)
    .await
    .unwrap();
    OrganizationId(id)
}

async fn seed_member(
    tx: &SharedTx<'_>,
    organization: OrganizationId,
    user: UserId,
    last_name: &str,
    role: Option<&str>,
) {
    let member_id = generate_uuid_v7();
    let mut guard = tx.lock().await;
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name, first_name)
         VALUES ($1, $2, $3, $4, 'Prénom')",
    )
    .bind(member_id)
    .bind(organization.0)
    .bind(user.0)
    .bind(last_name)
    .execute(&mut ***guard)
    .await
    .unwrap();
    if let Some(role) = role {
        let role_id = generate_uuid_v7();
        sqlx::query("INSERT INTO roles (id, organization_id, name) VALUES ($1, $2, $3)")
            .bind(role_id)
            .bind(organization.0)
            .bind(role)
            .execute(&mut ***guard)
            .await
            .unwrap();
        sqlx::query("INSERT INTO member_roles (member_id, role_id) VALUES ($1, $2)")
            .bind(member_id)
            .bind(role_id)
            .execute(&mut ***guard)
            .await
            .unwrap();
    }
}

async fn seed_work(
    tx: &SharedTx<'_>,
    organization: OrganizationId,
    created_at: DateTime<Utc>,
    kinds: &[&str],
) {
    let mut guard = tx.lock().await;
    let customer = generate_uuid_v7();
    let context = generate_uuid_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, 'C')")
        .bind(customer)
        .bind(organization.0)
        .execute(&mut ***guard)
        .await
        .unwrap();
    sqlx::query("INSERT INTO customer_contexts (id, customer_id, label) VALUES ($1, $2, 'L')")
        .bind(context)
        .bind(customer)
        .execute(&mut ***guard)
        .await
        .unwrap();
    for kind in kinds {
        let id = generate_uuid_v7();
        let query = match *kind {
            "quote" => sqlx::query(
                "INSERT INTO quotes (id, org_id, customer_id, customer_context_id, status, net_cents, gross_cents, title, created_at)
                 VALUES ($1, $2, $3, $4, 'DRAFT', 0, 0, 'T', $5)",
            )
            .bind(id)
            .bind(organization.0)
            .bind(customer)
            .bind(context)
            .bind(created_at),
            "invoice" => sqlx::query(
                "INSERT INTO invoices (id, org_id, customer_id, customer_context_id, created_at)
                 VALUES ($1, $2, $3, $4, $5)",
            )
            .bind(id)
            .bind(organization.0)
            .bind(customer)
            .bind(context)
            .bind(created_at),
            "project" => sqlx::query(
                "INSERT INTO projects (id, org_id, name, created_at) VALUES ($1, $2, 'P', $3)",
            )
            .bind(id)
            .bind(organization.0)
            .bind(created_at),
            _ => sqlx::query(
                "INSERT INTO tasks (id, org_id, status, title, created_at)
                 VALUES ($1, $2, 'BACKLOG', 'T', $3)",
            )
            .bind(id)
            .bind(organization.0)
            .bind(created_at),
        };
        query.execute(&mut ***guard).await.unwrap();
    }
}

fn instant(year: i32, month: u32, day: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, 10, 0, 0).unwrap()
}

fn token() -> String {
    generate_uuid_v7().simple().to_string()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn organizations_are_searched_by_name_or_slug_ignoring_case_and_paginated() {
    run(async |tx| {
        let now = Utc::now();
        let marker = token();
        let owner = seed_user(&tx, "Owner", &format!("owner-{marker}@example.com"), now).await;
        for index in 0..3 {
            seed_org(
                &tx,
                &format!("Treyve Paysages {marker} {index}"),
                &format!("treyve-{marker}-{index}"),
                owner,
                now - Duration::minutes(index),
            )
            .await;
        }
        seed_org(&tx, "Autre", &format!("slug-only-{marker}"), owner, now).await;
        let mut repository = PgPlatformAdminRepository::new(&tx);

        let (first, total) = repository
            .list_organizations(Some(&format!("TREYVE PAYSAGES {marker}")), 2, 0)
            .await?;
        let (second, _) = repository
            .list_organizations(Some(&format!("TREYVE PAYSAGES {marker}")), 2, 2)
            .await?;
        let (by_slug, by_slug_total) = repository
            .list_organizations(Some(&format!("SLUG-ONLY-{marker}")), 10, 0)
            .await?;

        assert_eq!(total, 3);
        assert_eq!(first.len(), 2);
        assert_eq!(second.len(), 1);
        assert_eq!(by_slug_total, 1);
        assert_eq!(by_slug[0].name, "Autre");
        assert!(first[0].created_at >= first[1].created_at);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_search_treats_wildcards_literally_and_skips_deleted_organizations() {
    run(async |tx| {
        let now = Utc::now();
        let marker = token();
        let owner = seed_user(&tx, "Owner", &format!("owner-{marker}@example.com"), now).await;
        seed_org(
            &tx,
            &format!("a_b {marker}"),
            &format!("ab-{marker}"),
            owner,
            now,
        )
        .await;
        seed_org(
            &tx,
            &format!("axb {marker}"),
            &format!("axb-{marker}"),
            owner,
            now,
        )
        .await;
        let deleted = seed_org(
            &tx,
            &format!("a_b gone {marker}"),
            &format!("gone-{marker}"),
            owner,
            now,
        )
        .await;
        sqlx::query("UPDATE organizations SET deleted_at = now() WHERE id = $1")
            .bind(deleted.0)
            .execute(&mut ***tx.lock().await)
            .await
            .unwrap();
        let mut repository = PgPlatformAdminRepository::new(&tx);

        let (found, total) = repository.list_organizations(Some("a_b"), 50, 0).await?;

        let mine: Vec<_> = found.iter().filter(|o| o.slug.ends_with(&marker)).collect();
        assert_eq!(mine.len(), 1);
        assert_eq!(mine[0].slug, format!("ab-{marker}"));
        assert!(total >= 1);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_organization_summary_carries_its_member_count_and_enabled_flags() {
    run(async |tx| {
        let now = Utc::now();
        let marker = token();
        let owner = seed_user(&tx, "Owner", &format!("owner-{marker}@example.com"), now).await;
        let other = seed_user(&tx, "Other", &format!("other-{marker}@example.com"), now).await;
        let org = seed_org(
            &tx,
            &format!("Org {marker}"),
            &format!("org-{marker}"),
            owner,
            now,
        )
        .await;
        seed_member(&tx, org, owner, "Owner", Some("owner")).await;
        seed_member(&tx, org, other, "Other", None).await;
        PgFeatureFlagRepository::new(&tx)
            .enable(org, FeatureFlag::DriveAi, owner)
            .await?;
        sqlx::query(
            "INSERT INTO organization_feature_flags (org_id, flag) VALUES ($1, 'retired_flag')",
        )
        .bind(org.0)
        .execute(&mut ***tx.lock().await)
        .await
        .unwrap();
        let mut repository = PgPlatformAdminRepository::new(&tx);

        let (found, _) = repository.list_organizations(Some(&marker), 10, 0).await?;

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].member_count, 2);
        assert_eq!(found[0].features, vec![FeatureFlag::DriveAi]);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_overview_lists_members_with_name_email_and_roles() {
    run(async |tx| {
        let now = Utc::now();
        let marker = token();
        let owner = seed_user(&tx, "Owner", &format!("owner-{marker}@example.com"), now).await;
        let org = seed_org(
            &tx,
            &format!("Org {marker}"),
            &format!("org-{marker}"),
            owner,
            now,
        )
        .await;
        seed_member(&tx, org, owner, "Dupont", Some("owner")).await;
        let mut repository = PgPlatformAdminRepository::new(&tx);

        let overview = repository.organization_overview(org).await?.unwrap();
        let missing = repository
            .organization_overview(OrganizationId(Uuid::from_u128(7)))
            .await?;

        assert_eq!(overview.slug, format!("org-{marker}"));
        assert_eq!(overview.members.len(), 1);
        assert_eq!(overview.members[0].name, "Dupont Prénom");
        assert_eq!(
            overview.members[0].email.as_deref(),
            Some(format!("owner-{marker}@example.com").as_str())
        );
        assert_eq!(overview.members[0].roles, vec!["owner".to_owned()]);
        assert!(missing.is_none());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn users_are_searched_by_name_email_or_username_with_their_organizations() {
    run(async |tx| {
        let now = Utc::now();
        let marker = token();
        let erwan = seed_user(
            &tx,
            &format!("Erwan {marker}"),
            &format!("e-{marker}@example.com"),
            now,
        )
        .await;
        seed_user(
            &tx,
            "Autre",
            &format!("mail-{marker}@example.com"),
            now - Duration::days(1),
        )
        .await;
        let gone = seed_user(
            &tx,
            &format!("Erwan gone {marker}"),
            &format!("g-{marker}@example.com"),
            now,
        )
        .await;
        sqlx::query("UPDATE users SET deleted_at = now() WHERE id = $1")
            .bind(gone.0)
            .execute(&mut ***tx.lock().await)
            .await
            .unwrap();
        let org = seed_org(
            &tx,
            &format!("Treyve {marker}"),
            &format!("treyve-{marker}"),
            erwan,
            now,
        )
        .await;
        seed_member(&tx, org, erwan, "Erwan", None).await;
        let mut repository = PgPlatformAdminRepository::new(&tx);

        let (by_name, by_name_total) = repository
            .list_users(Some(&format!("ERWAN {marker}")), 10, 0)
            .await?;
        let (by_email, _) = repository
            .list_users(Some(&format!("MAIL-{marker}")), 10, 0)
            .await?;
        let (page_two, paged_total) = repository.list_users(Some(&marker), 1, 1).await?;

        assert_eq!(by_name_total, 1);
        assert_eq!(by_name[0].id, erwan);
        assert_eq!(by_name[0].organizations.len(), 1);
        assert_eq!(by_name[0].organizations[0].slug, format!("treyve-{marker}"));
        assert_eq!(by_email.len(), 1);
        assert_eq!(paged_total, 2);
        assert_eq!(page_two.len(), 1);
        Ok(())
    })
    .await;
}

fn weekly(stats: &PlatformStats, signups: bool) -> Vec<(NaiveDate, u64)> {
    let source = if signups {
        &stats.signups_per_week
    } else {
        &stats.organizations_per_week
    };
    source.iter().map(|w| (w.week_start, w.count)).collect()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn stats_count_a_known_dataset_and_show_empty_weeks() {
    run(async |tx| {
        let now = instant(2026, 11, 12);
        let marker = token();
        let before = PgPlatformAdminRepository::new(&tx).stats(now).await?;

        let owner = seed_user(
            &tx,
            "Owner",
            &format!("o-{marker}@example.com"),
            instant(2026, 11, 10),
        )
        .await;
        seed_user(
            &tx,
            "Two",
            &format!("t-{marker}@example.com"),
            instant(2026, 11, 2),
        )
        .await;
        seed_user(
            &tx,
            "Old",
            &format!("old-{marker}@example.com"),
            instant(2026, 8, 1),
        )
        .await;
        let org = seed_org(
            &tx,
            &format!("Org {marker}"),
            &format!("org-{marker}"),
            owner,
            instant(2026, 11, 3),
        )
        .await;
        seed_work(
            &tx,
            org,
            instant(2026, 11, 1),
            &["quote", "invoice", "project", "task"],
        )
        .await;
        seed_work(&tx, org, instant(2026, 8, 1), &["quote", "task", "task"]).await;
        PgFeatureFlagRepository::new(&tx)
            .enable(org, FeatureFlag::DriveAi, owner)
            .await?;

        let after = PgPlatformAdminRepository::new(&tx).stats(now).await?;

        assert_eq!(after.signups_per_week.len(), 12);
        assert_eq!(after.organizations_per_week.len(), 12);
        assert_eq!(
            after.signups_per_week[0].week_start,
            NaiveDate::from_ymd_opt(2026, 8, 24).unwrap()
        );
        assert_eq!(
            after.signups_per_week[11].week_start,
            NaiveDate::from_ymd_opt(2026, 11, 9).unwrap()
        );
        let delta = |a: &[(NaiveDate, u64)], b: &[(NaiveDate, u64)], i: usize| a[i].1 - b[i].1;
        let (signups_after, signups_before) = (weekly(&after, true), weekly(&before, true));
        assert_eq!(delta(&signups_after, &signups_before, 11), 1);
        assert_eq!(delta(&signups_after, &signups_before, 10), 1);
        assert_eq!(delta(&signups_after, &signups_before, 0), 0);
        let (orgs_after, orgs_before) = (weekly(&after, false), weekly(&before, false));
        assert_eq!(delta(&orgs_after, &orgs_before, 10), 1);
        assert_eq!(after.active_users - before.active_users, 3);
        assert_eq!(after.active_organizations - before.active_organizations, 1);
        assert_eq!(after.quotes.total - before.quotes.total, 2);
        assert_eq!(after.quotes.last_30_days - before.quotes.last_30_days, 1);
        assert_eq!(after.invoices.total - before.invoices.total, 1);
        assert_eq!(
            after.invoices.last_30_days - before.invoices.last_30_days,
            1
        );
        assert_eq!(after.projects.total - before.projects.total, 1);
        assert_eq!(after.tasks.total - before.tasks.total, 3);
        assert_eq!(after.tasks.last_30_days - before.tasks.last_30_days, 1);
        assert_eq!(
            after.flag_adoption[0].organizations - before.flag_adoption[0].organizations,
            1
        );
        Ok(())
    })
    .await;
}
