//! The connector catalogue: what the product knows about every connector
//! kind a workflow graph can pose, mirroring `events::EventCatalogue` on the
//! trigger side — see [`catalogue::connector_catalogue`] for the assembly
//! point.

mod action;
mod actions;
mod catalogue;
mod descriptor;
mod field;
pub mod smtp;

pub use action::{ACTION_FAMILY, ActionSpec, action_descriptors, action_specs, find_action};
pub use catalogue::{ConnectorCatalogue, ConnectorCatalogueError, connector_catalogue};
pub use descriptor::{AuthRequirement, ConnectorDescriptor, ExecutionMode};
pub use field::{Field, FieldKind, SelectOption, VisibleWhen};
