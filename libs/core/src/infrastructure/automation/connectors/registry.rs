use thiserror::Error;

use crate::application::MestierUseCase;
use crate::domain::automation::connector::ConnectorDescriptor;
use crate::domain::automation::run::{
    Connector, ConnectorInput, ConnectorOutcome, DryRunDispatch, DryRunInput,
};
use crate::infrastructure::automation::webhook::address_policy::PrivateNetworkAccess;

use super::actions;
use super::chat_post_message::ChatPostMessageConnector;
use super::collection_operation::CollectionOperationConnector;
use super::customer_create::CustomerCreateConnector;
use super::flow_condition::FlowConditionConnector;
use super::flow_config::FlowConfigConnector;
use super::flow_loop::FlowLoopConnector;
use super::flow_merge::FlowMergeConnector;
use super::flow_wait::FlowWaitConnector;
use super::http_request::HttpRequestConnector;
use super::note_add::NoteAddConnector;
use super::odoo::{
    OdooCreateInvoiceConnector, OdooCreatePartnerConnector, OdooUpdatePartnerConnector,
};
use super::project_create::ProjectCreateConnector;
use super::quote_search::QuoteSearchConnector;
use super::smtp_send::SmtpSendConnector;
use super::support_add_badge::SupportAddBadgeConnector;
use super::support_assign_ticket::SupportAssignTicketConnector;
use super::task_recurrence_extend_horizon::TaskRecurrenceExtendHorizonConnector;

#[derive(Debug, Error, PartialEq, Eq)]
#[error("no connector implementation for `{kind}` version {version}")]
pub struct UnknownConnectorKind {
    pub kind: String,
    pub version: u16,
}

/// Every executable connector kind, dispatched by `(kind, version)`. A
/// concrete struct rather than a `HashMap<_, Box<dyn Connector>>`:
/// `Connector::execute` returning `impl Future` (not object-safe) would
/// otherwise force boxing every call.
///
/// [`Self::known_kinds`] is the anti-drift test's other half — see
/// `domain::automation::connector::catalogue` for the descriptors this is
/// checked against: every catalogue entry must have an implementation here,
/// and every implementation here must be in the catalogue, or the two silently
/// drift apart the way #197 could not yet guard against.
pub struct ConnectorRegistry {
    usecase: MestierUseCase,
    flow_loop: FlowLoopConnector,
    flow_condition: FlowConditionConnector,
    flow_config: FlowConfigConnector,
    flow_wait: FlowWaitConnector,
    flow_merge: FlowMergeConnector,
    customer_create: CustomerCreateConnector,
    support_assign_ticket: SupportAssignTicketConnector,
    support_add_badge: SupportAddBadgeConnector,
    note_add: NoteAddConnector,
    quote_search: QuoteSearchConnector,
    project_create: ProjectCreateConnector,
    chat_post_message: ChatPostMessageConnector,
    http_request: HttpRequestConnector,
    collection_operation: CollectionOperationConnector,
    odoo_create_partner: OdooCreatePartnerConnector,
    odoo_update_partner: OdooUpdatePartnerConnector,
    odoo_create_invoice: OdooCreateInvoiceConnector,
    smtp_send: SmtpSendConnector,
    task_recurrence_extend_horizon: TaskRecurrenceExtendHorizonConnector,
}

impl ConnectorRegistry {
    /// Network connectors default to [`PrivateNetworkAccess::Denied`] — the
    /// same fail-closed default `PrivateNetworkAccess` itself declares.
    /// Callers who need the instance's actual `AutomationConfig`-sourced
    /// policy build through [`Self::with_private_network_access`] instead.
    pub fn new(usecase: MestierUseCase) -> Self {
        Self::with_private_network_access(usecase, PrivateNetworkAccess::default())
    }

    /// Builds the registry with an explicit private-network policy — see
    /// `AutomationConfig::allow_private_network`. Only `http.request` and
    /// the `odoo.*` connectors read it: every other kind here makes no
    /// outbound call.
    pub fn with_private_network_access(
        usecase: MestierUseCase,
        access: PrivateNetworkAccess,
    ) -> Self {
        Self {
            usecase: usecase.clone(),
            flow_loop: FlowLoopConnector,
            flow_condition: FlowConditionConnector,
            flow_config: FlowConfigConnector,
            flow_wait: FlowWaitConnector,
            flow_merge: FlowMergeConnector,
            customer_create: CustomerCreateConnector::new(usecase.clone()),
            support_assign_ticket: SupportAssignTicketConnector::new(usecase.clone()),
            support_add_badge: SupportAddBadgeConnector::new(usecase.clone()),
            note_add: NoteAddConnector::new(usecase.clone()),
            quote_search: QuoteSearchConnector::new(usecase.clone()),
            project_create: ProjectCreateConnector::new(usecase.clone()),
            chat_post_message: ChatPostMessageConnector::new(usecase.clone()),
            http_request: HttpRequestConnector::new(usecase.clone(), access),
            collection_operation: CollectionOperationConnector::new(usecase.clone(), access),
            odoo_create_partner: OdooCreatePartnerConnector::new(usecase.clone(), access),
            odoo_update_partner: OdooUpdatePartnerConnector::new(usecase.clone(), access),
            odoo_create_invoice: OdooCreateInvoiceConnector::new(usecase.clone(), access),
            smtp_send: SmtpSendConnector::new(usecase.clone(), access),
            task_recurrence_extend_horizon: TaskRecurrenceExtendHorizonConnector::new(usecase),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_collection_client(
        mut self,
        usecase: MestierUseCase,
        client: reqwest::Client,
    ) -> Self {
        self.collection_operation = CollectionOperationConnector::with_client(usecase, client);
        self
    }

    /// Every `(kind, version)` this registry can dispatch to.
    pub fn known_kinds() -> &'static [(&'static str, u16)] {
        &[
            ("flow.loop", 1),
            ("flow.condition", 1),
            ("flow.config", 1),
            ("flow.wait", 1),
            ("flow.merge", 1),
            ("mestier.customer.create", 1),
            ("mestier.support.assign_ticket", 1),
            ("mestier.support.add_badge", 1),
            ("mestier.note.add", 1),
            ("mestier.quote.search", 1),
            ("mestier.project.create", 1),
            ("mestier.chat.post_message", 1),
            ("http.request", 1),
            ("collection.operation", 1),
            ("odoo.create_partner", 1),
            ("odoo.update_partner", 1),
            ("odoo.create_invoice", 1),
            ("smtp.send", 1),
            ("mestier.task_recurrence.extend_horizon", 1),
        ]
    }

    pub fn dry_run(
        descriptor: &ConnectorDescriptor,
        input: &DryRunInput<'_>,
    ) -> Option<ConnectorOutcome> {
        Some(match (descriptor.kind, descriptor.version) {
            ("flow.loop", 1) => FlowLoopConnector::dry_run(descriptor, input),
            ("flow.condition", 1) => FlowConditionConnector::dry_run(descriptor, input),
            ("flow.config", 1) => FlowConfigConnector::dry_run(descriptor, input),
            ("flow.wait", 1) => FlowWaitConnector::dry_run(descriptor, input),
            ("flow.merge", 1) => FlowMergeConnector::dry_run(descriptor, input),
            ("mestier.customer.create", 1) => CustomerCreateConnector::dry_run(descriptor, input),
            ("mestier.support.assign_ticket", 1) => {
                SupportAssignTicketConnector::dry_run(descriptor, input)
            }
            ("mestier.support.add_badge", 1) => {
                SupportAddBadgeConnector::dry_run(descriptor, input)
            }
            ("mestier.note.add", 1) => NoteAddConnector::dry_run(descriptor, input),
            ("mestier.quote.search", 1) => QuoteSearchConnector::dry_run(descriptor, input),
            ("mestier.project.create", 1) => ProjectCreateConnector::dry_run(descriptor, input),
            ("mestier.chat.post_message", 1) => {
                ChatPostMessageConnector::dry_run(descriptor, input)
            }
            ("http.request", 1) => HttpRequestConnector::dry_run(descriptor, input),
            ("collection.operation", 1) => CollectionOperationConnector::dry_run(descriptor, input),
            ("odoo.create_partner", 1) => OdooCreatePartnerConnector::dry_run(descriptor, input),
            ("odoo.update_partner", 1) => OdooUpdatePartnerConnector::dry_run(descriptor, input),
            ("odoo.create_invoice", 1) => OdooCreateInvoiceConnector::dry_run(descriptor, input),
            ("smtp.send", 1) => SmtpSendConnector::dry_run(descriptor, input),
            ("mestier.task_recurrence.extend_horizon", 1) => {
                TaskRecurrenceExtendHorizonConnector::dry_run(descriptor, input)
            }
            _ => return actions::dry_run(descriptor),
        })
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
            ("flow.config", 1) => Ok(self.flow_config.execute(input).await),
            ("flow.wait", 1) => Ok(self.flow_wait.execute(input).await),
            ("flow.merge", 1) => Ok(self.flow_merge.execute(input).await),
            ("mestier.customer.create", 1) => Ok(self.customer_create.execute(input).await),
            ("mestier.support.assign_ticket", 1) => {
                Ok(self.support_assign_ticket.execute(input).await)
            }
            ("mestier.support.add_badge", 1) => Ok(self.support_add_badge.execute(input).await),
            ("mestier.note.add", 1) => Ok(self.note_add.execute(input).await),
            ("mestier.quote.search", 1) => Ok(self.quote_search.execute(input).await),
            ("mestier.project.create", 1) => Ok(self.project_create.execute(input).await),
            ("mestier.chat.post_message", 1) => Ok(self.chat_post_message.execute(input).await),
            ("http.request", 1) => Ok(self.http_request.execute(input).await),
            ("collection.operation", 1) => Ok(self.collection_operation.execute(input).await),
            ("odoo.create_partner", 1) => Ok(self.odoo_create_partner.execute(input).await),
            ("odoo.update_partner", 1) => Ok(self.odoo_update_partner.execute(input).await),
            ("odoo.create_invoice", 1) => Ok(self.odoo_create_invoice.execute(input).await),
            ("smtp.send", 1) => Ok(self.smtp_send.execute(input).await),
            ("mestier.task_recurrence.extend_horizon", 1) => {
                Ok(self.task_recurrence_extend_horizon.execute(input).await)
            }
            _ => match version {
                1 => actions::execute(&self.usecase, kind, input).await,
                _ => None,
            }
            .ok_or_else(|| UnknownConnectorKind {
                kind: kind.to_string(),
                version,
            }),
        }
    }
}

pub struct ConnectorDryRun;

impl DryRunDispatch for ConnectorDryRun {
    fn dry_run(
        &self,
        descriptor: &ConnectorDescriptor,
        input: &DryRunInput<'_>,
    ) -> Option<ConnectorOutcome> {
        ConnectorRegistry::dry_run(descriptor, input)
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
    use crate::domain::automation::connector::{action_specs, connector_catalogue};
    use crate::domain::automation::run::DryRunDispatch;
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
            .chain(action_specs().map(|spec| (spec.kind.to_string(), 1)))
            .collect();

        assert_eq!(
            catalogue_kinds, registry_kinds,
            "the catalogue and the connector registry have drifted apart"
        );
    }

    fn dry_run_input(config: &serde_json::Map<String, serde_json::Value>) -> DryRunInput<'_> {
        DryRunInput {
            config,
            stored: None,
        }
    }

    #[test]
    fn every_catalogue_kind_has_a_dry_run() {
        let catalogue = connector_catalogue();
        let config = serde_json::Map::new();

        for descriptor in catalogue.descriptors() {
            assert!(
                ConnectorRegistry::dry_run(descriptor, &dry_run_input(&config)).is_some(),
                "`{}` has no dry run entry",
                descriptor.kind
            );
        }
    }

    #[test]
    fn a_connector_without_its_own_dry_run_answers_with_its_descriptors_sample() {
        let catalogue = connector_catalogue();
        let config = serde_json::Map::new();

        for kind in [
            "flow.wait",
            "flow.merge",
            "smtp.send",
            "mestier.note.add",
            "mestier.quote.search",
            "mestier.project.create",
            "mestier.chat.post_message",
            "mestier.customer.create",
            "mestier.support.assign_ticket",
            "mestier.support.add_badge",
            "http.request",
            "odoo.create_partner",
            "odoo.update_partner",
            "odoo.create_invoice",
            "mestier.task_recurrence.extend_horizon",
        ] {
            let descriptor = catalogue.get(kind, 1).unwrap();

            let outcome = ConnectorRegistry::dry_run(descriptor, &dry_run_input(&config));

            assert_eq!(
                outcome,
                Some(ConnectorOutcome::Produced(descriptor.sample_output.clone())),
                "{kind}"
            );
        }
    }

    #[test]
    fn a_kind_the_registry_does_not_know_has_no_dry_run() {
        let mut descriptor = connector_catalogue().get("flow.config", 1).unwrap().clone();
        descriptor.kind = "not.a.real.kind";
        let config = serde_json::Map::new();

        assert_eq!(
            ConnectorRegistry::dry_run(&descriptor, &dry_run_input(&config)),
            None
        );
        assert_eq!(
            ConnectorDryRun.dry_run(&descriptor, &dry_run_input(&config)),
            None
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
            operation: None,
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
            operation: None,
        };

        let outcome = registry()
            .execute("flow.condition", 1, input)
            .await
            .unwrap();

        assert!(matches!(outcome, ConnectorOutcome::Branch { .. }));
    }

    #[tokio::test]
    async fn execute_dispatches_flow_config_to_its_implementation() {
        let mut config = serde_json::Map::new();
        config.insert("variables".to_string(), json!({ "customer_id": "c-1" }));
        let input = ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config: &config,
            credential_id: None,
            operation: None,
        };

        let outcome = registry().execute("flow.config", 1, input).await.unwrap();

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(json!({ "customer_id": "c-1" }))
        );
    }

    #[tokio::test]
    async fn an_unknown_kind_is_refused_and_named() {
        let config = serde_json::Map::new();
        let input = ConnectorInput {
            org_id: OrganizationId(Uuid::from_u128(1)),
            run_id: Uuid::from_u128(2),
            config: &config,
            credential_id: None,
            operation: None,
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
