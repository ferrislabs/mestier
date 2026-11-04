use authz::Subject;
use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    DisplayFields, GetDisplayFieldsCommand, Permissions, ReplaceDisplayFieldsCommand,
    SupportSettingsRepository,
    application::{
        MestierUseCase, default_authorizer, policy,
        test_support::{purge, scratch_pool},
    },
    infrastructure::{
        postgres::with_tx, realtime::EventHub,
        support_settings::postgres::PgSupportSettingsRepository,
    },
};

static SETTINGS_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_support_settings_test_", &SETTINGS_SCRATCH).await
}

struct Person {
    user_id: Uuid,
}

impl Person {
    fn subject(&self) -> Subject {
        policy::user_subject(UserId(self.user_id), Vec::new())
    }
}

struct World {
    pool: PgPool,
    usecase: MestierUseCase,
    organization_id: OrganizationId,
    manager: Person,
    plain: Person,
}

async fn seed_user(pool: &PgPool, label: &str) -> Uuid {
    let user_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("{label}-{user_id}@example.com"))
    .bind(format!("{label}-{user_id}"))
    .bind(label)
    .bind(format!("sub-{user_id}"))
    .execute(pool)
    .await
    .unwrap();
    user_id
}

async fn seed_member(
    pool: &PgPool,
    organization_id: OrganizationId,
    user_id: Uuid,
    label: &str,
    role_id: Option<Uuid>,
) -> Person {
    let member_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(member_id)
    .bind(organization_id.0)
    .bind(user_id)
    .bind(label)
    .execute(pool)
    .await
    .unwrap();
    if let Some(role_id) = role_id {
        sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
            .bind(generate_uuid_v7())
            .bind(member_id)
            .bind(role_id)
            .execute(pool)
            .await
            .unwrap();
    }
    Person { user_id }
}

fn fields(values: &[&str]) -> DisplayFields {
    DisplayFields::try_from(
        values
            .iter()
            .map(|value| (*value).to_owned())
            .collect::<Vec<_>>(),
    )
    .unwrap()
}

impl World {
    async fn new(pool: &PgPool) -> Self {
        let manager_user = seed_user(pool, "Manager").await;
        let organization_id = OrganizationId(generate_uuid_v7());
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id.0)
            .bind("Treyve Paysages")
            .bind(format!("org-{}", organization_id.0))
            .bind(manager_user)
            .execute(pool)
            .await
            .unwrap();

        let role_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
        )
        .bind(role_id)
        .bind(organization_id.0)
        .bind("Support")
        .bind(Permissions::MANAGE_SUPPORT.bits())
        .execute(pool)
        .await
        .unwrap();

        let plain_user = seed_user(pool, "Plain").await;
        Self {
            pool: pool.clone(),
            usecase: MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new()),
            organization_id,
            manager: seed_member(
                pool,
                organization_id,
                manager_user,
                "Manager",
                Some(role_id),
            )
            .await,
            plain: seed_member(pool, organization_id, plain_user, "Plain", None).await,
        }
    }

    async fn read(&self) -> DisplayFields {
        self.usecase
            .get_support_display_fields(GetDisplayFieldsCommand {
                organization_id: self.organization_id,
            })
            .await
            .unwrap()
    }

    async fn replace(&self, display_fields: DisplayFields) -> DisplayFields {
        self.usecase
            .replace_support_display_fields(ReplaceDisplayFieldsCommand {
                actor: self.manager.subject(),
                organization_id: self.organization_id,
                display_fields,
            })
            .await
            .unwrap()
    }

    async fn rows(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM support_settings WHERE org_id = $1")
            .bind(self.organization_id.0)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn timestamps(&self) -> (DateTime<Utc>, DateTime<Utc>) {
        sqlx::query_as("SELECT created_at, updated_at FROM support_settings WHERE org_id = $1")
            .bind(self.organization_id.0)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn events(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM automation.event WHERE org_id = $1")
            .bind(self.organization_id.0)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn cleanup(self) {
        for statement in [
            "DELETE FROM support_settings WHERE org_id = $1",
            "DELETE FROM automation.event WHERE org_id = $1",
            "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
            "DELETE FROM roles WHERE organization_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(&self.pool, statement, self.organization_id.0).await;
        }
        for person in [&self.manager, &self.plain] {
            purge(
                &self.pool,
                "DELETE FROM users WHERE id = $1",
                person.user_id,
            )
            .await;
        }
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_organization_without_a_row_reads_an_empty_list_and_nothing_is_written() {
    let world = World::new(&pool().await).await;

    let read = world.read().await;

    assert_eq!(read, DisplayFields::default());
    assert!(read.as_slice().is_empty());
    assert_eq!(world.rows().await, 0);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn replace_then_read_keeps_the_order() {
    let world = World::new(&pool().await).await;

    let replaced = world
        .replace(fields(&["plan", "team", "deployment_id"]))
        .await;

    assert_eq!(replaced, fields(&["plan", "team", "deployment_id"]));
    assert_eq!(
        world.read().await,
        fields(&["plan", "team", "deployment_id"])
    );
    assert_eq!(world.rows().await, 1);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_second_replace_overwrites_the_first_and_bumps_updated_at() {
    let world = World::new(&pool().await).await;

    world.replace(fields(&["plan", "team"])).await;
    let (created_at, first_updated_at) = world.timestamps().await;
    let second = world.replace(fields(&["seats"])).await;
    let (still_created_at, second_updated_at) = world.timestamps().await;

    assert_eq!(second, fields(&["seats"]));
    assert_eq!(world.read().await, fields(&["seats"]));
    assert_eq!(world.rows().await, 1);
    assert_eq!(still_created_at, created_at);
    assert!(second_updated_at > first_updated_at);

    let cleared = world.replace(DisplayFields::default()).await;
    assert!(cleared.as_slice().is_empty());
    assert!(world.read().await.as_slice().is_empty());
    assert_eq!(world.rows().await, 1);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn another_organization_is_unaffected() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;

    acme.replace(fields(&["plan", "team"])).await;

    assert_eq!(acme.read().await, fields(&["plan", "team"]));
    assert!(globex.read().await.as_slice().is_empty());
    assert_eq!(globex.rows().await, 0);

    globex.replace(fields(&["seats"])).await;
    acme.replace(fields(&["team"])).await;

    assert_eq!(acme.read().await, fields(&["team"]));
    assert_eq!(globex.read().await, fields(&["seats"]));

    acme.cleanup().await;
    globex.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_without_manage_support_cannot_replace() {
    let world = World::new(&pool().await).await;
    world.replace(fields(&["plan"])).await;

    let outcome = world
        .usecase
        .replace_support_display_fields(ReplaceDisplayFieldsCommand {
            actor: world.plain.subject(),
            organization_id: world.organization_id,
            display_fields: fields(&["team"]),
        })
        .await;

    assert!(matches!(outcome, Err(CoreError::Forbidden { .. })));
    assert_eq!(world.read().await, fields(&["plan"]));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_refused_replace_writes_no_row() {
    let world = World::new(&pool().await).await;

    let outcome = world
        .usecase
        .replace_support_display_fields(ReplaceDisplayFieldsCommand {
            actor: world.plain.subject(),
            organization_id: world.organization_id,
            display_fields: fields(&["team"]),
        })
        .await;

    assert!(matches!(outcome, Err(CoreError::Forbidden { .. })));
    assert_eq!(world.rows().await, 0);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_system_subject_can_replace() {
    let world = World::new(&pool().await).await;

    let replaced = world
        .usecase
        .replace_support_display_fields(ReplaceDisplayFieldsCommand {
            actor: Subject::system(),
            organization_id: world.organization_id,
            display_fields: fields(&["plan"]),
        })
        .await
        .unwrap();

    assert_eq!(replaced, fields(&["plan"]));
    assert_eq!(world.read().await, fields(&["plan"]));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn replacing_and_reading_emit_no_domain_event() {
    let world = World::new(&pool().await).await;

    world.replace(fields(&["plan", "team"])).await;
    world.read().await;

    assert_eq!(world.events().await, 0);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_repository_reads_and_upserts_through_one_transaction() {
    let world = World::new(&pool().await).await;
    let organization_id = world.organization_id;

    let (before, replaced, after) = with_tx(&world.pool, async |tx| {
        let mut repository = PgSupportSettingsRepository::new(&tx);
        let before = repository.display_fields(organization_id).await?;
        let replaced = repository
            .replace_display_fields(organization_id, &fields(&["team", "plan"]))
            .await?;
        let after = repository.display_fields(organization_id).await?;
        Ok((before, replaced, after))
    })
    .await
    .unwrap();

    assert!(before.as_slice().is_empty());
    assert_eq!(replaced, fields(&["team", "plan"]));
    assert_eq!(after, replaced);
    let stored: Value =
        sqlx::query_scalar("SELECT display_fields FROM support_settings WHERE org_id = $1")
            .bind(organization_id.0)
            .fetch_one(&world.pool)
            .await
            .unwrap();
    assert_eq!(stored, json!(["team", "plan"]));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_stored_list_that_breaks_the_invariants_is_an_internal_error() {
    let world = World::new(&pool().await).await;

    for stored in [
        json!(["team", "team"]),
        json!([""]),
        json!({ "team": 1 }),
        json!([1, 2]),
    ] {
        sqlx::query(
            "INSERT INTO support_settings (org_id, display_fields) VALUES ($1, $2) \
             ON CONFLICT (org_id) DO UPDATE SET display_fields = EXCLUDED.display_fields",
        )
        .bind(world.organization_id.0)
        .bind(&stored)
        .execute(&world.pool)
        .await
        .unwrap();

        let outcome = world
            .usecase
            .get_support_display_fields(GetDisplayFieldsCommand {
                organization_id: world.organization_id,
            })
            .await;

        assert!(
            matches!(outcome, Err(CoreError::Internal(_))),
            "{stored} read as {outcome:?}"
        );
    }

    world.cleanup().await;
}
