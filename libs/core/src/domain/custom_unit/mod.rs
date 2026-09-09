use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct CustomUnitId(pub Uuid);

impl FromStr for CustomUnitId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(CustomUnitId)
    }
}

impl Display for CustomUnitId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// An organization's own addition to the ten built-in [`crate::ServiceRateUnit`]
/// codes (#449) — a sack, a pallet, a linear metre of a specific profile,
/// whatever a trade needs that the built-in set does not name. `code` is
/// what actually gets stored in `products.unit`/`service_rates.unit`/
/// `quote_lines.unit`, exactly like a built-in unit's wire form
/// (`ServiceRateUnit::as_str`) — those three columns stayed plain `TEXT`
/// from the day they were added, so no migration was needed to accept a
/// custom code, only the Rust-side type relaxing from the closed enum to a
/// validated string.
///
/// Deliberately a single field, not a code/label pair like a built-in unit:
/// `ServiceRateUnit` needs both because its wire form (`HOUR`) and its
/// French reading (`heure`) genuinely differ, but a custom unit's code *is*
/// whatever the artisan typed — there is no separate technical form to
/// reconcile it with.
#[derive(Debug, Clone, PartialEq)]
pub struct CustomUnit {
    pub id: CustomUnitId,
    pub organization_id: OrganizationId,
    pub code: String,
    pub created_at: DateTime<Utc>,
}
