use std::fmt;

pub const DEFAULT_LATE_PENALTY: &str = "trois fois le taux d’intérêt légal";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BillingSettingsField {
    PaymentTermsDays,
    QuoteValidityDays,
    DefaultVatRateBp,
    LatePenalty,
    RecoveryFeeCents,
    NumberPadding,
}

impl BillingSettingsField {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PaymentTermsDays => "payment_terms_days",
            Self::QuoteValidityDays => "quote_validity_days",
            Self::DefaultVatRateBp => "default_vat_rate_bp",
            Self::LatePenalty => "late_penalty",
            Self::RecoveryFeeCents => "recovery_fee_cents",
            Self::NumberPadding => "number_padding",
        }
    }
}

impl fmt::Display for BillingSettingsField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("{field}: {rule}")]
pub struct BillingSettingsError {
    pub field: BillingSettingsField,
    pub rule: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PaymentTermsDays(u8);

impl PaymentTermsDays {
    pub const MAX: i64 = 60;

    pub fn get(self) -> i32 {
        i32::from(self.0)
    }
}

impl TryFrom<i64> for PaymentTermsDays {
    type Error = BillingSettingsError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .ok()
            .filter(|days| i64::from(*days) <= Self::MAX)
            .map(Self)
            .ok_or(BillingSettingsError {
                field: BillingSettingsField::PaymentTermsDays,
                rule: "must be between 0 and 60 days",
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct QuoteValidityDays(u32);

impl QuoteValidityDays {
    pub fn get(self) -> i32 {
        i32::try_from(self.0).unwrap_or(i32::MAX)
    }
}

impl TryFrom<i64> for QuoteValidityDays {
    type Error = BillingSettingsError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        u32::try_from(value)
            .ok()
            .filter(|days| *days >= 1 && i32::try_from(*days).is_ok())
            .map(Self)
            .ok_or(BillingSettingsError {
                field: BillingSettingsField::QuoteValidityDays,
                rule: "must be at least 1 day",
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DefaultVatRate {
    Zero,
    Reduced,
    Intermediate,
    Standard,
}

impl DefaultVatRate {
    pub fn basis_points(self) -> i32 {
        match self {
            Self::Zero => 0,
            Self::Reduced => 550,
            Self::Intermediate => 1000,
            Self::Standard => 2000,
        }
    }
}

impl TryFrom<i64> for DefaultVatRate {
    type Error = BillingSettingsError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Zero),
            550 => Ok(Self::Reduced),
            1000 => Ok(Self::Intermediate),
            2000 => Ok(Self::Standard),
            _ => Err(BillingSettingsError {
                field: BillingSettingsField::DefaultVatRateBp,
                rule: "must be one of 0, 550, 1000, 2000",
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatePenalty(String);

impl LatePenalty {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for LatePenalty {
    type Error = BillingSettingsError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(BillingSettingsError {
                field: BillingSettingsField::LatePenalty,
                rule: "must not be empty",
            });
        }
        Ok(Self(trimmed.to_owned()))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryFeeCents(u32);

impl RecoveryFeeCents {
    pub fn get(self) -> i32 {
        i32::try_from(self.0).unwrap_or(i32::MAX)
    }
}

impl TryFrom<i64> for RecoveryFeeCents {
    type Error = BillingSettingsError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        i32::try_from(value)
            .ok()
            .and_then(|cents| u32::try_from(cents).ok())
            .map(Self)
            .ok_or(BillingSettingsError {
                field: BillingSettingsField::RecoveryFeeCents,
                rule: "must be zero or more cents",
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumberPadding(u8);

impl NumberPadding {
    pub const DEFAULT: Self = Self(4);

    pub fn get(self) -> usize {
        usize::from(self.0)
    }

    pub fn as_i32(self) -> i32 {
        i32::from(self.0)
    }
}

impl TryFrom<i64> for NumberPadding {
    type Error = BillingSettingsError;

    fn try_from(value: i64) -> Result<Self, Self::Error> {
        u8::try_from(value)
            .ok()
            .filter(|digits| (1..=6).contains(digits))
            .map(Self)
            .ok_or(BillingSettingsError {
                field: BillingSettingsField::NumberPadding,
                rule: "must be between 1 and 6 digits",
            })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillingSettingsInput {
    pub payment_terms_days: i64,
    pub quote_validity_days: i64,
    pub default_vat_rate_bp: i64,
    pub late_penalty: String,
    pub recovery_fee_cents: i64,
    pub invoice_footer: String,
    pub number_padding: i64,
    pub iban: Option<String>,
    pub bic: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BillingSettings {
    pub payment_terms_days: PaymentTermsDays,
    pub quote_validity_days: QuoteValidityDays,
    pub default_vat_rate: DefaultVatRate,
    pub late_penalty: LatePenalty,
    pub recovery_fee: RecoveryFeeCents,
    pub invoice_footer: String,
    pub number_padding: NumberPadding,
    pub iban: Option<String>,
    pub bic: Option<String>,
}

impl Default for BillingSettings {
    fn default() -> Self {
        Self {
            payment_terms_days: PaymentTermsDays(30),
            quote_validity_days: QuoteValidityDays(30),
            default_vat_rate: DefaultVatRate::Standard,
            late_penalty: LatePenalty(DEFAULT_LATE_PENALTY.to_owned()),
            recovery_fee: RecoveryFeeCents(4000),
            invoice_footer: String::new(),
            number_padding: NumberPadding::DEFAULT,
            iban: None,
            bic: None,
        }
    }
}

impl TryFrom<BillingSettingsInput> for BillingSettings {
    type Error = BillingSettingsError;

    fn try_from(input: BillingSettingsInput) -> Result<Self, Self::Error> {
        Ok(Self {
            payment_terms_days: PaymentTermsDays::try_from(input.payment_terms_days)?,
            quote_validity_days: QuoteValidityDays::try_from(input.quote_validity_days)?,
            default_vat_rate: DefaultVatRate::try_from(input.default_vat_rate_bp)?,
            late_penalty: LatePenalty::try_from(input.late_penalty)?,
            recovery_fee: RecoveryFeeCents::try_from(input.recovery_fee_cents)?,
            invoice_footer: input.invoice_footer.trim().to_owned(),
            number_padding: NumberPadding::try_from(input.number_padding)?,
            iban: non_blank(input.iban),
            bic: non_blank(input.bic),
        })
    }
}

fn non_blank(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IssuerBilling {
    pub iban: Option<String>,
    pub bic: Option<String>,
    pub late_penalty: String,
    pub recovery_fee_cents: i32,
    pub invoice_footer: String,
}

impl From<&BillingSettings> for IssuerBilling {
    fn from(settings: &BillingSettings) -> Self {
        Self {
            iban: settings.iban.clone(),
            bic: settings.bic.clone(),
            late_penalty: settings.late_penalty.as_str().to_owned(),
            recovery_fee_cents: settings.recovery_fee.get(),
            invoice_footer: settings.invoice_footer.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> BillingSettingsInput {
        BillingSettingsInput {
            payment_terms_days: 30,
            quote_validity_days: 30,
            default_vat_rate_bp: 2000,
            late_penalty: "trois fois le taux légal".to_owned(),
            recovery_fee_cents: 4000,
            invoice_footer: " Merci. ".to_owned(),
            number_padding: 4,
            iban: Some("  ".to_owned()),
            bic: Some(" AGRIFRPP ".to_owned()),
        }
    }

    fn refused_field(input: BillingSettingsInput) -> BillingSettingsField {
        BillingSettings::try_from(input).unwrap_err().field
    }

    #[test]
    fn defaults_follow_the_real_code() {
        let defaults = BillingSettings::default();

        assert_eq!(defaults.payment_terms_days.get(), 30);
        assert_eq!(defaults.quote_validity_days.get(), 30);
        assert_eq!(defaults.default_vat_rate.basis_points(), 2000);
        assert_eq!(defaults.late_penalty.as_str(), DEFAULT_LATE_PENALTY);
        assert_eq!(defaults.recovery_fee.get(), 4000);
        assert_eq!(defaults.invoice_footer, "");
        assert_eq!(defaults.number_padding.get(), 4);
        assert_eq!(defaults.iban, None);
        assert_eq!(defaults.bic, None);
    }

    #[test]
    fn a_valid_input_is_trimmed_and_blank_bank_fields_become_none() {
        let settings = BillingSettings::try_from(input()).unwrap();

        assert_eq!(settings.invoice_footer, "Merci.");
        assert_eq!(settings.iban, None);
        assert_eq!(settings.bic.as_deref(), Some("AGRIFRPP"));
    }

    #[test]
    fn payment_terms_are_bounded_between_0_and_60() {
        for days in [0, 60] {
            let accepted = BillingSettingsInput {
                payment_terms_days: days,
                ..input()
            };
            assert!(BillingSettings::try_from(accepted).is_ok());
        }
        for days in [-1, 61, 100_000] {
            let refused = BillingSettingsInput {
                payment_terms_days: days,
                ..input()
            };
            assert_eq!(
                refused_field(refused),
                BillingSettingsField::PaymentTermsDays
            );
        }
    }

    #[test]
    fn padding_is_bounded_between_1_and_6() {
        for digits in [1, 6] {
            let accepted = BillingSettingsInput {
                number_padding: digits,
                ..input()
            };
            assert!(BillingSettings::try_from(accepted).is_ok());
        }
        for digits in [0, 7] {
            let refused = BillingSettingsInput {
                number_padding: digits,
                ..input()
            };
            assert_eq!(refused_field(refused), BillingSettingsField::NumberPadding);
        }
    }

    #[test]
    fn validity_needs_at_least_one_day() {
        let refused = BillingSettingsInput {
            quote_validity_days: 0,
            ..input()
        };
        assert_eq!(
            refused_field(refused),
            BillingSettingsField::QuoteValidityDays
        );
    }

    #[test]
    fn recovery_fee_cannot_be_negative() {
        let refused = BillingSettingsInput {
            recovery_fee_cents: -1,
            ..input()
        };
        assert_eq!(
            refused_field(refused),
            BillingSettingsField::RecoveryFeeCents
        );
    }

    #[test]
    fn late_penalty_cannot_be_blank() {
        let refused = BillingSettingsInput {
            late_penalty: "   ".to_owned(),
            ..input()
        };
        assert_eq!(refused_field(refused), BillingSettingsField::LatePenalty);
    }

    #[test]
    fn vat_rate_is_one_of_the_four_french_rates() {
        for rate in [0, 550, 1000, 2000] {
            let accepted = BillingSettingsInput {
                default_vat_rate_bp: rate,
                ..input()
            };
            assert!(BillingSettings::try_from(accepted).is_ok());
        }
        let refused = BillingSettingsInput {
            default_vat_rate_bp: 700,
            ..input()
        };
        assert_eq!(
            refused_field(refused),
            BillingSettingsField::DefaultVatRateBp
        );
    }

    #[test]
    fn the_issuer_snapshot_copies_what_an_invoice_freezes() {
        let settings = BillingSettings::try_from(input()).unwrap();

        let snapshot = IssuerBilling::from(&settings);

        assert_eq!(snapshot.late_penalty, "trois fois le taux légal");
        assert_eq!(snapshot.recovery_fee_cents, 4000);
        assert_eq!(snapshot.invoice_footer, "Merci.");
        assert_eq!(snapshot.iban, None);
        assert_eq!(snapshot.bic.as_deref(), Some("AGRIFRPP"));
    }
}
