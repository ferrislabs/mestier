use discord::{Channel, ChannelId, ChannelType, CreateMessageCommand, MessageAuthor};
use serde_json::{Map, Value, json};

use super::fields;
use crate::OrganizationId;
use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};

pub const MAX_MESSAGE_CHARS: usize = 4_000;

pub struct ChatPostMessageConnector {
    usecase: MestierUseCase,
}

impl ChatPostMessageConnector {
    pub fn new(usecase: MestierUseCase) -> Self {
        Self { usecase }
    }
}

enum Target<'a> {
    Id(ChannelId),
    Name(&'a str),
}

fn target_from(config: &Map<String, Value>) -> Result<Target<'_>, String> {
    let channel = fields::filled(config, "channel", 200)?.trim();
    if let Ok(id) = channel.parse::<uuid::Uuid>() {
        return Ok(Target::Id(ChannelId(id)));
    }
    let name = channel.trim_start_matches('#').trim();
    if name.is_empty() {
        return Err("`channel` must not be blank".to_owned());
    }
    Ok(Target::Name(name))
}

fn resolve<'a>(channels: &'a [Channel], target: &Target<'_>) -> Result<&'a Channel, String> {
    match target {
        Target::Id(id) => channels
            .iter()
            .find(|channel| channel.id == *id)
            .ok_or_else(not_in_organization),
        Target::Name(name) => {
            let mut named = channels.iter().filter(|channel| {
                channel.channel_type == ChannelType::Text && channel.name.eq_ignore_ascii_case(name)
            });
            match (named.next(), named.next()) {
                (Some(channel), None) => Ok(channel),
                (Some(_), Some(_)) => {
                    Err("several channels bear that name, give the channel id instead".to_owned())
                }
                (None, _) => Err(not_in_organization()),
            }
        }
    }
}

fn not_in_organization() -> String {
    "`channel` does not match a channel of this organization".to_owned()
}

async fn post(
    usecase: &MestierUseCase,
    organization_id: OrganizationId,
    config: &Map<String, Value>,
) -> Result<Value, String> {
    let target = target_from(config)?;
    let message = fields::filled(config, "message", MAX_MESSAGE_CHARS)?;

    let channels = usecase
        .list_channels(organization_id)
        .await
        .map_err(|error| error.to_string())?;
    let channel = resolve(&channels, &target)?;

    let posted = usecase
        .create_message(CreateMessageCommand {
            organization_id,
            channel_id: channel.id,
            author: MessageAuthor::System,
            content: message.to_owned(),
            components: None,
            attachments: Vec::new(),
        })
        .await
        .map_err(|error| error.to_string())?;

    Ok(json!({
        "message_id": posted.id.0,
        "channel_id": posted.channel_id.0,
    }))
}

impl Connector for ChatPostMessageConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        match post(&self.usecase, input.org_id, input.config).await {
            Ok(output) => ConnectorOutcome::Produced(output),
            Err(error) => ConnectorOutcome::Failed { error },
        }
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::application::default_authorizer;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::connector::connector_catalogue;
    use crate::domain::automation::run::DryRunInput;
    use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
    use crate::infrastructure::automation::connectors::world_support;
    use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
    use crate::infrastructure::realtime::EventHub;

    fn lazy_connector() -> ChatPostMessageConnector {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        ChatPostMessageConnector::new(MestierUseCase::new(
            pool,
            default_authorizer(),
            EventHub::new(),
        ))
    }

    fn config(channel: Value, message: Value) -> Map<String, Value> {
        let mut config = Map::new();
        config.insert("channel".to_owned(), channel);
        config.insert("message".to_owned(), message);
        config
    }

    fn input(org: OrganizationId, config: &Map<String, Value>) -> ConnectorInput<'_> {
        ConnectorInput {
            org_id: org,
            run_id: Uuid::from_u128(2),
            config,
            credential_id: None,
            operation: None,
        }
    }

    fn failure(outcome: ConnectorOutcome) -> String {
        match outcome {
            ConnectorOutcome::Failed { error } => error,
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn an_invalid_config_fails_naming_the_field() {
        let cases = [
            (
                config(json!(3), json!("hello")),
                "`channel` must resolve to a string",
            ),
            (
                config(json!("  "), json!("hello")),
                "`channel` must not be blank",
            ),
            (
                config(json!("#"), json!("hello")),
                "`channel` must not be blank",
            ),
            (
                config(json!("general"), json!(7)),
                "`message` must resolve to a string",
            ),
            (
                config(json!("general"), json!(" ")),
                "`message` must not be blank",
            ),
            (
                config(json!("general"), json!("x".repeat(MAX_MESSAGE_CHARS + 1))),
                "`message` holds at most 4000 characters",
            ),
        ];

        for (config, expected) in cases {
            let outcome = lazy_connector()
                .execute(input(OrganizationId(Uuid::from_u128(1)), &config))
                .await;

            assert_eq!(failure(outcome), expected);
        }
    }

    #[test]
    fn the_dry_run_answers_with_the_sample_and_the_sample_names_what_a_run_returns() {
        let catalogue = connector_catalogue();
        let descriptor = catalogue.get("mestier.chat.post_message", 1).unwrap();
        let config = Map::new();

        let outcome = ChatPostMessageConnector::dry_run(
            descriptor,
            &DryRunInput {
                config: &config,
                stored: None,
            },
        );

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(descriptor.sample_output.clone())
        );
        assert_same_shape(
            &json!({ "message_id": "x", "channel_id": "y" }),
            "mestier.chat.post_message",
        );
    }

    fn connector(pool: &PgPool) -> ChatPostMessageConnector {
        ChatPostMessageConnector::new(MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            EventHub::new(),
        ))
    }

    async fn messages_in(pool: &PgPool, channel: Uuid) -> Vec<(String, String)> {
        sqlx::query_as("SELECT content, author_type::text FROM chat.messages WHERE channel_id = $1")
            .bind(channel)
            .fetch_all(pool)
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_message_is_posted_in_a_channel_of_the_run_organization_by_id_or_by_name() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "chat-post").await;
        let general = world_support::channel(&pool, org, "general").await;
        let other = world_support::channel(&pool, org, "devis").await;
        let connector = connector(&pool);

        let by_id = connector
            .execute(input(
                org,
                &config(json!(general.to_string()), json!("Par identifiant")),
            ))
            .await;
        let by_name = connector
            .execute(input(org, &config(json!("#General"), json!("Par nom"))))
            .await;

        let ConnectorOutcome::Produced(output) = by_id else {
            panic!("expected an output, got {by_id:?}");
        };
        assert_eq!(output["channel_id"], general.to_string());
        assert_same_shape(&output, "mestier.chat.post_message");
        assert!(
            matches!(by_name, ConnectorOutcome::Produced(_)),
            "{by_name:?}"
        );
        let posted = messages_in(&pool, general).await;
        assert_eq!(posted.len(), 2);
        assert!(posted.iter().all(|(_, author)| author == "SYSTEM"));
        assert!(posted.iter().any(|(content, _)| content == "Par nom"));
        assert!(messages_in(&pool, other).await.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_channel_of_another_organization_or_an_ambiguous_name_is_refused_and_posts_nothing() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let acme = world_support::organization(&pool, "chat-acme").await;
        let globex = world_support::organization(&pool, "chat-globex").await;
        let acme_channel = world_support::channel(&pool, acme, "general").await;
        let globex_channel = world_support::channel(&pool, globex, "general").await;
        let twin_a = world_support::channel(&pool, acme, "annonces").await;
        let twin_b = world_support::channel(&pool, acme, "Annonces").await;
        let connector = connector(&pool);

        let foreign_id = connector
            .execute(input(
                acme,
                &config(json!(globex_channel.to_string()), json!("Intrus")),
            ))
            .await;
        let unknown_name = connector
            .execute(input(acme, &config(json!("inexistant"), json!("Perdu"))))
            .await;
        let ambiguous = connector
            .execute(input(acme, &config(json!("annonces"), json!("Lequel"))))
            .await;

        let not_here = "`channel` does not match a channel of this organization";
        assert_eq!(failure(foreign_id), not_here);
        assert_eq!(failure(unknown_name), not_here);
        assert_eq!(
            failure(ambiguous),
            "several channels bear that name, give the channel id instead"
        );
        for channel in [acme_channel, globex_channel, twin_a, twin_b] {
            assert!(messages_in(&pool, channel).await.is_empty());
        }
    }
}
