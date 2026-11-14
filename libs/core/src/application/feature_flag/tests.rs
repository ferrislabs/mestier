use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    EventHub, FEATURE_DISABLED, FEATURE_ENABLED, FeatureFlag, MestierUseCase, Permissions,
    PlatformAdmin, SetSelfServiceFeatureCommand,
    application::{
        default_authorizer, policy,
        test_support::{dev_pool, purge},
    },
};

struct Fixture {
    pool: PgPool,
    usecase: MestierUseCase,
    manager: UserId,
    member: UserId,
    stranger: UserId,
    a: OrganizationId,
    b: OrganizationId,
}

async fn seed_user(pool: &PgPool) -> UserId {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, 'Self Service', $4)",
    )
    .bind(id)
    .bind(format!("self-service-{id}@example.com"))
    .bind(format!("self-service-{id}"))
    .bind(format!("sub-self-service-{id}"))
    .execute(pool)
    .await
    .unwrap();
    UserId(id)
}

async fn seed_org(pool: &PgPool, owner: UserId) -> OrganizationId {
    let id = generate_uuid_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, 'Org', $2, $3)")
        .bind(id)
        .bind(format!("self-service-{id}"))
        .bind(owner.0)
        .execute(pool)
        .await
        .unwrap();
    OrganizationId(id)
}

async fn seed_seat(
    pool: &PgPool,
    organization: OrganizationId,
    user: UserId,
    permissions: Option<Permissions>,
) {
    let member_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, 'Seat')",
    )
    .bind(member_id)
    .bind(organization.0)
    .bind(user.0)
    .execute(pool)
    .await
    .unwrap();
    let Some(permissions) = permissions else {
        return;
    };
    let role_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
    )
    .bind(role_id)
    .bind(organization.0)
    .bind(format!("Self service {role_id}"))
    .bind(permissions.bits())
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(generate_uuid_v7())
        .bind(member_id)
        .bind(role_id)
        .execute(pool)
        .await
        .unwrap();
}

async fn fixture() -> Fixture {
    let pool = dev_pool().await;
    let manager = seed_user(&pool).await;
    let member = seed_user(&pool).await;
    let stranger = seed_user(&pool).await;
    let a = seed_org(&pool, manager).await;
    let b = seed_org(&pool, stranger).await;
    seed_seat(&pool, a, manager, Some(Permissions::MANAGE_ORG)).await;
    seed_seat(&pool, a, member, Some(Permissions::MANAGE_SURVEYS)).await;
    seed_seat(&pool, b, stranger, Some(Permissions::MANAGE_ORG)).await;
    let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());
    Fixture {
        pool,
        usecase,
        manager,
        member,
        stranger,
        a,
        b,
    }
}

async fn cleanup(fixture: Fixture) {
    for org in [fixture.a, fixture.b] {
        for statement in [
            "DELETE FROM automation.event WHERE org_id = $1",
            "DELETE FROM member_roles WHERE role_id IN (SELECT id FROM roles WHERE organization_id = $1)",
            "DELETE FROM roles WHERE organization_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(&fixture.pool, statement, org.0).await;
        }
    }
    for user in [fixture.manager, fixture.member, fixture.stranger] {
        purge(&fixture.pool, "DELETE FROM users WHERE id = $1", user.0).await;
    }
}

fn command(
    user: UserId,
    organization_id: OrganizationId,
    flag: FeatureFlag,
    enabled: bool,
) -> SetSelfServiceFeatureCommand {
    SetSelfServiceFeatureCommand {
        actor: policy::user_subject(user, Vec::new()),
        user_id: user,
        organization_id,
        flag,
        enabled,
    }
}

async fn events(pool: &PgPool, org: OrganizationId) -> Vec<(String, Option<Uuid>)> {
    sqlx::query_as::<_, (String, Option<Uuid>)>(
        "SELECT name, actor_id FROM automation.event WHERE org_id = $1 ORDER BY occurred_at, id",
    )
    .bind(org.0)
    .fetch_all(pool)
    .await
    .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_manager_turns_surveys_on_then_off_and_each_change_records_the_member() {
    let f = fixture().await;

    let enabled = f
        .usecase
        .set_self_service_feature(command(f.manager, f.a, FeatureFlag::Surveys, true))
        .await
        .unwrap();
    let on = f
        .usecase
        .is_feature_enabled(f.a, FeatureFlag::Surveys)
        .await
        .unwrap();
    let replay = f
        .usecase
        .set_self_service_feature(command(f.manager, f.a, FeatureFlag::Surveys, true))
        .await
        .unwrap();
    let disabled = f
        .usecase
        .set_self_service_feature(command(f.manager, f.a, FeatureFlag::Surveys, false))
        .await
        .unwrap();
    let off = f
        .usecase
        .is_feature_enabled(f.a, FeatureFlag::Surveys)
        .await
        .unwrap();

    assert_eq!((enabled, replay, disabled), (true, false, true));
    assert!(on);
    assert!(!off);
    assert_eq!(
        events(&f.pool, f.a).await,
        vec![
            (FEATURE_ENABLED.to_owned(), Some(f.manager.0)),
            (FEATURE_DISABLED.to_owned(), Some(f.manager.0)),
        ]
    );
    cleanup(f).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_without_manage_org_is_forbidden_and_nothing_changes() {
    let f = fixture().await;

    let outcome = f
        .usecase
        .set_self_service_feature(command(f.member, f.a, FeatureFlag::Surveys, true))
        .await;

    assert!(matches!(outcome, Err(CoreError::Forbidden { .. })));
    assert!(
        !f.usecase
            .is_feature_enabled(f.a, FeatureFlag::Surveys)
            .await
            .unwrap()
    );
    assert!(events(&f.pool, f.a).await.is_empty());
    cleanup(f).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_manager_of_another_organization_gets_not_found() {
    let f = fixture().await;

    let outcome = f
        .usecase
        .set_self_service_feature(command(f.stranger, f.a, FeatureFlag::Surveys, true))
        .await;
    let unknown = f
        .usecase
        .set_self_service_feature(command(
            f.manager,
            OrganizationId(Uuid::from_u128(9)),
            FeatureFlag::Surveys,
            true,
        ))
        .await;

    assert!(matches!(outcome, Err(CoreError::NotFound)));
    assert!(matches!(unknown, Err(CoreError::NotFound)));
    assert!(
        !f.usecase
            .is_feature_enabled(f.a, FeatureFlag::Surveys)
            .await
            .unwrap()
    );
    cleanup(f).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_staff_only_flag_is_not_found_and_its_staff_activation_is_untouched() {
    let f = fixture().await;
    let admin = PlatformAdmin::from_realm_roles(&["mestier:admin".to_owned()]).unwrap();
    f.usecase
        .set_organization_feature(admin, f.manager, f.a, FeatureFlag::DriveAi, true)
        .await
        .unwrap();

    let disable = f
        .usecase
        .set_self_service_feature(command(f.manager, f.a, FeatureFlag::DriveAi, false))
        .await;
    let enable = f
        .usecase
        .set_self_service_feature(command(f.stranger, f.b, FeatureFlag::DriveAi, true))
        .await;

    assert!(matches!(disable, Err(CoreError::NotFound)));
    assert!(matches!(enable, Err(CoreError::NotFound)));
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
    cleanup(f).await;
}
