use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::error::DealError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DealStageName {
    New,
    Visit,
    QuoteSent,
    Negotiation,
    Won,
    Lost,
}

impl DealStageName {
    pub const ALL: [DealStageName; 6] = [
        Self::New,
        Self::Visit,
        Self::QuoteSent,
        Self::Negotiation,
        Self::Won,
        Self::Lost,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::New => "NEW",
            Self::Visit => "VISIT",
            Self::QuoteSent => "QUOTE_SENT",
            Self::Negotiation => "NEGOTIATION",
            Self::Won => "WON",
            Self::Lost => "LOST",
        }
    }

    pub fn is_open(self) -> bool {
        !matches!(self, Self::Won | Self::Lost)
    }
}

impl fmt::Display for DealStageName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for DealStageName {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NEW" => Ok(Self::New),
            "VISIT" => Ok(Self::Visit),
            "QUOTE_SENT" => Ok(Self::QuoteSent),
            "NEGOTIATION" => Ok(Self::Negotiation),
            "WON" => Ok(Self::Won),
            "LOST" => Ok(Self::Lost),
            other => Err(format!("invalid deal stage `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LostReason(String);

pub const ARCHIVED_ACCOUNT_REASON: &str = "Compte archivé";

impl LostReason {
    pub fn archived_account() -> Self {
        Self(ARCHIVED_ACCOUNT_REASON.to_owned())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

impl TryFrom<&str> for LostReason {
    type Error = DealError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(DealError::LostReasonRequired)
        } else {
            Ok(Self(trimmed.to_owned()))
        }
    }
}

impl TryFrom<String> for LostReason {
    type Error = DealError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DealStage {
    New,
    Visit,
    QuoteSent,
    Negotiation,
    Won,
    Lost(LostReason),
}

impl DealStage {
    pub fn name(&self) -> DealStageName {
        match self {
            Self::New => DealStageName::New,
            Self::Visit => DealStageName::Visit,
            Self::QuoteSent => DealStageName::QuoteSent,
            Self::Negotiation => DealStageName::Negotiation,
            Self::Won => DealStageName::Won,
            Self::Lost(_) => DealStageName::Lost,
        }
    }

    pub fn is_open(&self) -> bool {
        self.name().is_open()
    }

    pub fn lost_reason(&self) -> Option<&LostReason> {
        match self {
            Self::Lost(reason) => Some(reason),
            _ => None,
        }
    }
}

impl TryFrom<(DealStageName, Option<&str>)> for DealStage {
    type Error = DealError;

    fn try_from((name, reason): (DealStageName, Option<&str>)) -> Result<Self, Self::Error> {
        Ok(match name {
            DealStageName::New => Self::New,
            DealStageName::Visit => Self::Visit,
            DealStageName::QuoteSent => Self::QuoteSent,
            DealStageName::Negotiation => Self::Negotiation,
            DealStageName::Won => Self::Won,
            DealStageName::Lost => Self::Lost(LostReason::try_from(reason.unwrap_or_default())?),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lost_needs_a_non_empty_reason() {
        assert_eq!(
            DealStage::try_from((DealStageName::Lost, None)),
            Err(DealError::LostReasonRequired)
        );
        assert_eq!(
            DealStage::try_from((DealStageName::Lost, Some("   "))),
            Err(DealError::LostReasonRequired)
        );
    }

    #[test]
    fn lost_trims_its_reason() {
        let stage = DealStage::try_from((DealStageName::Lost, Some("  Prix trop élevé "))).unwrap();

        assert_eq!(stage.lost_reason().unwrap().as_str(), "Prix trop élevé");
    }

    #[test]
    fn an_open_or_won_stage_ignores_the_reason() {
        let stage = DealStage::try_from((DealStageName::Won, Some("x"))).unwrap();

        assert_eq!(stage, DealStage::Won);
        assert_eq!(stage.lost_reason(), None);
    }

    #[test]
    fn four_stages_are_open() {
        let open: Vec<_> = DealStageName::ALL
            .into_iter()
            .filter(|stage| stage.is_open())
            .collect();

        assert_eq!(
            open,
            [
                DealStageName::New,
                DealStageName::Visit,
                DealStageName::QuoteSent,
                DealStageName::Negotiation
            ]
        );
    }

    #[test]
    fn stage_names_round_trip() {
        for name in DealStageName::ALL {
            assert_eq!(name.as_str().parse::<DealStageName>().unwrap(), name);
        }
        assert!("CONTACTED".parse::<DealStageName>().is_err());
    }
}
