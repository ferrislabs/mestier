use std::time::Duration;

use authz::Subject;
use aws_lc_rs::digest;
use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use tokio::{sync::OnceCell, time::sleep};
use uuid::Uuid;

use crate::{
    CreateApiKeyCommand, CreatedApiKey, ListApiKeysCommand, Permissions, PresentedApiKey,
    RevokeApiKeyCommand, SupportApiKeyId, api_key_matches,
    application::{
        MestierUseCase, default_authorizer, policy,
        test_support::{purge, scratch_pool},
    },
    infrastructure::realtime::EventHub,
};

static SUPPORT_API_KEY_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_support_api_key_test_", &SUPPORT_API_KEY_SCRATCH).await
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
    other_organization_id: OrganizationId,
    admin: Person,
    support_manager: Person,
    plain: Person,
    outsider: Person,
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

async fn seed_organization(pool: &PgPool, owner_id: Uuid) -> OrganizationId {
    let organization_id = OrganizationId(generate_uuid_v7());
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id.0)
        .bind("Treyve Paysages")
        .bind(format!("org-{}", organization_id.0))
        .bind(owner_id)
        .execute(pool)
        .await
        .unwrap();
    organization_id
}

async fn seed_member(
    pool: &PgPool,
    organization_id: OrganizationId,
    user_id: Uuid,
    label: &str,
    permissions: Option<Permissions>,
) {
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
    if let Some(permissions) = permissions {
        let role_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
        )
        .bind(role_id)
        .bind(organization_id.0)
        .bind(label)
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
}

impl World {
    async fn new(pool: &PgPool) -> Self {
        let admin = seed_user(pool, "Admin").await;
        let support_manager = seed_user(pool, "SupportManager").await;
        let plain = seed_user(pool, "Plain").await;
        let outsider = seed_user(pool, "Outsider").await;

        let organization_id = seed_organization(pool, admin).await;
        let other_organization_id = seed_organization(pool, outsider).await;

        seed_member(
            pool,
            organization_id,
            admin,
            "Admin",
            Some(Permissions::MANAGE_ORG),
        )
        .await;
        seed_member(
            pool,
            organization_id,
            support_manager,
            "SupportManager",
            Some(Permissions::MANAGE_SUPPORT | Permissions::VIEW_SUPPORT),
        )
        .await;
        seed_member(pool, organization_id, plain, "Plain", None).await;
        seed_member(
            pool,
            other_organization_id,
            outsider,
            "Outsider",
            Some(Permissions::MANAGE_ORG),
        )
        .await;

        Self {
            pool: pool.clone(),
            usecase: MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new()),
            organization_id,
            other_organization_id,
            admin: Person { user_id: admin },
            support_manager: Person {
                user_id: support_manager,
            },
            plain: Person { user_id: plain },
            outsider: Person { user_id: outsider },
        }
    }

    async fn create_in(
        &self,
        actor: &Person,
        organization_id: OrganizationId,
        name: &str,
    ) -> Result<CreatedApiKey, CoreError> {
        self.usecase
            .create_support_api_key(CreateApiKeyCommand {
                actor: actor.subject(),
                organization_id,
                name: name.to_owned(),
            })
            .await
    }

    async fn create(&self, name: &str) -> CreatedApiKey {
        self.create_in(&self.admin, self.organization_id, name)
            .await
            .unwrap()
    }

    async fn create_for_other_organization(&self, name: &str) -> CreatedApiKey {
        self.create_in(&self.outsider, self.other_organization_id, name)
            .await
            .unwrap()
    }

    async fn revoke_as(
        &self,
        actor: &Person,
        organization_id: OrganizationId,
        key_id: SupportApiKeyId,
    ) -> Result<(), CoreError> {
        self.usecase
            .revoke_support_api_key(RevokeApiKeyCommand {
                actor: actor.subject(),
                organization_id,
                key_id,
            })
            .await
    }

    async fn revoke(&self, key_id: SupportApiKeyId) {
        self.revoke_as(&self.admin, self.organization_id, key_id)
            .await
            .unwrap();
    }

    async fn authenticate(&self, plaintext: &str) -> Option<OrganizationId> {
        self.usecase
            .authenticate_support_api_key(plaintext)
            .await
            .unwrap()
    }

    async fn key_rows(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM support_api_keys WHERE org_id = ANY($1)")
            .bind(vec![self.organization_id.0, self.other_organization_id.0])
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn event_rows(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM automation.event WHERE org_id = ANY($1)")
            .bind(vec![self.organization_id.0, self.other_organization_id.0])
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn revoked_at(&self, key_id: SupportApiKeyId) -> Option<DateTime<Utc>> {
        sqlx::query_scalar("SELECT revoked_at FROM support_api_keys WHERE id = $1")
            .bind(key_id.0)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn last_used_at(&self, key_id: SupportApiKeyId) -> Option<DateTime<Utc>> {
        sqlx::query_scalar("SELECT last_used_at FROM support_api_keys WHERE id = $1")
            .bind(key_id.0)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn age_last_used_at(&self, key_id: SupportApiKeyId) {
        sqlx::query(
            "UPDATE support_api_keys SET last_used_at = last_used_at - interval '2 minutes' WHERE id = $1",
        )
        .bind(key_id.0)
        .execute(&self.pool)
        .await
        .unwrap();
    }

    async fn cleanup(self) {
        for organization_id in [self.organization_id, self.other_organization_id] {
            for statement in [
                "DELETE FROM automation.event WHERE org_id = $1",
                "DELETE FROM support_api_keys WHERE org_id = $1",
                "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
                "DELETE FROM roles WHERE organization_id = $1",
                "DELETE FROM organization_members WHERE organization_id = $1",
                "DELETE FROM organizations WHERE id = $1",
            ] {
                purge(&self.pool, statement, organization_id.0).await;
            }
        }
        for person in [
            &self.admin,
            &self.support_manager,
            &self.plain,
            &self.outsider,
        ] {
            purge(
                &self.pool,
                "DELETE FROM users WHERE id = $1",
                person.user_id,
            )
            .await;
        }
    }
}

fn secret_of(created: &CreatedApiKey) -> &str {
    created.plaintext.split_once('.').unwrap().1
}

fn with_secret_changed(plaintext: &str) -> String {
    let (prefix, secret) = plaintext.split_once('.').unwrap();
    let flipped = if secret.starts_with('0') { '1' } else { '0' };
    format!("{prefix}.{flipped}{}", &secret[1..])
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn create_returns_the_plaintext_once_and_stores_only_its_hash() {
    let world = World::new(&pool().await).await;

    let created = world.create("  Formulaire de contact  ").await;

    let presented = PresentedApiKey::parse(&created.plaintext).unwrap();
    assert_eq!(created.key.organization_id, world.organization_id);
    assert_eq!(created.key.name, "Formulaire de contact");
    assert_eq!(created.key.prefix, presented.prefix);
    assert_eq!(created.key.last_used_at, None);
    assert_eq!(created.key.revoked_at, None);
    assert!(api_key_matches(&created.key.secret_hash, &presented));

    let stored_hash: Vec<u8> =
        sqlx::query_scalar("SELECT secret_hash FROM support_api_keys WHERE id = $1")
            .bind(created.key.id.0)
            .fetch_one(&world.pool)
            .await
            .unwrap();
    let expected = digest::digest(&digest::SHA256, secret_of(&created).as_bytes());
    assert_eq!(stored_hash.as_slice(), expected.as_ref());
    assert_eq!(created.key.secret_hash, stored_hash);

    let whole_row: String =
        sqlx::query_scalar("SELECT t::text FROM support_api_keys t WHERE t.id = $1")
            .bind(created.key.id.0)
            .fetch_one(&world.pool)
            .await
            .unwrap();
    assert!(!whole_row.contains(secret_of(&created)), "{whole_row}");
    assert!(!whole_row.contains(&created.plaintext), "{whole_row}");
    assert!(
        !stored_hash
            .windows(secret_of(&created).len())
            .any(|window| window == secret_of(&created).as_bytes())
    );
    assert_eq!(world.event_rows().await, 0);
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn two_keys_get_distinct_prefixes_and_secrets() {
    let world = World::new(&pool().await).await;

    let first = world.create("Premier").await;
    let second = world.create("Second").await;

    assert_ne!(first.key.prefix, second.key.prefix);
    assert_ne!(first.plaintext, second.plaintext);
    assert_ne!(first.key.secret_hash, second.key.secret_hash);
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_blank_or_overlong_name_is_refused_and_nothing_is_written() {
    let world = World::new(&pool().await).await;

    for name in ["", "   ", &"x".repeat(101)] {
        let result = world
            .create_in(&world.admin, world.organization_id, name)
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))), "{name:?}");
    }

    assert_eq!(world.key_rows().await, 0);
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn list_never_exposes_the_plaintext_and_keeps_revoked_keys_newest_first() {
    let world = World::new(&pool().await).await;
    let first = world.create("Premier").await;
    let second = world.create("Second").await;
    let third = world.create("Troisième").await;
    world.create_for_other_organization("Ailleurs").await;
    world.revoke(second.key.id).await;

    let listed = world
        .usecase
        .list_support_api_keys(ListApiKeysCommand {
            actor: world.admin.subject(),
            organization_id: world.organization_id,
        })
        .await
        .unwrap();

    let ids: Vec<SupportApiKeyId> = listed.iter().map(|key| key.id).collect();
    assert_eq!(ids, [third.key.id, second.key.id, first.key.id]);
    assert_eq!(listed[0].revoked_at, None);
    assert!(listed[1].revoked_at.is_some());
    assert_eq!(listed[2].revoked_at, None);
    let rendered = format!("{listed:?}");
    for created in [&first, &second, &third] {
        assert!(!rendered.contains(secret_of(created)), "{rendered}");
        assert!(!rendered.contains(&created.plaintext), "{rendered}");
    }
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_without_manage_org_is_refused_on_create_list_and_revoke() {
    let world = World::new(&pool().await).await;
    let existing = world.create("Existant").await;

    for person in [&world.support_manager, &world.plain] {
        let created = world
            .create_in(person, world.organization_id, "Intrus")
            .await;
        let listed = world
            .usecase
            .list_support_api_keys(ListApiKeysCommand {
                actor: person.subject(),
                organization_id: world.organization_id,
            })
            .await;
        let revoked = world
            .revoke_as(person, world.organization_id, existing.key.id)
            .await;

        assert!(matches!(created, Err(CoreError::Forbidden { .. })));
        assert!(matches!(listed, Err(CoreError::Forbidden { .. })));
        assert!(matches!(revoked, Err(CoreError::Forbidden { .. })));
    }

    assert_eq!(world.key_rows().await, 1);
    assert_eq!(world.revoked_at(existing.key.id).await, None);
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_administrator_of_another_organization_is_refused_like_any_non_member() {
    let world = World::new(&pool().await).await;
    let existing = world.create("Existant").await;

    let created = world
        .create_in(&world.outsider, world.organization_id, "Intrus")
        .await;
    let listed = world
        .usecase
        .list_support_api_keys(ListApiKeysCommand {
            actor: world.outsider.subject(),
            organization_id: world.organization_id,
        })
        .await;
    let revoked = world
        .revoke_as(&world.outsider, world.organization_id, existing.key.id)
        .await;

    assert!(matches!(created, Err(CoreError::Forbidden { .. })));
    assert!(matches!(listed, Err(CoreError::Forbidden { .. })));
    assert!(matches!(revoked, Err(CoreError::Forbidden { .. })));
    assert_eq!(world.key_rows().await, 1);
    assert_eq!(world.revoked_at(existing.key.id).await, None);
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn revoke_is_idempotent_and_keeps_the_first_revoked_at() {
    let world = World::new(&pool().await).await;
    let created = world.create("Formulaire").await;

    world.revoke(created.key.id).await;
    let first_revocation = world.revoked_at(created.key.id).await;
    assert!(first_revocation.is_some());
    sleep(Duration::from_millis(50)).await;
    world.revoke(created.key.id).await;

    assert_eq!(world.revoked_at(created.key.id).await, first_revocation);
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn revoking_a_key_of_another_organization_is_not_found_and_leaves_it_active() {
    let world = World::new(&pool().await).await;
    let foreign = world.create_for_other_organization("Ailleurs").await;

    let result = world
        .revoke_as(&world.admin, world.organization_id, foreign.key.id)
        .await;
    let unknown = world
        .revoke_as(
            &world.admin,
            world.organization_id,
            SupportApiKeyId(generate_uuid_v7()),
        )
        .await;

    assert!(matches!(result, Err(CoreError::NotFound)));
    assert!(matches!(unknown, Err(CoreError::NotFound)));
    assert_eq!(world.revoked_at(foreign.key.id).await, None);
    assert_eq!(
        world.authenticate(&foreign.plaintext).await,
        Some(world.other_organization_id)
    );
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_valid_key_authenticates_as_its_organization_and_stamps_last_used_at_on_first_use() {
    let world = World::new(&pool().await).await;
    let created = world.create("Formulaire").await;
    let other = world.create_for_other_organization("Ailleurs").await;
    assert_eq!(world.last_used_at(created.key.id).await, None);

    let authenticated = world.authenticate(&created.plaintext).await;
    let first_use = world.last_used_at(created.key.id).await;
    sleep(Duration::from_millis(50)).await;
    let authenticated_again = world.authenticate(&created.plaintext).await;
    let second_use = world.last_used_at(created.key.id).await;

    assert_eq!(authenticated, Some(world.organization_id));
    assert_eq!(authenticated_again, Some(world.organization_id));
    assert!(first_use.is_some());
    assert_eq!(second_use, first_use);
    assert_eq!(world.last_used_at(other.key.id).await, None);
    assert_eq!(world.event_rows().await, 0);
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn last_used_at_moves_again_once_its_stamp_is_older_than_a_minute() {
    let world = World::new(&pool().await).await;
    let created = world.create("Formulaire").await;
    world.authenticate(&created.plaintext).await.unwrap();
    let first_use = world.last_used_at(created.key.id).await.unwrap();
    world.age_last_used_at(created.key.id).await;
    let aged = world.last_used_at(created.key.id).await.unwrap();

    let authenticated = world.authenticate(&created.plaintext).await;
    let renewed = world.last_used_at(created.key.id).await.unwrap();

    assert_eq!(authenticated, Some(world.organization_id));
    assert!(aged < first_use);
    assert!(renewed >= first_use);
    assert!(renewed > aged + chrono::Duration::minutes(1));
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_revoked_key_no_longer_authenticates_and_is_not_touched() {
    let world = World::new(&pool().await).await;
    let created = world.create("Formulaire").await;
    world.authenticate(&created.plaintext).await.unwrap();
    let last_use = world.last_used_at(created.key.id).await;
    world.revoke(created.key.id).await;
    sleep(Duration::from_millis(50)).await;

    let authenticated = world.authenticate(&created.plaintext).await;

    assert_eq!(authenticated, None);
    assert_eq!(world.last_used_at(created.key.id).await, last_use);
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_wrong_secret_does_not_authenticate_and_does_not_touch_the_key() {
    let world = World::new(&pool().await).await;
    let created = world.create("Formulaire").await;

    let authenticated = world
        .authenticate(&with_secret_changed(&created.plaintext))
        .await;

    assert_eq!(authenticated, None);
    assert_eq!(world.last_used_at(created.key.id).await, None);
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_unknown_prefix_does_not_authenticate() {
    let world = World::new(&pool().await).await;
    let created = world.create("Formulaire").await;
    let (prefix, secret) = created.plaintext.split_once('.').unwrap();
    let unknown_prefix = if prefix.starts_with('0') { "1" } else { "0" };

    let authenticated = world
        .authenticate(&format!("{unknown_prefix}{}.{secret}", &prefix[1..]))
        .await;

    assert_eq!(authenticated, None);
    assert_eq!(world.last_used_at(created.key.id).await, None);
    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_malformed_key_does_not_authenticate() {
    let world = World::new(&pool().await).await;
    let created = world.create("Formulaire").await;
    let (prefix, secret) = created.plaintext.split_once('.').unwrap();

    for malformed in [
        String::new(),
        ".".to_owned(),
        prefix.to_owned(),
        secret.to_owned(),
        format!("{prefix}{secret}"),
        format!("{prefix}.{secret}.{secret}"),
        format!("{prefix}.{secret} "),
        format!(" {prefix}.{secret}"),
        format!("{}.{secret}", prefix.to_uppercase()),
        format!("{prefix}.{}", secret.to_uppercase()),
        format!("{prefix}.{}", &secret[..63]),
    ] {
        if malformed == created.plaintext {
            continue;
        }
        assert_eq!(world.authenticate(&malformed).await, None, "{malformed:?}");
    }

    assert_eq!(world.last_used_at(created.key.id).await, None);
    world.cleanup().await;
}
