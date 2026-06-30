use rust_decimal::prelude::*;
use rust_decimal::RoundingStrategy;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DepositBasis {
    Percent,
    Fixed,
}

#[derive(Debug, Clone)]
pub struct BillingLine {
    pub quantity: Decimal,
    pub unit_price_cents: i64,
    pub vat_rate: Decimal, // percentage, e.g. 20 means 20%
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VatBucket {
    pub rate: Decimal,       // the VAT percentage for this bucket
    pub base_ht_cents: i64,  // sum of line HT (excl. VAT) at this rate
    pub vat_cents: i64,      // VAT amount for this bucket
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Totals {
    pub total_ht_cents: i64,
    pub total_vat_cents: i64,
    pub total_ttc_cents: i64,
    pub vat_breakdown: Vec<VatBucket>, // sorted by rate ascending
}

/// Round a Decimal to 0 decimal places using MidpointAwayFromZero (half-up), then convert to i64.
fn round_to_i64(d: Decimal) -> i64 {
    d.round_dp_with_strategy(0, RoundingStrategy::MidpointAwayFromZero)
        .to_i64()
        .expect("billing decimal out of i64 range")
}

pub fn compute_totals(lines: &[BillingLine]) -> Totals {
    if lines.is_empty() {
        return Totals {
            total_ht_cents: 0,
            total_vat_cents: 0,
            total_ttc_cents: 0,
            vat_breakdown: vec![],
        };
    }

    // BTreeMap keyed by a string representation of rate for ordering; we'll collect rate alongside.
    // We use Decimal's canonical string so equal decimals map to the same key.
    // Actually use a Vec of (rate, base_ht) pairs collected into a BTreeMap<String, (Decimal, i64)>.
    let mut buckets: BTreeMap<String, (Decimal, i64)> = BTreeMap::new();

    for line in lines {
        let ht = round_to_i64(line.quantity * Decimal::from(line.unit_price_cents));
        let key = line.vat_rate.normalize().to_string();
        let entry = buckets.entry(key).or_insert((line.vat_rate, 0i64));
        entry.1 += ht;
    }

    // Build sorted VatBucket vec (BTreeMap on string key — strings of decimals don't sort
    // numerically when digits differ, so collect and sort by rate value instead).
    let mut bucket_vec: Vec<VatBucket> = buckets
        .into_values()
        .map(|(rate, base_ht_cents)| {
            let rate_decimal = Decimal::from(base_ht_cents) * rate
                / Decimal::from(100u32);
            let vat_cents = round_to_i64(rate_decimal);
            VatBucket {
                rate,
                base_ht_cents,
                vat_cents,
            }
        })
        .collect();

    bucket_vec.sort_by(|a, b| a.rate.cmp(&b.rate));

    let total_ht_cents: i64 = bucket_vec.iter().map(|b| b.base_ht_cents).sum();
    let total_vat_cents: i64 = bucket_vec.iter().map(|b| b.vat_cents).sum();

    Totals {
        total_ht_cents,
        total_vat_cents,
        total_ttc_cents: total_ht_cents + total_vat_cents,
        vat_breakdown: bucket_vec,
    }
}

pub fn resolve_deposit(total_ttc_cents: i64, basis: DepositBasis, value: Decimal) -> i64 {
    let raw = match basis {
        DepositBasis::Percent => {
            let amount = Decimal::from(total_ttc_cents) * value / Decimal::from(100u32);
            round_to_i64(amount)
        }
        DepositBasis::Fixed => {
            round_to_i64(value)
        }
    };
    raw.clamp(0, total_ttc_cents)
}

pub fn remaining_to_pay(total_ttc_cents: i64, already_invoiced_cents: i64) -> i64 {
    (total_ttc_cents - already_invoiced_cents).max(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn dec(s: &str) -> Decimal {
        Decimal::from_str(s).unwrap()
    }

    // 1. Single line qty 1, unit 10000, vat 20 → HT 10000, VAT 2000, TTC 12000
    #[test]
    fn single_line_20_pct_vat() {
        let lines = vec![BillingLine {
            quantity: Decimal::from(1u32),
            unit_price_cents: 10000,
            vat_rate: Decimal::from(20u32),
        }];
        let totals = compute_totals(&lines);
        assert_eq!(totals.total_ht_cents, 10000);
        assert_eq!(totals.total_vat_cents, 2000);
        assert_eq!(totals.total_ttc_cents, 12000);
        assert_eq!(
            totals.vat_breakdown,
            vec![VatBucket {
                rate: Decimal::from(20u32),
                base_ht_cents: 10000,
                vat_cents: 2000,
            }]
        );
    }

    // 2. Multi-rate: lineA{qty1, 10000, 20}, lineB{qty1, 5000, 10}
    // → HT 15000, VAT 2500, TTC 17500; breakdown sorted [{10,5000,500},{20,10000,2000}]
    #[test]
    fn multi_rate_sorted_breakdown() {
        let lines = vec![
            BillingLine {
                quantity: Decimal::from(1u32),
                unit_price_cents: 10000,
                vat_rate: Decimal::from(20u32),
            },
            BillingLine {
                quantity: Decimal::from(1u32),
                unit_price_cents: 5000,
                vat_rate: Decimal::from(10u32),
            },
        ];
        let totals = compute_totals(&lines);
        assert_eq!(totals.total_ht_cents, 15000);
        assert_eq!(totals.total_vat_cents, 2500);
        assert_eq!(totals.total_ttc_cents, 17500);
        assert_eq!(
            totals.vat_breakdown,
            vec![
                VatBucket {
                    rate: Decimal::from(10u32),
                    base_ht_cents: 5000,
                    vat_cents: 500,
                },
                VatBucket {
                    rate: Decimal::from(20u32),
                    base_ht_cents: 10000,
                    vat_cents: 2000,
                },
            ]
        );
    }

    // 3. Decimal qty 2.5, unit 4000, vat 20 → HT 10000, VAT 2000, TTC 12000
    #[test]
    fn decimal_quantity() {
        let lines = vec![BillingLine {
            quantity: dec("2.5"),
            unit_price_cents: 4000,
            vat_rate: Decimal::from(20u32),
        }];
        let totals = compute_totals(&lines);
        assert_eq!(totals.total_ht_cents, 10000);
        assert_eq!(totals.total_vat_cents, 2000);
        assert_eq!(totals.total_ttc_cents, 12000);
    }

    // 4. Rounding: qty 3, unit 333, vat 20 → HT 999, VAT 200 (199.8 → 200), TTC 1199
    #[test]
    fn vat_rounding_half_up() {
        let lines = vec![BillingLine {
            quantity: Decimal::from(3u32),
            unit_price_cents: 333,
            vat_rate: Decimal::from(20u32),
        }];
        let totals = compute_totals(&lines);
        assert_eq!(totals.total_ht_cents, 999);
        assert_eq!(totals.total_vat_cents, 200);
        assert_eq!(totals.total_ttc_cents, 1199);
    }

    // 5. Empty lines → Totals all zero, empty breakdown
    #[test]
    fn empty_lines() {
        let totals = compute_totals(&[]);
        assert_eq!(totals.total_ht_cents, 0);
        assert_eq!(totals.total_vat_cents, 0);
        assert_eq!(totals.total_ttc_cents, 0);
        assert!(totals.vat_breakdown.is_empty());
    }

    // 6. resolve_deposit cases
    #[test]
    fn resolve_deposit_percent() {
        let result = resolve_deposit(120000, DepositBasis::Percent, dec("30"));
        assert_eq!(result, 36000);
    }

    #[test]
    fn resolve_deposit_fixed() {
        let result = resolve_deposit(120000, DepositBasis::Fixed, dec("50000"));
        assert_eq!(result, 50000);
    }

    #[test]
    fn resolve_deposit_percent_clamp_above_total() {
        let result = resolve_deposit(120000, DepositBasis::Percent, dec("200"));
        assert_eq!(result, 120000);
    }

    // 7. remaining_to_pay cases
    #[test]
    fn remaining_to_pay_normal() {
        assert_eq!(remaining_to_pay(120000, 36000), 84000);
    }

    #[test]
    fn remaining_to_pay_clamped_to_zero() {
        assert_eq!(remaining_to_pay(120000, 150000), 0);
    }

    // Edge: zero vat rate
    #[test]
    fn zero_vat_rate() {
        let lines = vec![BillingLine {
            quantity: Decimal::from(1u32),
            unit_price_cents: 5000,
            vat_rate: Decimal::from(0u32),
        }];
        let totals = compute_totals(&lines);
        assert_eq!(totals.total_ht_cents, 5000);
        assert_eq!(totals.total_vat_cents, 0);
        assert_eq!(totals.total_ttc_cents, 5000);
    }

    // Edge: same vat rate on two lines aggregates into one bucket
    #[test]
    fn same_rate_two_lines_one_bucket() {
        let lines = vec![
            BillingLine {
                quantity: Decimal::from(1u32),
                unit_price_cents: 3000,
                vat_rate: Decimal::from(20u32),
            },
            BillingLine {
                quantity: Decimal::from(2u32),
                unit_price_cents: 2000,
                vat_rate: Decimal::from(20u32),
            },
        ];
        let totals = compute_totals(&lines);
        // 3000 + 4000 = 7000 HT; VAT = round(7000 * 20/100) = 1400
        assert_eq!(totals.total_ht_cents, 7000);
        assert_eq!(totals.total_vat_cents, 1400);
        assert_eq!(totals.total_ttc_cents, 8400);
        assert_eq!(totals.vat_breakdown.len(), 1);
    }
}
