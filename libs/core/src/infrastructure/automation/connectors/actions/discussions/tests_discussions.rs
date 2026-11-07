use common::{OrganizationId, generate_uuid_v7};
use serde_json::{Map, Value, json};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::application::test_support::scratch_pool;
use crate::application::{MestierUseCase, default_authorizer};
use crate::domain::automation::connector::connector_catalogue;
use crate::domain::automation::run::{ConnectorInput, ConnectorOutcome, DryRunInput};
use crate::infrastructure::automation::connectors::ConnectorRegistry;
use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
use crate::infrastructure::realtime::EventHub;

static SCRATCH: OnceCell<String> = OnceCell::const_new();

const NOT_FOUND: &str = "not found: the targeted object does not exist in this organization";

struct World {
    pool: PgPool,
    registry: ConnectorRegistry,
    organization_id: OrganizationId,
}

impl World {
    async fn new(pool: &PgPool) -> Self {
        let owner = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(owner)
        .bind(format!("owner-{owner}@example.com"))
        .bind(format!("owner-{owner}"))
        .bind("Owner")
        .bind(format!("sub-{owner}"))
        .execute(pool)
        .await
        .unwrap();
        let organization_id = OrganizationId(generate_uuid_v7());
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id.0)
            .bind("Treyve Paysages")
            .bind(format!("org-{}", organization_id.0))
            .bind(owner)
            .execute(pool)
            .await
            .unwrap();
        let usecase = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());
        Self {
            pool: pool.clone(),
            registry: ConnectorRegistry::new(usecase),
            organization_id,
        }
    }

    async fn run(&self, kind: &str, pairs: &[(&str, Value)]) -> ConnectorOutcome {
        self.run_as(self.organization_id, kind, pairs).await
    }

    async fn run_as(
        &self,
        organization_id: OrganizationId,
        kind: &str,
        pairs: &[(&str, Value)],
    ) -> ConnectorOutcome {
        let config: Map<String, Value> = pairs
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect();
        self.registry
            .execute(
                kind,
                1,
                ConnectorInput {
                    org_id: organization_id,
                    run_id: Uuid::from_u128(2),
                    config: &config,
                    credential_id: None,
                    operation: None,
                },
            )
            .await
            .unwrap()
    }

    async fn ok(&self, kind: &str, pairs: &[(&str, Value)]) -> Value {
        match self.run(kind, pairs).await {
            ConnectorOutcome::Produced(value) => value,
            other => panic!("{kind}: expected an output, got {other:?}"),
        }
    }

    async fn message(&self, channel: &Value, content: &str) -> Uuid {
        let id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO chat.messages (id, org_id, channel_id, author_type, content) \
             VALUES ($1, $2, $3, 'SYSTEM', $4)",
        )
        .bind(id)
        .bind(self.organization_id.0)
        .bind(channel["id"].as_str().unwrap().parse::<Uuid>().unwrap())
        .bind(content)
        .execute(&self.pool)
        .await
        .unwrap();
        id
    }
}

fn failed(outcome: ConnectorOutcome) -> String {
    match outcome {
        ConnectorOutcome::Failed { error } => error,
        other => panic!("expected a failure, got {other:?}"),
    }
}

async fn pool() -> PgPool {
    scratch_pool("mestier_lot9b_disc_", &SCRATCH).await
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_category_and_a_channel_are_created_read_and_listed() {
    let world = World::new(&pool().await).await;

    let category = world
        .ok(
            "mestier.discussions.create_category",
            &[("name", json!("Chantiers")), ("position", json!(2))],
        )
        .await;
    let channel = world
        .ok(
            "mestier.discussions.create_channel",
            &[
                ("name", json!("general")),
                ("category_id", category["id"].clone()),
            ],
        )
        .await;
    let got = world
        .ok(
            "mestier.discussions.get_channel",
            &[("channel_id", channel["id"].clone())],
        )
        .await;
    let plain = world
        .ok(
            "mestier.discussions.create_channel",
            &[("name", json!("plain"))],
        )
        .await;
    let plain_got = world
        .ok(
            "mestier.discussions.get_channel",
            &[("channel_id", plain["id"].clone())],
        )
        .await;
    let listed = world.ok("mestier.discussions.list_channels", &[]).await;
    let categories = world.ok("mestier.discussions.list_categories", &[]).await;

    assert_eq!(got["name"], "general");
    assert_eq!(got["category_id"], category["id"]);
    assert_eq!(listed.as_array().unwrap().len(), 2);
    assert_eq!(categories.as_array().unwrap().len(), 1);
    assert_same_shape(&category, "mestier.discussions.create_category");
    assert_same_shape(&plain_got, "mestier.discussions.get_channel");
    assert_same_shape(&plain, "mestier.discussions.create_channel");
    assert_same_shape(&categories, "mestier.discussions.list_categories");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_channel_keeps_what_an_update_does_not_name_and_a_thread_is_managed() {
    let world = World::new(&pool().await).await;
    let channel = world
        .ok(
            "mestier.discussions.create_channel",
            &[("name", json!("general")), ("topic", json!("Accueil"))],
        )
        .await;

    let renamed = world
        .ok(
            "mestier.discussions.update_channel",
            &[
                ("channel_id", channel["id"].clone()),
                ("name", json!("accueil")),
            ],
        )
        .await;
    let thread = world
        .ok(
            "mestier.discussions.create_thread",
            &[
                ("channel_id", channel["id"].clone()),
                ("name", json!("Fil")),
            ],
        )
        .await;
    let archived = world
        .ok(
            "mestier.discussions.update_thread",
            &[
                ("thread_id", thread["id"].clone()),
                ("archived", json!(true)),
            ],
        )
        .await;
    let threads = world
        .ok(
            "mestier.discussions.list_threads",
            &[("channel_id", channel["id"].clone())],
        )
        .await;
    let deleted = world
        .ok(
            "mestier.discussions.delete_thread",
            &[("thread_id", thread["id"].clone())],
        )
        .await;

    assert_eq!(renamed["name"], "accueil");
    assert_eq!(renamed["topic"], "Accueil");
    assert_eq!(archived["archived"], json!(true));
    assert_eq!(threads.as_array().unwrap().len(), 1);
    assert_eq!(deleted["deleted"], json!(true));
    assert_same_shape(&thread, "mestier.discussions.create_thread");
    assert_same_shape(&deleted, "mestier.discussions.delete_thread");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn messages_are_listed_edited_when_the_automation_wrote_them_and_deleted() {
    let world = World::new(&pool().await).await;
    let channel = world
        .ok(
            "mestier.discussions.create_channel",
            &[("name", json!("general"))],
        )
        .await;
    let message = world.message(&channel, "Bonjour").await;
    let target = json!(message.to_string());

    let listed = world
        .ok(
            "mestier.discussions.list_messages",
            &[("channel_id", channel["id"].clone()), ("limit", json!(10))],
        )
        .await;
    let edited = world
        .ok(
            "mestier.discussions.edit_message",
            &[("message_id", target.clone()), ("content", json!("Salut"))],
        )
        .await;
    let reactors = world
        .ok(
            "mestier.discussions.list_reactors",
            &[("message_id", target.clone()), ("emoji", json!("👍"))],
        )
        .await;
    let deleted = world
        .ok(
            "mestier.discussions.delete_message",
            &[("message_id", target)],
        )
        .await;

    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(edited["content"], "Salut");
    assert!(edited["edited_at"].is_string());
    assert_eq!(reactors, json!([]));
    assert_eq!(deleted["deleted"], json!(true));
    assert_same_shape(&listed, "mestier.discussions.list_messages");
    assert_same_shape(&deleted, "mestier.discussions.delete_message");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn permissions_are_set_listed_and_removed_for_everyone() {
    let world = World::new(&pool().await).await;
    let channel = world
        .ok(
            "mestier.discussions.create_channel",
            &[("name", json!("general"))],
        )
        .await;
    let id = channel["id"].clone();

    let set = world
        .ok(
            "mestier.discussions.set_everyone_permissions",
            &[
                ("channel_id", id.clone()),
                ("allow", json!(32)),
                ("deny", json!(64)),
            ],
        )
        .await;
    let listed = world
        .ok(
            "mestier.discussions.list_channel_permissions",
            &[("channel_id", id.clone())],
        )
        .await;
    let removed = world
        .ok(
            "mestier.discussions.remove_everyone_permissions",
            &[("channel_id", id)],
        )
        .await;

    assert_eq!(set["target_type"], "everyone");
    assert_eq!(listed.as_array().unwrap().len(), 1);
    assert_eq!(removed["removed"], json!(true));
    assert_same_shape(&set, "mestier.discussions.set_everyone_permissions");
    assert_same_shape(&removed, "mestier.discussions.remove_everyone_permissions");
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_object_of_another_organization_fails_with_a_named_error_and_nothing_changes() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let category = acme
        .ok(
            "mestier.discussions.create_category",
            &[("name", json!("Chez Acme"))],
        )
        .await;
    let channel = acme
        .ok(
            "mestier.discussions.create_channel",
            &[("name", json!("acme"))],
        )
        .await;
    let message = acme.message(&channel, "Secret").await;
    let channel_id = channel["id"].clone();

    let attempts = [
        (
            "mestier.discussions.update_category",
            vec![
                ("category_id", category["id"].clone()),
                ("name", json!("X")),
            ],
        ),
        (
            "mestier.discussions.delete_category",
            vec![("category_id", category["id"].clone())],
        ),
        (
            "mestier.discussions.get_channel",
            vec![("channel_id", channel_id.clone())],
        ),
        (
            "mestier.discussions.update_channel",
            vec![("channel_id", channel_id.clone()), ("name", json!("X"))],
        ),
        (
            "mestier.discussions.delete_channel",
            vec![("channel_id", channel_id.clone())],
        ),
        (
            "mestier.discussions.create_thread",
            vec![("channel_id", channel_id.clone()), ("name", json!("Fil"))],
        ),
        (
            "mestier.discussions.list_messages",
            vec![("channel_id", channel_id.clone())],
        ),
        (
            "mestier.discussions.delete_message",
            vec![("message_id", json!(message.to_string()))],
        ),
        (
            "mestier.discussions.set_everyone_permissions",
            vec![
                ("channel_id", channel_id.clone()),
                ("allow", json!(1)),
                ("deny", json!(0)),
            ],
        ),
    ];

    for (kind, pairs) in attempts {
        let outcome = acme.run_as(globex.organization_id, kind, &pairs).await;
        assert_eq!(failed(outcome), NOT_FOUND, "{kind}");
    }

    let channels = acme.ok("mestier.discussions.list_channels", &[]).await;
    let categories = acme.ok("mestier.discussions.list_categories", &[]).await;
    let remaining: i64 = sqlx::query_scalar("SELECT count(*) FROM chat.messages WHERE id = $1")
        .bind(message)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(channels.as_array().unwrap().len(), 1);
    assert_eq!(channels[0]["name"], "acme");
    assert_eq!(categories[0]["name"], "Chez Acme");
    assert_eq!(remaining, 1);
    let stranger = globex.ok("mestier.discussions.list_channels", &[]).await;
    assert_eq!(stranger, json!([]));
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_dry_run_of_a_write_action_writes_nothing() {
    let world = World::new(&pool().await).await;
    let catalogue = connector_catalogue();
    let descriptor = catalogue
        .get("mestier.discussions.create_channel", 1)
        .unwrap();
    let config: Map<String, Value> = [("name".to_owned(), json!("fantome"))]
        .into_iter()
        .collect();

    let outcome = ConnectorRegistry::dry_run(
        descriptor,
        &DryRunInput {
            config: &config,
            stored: None,
        },
    );

    assert_eq!(
        outcome,
        Some(ConnectorOutcome::Produced(descriptor.sample_output.clone()))
    );
    let channels = world.ok("mestier.discussions.list_channels", &[]).await;
    assert_eq!(channels, json!([]));
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_target_permission_needs_a_known_kind_and_a_member_of_the_organization() {
    let world = World::new(&pool().await).await;
    let channel = world
        .ok(
            "mestier.discussions.create_channel",
            &[("name", json!("general"))],
        )
        .await;

    let stranger = world
        .run(
            "mestier.discussions.set_target_permissions",
            &[
                ("channel_id", channel["id"].clone()),
                ("target_type", json!("member")),
                ("target_id", json!(Uuid::new_v4().to_string())),
                ("allow", json!(1)),
                ("deny", json!(0)),
            ],
        )
        .await;
    let unknown_kind = world
        .run(
            "mestier.discussions.remove_target_permissions",
            &[
                ("channel_id", channel["id"].clone()),
                ("target_type", json!("group")),
                ("target_id", json!(Uuid::new_v4().to_string())),
            ],
        )
        .await;

    assert_eq!(failed(stranger), NOT_FOUND);
    assert_eq!(
        failed(unknown_kind),
        "`target_type` must be `role` or `member`"
    );
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_webhook_and_a_reaction_are_written_by_the_system_user() {
    let world = World::new(&pool().await).await;
    let channel = world
        .ok(
            "mestier.discussions.create_channel",
            &[("name", json!("general"))],
        )
        .await;
    let message = json!(world.message(&channel, "Bonjour").await.to_string());

    let webhook = world
        .ok(
            "mestier.discussions.create_webhook",
            &[
                ("channel_id", channel["id"].clone()),
                ("name", json!("Alertes")),
            ],
        )
        .await;
    let added = world
        .ok(
            "mestier.discussions.add_reaction",
            &[("message_id", message.clone()), ("emoji", json!("👍"))],
        )
        .await;
    let reactors = world
        .ok(
            "mestier.discussions.list_reactors",
            &[("message_id", message.clone()), ("emoji", json!("👍"))],
        )
        .await;
    let removed = world
        .ok(
            "mestier.discussions.remove_reaction",
            &[("message_id", message.clone()), ("emoji", json!("👍"))],
        )
        .await;
    let after = world
        .ok(
            "mestier.discussions.list_reactors",
            &[("message_id", message), ("emoji", json!("👍"))],
        )
        .await;

    let author: String = sqlx::query_scalar("SELECT sub FROM users WHERE id = $1")
        .bind(
            webhook["created_by"]
                .as_str()
                .unwrap()
                .parse::<Uuid>()
                .unwrap(),
        )
        .fetch_one(&world.pool)
        .await
        .unwrap();
    assert_same_shape(&webhook, "mestier.discussions.create_webhook");
    assert_same_shape(&added, "mestier.discussions.add_reaction");
    assert_same_shape(&removed, "mestier.discussions.remove_reaction");
    assert!(webhook.get("token").is_none());
    assert_eq!(author, "system:automation");
    assert_eq!(reactors.as_array().unwrap().len(), 1);
    assert_eq!(reactors[0]["user_id"], webhook["created_by"]);
    assert_eq!(after, json!([]));
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_webhook_or_reaction_on_another_organization_fails_with_a_named_error() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let channel = acme
        .ok(
            "mestier.discussions.create_channel",
            &[("name", json!("acme"))],
        )
        .await;
    let message = json!(acme.message(&channel, "Secret").await.to_string());

    let attempts = [
        (
            "mestier.discussions.create_webhook",
            vec![("channel_id", channel["id"].clone()), ("name", json!("X"))],
        ),
        (
            "mestier.discussions.add_reaction",
            vec![("message_id", message.clone()), ("emoji", json!("👍"))],
        ),
        (
            "mestier.discussions.remove_reaction",
            vec![("message_id", message.clone()), ("emoji", json!("👍"))],
        ),
    ];
    for (kind, pairs) in attempts {
        assert_eq!(failed(globex.run(kind, &pairs).await), NOT_FOUND, "{kind}");
    }

    let webhooks: i64 = sqlx::query_scalar("SELECT count(*) FROM chat.webhooks WHERE org_id = $1")
        .bind(acme.organization_id.0)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(webhooks, 0);
}
