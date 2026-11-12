use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    EventHub, FEATURE_DISABLED, FEATURE_ENABLED, FeatureFlag, MestierUseCase, PlatformAdmin,
    application::{
        default_authorizer,
        test_support::{dev_pool, purge},
    },
};

struct Fixture {
    pool: PgPool,
    usecase: MestierUseCase,
    admin: PlatformAdmin,
    staff: UserId,
    a: OrganizationId,
    b: OrganizationId,
}

fn admin() -> PlatformAdmin {
    PlatformAdmin::from_realm_roles(&["mestier:admin".to_owned()]).unwrap()
}

async fn seed_user(pool: &PgPool, display_name: &str) -> UserId {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(format!("user-{id}@example.com"))
    .bind(format!("user-{id}"))
    .bind(display_name)
    .bind(format!("sub-{id}"))
    .execute(pool)
    .await
    .unwrap();
    UserId(id)
}

async fn seed_org(pool: &PgPool, owner: UserId) -> OrganizationId {
    let id = generate_uuid_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(format!("Org {id}"))
        .bind(format!("org-{id}"))
        .bind(owner.0)
        .execute(pool)
        .await
        .unwrap();
    OrganizationId(id)
}

async fn fixture() -> Fixture {
    let pool = dev_pool().await;
    let staff = seed_user(&pool, "Admin Staff").await;
    let a = seed_org(&pool, staff).await;
    let b = seed_org(&pool, staff).await;
    let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());
    Fixture {
        pool,
        usecase,
        admin: admin(),
        staff,
        a,
        b,
    }
}

async fn cleanup(fixture: Fixture) {
    for org in [fixture.a, fixture.b] {
        purge(
            &fixture.pool,
            "DELETE FROM organizations WHERE id = $1",
            org.0,
        )
        .await;
    }
    purge(
        &fixture.pool,
        "DELETE FROM users WHERE id = $1",
        fixture.staff.0,
    )
    .await;
}

async fn events(pool: &PgPool, org: OrganizationId) -> Vec<(String, String, Option<Uuid>)> {
    sqlx::query_as::<_, (String, String, Option<Uuid>)>(
        "SELECT name, actor_kind, actor_id FROM automation.event
         WHERE org_id = $1 ORDER BY occurred_at, id",
    )
    .bind(org.0)
    .fetch_all(pool)
    .await
    .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_flag_is_off_everywhere_until_enabled_and_enabling_for_a_leaves_b_closed() {
    let f = fixture().await;

    let initially = f
        .usecase
        .is_feature_enabled(f.a, FeatureFlag::DriveAi)
        .await
        .unwrap();
    f.usecase
        .set_organization_feature(f.admin, f.staff, f.a, FeatureFlag::DriveAi, true)
        .await
        .unwrap();

    assert!(!initially);
    assert!(
        f.usecase
            .is_feature_enabled(f.a, FeatureFlag::DriveAi)
            .await
            .unwrap()
    );
    assert!(
        !f.usecase
            .is_feature_enabled(f.b, FeatureFlag::DriveAi)
            .await
            .unwrap()
    );
    assert_eq!(
        f.usecase.enabled_features(f.a).await.unwrap(),
        vec![FeatureFlag::DriveAi]
    );
    assert!(f.usecase.enabled_features(f.b).await.unwrap().is_empty());
    cleanup(f).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn each_change_writes_one_event_with_the_admin_as_actor_and_replays_write_none() {
    let f = fixture().await;
    let set = |enabled| {
        f.usecase
            .set_organization_feature(f.admin, f.staff, f.a, FeatureFlag::DriveAi, enabled)
    };

    let disable_inactive = set(false).await.unwrap();
    let enable = set(true).await.unwrap();
    let enable_again = set(true).await.unwrap();
    let disable = set(false).await.unwrap();
    let disable_again = set(false).await.unwrap();

    assert_eq!(
        [
            disable_inactive,
            enable,
            enable_again,
            disable,
            disable_again
        ],
        [false, true, false, true, false]
    );
    let written = events(&f.pool, f.a).await;
    assert_eq!(
        written,
        vec![
            (
                FEATURE_ENABLED.to_owned(),
                "user".to_owned(),
                Some(f.staff.0)
            ),
            (
                FEATURE_DISABLED.to_owned(),
                "user".to_owned(),
                Some(f.staff.0)
            ),
        ]
    );
    assert!(events(&f.pool, f.b).await.is_empty());
    cleanup(f).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn changing_a_flag_of_an_unknown_organization_is_not_found() {
    let f = fixture().await;

    let outcome = f
        .usecase
        .set_organization_feature(
            f.admin,
            f.staff,
            OrganizationId(Uuid::from_u128(9)),
            FeatureFlag::DriveAi,
            true,
        )
        .await;

    assert!(matches!(outcome, Err(CoreError::NotFound)));
    cleanup(f).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_detail_lists_the_whole_catalogue_and_a_newest_first_history() {
    let f = fixture().await;
    for enabled in [true, false, true] {
        f.usecase
            .set_organization_feature(f.admin, f.staff, f.a, FeatureFlag::DriveAi, enabled)
            .await
            .unwrap();
    }

    let detail = f
        .usecase
        .get_organization_for_admin(f.admin, f.a)
        .await
        .unwrap();

    assert_eq!(detail.overview.id, f.a);
    assert_eq!(detail.features.len(), FeatureFlag::ALL.len());
    assert_eq!(detail.features[0].flag, FeatureFlag::DriveAi);
    assert!(detail.features[0].enabled());
    assert_eq!(
        detail.features[0].enabled_by_name.as_deref(),
        Some("Admin Staff")
    );
    let order: Vec<bool> = detail.history.iter().map(|change| change.enabled).collect();
    assert_eq!(order, vec![true, false, true]);
    assert!(detail.history.iter().all(|change| change.flag == "drive_ai"
        && change.actor_id == Some(f.staff)
        && change.actor_name.as_deref() == Some("Admin Staff")));
    assert!(detail.history[0].occurred_at >= detail.history[1].occurred_at);

    let untouched = f
        .usecase
        .get_organization_for_admin(f.admin, f.b)
        .await
        .unwrap();
    assert!(!untouched.features[0].enabled());
    assert!(untouched.history.is_empty());
    cleanup(f).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_detail_of_an_unknown_organization_is_not_found() {
    let f = fixture().await;

    let outcome = f
        .usecase
        .get_organization_for_admin(f.admin, OrganizationId(Uuid::from_u128(9)))
        .await;

    assert!(matches!(outcome, Err(CoreError::NotFound)));
    cleanup(f).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_back_office_lists_and_stats_run_through_the_use_cases() {
    let f = fixture().await;

    let (organizations, total) = f
        .usecase
        .list_organizations_for_admin(f.admin, None, 5, 0)
        .await
        .unwrap();
    let (users, _) = f
        .usecase
        .list_users_for_admin(f.admin, Some("Admin Staff".to_owned()), 5, 0)
        .await
        .unwrap();
    let stats = f.usecase.platform_stats(f.admin).await.unwrap();

    assert!(total >= 2);
    assert!(organizations.len() <= 5);
    assert!(users.iter().any(|user| user.id == f.staff));
    assert_eq!(stats.signups_per_week.len(), 12);
    assert_eq!(f.usecase.list_feature_catalogue(f.admin), FeatureFlag::ALL);
    cleanup(f).await;
}
