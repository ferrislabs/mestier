use serde_json::{Value, json};

use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};
use crate::domain::customer::commands::CreateCustomerCommand;
use crate::{CustomerProfile, CustomerStatus};

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
                name: name.to_string(),
                registration_number: None,
                phone,
                email,
                profile: CustomerProfile::default(),
            })
            .await
        {
            Ok(Ok(customer)) => ConnectorOutcome::Produced(json!({
                "id": customer.id.0,
                "name": customer.name,
            })),
            Ok(Err(refusal)) => ConnectorOutcome::Failed {
                error: refusal.to_string(),
            },
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
            operation: None,
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

    async fn seed_organization(pool: &PgPool) -> OrganizationId {
        let owner_id = common::generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = common::generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{} Org", "shape"),
            format!("{}-{org_id}", "shape"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_created_customer_comes_back_in_the_shape_the_catalogue_samples() {
        let pool = crate::application::test_support::automation_pool().await;
        let org_id = seed_organization(&pool).await;
        let connector = CustomerCreateConnector::new(MestierUseCase::new(
            pool,
            default_authorizer(),
            EventHub::new(),
        ));
        let mut config = serde_json::Map::new();
        config.insert("name".to_string(), json!("Ada Lovelace"));

        let outcome = connector
            .execute(ConnectorInput {
                org_id,
                run_id: Uuid::from_u128(2),
                config: &config,
                credential_id: None,
                operation: None,
            })
            .await;

        let ConnectorOutcome::Produced(value) = outcome else {
            panic!("expected Produced, got {outcome:?}");
        };
        assert_eq!(value["name"], "Ada Lovelace");
        crate::infrastructure::automation::connectors::test_support::assert_same_shape(
            &value,
            "mestier.customer.create",
        );
    }
}
