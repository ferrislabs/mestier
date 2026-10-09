//! #304's own binding requirement: "the owner of an existing organization
//! can still read costs" must be a sentence a test can hold, not a claim
//! read off the SQL. Drives the real `sqlx` CLI against a scratch database
//! the same way `application::task::tests::migration_backfills_title_and_
//! renames_note_to_description` does — see that test's own doc comment for
//! why a CLI subprocess, not `sqlx::migrate!`, is how this repo drives a
//! migration from a test.

use common::generate_uuid_v7;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{application::test_support::split_database_url, domain::role::Permissions};

fn migrations_dir() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations")
}

fn run_sqlx_migrate(database_url: &str, target_version: &str) {
    let status = std::process::Command::new("sqlx")
        .args([
            "migrate",
            "run",
            "--source",
            migrations_dir().to_str().unwrap(),
            "--database-url",
            database_url,
            "--target-version",
            target_version,
        ])
        .status()
        .expect("the `sqlx` CLI must be on PATH to run this migration integration test");

    assert!(
        status.success(),
        "`sqlx migrate run --target-version {target_version}` failed"
    );
}

/// Same device as `run_sqlx_migrate`, without a `--target-version`: brings
/// the scratch database the rest of the way to the latest migration.
fn run_sqlx_migrate_to_latest(database_url: &str) {
    let status = std::process::Command::new("sqlx")
        .args([
            "migrate",
            "run",
            "--source",
            migrations_dir().to_str().unwrap(),
            "--database-url",
            database_url,
        ])
        .status()
        .expect("the `sqlx` CLI must be on PATH to run this migration integration test");

    assert!(status.success(), "`sqlx migrate run` to latest failed");
}

#[tokio::test]
#[ignore = "requires live postgres and the sqlx CLI binary on PATH"]
async fn backfill_grants_business_permissions_without_leaking_payroll_to_members() {
    let (base_url, _current_db) =
        split_database_url(&std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"));
    let admin_url = format!("{base_url}/postgres");
    let scratch_db = format!("mestier_migration_test_{}", Uuid::new_v4().simple());
    let scratch_url = format!("{base_url}/{scratch_db}");

    let admin_pool = PgPool::connect(&admin_url).await.unwrap();
    sqlx::query(&format!(r#"CREATE DATABASE "{scratch_db}""#))
        .execute(&admin_pool)
        .await
        .expect("creating the scratch database must succeed");

    // Bring the scratch database to right before this issue's migration,
    // then seed roles shaped like an organization created before #304
    // shipped: owner already `Permissions::ALL`, admin with only
    // `MANAGE_MEMBERS`, member with none at all.
    run_sqlx_migrate(&scratch_url, "20260829000001");

    let scratch_pool = PgPool::connect(&scratch_url).await.unwrap();
    let owner_id = generate_uuid_v7();
    sqlx::query(
        r#"INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)"#,
    )
    .bind(owner_id)
    .bind(format!("owner-{owner_id}@example.com"))
    .bind(format!("owner-{owner_id}"))
    .bind("Owner User")
    .bind(format!("sub-owner-{owner_id}"))
    .execute(&scratch_pool)
    .await
    .unwrap();

    let org_id = generate_uuid_v7();
    sqlx::query(r#"INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)"#)
        .bind(org_id)
        .bind("Test Org")
        .bind(format!("test-org-{org_id}"))
        .bind(owner_id)
        .execute(&scratch_pool)
        .await
        .unwrap();

    let owner_role_id = generate_uuid_v7();
    let admin_role_id = generate_uuid_v7();
    let member_role_id = generate_uuid_v7();
    for (id, name, permissions) in [
        (owner_role_id, "owner", Permissions::ALL.bits()),
        (admin_role_id, "admin", Permissions::MANAGE_MEMBERS.bits()),
        (member_role_id, "member", Permissions::NONE.bits()),
    ] {
        sqlx::query(
            r#"INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)"#,
        )
        .bind(id)
        .bind(org_id)
        .bind(name)
        .bind(permissions)
        .execute(&scratch_pool)
        .await
        .unwrap();
    }

    scratch_pool.close().await;

    run_sqlx_migrate_to_latest(&scratch_url);

    let scratch_pool = PgPool::connect(&scratch_url).await.unwrap();
    let permissions_of = |id: Uuid| {
        let pool = scratch_pool.clone();
        async move {
            let bits: i64 = sqlx::query_scalar(r#"SELECT permissions FROM roles WHERE id = $1"#)
                .bind(id)
                .fetch_one(&pool)
                .await
                .unwrap();
            Permissions(bits)
        }
    };

    let owner_permissions = permissions_of(owner_role_id).await;
    assert!(
        owner_permissions.contains(Permissions::VIEW_COST),
        "the owner of an existing organization must still be able to read costs"
    );

    let admin_permissions = permissions_of(admin_role_id).await;
    assert!(admin_permissions.contains(Permissions::MANAGE_MEMBERS));
    assert!(admin_permissions.contains(Permissions::VIEW_PLANNING));
    assert!(admin_permissions.contains(Permissions::MANAGE_PLANNING));
    assert!(admin_permissions.contains(Permissions::VIEW_REPORTS));
    assert!(admin_permissions.contains(Permissions::MANAGE_CUSTOMERS));
    assert!(admin_permissions.contains(Permissions::MANAGE_QUOTES));
    assert!(admin_permissions.contains(Permissions::MANAGE_REFERENCE));
    assert!(
        !admin_permissions.contains(Permissions::VIEW_COST),
        "an existing admin must not silently gain payroll visibility"
    );
    assert!(!admin_permissions.contains(Permissions::MANAGE_COST));

    let member_permissions = permissions_of(member_role_id).await;
    assert!(member_permissions.contains(Permissions::VIEW_PLANNING));
    assert!(member_permissions.contains(Permissions::MANAGE_PLANNING));
    assert!(
        !member_permissions.contains(Permissions::VIEW_COST),
        "an existing plain member must not gain payroll visibility — this is the leak #283 closes"
    );
    assert!(!member_permissions.contains(Permissions::MANAGE_COST));
    assert!(!member_permissions.contains(Permissions::VIEW_REPORTS));
    assert!(!member_permissions.contains(Permissions::MANAGE_CUSTOMERS));

    scratch_pool.close().await;
    sqlx::query(&format!(r#"DROP DATABASE IF EXISTS "{scratch_db}""#))
        .execute(&admin_pool)
        .await
        .unwrap_or_else(|error| panic!("dropping {scratch_db} failed: {error}"));
}

mod mcp_enabled {
    use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
    use sqlx::PgPool;
    use tokio::sync::OnceCell;
    use uuid::Uuid;

    use crate::{
        UpdateOrganizationCommand,
        application::{
            MestierUseCase, default_authorizer, policy,
            test_support::{purge, scratch_pool},
        },
        domain::role::Permissions,
        infrastructure::realtime::EventHub,
    };

    static SCRATCH: OnceCell<String> = OnceCell::const_new();

    struct Fixture {
        organization_id: OrganizationId,
        owner_id: Uuid,
        plain_member_id: Uuid,
    }

    async fn seed_user(pool: &PgPool) -> Uuid {
        let id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(id)
        .bind(format!("user-{id}@example.com"))
        .bind(format!("user-{id}"))
        .bind("Test User")
        .bind(format!("sub-{id}"))
        .execute(pool)
        .await
        .unwrap();
        id
    }

    async fn seed_member(pool: &PgPool, organization_id: Uuid, user_id: Uuid, permissions: i64) {
        let member_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(member_id)
        .bind(organization_id)
        .bind(user_id)
        .bind("Member")
        .execute(pool)
        .await
        .unwrap();
        let role_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
        )
        .bind(role_id)
        .bind(organization_id)
        .bind(format!("role-{role_id}"))
        .bind(permissions)
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

    async fn seed(pool: &PgPool) -> Fixture {
        let owner_id = seed_user(pool).await;
        let plain_member_id = seed_user(pool).await;
        let organization_id = generate_uuid_v7();
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id)
            .bind("Mcp Org")
            .bind(format!("mcp-org-{organization_id}"))
            .bind(owner_id)
            .execute(pool)
            .await
            .unwrap();
        seed_member(pool, organization_id, owner_id, Permissions::ALL.0).await;
        seed_member(pool, organization_id, plain_member_id, Permissions::NONE.0).await;
        Fixture {
            organization_id: OrganizationId(organization_id),
            owner_id,
            plain_member_id,
        }
    }

    async fn cleanup(pool: &PgPool, fixture: &Fixture) {
        for statement in [
            "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM roles WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(pool, statement, fixture.organization_id.0).await;
        }
        for user_id in [fixture.owner_id, fixture.plain_member_id] {
            purge(pool, "DELETE FROM users WHERE id = $1", user_id).await;
        }
    }

    fn command(
        fixture: &Fixture,
        user_id: Uuid,
        mcp_enabled: Option<bool>,
    ) -> UpdateOrganizationCommand {
        UpdateOrganizationCommand {
            actor: policy::user_subject(UserId(user_id), Vec::new()),
            id: fixture.organization_id,
            name: "Mcp Org".to_owned(),
            slug: format!("mcp-org-{}", fixture.organization_id.0),
            field_clock_enabled: false,
            mcp_enabled,
        }
    }

    #[tokio::test]
    #[ignore = "needs DATABASE_URL"]
    async fn spec_mcp_13_the_flag_defaults_to_false_and_toggles() {
        let pool = scratch_pool("mestier_orgmcp_test_", &SCRATCH).await;
        let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());
        let fixture = seed(&pool).await;

        let initial = usecase
            .get_organization(fixture.organization_id)
            .await
            .unwrap();
        assert!(!initial.mcp_enabled);

        let enabled = usecase
            .update_organization(command(&fixture, fixture.owner_id, Some(true)))
            .await
            .unwrap();
        assert!(enabled.mcp_enabled);

        let unchanged = usecase
            .update_organization(command(&fixture, fixture.owner_id, None))
            .await
            .unwrap();
        assert!(unchanged.mcp_enabled, "absent leaves the flag untouched");

        let disabled = usecase
            .update_organization(command(&fixture, fixture.owner_id, Some(false)))
            .await
            .unwrap();
        assert!(!disabled.mcp_enabled);

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "needs DATABASE_URL"]
    async fn spec_mcp_13_a_member_without_manage_org_cannot_change_it() {
        let pool = scratch_pool("mestier_orgmcp_test_", &SCRATCH).await;
        let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());
        let fixture = seed(&pool).await;

        let err = usecase
            .update_organization(command(&fixture, fixture.plain_member_id, Some(true)))
            .await
            .unwrap_err();
        assert!(matches!(err, CoreError::Forbidden { .. }), "{err:?}");

        let reloaded = usecase
            .get_organization(fixture.organization_id)
            .await
            .unwrap();
        assert!(!reloaded.mcp_enabled);

        cleanup(&pool, &fixture).await;
    }
}
