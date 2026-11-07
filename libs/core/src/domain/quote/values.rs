use std::{fmt, str::FromStr};

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::error::{QuoteError, QuoteField};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BillingFrequency {
    #[default]
    OneShot,
    Monthly,
    Quarterly,
    Yearly,
}

impl BillingFrequency {
    pub const RECURRING: [BillingFrequency; 3] = [Self::Monthly, Self::Quarterly, Self::Yearly];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::OneShot => "ONE_SHOT",
            Self::Monthly => "MONTHLY",
            Self::Quarterly => "QUARTERLY",
            Self::Yearly => "YEARLY",
        }
    }

    pub fn is_recurring(self) -> bool {
        self != Self::OneShot
    }
}

impl fmt::Display for BillingFrequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for BillingFrequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ONE_SHOT" => Ok(Self::OneShot),
            "MONTHLY" => Ok(Self::Monthly),
            "QUARTERLY" => Ok(Self::Quarterly),
            "YEARLY" => Ok(Self::Yearly),
            other => Err(format!("invalid billing frequency `{other}`")),
        }
    }
}

macro_rules! percent {
    ($name:ident, $field:expr, $scale:expr, $rule:expr) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default)]
        pub struct $name(Decimal);

        impl $name {
            pub const ZERO: Self = Self(Decimal::ZERO);

            pub fn get(self) -> Decimal {
                self.0
            }
        }

        impl TryFrom<Decimal> for $name {
            type Error = QuoteError;

            fn try_from(value: Decimal) -> Result<Self, Self::Error> {
                let in_range = value >= Decimal::ZERO && value <= Decimal::ONE_HUNDRED;
                if in_range && value.normalize().scale() <= $scale {
                    Ok(Self(value))
                } else {
                    Err(QuoteError::invalid($field, $rule))
                }
            }
        }
    };
}

percent!(
    DepositPercent,
    QuoteField::DepositPercent,
    1,
    "must be between 0 and 100 with at most one decimal"
);
percent!(
    DiscountPercent,
    QuoteField::DiscountPercent,
    2,
    "must be between 0 and 100 with at most two decimals"
);

#[cfg(test)]
mod tests {
    use super::*;

    fn dec(text: &str) -> Decimal {
        text.parse().unwrap()
    }

    #[test]
    fn a_deposit_holds_zero_to_one_hundred_with_one_decimal() {
        assert!(DepositPercent::try_from(dec("0")).is_ok());
        assert!(DepositPercent::try_from(dec("30.5")).is_ok());
        assert!(DepositPercent::try_from(dec("100")).is_ok());
        assert!(DepositPercent::try_from(dec("30.50")).is_ok());
        assert_eq!(
            DepositPercent::try_from(dec("100.1")),
            Err(QuoteError::invalid(
                QuoteField::DepositPercent,
                "must be between 0 and 100 with at most one decimal"
            ))
        );
        assert!(DepositPercent::try_from(dec("-1")).is_err());
        assert!(DepositPercent::try_from(dec("30.55")).is_err());
    }

    #[test]
    fn a_discount_holds_zero_to_one_hundred_with_two_decimals() {
        assert!(DiscountPercent::try_from(dec("12.25")).is_ok());
        assert!(DiscountPercent::try_from(dec("100")).is_ok());
        assert!(DiscountPercent::try_from(dec("100.01")).is_err());
        assert!(DiscountPercent::try_from(dec("-0.01")).is_err());
        assert!(DiscountPercent::try_from(dec("12.255")).is_err());
    }

    #[test]
    fn a_frequency_parses_its_wire_form_and_only_one_shot_is_not_recurring() {
        for frequency in [
            BillingFrequency::OneShot,
            BillingFrequency::Monthly,
            BillingFrequency::Quarterly,
            BillingFrequency::Yearly,
        ] {
            assert_eq!(
                frequency.as_str().parse::<BillingFrequency>(),
                Ok(frequency)
            );
        }
        assert!("WEEKLY".parse::<BillingFrequency>().is_err());
        assert!(!BillingFrequency::OneShot.is_recurring());
        assert!(BillingFrequency::RECURRING.iter().all(|f| f.is_recurring()));
    }
}
