use common::CoreError;
use serde_json::{Map, Value, json};

use super::fields;
use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::domain::project::commands::CreateProjectFromQuoteCommand;
use crate::{OrganizationId, QuoteId};

pub const MAX_PROJECT_NAME_CHARS: usize = 200;

pub struct ProjectCreateConnector {
    usecase: MestierUseCase,
}

impl ProjectCreateConnector {
    pub fn new(usecase: MestierUseCase) -> Self {
        Self { usecase }
    }
}

struct Request {
    quote_id: QuoteId,
    name: Option<String>,
}

fn request_from(config: &Map<String, Value>) -> Result<Request, String> {
    let quote_id = fields::uuid(config, "quote_id").map(QuoteId)?;
    let name = fields::optional_text(config, "name")?;
    if let Some(name) = name
        && name.chars().count() > MAX_PROJECT_NAME_CHARS
    {
        return Err(format!(
            "`name` holds at most {MAX_PROJECT_NAME_CHARS} characters"
        ));
    }
    Ok(Request {
        quote_id,
        name: name.map(str::to_owned),
    })
}

fn foreign_quote() -> ConnectorOutcome {
    ConnectorOutcome::Failed {
        error: "`quote_id` does not match a quote of this organization".to_owned(),
    }
}

async fn quote_title_of(
    usecase: &MestierUseCase,
    organization_id: OrganizationId,
    quote_id: QuoteId,
) -> Result<String, ConnectorOutcome> {
    match usecase.get_quote(quote_id).await {
        Ok(quote) if quote.organization_id == organization_id => Ok(quote.title),
        Ok(_) | Err(CoreError::NotFound) => Err(foreign_quote()),
        Err(error) => Err(ConnectorOutcome::Failed {
            error: error.to_string(),
        }),
    }
}

impl Connector for ProjectCreateConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let request = match request_from(input.config) {
            Ok(request) => request,
            Err(error) => return ConnectorOutcome::Failed { error },
        };

        let usecase = self.usecase.acting_as_automation(input.run_id);

        let title = match quote_title_of(&usecase, input.org_id, request.quote_id).await {
            Ok(title) => title,
            Err(failure) => return failure,
        };

        match usecase
            .create_project_from_quote(CreateProjectFromQuoteCommand {
                actor: authz::Subject::system(),
                quote_id: request.quote_id,
                name: request.name.unwrap_or(title),
                force_new: false,
                tasks: Vec::new(),
            })
            .await
        {
            Ok((project, _)) => ConnectorOutcome::Produced(json!({
                "project_id": project.id.0,
                "name": project.name,
                "quote_id": request.quote_id.0,
            })),
            Err(CoreError::NotFound) => foreign_quote(),
            Err(error) => ConnectorOutcome::Failed {
                error: error.to_string(),
            },
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

    const A_UUID: &str = "018f3b2a-0000-7000-8000-000000000012";

    fn lazy_connector() -> ProjectCreateConnector {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        ProjectCreateConnector::new(MestierUseCase::new(
            pool,
            default_authorizer(),
            EventHub::new(),
        ))
    }

    fn config(entries: &[(&str, Value)]) -> Map<String, Value> {
        entries
            .iter()
            .map(|(name, value)| ((*name).to_owned(), value.clone()))
            .collect()
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
    async fn an_invalid_config_fails_naming_the_field_without_echoing_it() {
        let secret = "do-not-leak-this-token";
        let cases = [
            (config(&[]), "`quote_id` must resolve to a string"),
            (
                config(&[("quote_id", json!(3))]),
                "`quote_id` must resolve to a string",
            ),
            (
                config(&[("quote_id", json!(secret))]),
                "`quote_id` must be a UUID",
            ),
            (
                config(&[("quote_id", json!(A_UUID)), ("name", json!(4))]),
                "`name` must resolve to a string",
            ),
            (
                config(&[
                    ("quote_id", json!(A_UUID)),
                    ("name", json!("x".repeat(MAX_PROJECT_NAME_CHARS + 1))),
                ]),
                "`name` holds at most 200 characters",
            ),
        ];

        for (config, expected) in cases {
            let outcome = lazy_connector()
                .execute(input(OrganizationId(Uuid::from_u128(1)), &config))
                .await;

            let error = failure(outcome);
            assert_eq!(error, expected);
            assert!(!error.contains(secret));
        }
    }

    #[test]
    fn the_dry_run_answers_with_the_sample_and_the_sample_names_what_a_run_returns() {
        let catalogue = connector_catalogue();
        let descriptor = catalogue.get("mestier.project.create", 1).unwrap();
        let config = Map::new();

        let outcome = ProjectCreateConnector::dry_run(
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
            &json!({ "project_id": A_UUID, "name": "x", "quote_id": A_UUID }),
            "mestier.project.create",
        );
    }

    async fn projects_of(pool: &PgPool, quote_id: Uuid) -> i64 {
        world_support::count(
            pool,
            "SELECT count(*) FROM projects WHERE quote_id = $1",
            quote_id,
        )
        .await
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_accepted_quote_of_the_run_organization_becomes_a_project() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "project-create").await;
        let customer = world_support::customer(&pool, org).await;
        let quote = world_support::quote(&pool, org, &customer, "ACCEPTED", "Rénovation", 1).await;
        let connector = ProjectCreateConnector::new(MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            EventHub::new(),
        ));

        let named = connector
            .execute(input(
                org,
                &config(&[
                    ("quote_id", json!(quote.to_string())),
                    ("name", json!("Chantier Fontaine")),
                ]),
            ))
            .await;

        let ConnectorOutcome::Produced(output) = named else {
            panic!("expected an output, got {named:?}");
        };
        assert_eq!(output["name"], "Chantier Fontaine");
        assert_eq!(output["quote_id"], quote.to_string());
        assert_same_shape(&output, "mestier.project.create");
        let project_id: Uuid = output["project_id"].as_str().unwrap().parse().unwrap();
        let stored: (Uuid, String, Option<Uuid>) =
            sqlx::query_as("SELECT org_id, name, customer_id FROM projects WHERE id = $1")
                .bind(project_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(stored.0, org.0);
        assert_eq!(stored.1, "Chantier Fontaine");
        assert_eq!(stored.2, Some(customer.id));

        let again = connector
            .execute(input(
                org,
                &config(&[("quote_id", json!(quote.to_string()))]),
            ))
            .await;
        assert!(
            failure(again).contains("already has a project"),
            "a second project for the same quote is refused"
        );
        assert_eq!(projects_of(&pool, quote).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_missing_name_falls_back_to_the_quote_title() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "project-title").await;
        let customer = world_support::customer(&pool, org).await;
        let quote =
            world_support::quote(&pool, org, &customer, "ACCEPTED", "Pose de parquet", 1).await;
        let connector = ProjectCreateConnector::new(MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            EventHub::new(),
        ));

        let outcome = connector
            .execute(input(
                org,
                &config(&[
                    ("quote_id", json!(quote.to_string())),
                    ("name", json!("  ")),
                ]),
            ))
            .await;

        let ConnectorOutcome::Produced(output) = outcome else {
            panic!("expected an output, got {outcome:?}");
        };
        assert_eq!(output["name"], "Pose de parquet");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_quote_that_is_not_accepted_is_refused_and_creates_nothing() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "project-draft").await;
        let customer = world_support::customer(&pool, org).await;
        let quote = world_support::quote(&pool, org, &customer, "DRAFT", "Brouillon", 1).await;
        let connector = ProjectCreateConnector::new(MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            EventHub::new(),
        ));

        let outcome = connector
            .execute(input(
                org,
                &config(&[("quote_id", json!(quote.to_string()))]),
            ))
            .await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
        assert_eq!(projects_of(&pool, quote).await, 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_quote_of_another_organization_is_refused_and_creates_nothing() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let acme = world_support::organization(&pool, "project-acme").await;
        let globex = world_support::organization(&pool, "project-globex").await;
        let customer = world_support::customer(&pool, acme).await;
        let quote = world_support::quote(&pool, acme, &customer, "ACCEPTED", "Chez Acme", 1).await;
        let connector = ProjectCreateConnector::new(MestierUseCase::new(
            pool.clone(),
            default_authorizer(),
            EventHub::new(),
        ));

        let from_globex = connector
            .execute(input(
                globex,
                &config(&[("quote_id", json!(quote.to_string()))]),
            ))
            .await;
        let unknown = connector
            .execute(input(
                acme,
                &config(&[("quote_id", json!(Uuid::from_u128(0x99).to_string()))]),
            ))
            .await;

        let expected = "`quote_id` does not match a quote of this organization";
        assert_eq!(failure(from_globex), expected);
        assert_eq!(failure(unknown), expected);
        assert_eq!(projects_of(&pool, quote).await, 0);
        let projects_in_globex: i64 = world_support::count(
            &pool,
            "SELECT count(*) FROM projects WHERE org_id = $1",
            globex.0,
        )
        .await;
        assert_eq!(projects_in_globex, 0);
    }
}
