//! Executable connectors: one module per `kind`, dispatched through
//! [`ConnectorRegistry`]. Every kind here is proved against the connector
//! contract in `domain::automation::run` (`Connector`, `ConnectorInput`,
//! `ConnectorOutcome`) and checked against
//! `domain::automation::connector::catalogue` by an anti-drift test in
//! `registry`.

mod chat_post_message;
mod collection_operation;
mod customer_create;
mod digest_auth;
mod fields;
mod flow_condition;
mod flow_config;
mod flow_loop;
mod flow_merge;
mod flow_wait;
pub(crate) mod http_auth;
pub(crate) mod http_client;
mod http_request;
mod jwt_auth;
mod note_add;
pub(crate) mod oauth1;
mod odoo;
mod project_create;
mod quote_search;
mod registry;
mod smtp_send;
mod support_add_badge;
mod support_assign_ticket;
mod task_recurrence_extend_horizon;
#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
pub(crate) mod world_support;

pub use registry::{ConnectorDryRun, ConnectorRegistry, UnknownConnectorKind};
