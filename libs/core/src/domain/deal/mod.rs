use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{CustomerId, LeadSource, MemberId, OrganizationId};

pub mod commands;
pub mod error;
pub mod ports;
pub mod quote_link;
pub mod service;
pub mod stage;
pub mod summary;

pub use error::{DealError, DealField, DealOutcome};
pub use quote_link::DealQuoteLinks;
pub use service::DealService;
pub use stage::{ARCHIVED_ACCOUNT_REASON, DealStage, DealStageName, LostReason};
pub use summary::{PipelineSummary, SUMMARY_WINDOW_DAYS, StageTotal};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct DealId(pub Uuid);

impl FromStr for DealId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(DealId)
    }
}

impl Display for DealId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DealTitle(String);

impl DealTitle {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl TryFrom<&str> for DealTitle {
    type Error = DealError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(DealError::TitleRequired)
        } else {
            Ok(Self(trimmed.to_owned()))
        }
    }
}

impl TryFrom<String> for DealTitle {
    type Error = DealError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DealAmount(i64);

impl DealAmount {
    pub const ZERO: Self = Self(0);

    pub fn get(self) -> i64 {
        self.0
    }
}

impl TryFrom<i64> for DealAmount {
    type Error = DealError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        if value < 0 {
            Err(DealError::AmountNegative)
        } else {
            Ok(Self(value))
        }
    }
}

pub fn normalized_notes(notes: Option<String>) -> Option<String> {
    notes
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

#[derive(Debug, Clone, PartialEq)]
pub struct Deal {
    pub id: DealId,
    pub organization_id: OrganizationId,
    pub customer_id: CustomerId,
    pub title: DealTitle,
    pub amount: DealAmount,
    pub stage: DealStage,
    pub expected_close_date: Option<NaiveDate>,
    pub owner_member_id: Option<MemberId>,
    pub source: Option<LeadSource>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub stage_changed_at: DateTime<Utc>,
    pub closed_at: Option<DateTime<Utc>>,
    pub quote_count: i64,
}

impl Deal {
    pub fn move_to(&mut self, stage: DealStage, now: DateTime<Utc>) -> bool {
        if self.stage.name() == stage.name() {
            return false;
        }
        self.closed_at = if stage.is_open() { None } else { Some(now) };
        self.stage = stage;
        self.stage_changed_at = now;
        true
    }

    pub fn is_linked_to_a_quote(&self) -> bool {
        self.quote_count > 0
    }
}

#[cfg(test)]
pub(crate) mod fixtures {
    use super::*;

    pub fn at(day: u32) -> DateTime<Utc> {
        chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 11, day, 9, 0, 0).unwrap()
    }

    pub fn deal(stage: DealStage) -> Deal {
        Deal {
            id: DealId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            customer_id: CustomerId(Uuid::new_v4()),
            title: DealTitle::try_from("Rénovation salle de bain").unwrap(),
            amount: DealAmount::try_from(1_250_000).unwrap(),
            closed_at: if stage.is_open() { None } else { Some(at(1)) },
            stage,
            expected_close_date: None,
            owner_member_id: None,
            source: None,
            notes: None,
            created_at: at(1),
            stage_changed_at: at(1),
            quote_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::fixtures::{at, deal};
    use super::*;

    #[test]
    fn a_title_is_trimmed_and_never_empty() {
        assert_eq!(DealTitle::try_from("  Pose  ").unwrap().as_str(), "Pose");
        assert_eq!(DealTitle::try_from("   "), Err(DealError::TitleRequired));
    }

    #[test]
    fn an_amount_is_never_negative() {
        assert_eq!(DealAmount::try_from(0).unwrap(), DealAmount::ZERO);
        assert_eq!(DealAmount::try_from(-1), Err(DealError::AmountNegative));
    }

    #[test]
    fn winning_closes_the_deal_today() {
        let mut deal = deal(DealStage::Negotiation);

        let moved = deal.move_to(DealStage::Won, at(5));

        assert!(moved);
        assert_eq!(deal.stage, DealStage::Won);
        assert_eq!(deal.closed_at, Some(at(5)));
        assert_eq!(deal.stage_changed_at, at(5));
    }

    #[test]
    fn losing_stores_the_reason_and_closes_the_deal() {
        let mut deal = deal(DealStage::Visit);
        let lost = DealStage::try_from((DealStageName::Lost, Some("Prix trop élevé"))).unwrap();

        deal.move_to(lost, at(6));

        assert_eq!(deal.closed_at, Some(at(6)));
        assert_eq!(
            deal.stage.lost_reason().unwrap().as_str(),
            "Prix trop élevé"
        );
    }

    #[test]
    fn moving_to_the_same_stage_changes_nothing() {
        let mut deal = deal(DealStage::Won);

        let moved = deal.move_to(DealStage::Won, at(9));

        assert!(!moved);
        assert_eq!(deal.closed_at, Some(at(1)));
        assert_eq!(deal.stage_changed_at, at(1));
    }

    #[test]
    fn reopening_into_an_open_stage_clears_the_close() {
        let lost = DealStage::try_from((DealStageName::Lost, Some("Pas de réponse"))).unwrap();
        let mut deal = deal(lost);

        deal.move_to(DealStage::Negotiation, at(7));

        assert_eq!(deal.stage, DealStage::Negotiation);
        assert_eq!(deal.stage.lost_reason(), None);
        assert_eq!(deal.closed_at, None);
        assert_eq!(deal.stage_changed_at, at(7));
    }

    #[test]
    fn moving_between_open_stages_keeps_the_deal_open() {
        let mut deal = deal(DealStage::New);

        deal.move_to(DealStage::QuoteSent, at(3));

        assert_eq!(deal.closed_at, None);
        assert_eq!(deal.stage_changed_at, at(3));
    }

    #[test]
    fn notes_that_are_blank_are_dropped() {
        assert_eq!(normalized_notes(Some("  ".to_owned())), None);
        assert_eq!(
            normalized_notes(Some(" a ".to_owned())),
            Some("a".to_owned())
        );
    }
}
