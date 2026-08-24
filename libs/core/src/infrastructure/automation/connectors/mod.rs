//! Executable connectors: one module per `kind`, dispatched through
//! [`ConnectorRegistry`]. Every kind here is proved against the connector
//! contract in `domain::automation::run` (`Connector`, `ConnectorInput`,
//! `ConnectorOutcome`) and checked against
//! `domain::automation::connector::catalogue` by an anti-drift test in
//! `registry`.

mod customer_create;
mod flow_condition;
mod flow_loop;
mod http_client;
mod http_request;
mod odoo;
mod registry;
mod task_recurrence_extend_horizon;
#[cfg(test)]
mod test_support;

pub use registry::{ConnectorRegistry, UnknownConnectorKind};
