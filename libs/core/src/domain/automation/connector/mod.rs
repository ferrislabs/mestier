//! The connector catalogue: what the product knows about every connector
//! kind a workflow graph can pose, mirroring `events::EventCatalogue` on the
//! trigger side — see [`catalogue::connector_catalogue`] for the assembly
//! point.

mod catalogue;
mod descriptor;
mod field;
pub mod smtp;

pub use catalogue::{ConnectorCatalogue, ConnectorCatalogueError, connector_catalogue};
pub use descriptor::{AuthRequirement, ConnectorDescriptor};
pub use field::{Field, FieldKind, SelectOption, VisibleWhen};
