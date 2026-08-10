//! The connector catalogue: what the product knows about every connector
//! kind a workflow graph can pose, mirroring `events::EventCatalogue` on the
//! trigger side — see [`catalogue::connector_catalogue`] for the assembly
//! point.

mod catalogue;
mod descriptor;
mod field;
mod scheme;

pub use catalogue::{ConnectorCatalogue, ConnectorCatalogueError};
pub use descriptor::{AuthRequirement, ConnectorDescriptor};
pub use field::{Field, FieldKind, SelectOption, VisibleWhen};
pub use scheme::{AuthScheme, auth_scheme, auth_schemes};
