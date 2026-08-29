use serde_json::{Value, json};

use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::domain::customer::commands::CreateCustomerCommand;
use crate::{CustomerPipelineStage, CustomerStatus};

/// `mestier.customer.create`: the one internal connector the run engine
/// (#200) proves itself against. It calls the existing `create_customer` use
/// case rather than talking to Postgres itself, so every business rule
/// already enforced there (a blank name is refused, for instance) applies
/// here too, with no second implementation to keep in sync.
pub struct CustomerCreateConnector {
    usecase: MestierUseCase,
}

impl CustomerCreateConnector {
    pub fn new(usecase: MestierUseCase) -> Self {
        Self { usecase }
    }
}

impl Connector for CustomerCreateConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let Some(name) = input.config.get("name").and_then(Value::as_str) else {
            return ConnectorOutcome::Failed {
                error: "`name` must resolve to a string".to_string(),
            };
        };
        let email = input
            .config
            .get("email")
            .and_then(Value::as_str)
            .map(str::to_string);
        let phone = input
            .config
            .get("phone")
            .and_then(Value::as_str)
            .map(str::to_string);

        // Attributed to the run, not to the system: whatever this call
        // chain writes — today `create_customer`, and anything a future
        // connector calls at any depth — is attributable to the automation
        // that caused it.
        let usecase = self.usecase.acting_as_automation(input.run_id);

        match usecase
            .create_customer(CreateCustomerCommand {
                // Attributed to the run rather than to a human actor:
                // `authz::Subject::system()` short-circuits
                // `policy::enrich_for_organization` and is allowed every
                // action by the default policy engine, matching the
                // automation semantics documented above.
                actor: authz::Subject::system(),
                organization_id: input.org_id,
                status: CustomerStatus::Prospect,
                pipeline_stage: CustomerPipelineStage::New,
                name: name.to_string(),
                registration_number: None,
                phone,
                email,
            })
            .await
        {
            Ok(customer) => ConnectorOutcome::Produced(json!({
                "id": customer.id.0,
                "name": customer.name,
            })),
            Err(error) => ConnectorOutcome::Failed {
                error: error.to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use common::OrganizationId;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::application::default_authorizer;
    use crate::infrastructure::realtime::EventHub;

    /// `connect_lazy` builds a pool without touching the network, which is
    /// enough for the tests below: they never reach the use case because the
    /// config fails to resolve first.
    fn connector() -> CustomerCreateConnector {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        CustomerCreateConnector::new(MestierUseCase::new(
            pool,
            default_authorizer(),
            EventHub::new(),
        ))
    }

    fn input(config: &serde_json::Map<String, Value>) -> ConnectorInput<'_> {
        ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config,
            credential_id: None,
        }
    }

    #[tokio::test]
    async fn a_missing_name_fails_without_touching_the_use_case() {
        let config = serde_json::Map::new();

        let outcome = connector().execute(input(&config)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }

    #[tokio::test]
    async fn a_non_string_name_fails_without_touching_the_use_case() {
        let mut config = serde_json::Map::new();
        config.insert("name".to_string(), json!(42));

        let outcome = connector().execute(input(&config)).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }
}
