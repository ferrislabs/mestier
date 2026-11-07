use std::collections::BTreeMap;

use rust_decimal::{Decimal, RoundingStrategy, prelude::ToPrimitive};

use super::{
    BillingFrequency, DiscountPercent, QuoteError, QuoteField, QuoteLine, QuoteVatBreakdownLine,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuoteTotals {
    pub net_cents: i32,
    pub vat_breakdown: Vec<QuoteVatBreakdownLine>,
    pub gross_cents: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecurringTotal {
    pub frequency: BillingFrequency,
    pub net_cents: i32,
    pub vat_cents: i32,
    pub gross_cents: i32,
}

fn out_of_bounds() -> QuoteError {
    QuoteError::invalid(QuoteField::Lines, "the total is outside supported bounds")
}

pub fn line_net_cents(
    quantity: Decimal,
    unit_price_cents: i32,
    discount: DiscountPercent,
) -> Result<i64, QuoteError> {
    let remaining = Decimal::ONE_HUNDRED - discount.get();
    let net = quantity
        .checked_mul(Decimal::from(unit_price_cents))
        .and_then(|amount| amount.checked_mul(remaining))
        .map(|amount| amount / Decimal::ONE_HUNDRED)
        .ok_or_else(out_of_bounds)?;

    net.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .ok_or_else(out_of_bounds)
}

impl QuoteLine {
    pub fn net_cents(&self) -> Result<i64, QuoteError> {
        line_net_cents(self.quantity, self.unit_price_cents, self.discount_percent)
    }
}

pub fn vat_cents(base_cents: i64, rate_bp: i32) -> Result<i64, QuoteError> {
    let scaled = base_cents
        .checked_mul(i64::from(rate_bp))
        .ok_or_else(out_of_bounds)?;
    let quotient = scaled / 10_000;
    let remainder = scaled % 10_000;

    Ok(if remainder.abs() * 2 >= 10_000 {
        quotient + scaled.signum()
    } else {
        quotient
    })
}

struct Aggregate {
    net: i64,
    vat_by_rate: BTreeMap<i32, i64>,
}

fn aggregate<'a>(
    lines: impl Iterator<Item = &'a QuoteLine>,
    subject_to_vat: bool,
) -> Result<Aggregate, QuoteError> {
    let mut net: i64 = 0;
    let mut base_by_rate: BTreeMap<i32, i64> = BTreeMap::new();

    for line in lines {
        let line_net = line.net_cents()?;
        net = net.checked_add(line_net).ok_or_else(out_of_bounds)?;
        let base = base_by_rate
            .entry(line.vat_rate_bp.unwrap_or(0))
            .or_insert(0);
        *base = base.checked_add(line_net).ok_or_else(out_of_bounds)?;
    }

    let vat_by_rate = if subject_to_vat {
        base_by_rate
            .into_iter()
            .map(|(rate_bp, base)| vat_cents(base, rate_bp).map(|vat| (rate_bp, vat)))
            .collect::<Result<BTreeMap<_, _>, _>>()?
    } else {
        BTreeMap::new()
    };

    Ok(Aggregate { net, vat_by_rate })
}

fn to_i32(value: i64) -> Result<i32, QuoteError> {
    i32::try_from(value).map_err(|_| out_of_bounds())
}

pub fn totals_of(lines: &[QuoteLine], subject_to_vat: bool) -> Result<QuoteTotals, QuoteError> {
    let Aggregate { net, vat_by_rate } = aggregate(lines.iter(), subject_to_vat)?;
    let vat_total: i64 = vat_by_rate.values().sum();

    Ok(QuoteTotals {
        net_cents: to_i32(net)?,
        vat_breakdown: vat_by_rate
            .into_iter()
            .map(|(rate_bp, vat)| {
                to_i32(vat).map(|vat_cents| QuoteVatBreakdownLine { rate_bp, vat_cents })
            })
            .collect::<Result<Vec<_>, _>>()?,
        gross_cents: to_i32(net.checked_add(vat_total).ok_or_else(out_of_bounds)?)?,
    })
}

pub fn recurring_totals(lines: &[QuoteLine], subject_to_vat: bool) -> Vec<RecurringTotal> {
    BillingFrequency::RECURRING
        .into_iter()
        .filter_map(|frequency| {
            let mut matching = lines
                .iter()
                .filter(|line| line.billing_frequency == frequency)
                .peekable();
            matching.peek()?;
            let Aggregate { net, vat_by_rate } = aggregate(matching, subject_to_vat).ok()?;
            let vat: i64 = vat_by_rate.values().sum();

            Some(RecurringTotal {
                frequency,
                net_cents: to_i32(net).ok()?,
                vat_cents: to_i32(vat).ok()?,
                gross_cents: to_i32(net + vat).ok()?,
            })
        })
        .collect()
}

#[cfg(test)]
pub(crate) mod fixtures {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::{OrganizationId, QuoteId, QuoteLineId};

    pub fn priced_line(
        quantity: &str,
        unit_price_cents: i32,
        vat_rate_bp: i32,
        discount: &str,
        frequency: BillingFrequency,
    ) -> QuoteLine {
        let now = Utc::now();
        QuoteLine {
            id: QuoteLineId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            quote_id: QuoteId(Uuid::new_v4()),
            service_rate_id: None,
            product_id: None,
            label: "Pose".to_owned(),
            quantity: quantity.parse().unwrap(),
            unit: "U".to_owned(),
            unit_price_cents,
            vat_rate_bp: Some(vat_rate_bp),
            discount_percent: DiscountPercent::try_from(discount.parse::<Decimal>().unwrap())
                .unwrap(),
            billing_frequency: frequency,
            notes: None,
            photo_keys: Vec::new(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{fixtures::priced_line, *};

    fn once(quantity: &str, price: i32, vat_bp: i32, discount: &str) -> QuoteLine {
        priced_line(quantity, price, vat_bp, discount, BillingFrequency::OneShot)
    }

    #[test]
    fn spec_crm_4_10_a_ten_percent_discount_applies_before_vat_and_rounds_to_the_cent() {
        assert_eq!(once("3", 10_000, 2000, "10").net_cents().unwrap(), 27_000);
        assert_eq!(once("1.5", 333, 2000, "0").net_cents().unwrap(), 500);

        let totals = totals_of(&[once("3", 10_000, 2000, "10")], true).unwrap();
        assert_eq!(totals.net_cents, 27_000);
        assert_eq!(
            totals.vat_breakdown,
            vec![QuoteVatBreakdownLine {
                rate_bp: 2000,
                vat_cents: 5_400
            }]
        );
        assert_eq!(totals.gross_cents, 32_400);
    }

    #[test]
    fn a_line_net_rounds_half_away_from_zero_not_to_even() {
        assert_eq!(once("0.5", 5, 0, "0").net_cents().unwrap(), 3);
        assert_eq!(once("1.5", 333, 0, "0").net_cents().unwrap(), 500);
        assert_eq!(once("2.5", 1, 0, "0").net_cents().unwrap(), 3);
        assert_eq!(once("1", 1_000, 0, "12.5").net_cents().unwrap(), 875);
        assert_eq!(once("1", 1, 0, "50").net_cents().unwrap(), 1);
    }

    #[test]
    fn a_full_discount_leaves_nothing_to_pay() {
        assert_eq!(once("4", 2_500, 2000, "100").net_cents().unwrap(), 0);
    }

    #[test]
    fn vat_rounds_half_away_from_zero_on_the_base() {
        assert_eq!(vat_cents(20_066, 2000).unwrap(), 4_013);
        assert_eq!(vat_cents(5, 5000).unwrap(), 3);
        assert_eq!(vat_cents(-5, 5000).unwrap(), -3);
        assert_eq!(vat_cents(10_000, 1000).unwrap(), 1_000);
    }

    #[test]
    fn spec_crm_4_11_two_lines_of_100_33_at_20_percent_give_a_base_of_20066_and_vat_of_4013() {
        let totals = totals_of(
            &[once("1", 10_033, 2000, "0"), once("1", 10_033, 2000, "0")],
            true,
        )
        .unwrap();

        assert_eq!(totals.net_cents, 20_066);
        assert_eq!(
            totals.vat_breakdown,
            vec![QuoteVatBreakdownLine {
                rate_bp: 2000,
                vat_cents: 4_013
            }]
        );
        assert_eq!(totals.gross_cents, 24_079);
    }

    #[test]
    fn the_prototype_example_with_two_rates_groups_the_vat_by_rate() {
        let totals = totals_of(
            &[
                once("1", 10_033, 2000, "0"),
                once("1", 10_033, 2000, "0"),
                once("2", 5_000, 1000, "0"),
            ],
            true,
        )
        .unwrap();

        assert_eq!(totals.net_cents, 30_066);
        assert_eq!(
            totals.vat_breakdown,
            vec![
                QuoteVatBreakdownLine {
                    rate_bp: 1000,
                    vat_cents: 1_000
                },
                QuoteVatBreakdownLine {
                    rate_bp: 2000,
                    vat_cents: 4_013
                },
            ]
        );
        assert_eq!(totals.gross_cents, 35_079);
    }

    #[test]
    fn an_organization_not_subject_to_vat_has_no_breakdown() {
        let totals = totals_of(&[once("1", 10_033, 2000, "0")], false).unwrap();

        assert_eq!(totals.net_cents, 10_033);
        assert!(totals.vat_breakdown.is_empty());
        assert_eq!(totals.gross_cents, 10_033);
    }

    #[test]
    fn spec_crm_4_12_frequency_never_changes_the_totals() {
        let one_shot = [once("2", 5_000, 2000, "0"), once("1", 12_000, 2000, "0")];
        let recurring = [
            priced_line("2", 5_000, 2000, "0", BillingFrequency::Monthly),
            priced_line("1", 12_000, 2000, "0", BillingFrequency::Yearly),
        ];

        assert_eq!(
            totals_of(&one_shot, true).unwrap(),
            totals_of(&recurring, true).unwrap()
        );
    }

    #[test]
    fn spec_crm_4_12_recurring_totals_group_by_frequency_and_exclude_one_shot() {
        let lines = [
            priced_line("2", 10_000, 2000, "0", BillingFrequency::OneShot),
            priced_line("1", 2_000, 2000, "0", BillingFrequency::Monthly),
            priced_line("1", 12_000, 2000, "0", BillingFrequency::Yearly),
            priced_line("2", 1_700, 2000, "0", BillingFrequency::Monthly),
        ];

        let recurring = recurring_totals(&lines, true);

        assert_eq!(
            recurring,
            vec![
                RecurringTotal {
                    frequency: BillingFrequency::Monthly,
                    net_cents: 5_400,
                    vat_cents: 1_080,
                    gross_cents: 6_480,
                },
                RecurringTotal {
                    frequency: BillingFrequency::Yearly,
                    net_cents: 12_000,
                    vat_cents: 2_400,
                    gross_cents: 14_400,
                },
            ]
        );
    }

    #[test]
    fn a_quote_with_only_one_shot_lines_has_no_recurring_block() {
        assert!(recurring_totals(&[once("1", 100, 2000, "0")], true).is_empty());
    }

    #[test]
    fn recurring_totals_apply_the_discount_and_skip_vat_when_not_subject() {
        let lines = [priced_line(
            "1",
            10_000,
            2000,
            "10",
            BillingFrequency::Quarterly,
        )];

        assert_eq!(
            recurring_totals(&lines, false),
            vec![RecurringTotal {
                frequency: BillingFrequency::Quarterly,
                net_cents: 9_000,
                vat_cents: 0,
                gross_cents: 9_000,
            }]
        );
    }
}
