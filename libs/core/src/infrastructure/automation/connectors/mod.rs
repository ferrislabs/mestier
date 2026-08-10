//! Executable connectors: one module per `kind`, dispatched through
//! [`ConnectorRegistry`]. Every kind here is proved against the connector
//! contract in `domain::automation::run` (`Connector`, `ConnectorInput`,
//! `ConnectorOutcome`) and checked against
//! `domain::automation::connector::catalogue` by an anti-drift test in
//! `registry`.

mod customer_create;
mod flow_condition;
mod flow_loop;
mod registry;

pub use registry::{ConnectorRegistry, UnknownConnectorKind};
