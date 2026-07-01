use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
	CustomerContextId, CustomerId, LegalMentionTemplateId, OrganizationId, ServiceRateId,
	ServiceRateUnit,
};

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
    pub vat_rate: Decimal,
    pub notes: Option<String>,
    pub photo_keys: Vec<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Quote {
    pub id: QuoteId,
    pub organization_id: OrganizationId,
    pub reference: String,
    pub title: String,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub status: QuoteStatus,
    pub deposit_basis: Option<String>,
    pub deposit_value: Option<Decimal>,
    pub total_cents: i32,
    pub total_ht_cents: i32,
    pub total_vat_cents: i32,
    pub total_ttc_cents: i32,
    pub lines: Vec<QuoteLine>,
    pub legal_mention_template_ids: Vec<LegalMentionTemplateId>,
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
