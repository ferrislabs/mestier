//! Executable connectors: one module per `kind`, dispatched through
//! [`ConnectorRegistry`]. Every kind here is proved against the connector
//! contract in `domain::automation::run` (`Connector`, `ConnectorInput`,
//! `ConnectorOutcome`) and checked against
//! `domain::automation::connector::catalogue` by an anti-drift test in
//! `registry`.

mod collection_operation;
mod customer_create;
mod digest_auth;
mod flow_condition;
mod flow_config;
mod flow_loop;
pub(crate) mod http_auth;
pub(crate) mod http_client;
mod http_request;
mod jwt_auth;
mod md5;
pub(crate) mod oauth1;
mod odoo;
mod registry;
mod support_add_badge;
mod support_assign_ticket;
mod task_recurrence_extend_horizon;
#[cfg(test)]
pub(crate) mod test_support;

pub use registry::{ConnectorDryRun, ConnectorRegistry, UnknownConnectorKind};
