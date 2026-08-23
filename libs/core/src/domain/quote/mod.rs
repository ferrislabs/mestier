pub mod events;
use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{CustomerContextId, CustomerId, OrganizationId, ServiceRateId, ServiceRateUnit};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct QuoteId(pub Uuid);

impl FromStr for QuoteId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(QuoteId)
    }
}

impl Display for QuoteId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuoteStatus {
    Draft,
    Sent,
    Accepted,
    Declined,
    Cancelled,
}

impl QuoteStatus {
    /// Every variant, for exhaustive iteration. Adding one here is not
    /// enforced by the compiler, but naming its event is: `event_name` matches
    /// exhaustively, so a new status cannot be ignored silently.
    pub const ALL: [QuoteStatus; 5] = [
        QuoteStatus::Draft,
        QuoteStatus::Sent,
        QuoteStatus::Accepted,
        QuoteStatus::Declined,
        QuoteStatus::Cancelled,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Sent => "SENT",
            Self::Accepted => "ACCEPTED",
            Self::Declined => "DECLINED",
            Self::Cancelled => "CANCELLED",
        }
    }
}

impl Display for QuoteStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for QuoteStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DRAFT" => Ok(Self::Draft),
            "SENT" => Ok(Self::Sent),
            "ACCEPTED" => Ok(Self::Accepted),
            "DECLINED" => Ok(Self::Declined),
            "CANCELLED" => Ok(Self::Cancelled),
            other => Err(format!("invalid quote status `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct QuoteLineId(pub Uuid);

impl FromStr for QuoteLineId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(QuoteLineId)
    }
}

impl Display for QuoteLineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuoteLine {
    pub id: QuoteLineId,
    pub organization_id: OrganizationId,
    pub quote_id: QuoteId,
    pub service_rate_id: Option<ServiceRateId>,
    pub label: String,
    pub quantity: Decimal,
    pub unit: ServiceRateUnit,
    pub unit_price_cents: i32,
    /// Basis points (2000 = 20 %, 550 = 5.5 %). Travels with the line —
    /// never looked up from a referential at render time — so a document
    /// already sent cannot change because a rate was edited afterwards.
    /// `None` on an organization not subject to VAT, or on a line created
    /// before this field existed.
    pub vat_rate_bp: Option<i32>,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// One rate's contribution to a quote's VAT, grouped and summed across every
/// line at that rate. A quote can carry more than one — renovation work at
/// the reduced rate, supply at the standard one, on the same job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuoteVatBreakdownLine {
    pub rate_bp: i32,
    pub vat_cents: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    pub id: QuoteId,
    pub organization_id: OrganizationId,
    /// Allocated when the quote first leaves `Draft`, gapless and unique
    /// per organization, and never reallocated afterwards — not on edit,
    /// not on a further status change, not on a soft delete. `None` on a
    /// draft: a deleted draft must not leave a hole in the sequence.
    pub reference: Option<String>,
    pub title: String,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub status: QuoteStatus,
    /// Sum of the lines, before VAT.
    pub net_cents: i32,
    /// Per rate, not a single figure: a document has to show the
    /// breakdown. Empty — never a list of zeros — when the organization is
    /// not subject to VAT (see `QuoteVatBreakdownLine`).
    pub vat_breakdown: Vec<QuoteVatBreakdownLine>,
    /// `net_cents` plus the VAT breakdown's total. Equal to `net_cents`
    /// when the organization is not subject to VAT.
    pub gross_cents: i32,
    pub lines: Vec<QuoteLine>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_status_parses_known_values() {
        assert_eq!("DRAFT".parse::<QuoteStatus>().unwrap(), QuoteStatus::Draft);
        assert_eq!("SENT".parse::<QuoteStatus>().unwrap(), QuoteStatus::Sent);
        assert_eq!(
            "ACCEPTED".parse::<QuoteStatus>().unwrap(),
            QuoteStatus::Accepted
        );
    }

    #[test]
    fn quote_status_rejects_unknown_values() {
        assert!("APPROVED".parse::<QuoteStatus>().is_err());
    }

    #[test]
    fn quote_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = QuoteId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }
}
