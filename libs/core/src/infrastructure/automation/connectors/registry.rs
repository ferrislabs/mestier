use thiserror::Error;

use crate::application::MestierUseCase;
use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};

use super::customer_create::CustomerCreateConnector;
use super::flow_condition::FlowConditionConnector;
use super::flow_loop::FlowLoopConnector;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("no connector implementation for `{kind}` version {version}")]
pub struct UnknownConnectorKind {
    pub kind: String,
    pub version: u16,
}

/// Every executable connector kind, dispatched by `(kind, version)`. A
/// concrete struct rather than a `HashMap<_, Box<dyn Connector>>`: only three
/// kinds exist in this ticket, and `Connector::execute` returning
/// `impl Future` (not object-safe) would otherwise force boxing every call.
///
/// [`Self::known_kinds`] is the anti-drift test's other half — see
/// `domain::automation::connector::catalogue` for the descriptors this is
/// checked against: every catalogue entry must have an implementation here,
/// and every implementation here must be in the catalogue, or the two silently
/// drift apart the way #197 could not yet guard against.
pub struct ConnectorRegistry {
    flow_loop: FlowLoopConnector,
    flow_condition: FlowConditionConnector,
    customer_create: CustomerCreateConnector,
}

impl ConnectorRegistry {
    pub fn new(usecase: MestierUseCase) -> Self {
        Self {
            flow_loop: FlowLoopConnector,
            flow_condition: FlowConditionConnector,
            customer_create: CustomerCreateConnector::new(usecase),
        }
    }

    /// Every `(kind, version)` this registry can dispatch to.
    pub fn known_kinds() -> &'static [(&'static str, u16)] {
        &[
            ("flow.loop", 1),
            ("flow.condition", 1),
            ("mestier.customer.create", 1),
        ]
    }

    pub async fn execute(
        &self,
        kind: &str,
        version: u16,
        input: ConnectorInput<'_>,
    ) -> Result<ConnectorOutcome, UnknownConnectorKind> {
        match (kind, version) {
            ("flow.loop", 1) => Ok(self.flow_loop.execute(input).await),
            ("flow.condition", 1) => Ok(self.flow_condition.execute(input).await),
            ("mestier.customer.create", 1) => Ok(self.customer_create.execute(input).await),
            _ => Err(UnknownConnectorKind {
                kind: kind.to_string(),
                version,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use common::OrganizationId;
    use serde_json::json;
    use sqlx::PgPool;
    use uuid::Uuid;

    use super::*;
    use crate::application::default_authorizer;
    use crate::domain::automation::connector::connector_catalogue;
    use crate::infrastructure::realtime::EventHub;

    fn registry() -> ConnectorRegistry {
        let pool = PgPool::connect_lazy("postgres://unused:unused@localhost/unused")
            .expect("a lazy pool needs no server");
        ConnectorRegistry::new(MestierUseCase::new(
            pool,
            default_authorizer(),
            EventHub::new(),
        ))
    }

    /// The test #197 could not yet write: every `kind` the catalogue
    /// describes has an executor, and every executor is described. A
    /// connector added to one and forgotten in the other fails this, rather
    /// than surfacing only once a workflow using it actually runs.
    #[test]
    fn every_catalogue_kind_has_an_implementation_and_vice_versa() {
        let catalogue = connector_catalogue();
        let catalogue_kinds: BTreeSet<(String, u16)> = catalogue
            .descriptors()
            .map(|d| (d.kind.to_string(), d.version))
            .collect();
        let registry_kinds: BTreeSet<(String, u16)> = ConnectorRegistry::known_kinds()
            .iter()
            .map(|(kind, version)| (kind.to_string(), *version))
            .collect();

        assert_eq!(
            catalogue_kinds, registry_kinds,
            "the catalogue and the connector registry have drifted apart"
        );
    }

    #[tokio::test]
    async fn execute_dispatches_flow_loop_to_its_implementation() {
        let mut config = serde_json::Map::new();
        config.insert("items".to_string(), json!([1, 2]));
        let input = ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config: &config,
            credential_id: None,
        };

        let outcome = registry().execute("flow.loop", 1, input).await.unwrap();

        assert_eq!(
            outcome,
            ConnectorOutcome::Iterate {
                items: vec![json!(1), json!(2)]
            }
        );
    }

    #[tokio::test]
    async fn execute_dispatches_flow_condition_to_its_implementation() {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!(true));
        let input = ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config: &config,
            credential_id: None,
        };

        let outcome = registry()
            .execute("flow.condition", 1, input)
            .await
            .unwrap();

        assert!(matches!(outcome, ConnectorOutcome::Branch { .. }));
    }

    #[tokio::test]
    async fn an_unknown_kind_is_refused_and_named() {
        let config = serde_json::Map::new();
        let input = ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config: &config,
            credential_id: None,
        };

        let error = registry()
            .execute("not.a.real.kind", 1, input)
            .await
            .unwrap_err();

        assert_eq!(
            error,
            UnknownConnectorKind {
                kind: "not.a.real.kind".to_string(),
                version: 1,
            }
        );
    }
}
