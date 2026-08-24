use serde_json::json;

use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};

/// `mestier.task_recurrence.extend_horizon`: pushes forward the materialized
/// horizon of every one of the run's organization's recurrences that needs
/// it. Calls the existing `extend_recurrence_horizons_for_organization` use
/// case rather than touching Postgres itself — see
/// `CustomerCreateConnector`'s own doc for the same reasoning — so this
/// connector carries no logic of its own beyond wiring `input.org_id`
/// through.
///
/// Takes no config: unlike every other connector kind, this one is never
/// placed by a human editing a graph. `MestierUseCase::find_or_create_recurrence_horizon_workflow`
/// is its only placer, and the run it executes in always already knows
/// which organization it belongs to.
pub struct TaskRecurrenceExtendHorizonConnector {
    usecase: MestierUseCase,
}

impl TaskRecurrenceExtendHorizonConnector {
    pub fn new(usecase: MestierUseCase) -> Self {
        Self { usecase }
    }
}

impl Connector for TaskRecurrenceExtendHorizonConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        let usecase = self.usecase.acting_as_automation(input.run_id);

        match usecase
            .extend_recurrence_horizons_for_organization(input.org_id)
            .await
        {
            Ok(materialized) => ConnectorOutcome::Produced(json!({ "materialized": materialized })),
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

    fn connector() -> TaskRecurrenceExtendHorizonConnector {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        TaskRecurrenceExtendHorizonConnector::new(MestierUseCase::new(
            pool,
            default_authorizer(),
            EventHub::new(),
        ))
    }

    /// Proves the connector is wired to the use case rather than a stub:
    /// with no live database behind the lazy pool, calling `execute` must
    /// reach all the way down to a real I/O failure, not succeed on config
    /// alone (there is no config to fail on) the way a misrouted connector
    /// silently would.
    #[tokio::test]
    async fn execute_reaches_the_use_case_and_surfaces_its_failure() {
        let config = serde_json::Map::new();
        let input = ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config: &config,
            credential_id: None,
        };

        let outcome = connector().execute(input).await;

        assert!(
            matches!(outcome, ConnectorOutcome::Failed { .. }),
            "{outcome:?}"
        );
    }
}
