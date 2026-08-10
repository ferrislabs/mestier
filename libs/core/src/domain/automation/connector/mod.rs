//! The connector catalogue: what the product knows about every connector
//! kind a workflow graph can pose, mirroring `events::EventCatalogue` on the
//! trigger side — see [`catalogue::connector_catalogue`] for the assembly
//! point.

mod field;
mod scheme;

pub use field::{Field, FieldKind, SelectOption, VisibleWhen};
pub use scheme::{AuthScheme, auth_scheme, auth_schemes};
