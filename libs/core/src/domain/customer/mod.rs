use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod commands;
pub mod ports;
pub mod profile;
pub mod service;

pub use profile::{
    CustomerError, CustomerField, CustomerKind, CustomerOutcome, CustomerProfile, LeadSource,
    RegistrationNumber, payment_terms_of, siren_of,
};
pub use service::CustomerService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct CustomerId(pub Uuid);

impl FromStr for CustomerId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(CustomerId)
    }
}

impl Display for CustomerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomerStatus {
    Prospect,
    Client,
    Archived,
}

impl CustomerStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Prospect => "PROSPECT",
            Self::Client => "CLIENT",
            Self::Archived => "ARCHIVED",
        }
    }
}

impl Display for CustomerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CustomerStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PROSPECT" => Ok(Self::Prospect),
            "CLIENT" => Ok(Self::Client),
            "ARCHIVED" => Ok(Self::Archived),
            other => Err(format!("invalid customer status `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomerPipelineStage {
    New,
    Contacted,
    Qualified,
    QuoteSent,
    Won,
    Lost,
}

impl CustomerPipelineStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Contacted => "CONTACTED",
            Self::Qualified => "QUALIFIED",
            Self::QuoteSent => "QUOTE_SENT",
            Self::Won => "WON",
            Self::Lost => "LOST",
        }
    }
}

impl Display for CustomerPipelineStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for CustomerPipelineStage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NEW" => Ok(Self::New),
            "CONTACTED" => Ok(Self::Contacted),
            "QUALIFIED" => Ok(Self::Qualified),
            "QUOTE_SENT" => Ok(Self::QuoteSent),
            "WON" => Ok(Self::Won),
            "LOST" => Ok(Self::Lost),
            other => Err(format!("invalid customer pipeline stage `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Customer {
    pub id: CustomerId,
    pub organization_id: OrganizationId,
    pub status: CustomerStatus,
    pub pipeline_stage: CustomerPipelineStage,
    pub name: String,
    /// The customer's own SIREN (or equivalent), never carried before #341:
    /// the September 2026 e-invoicing reform requires at least one mention
    /// about the *customer*, not just the issuer. `None` for every
    /// customer that existed before this field, and for any customer that
    /// simply hasn't stated it — an invoice cannot be transmitted
    /// electronically to one until it does, see
    /// `organization::legal_identity::ElectronicInvoicingFacts`.
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub profile: CustomerProfile,
    pub converted_at: Option<DateTime<Utc>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Customer {
    pub fn change_status(&mut self, requested: CustomerStatus, now: DateTime<Utc>) {
        let restoring =
            self.status == CustomerStatus::Archived && requested == CustomerStatus::Prospect;
        let next = if restoring && self.converted_at.is_some() {
            CustomerStatus::Client
        } else {
            requested
        };

        match next {
            CustomerStatus::Client => {
                self.converted_at.get_or_insert(now);
                self.archived_at = None;
            }
            CustomerStatus::Prospect => self.archived_at = None,
            CustomerStatus::Archived => {
                if self.status != CustomerStatus::Archived {
                    self.archived_at = Some(now);
                }
            }
        }
        self.status = next;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn customer_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = CustomerId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn customer_id_rejects_invalid_uuid() {
        assert!(CustomerId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn customer_status_parses_known_values() {
        assert_eq!(
            "PROSPECT".parse::<CustomerStatus>().unwrap(),
            CustomerStatus::Prospect
        );
        assert_eq!(
            "CLIENT".parse::<CustomerStatus>().unwrap(),
            CustomerStatus::Client
        );
    }

    fn prospect() -> Customer {
        let now = Utc::now();
        Customer {
            id: CustomerId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            status: CustomerStatus::Prospect,
            pipeline_stage: CustomerPipelineStage::New,
            name: "Alice Dupont".to_owned(),
            registration_number: None,
            phone: None,
            email: None,
            profile: CustomerProfile::default(),
            converted_at: None,
            archived_at: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn at(day: u32) -> DateTime<Utc> {
        chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 11, day, 9, 0, 0).unwrap()
    }

    #[test]
    fn converting_a_prospect_sets_converted_at() {
        let mut customer = prospect();

        customer.change_status(CustomerStatus::Client, at(2));

        assert_eq!(customer.status, CustomerStatus::Client);
        assert_eq!(customer.converted_at, Some(at(2)));
    }

    #[test]
    fn converting_again_keeps_the_first_date() {
        let mut customer = prospect();
        customer.change_status(CustomerStatus::Client, at(2));

        customer.change_status(CustomerStatus::Client, at(20));

        assert_eq!(customer.converted_at, Some(at(2)));
    }

    #[test]
    fn archiving_sets_archived_at_and_keeps_the_conversion() {
        let mut customer = prospect();
        customer.change_status(CustomerStatus::Client, at(2));

        customer.change_status(CustomerStatus::Archived, at(5));

        assert_eq!(customer.status, CustomerStatus::Archived);
        assert_eq!(customer.archived_at, Some(at(5)));
        assert_eq!(customer.converted_at, Some(at(2)));
    }

    #[test]
    fn archiving_an_archived_customer_keeps_the_first_archive_date() {
        let mut customer = prospect();
        customer.change_status(CustomerStatus::Archived, at(5));

        customer.change_status(CustomerStatus::Archived, at(9));

        assert_eq!(customer.archived_at, Some(at(5)));
    }

    #[test]
    fn restoring_a_converted_customer_returns_to_client() {
        let mut customer = prospect();
        customer.change_status(CustomerStatus::Client, at(2));
        customer.change_status(CustomerStatus::Archived, at(5));

        customer.change_status(CustomerStatus::Prospect, at(8));

        assert_eq!(customer.status, CustomerStatus::Client);
        assert_eq!(customer.archived_at, None);
        assert_eq!(customer.converted_at, Some(at(2)));
    }

    #[test]
    fn restoring_a_never_converted_customer_returns_to_prospect() {
        let mut customer = prospect();
        customer.change_status(CustomerStatus::Archived, at(5));

        customer.change_status(CustomerStatus::Prospect, at(8));

        assert_eq!(customer.status, CustomerStatus::Prospect);
        assert_eq!(customer.archived_at, None);
        assert_eq!(customer.converted_at, None);
    }

    #[test]
    fn converting_an_archived_prospect_makes_it_a_client() {
        let mut customer = prospect();
        customer.change_status(CustomerStatus::Archived, at(5));

        customer.change_status(CustomerStatus::Client, at(8));

        assert_eq!(customer.status, CustomerStatus::Client);
        assert_eq!(customer.converted_at, Some(at(8)));
        assert_eq!(customer.archived_at, None);
    }

    #[test]
    fn customer_status_rejects_unknown_values() {
        assert!("LEAD".parse::<CustomerStatus>().is_err());
    }

    #[test]
    fn customer_pipeline_stage_parses_known_values() {
        assert_eq!(
            "NEW".parse::<CustomerPipelineStage>().unwrap(),
            CustomerPipelineStage::New
        );
        assert_eq!(
            "QUOTE_SENT".parse::<CustomerPipelineStage>().unwrap(),
            CustomerPipelineStage::QuoteSent
        );
    }

    #[test]
    fn customer_pipeline_stage_rejects_unknown_values() {
        assert!("WAITING".parse::<CustomerPipelineStage>().is_err());
    }
}
