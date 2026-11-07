use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{CustomerContactId, CustomerContextId, MemberId, PaymentTermsDays};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomerField {
    RegistrationNumber,
    PaymentTermsDays,
    OwnerMemberId,
    BillingContactId,
    BillingAddressId,
}

impl CustomerField {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RegistrationNumber => "registration_number",
            Self::PaymentTermsDays => "payment_terms_days",
            Self::OwnerMemberId => "owner_member_id",
            Self::BillingContactId => "billing_contact_id",
            Self::BillingAddressId => "billing_address_id",
        }
    }
}

impl fmt::Display for CustomerField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{field}: {rule}")]
pub struct CustomerError {
    pub field: CustomerField,
    pub rule: &'static str,
}

pub type CustomerOutcome<T> = Result<T, CustomerError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CustomerKind {
    Individual,
    #[default]
    Professional,
}

impl CustomerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Individual => "INDIVIDUAL",
            Self::Professional => "PROFESSIONAL",
        }
    }
}

impl FromStr for CustomerKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INDIVIDUAL" => Ok(Self::Individual),
            "PROFESSIONAL" => Ok(Self::Professional),
            other => Err(format!("invalid customer kind `{other}`")),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum LeadSource {
    #[serde(rename = "bouche_a_oreille")]
    WordOfMouth,
    #[serde(rename = "site_web")]
    Website,
    #[serde(rename = "recommandation")]
    Referral,
    #[serde(rename = "salon")]
    TradeShow,
    #[serde(rename = "architecte")]
    Architect,
    #[serde(rename = "autre")]
    Other,
}

impl LeadSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::WordOfMouth => "bouche_a_oreille",
            Self::Website => "site_web",
            Self::Referral => "recommandation",
            Self::TradeShow => "salon",
            Self::Architect => "architecte",
            Self::Other => "autre",
        }
    }
}

impl FromStr for LeadSource {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "bouche_a_oreille" => Ok(Self::WordOfMouth),
            "site_web" => Ok(Self::Website),
            "recommandation" => Ok(Self::Referral),
            "salon" => Ok(Self::TradeShow),
            "architecte" => Ok(Self::Architect),
            "autre" => Ok(Self::Other),
            other => Err(format!("invalid lead source `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationNumber(String);

impl RegistrationNumber {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn siren(&self) -> &str {
        &self.0[..SIREN_LEN]
    }

    pub fn into_inner(self) -> String {
        self.0
    }
}

const SIREN_LEN: usize = 9;
const SIRET_LEN: usize = 14;

impl TryFrom<&str> for RegistrationNumber {
    type Error = CustomerError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let digits: String = value.chars().filter(|c| !c.is_whitespace()).collect();
        let well_formed = digits.chars().all(|c| c.is_ascii_digit())
            && matches!(digits.len(), SIREN_LEN | SIRET_LEN);
        if well_formed {
            Ok(Self(digits))
        } else {
            Err(CustomerError {
                field: CustomerField::RegistrationNumber,
                rule: "must hold 9 or 14 digits",
            })
        }
    }
}

impl TryFrom<String> for RegistrationNumber {
    type Error = CustomerError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

pub fn siren_of(stored: &str) -> String {
    let digits: String = stored.chars().filter(|c| !c.is_whitespace()).collect();
    let numeric = digits.chars().all(|c| c.is_ascii_digit());
    if numeric && digits.len() == SIRET_LEN {
        digits[..SIREN_LEN].to_owned()
    } else if numeric && digits.len() == SIREN_LEN {
        digits
    } else {
        stored.to_owned()
    }
}

pub fn payment_terms_of(value: Option<i64>) -> Result<Option<PaymentTermsDays>, CustomerError> {
    value
        .map(|days| {
            PaymentTermsDays::try_from(days).map_err(|_| CustomerError {
                field: CustomerField::PaymentTermsDays,
                rule: "must be between 0 and 60 days",
            })
        })
        .transpose()
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct CustomerProfile {
    pub kind: CustomerKind,
    pub website: Option<String>,
    pub vat_number: Option<String>,
    pub owner_member_id: Option<MemberId>,
    pub lead_source: Option<LeadSource>,
    pub notes: Option<String>,
    pub payment_terms: Option<PaymentTermsDays>,
    pub billing_contact_id: Option<CustomerContactId>,
    pub billing_address_id: Option<CustomerContextId>,
}

impl CustomerProfile {
    pub fn normalized(self) -> Self {
        Self {
            website: present(self.website),
            vat_number: present(self.vat_number),
            notes: present(self.notes),
            ..self
        }
    }
}

fn present(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nine_and_fourteen_digits_are_accepted() {
        assert_eq!(
            RegistrationNumber::try_from("123456789").unwrap().as_str(),
            "123456789"
        );
        assert_eq!(
            RegistrationNumber::try_from("12345678900012")
                .unwrap()
                .as_str(),
            "12345678900012"
        );
    }

    #[test]
    fn spaces_are_ignored() {
        let number = RegistrationNumber::try_from("123 456 789 00012").unwrap();

        assert_eq!(number.as_str(), "12345678900012");
    }

    #[test]
    fn any_other_length_is_refused_naming_the_field() {
        for refused in [
            "",
            "12345678",
            "1234567890",
            "1234567890123",
            "123456789000123",
        ] {
            let error = RegistrationNumber::try_from(refused).unwrap_err();

            assert_eq!(error.field, CustomerField::RegistrationNumber, "{refused}");
        }
    }

    #[test]
    fn letters_are_refused() {
        assert!(RegistrationNumber::try_from("12345678A").is_err());
        assert!(RegistrationNumber::try_from("1234567890001A").is_err());
    }

    #[test]
    fn the_siren_is_the_first_nine_digits() {
        let siret = RegistrationNumber::try_from("12345678900012").unwrap();
        let siren = RegistrationNumber::try_from("123456789").unwrap();

        assert_eq!(siret.siren(), "123456789");
        assert_eq!(siren.siren(), "123456789");
    }

    #[test]
    fn the_siren_of_a_stored_value_derives_from_a_siret() {
        assert_eq!(siren_of("12345678900012"), "123456789");
        assert_eq!(siren_of("123 456 789 00012"), "123456789");
        assert_eq!(siren_of("123456789"), "123456789");
    }

    #[test]
    fn the_siren_of_a_legacy_value_is_left_unchanged() {
        assert_eq!(siren_of("RM-44"), "RM-44");
        assert_eq!(siren_of("12345"), "12345");
    }

    #[test]
    fn payment_terms_of_61_days_are_refused() {
        let error = payment_terms_of(Some(61)).unwrap_err();

        assert_eq!(error.field, CustomerField::PaymentTermsDays);
    }

    #[test]
    fn payment_terms_accept_zero_to_sixty_or_nothing() {
        assert_eq!(payment_terms_of(None).unwrap(), None);
        assert_eq!(payment_terms_of(Some(0)).unwrap().unwrap().get(), 0);
        assert_eq!(payment_terms_of(Some(60)).unwrap().unwrap().get(), 60);
        assert!(payment_terms_of(Some(-1)).is_err());
    }

    #[test]
    fn a_blank_profile_text_is_stored_as_nothing() {
        let profile = CustomerProfile {
            website: Some("  ".to_owned()),
            vat_number: Some(" FR123 ".to_owned()),
            notes: Some(String::new()),
            ..CustomerProfile::default()
        }
        .normalized();

        assert_eq!(profile.website, None);
        assert_eq!(profile.vat_number.as_deref(), Some("FR123"));
        assert_eq!(profile.notes, None);
    }

    #[test]
    fn existing_customers_default_to_professional() {
        assert_eq!(CustomerProfile::default().kind, CustomerKind::Professional);
    }

    #[test]
    fn lead_source_wire_values_follow_the_prototype() {
        for value in [
            "bouche_a_oreille",
            "site_web",
            "recommandation",
            "salon",
            "architecte",
            "autre",
        ] {
            let parsed: LeadSource = value.parse().unwrap();

            assert_eq!(parsed.as_str(), value);
            assert_eq!(
                serde_json::to_string(&parsed).unwrap(),
                format!("\"{value}\"")
            );
        }
        assert!("lead".parse::<LeadSource>().is_err());
    }
}
