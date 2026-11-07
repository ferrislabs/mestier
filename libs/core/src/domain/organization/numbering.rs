use std::str::FromStr;

use crate::domain::organization::billing_settings::NumberPadding;

pub const CREDIT_NOTE_NUMBER_PREFIX: &str = "AV";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NumberSeries {
    Quote,
    Invoice,
    CreditNote,
}

impl NumberSeries {
    pub const ALL: [NumberSeries; 3] = [Self::Quote, Self::Invoice, Self::CreditNote];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Quote => "QUOTE",
            Self::Invoice => "INVOICE",
            Self::CreditNote => "CREDIT_NOTE",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("unknown number series `{0}`")]
pub struct UnknownNumberSeries(pub String);

impl FromStr for NumberSeries {
    type Err = UnknownNumberSeries;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|series| series.as_str() == value)
            .ok_or_else(|| UnknownNumberSeries(value.to_owned()))
    }
}

pub fn format_number(prefix: &str, year: i32, sequence: i32, padding: NumberPadding) -> String {
    format!("{prefix}-{year}-{sequence:0width$}", width = padding.get())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SeriesNumbering {
    pub series: NumberSeries,
    pub prefix: String,
    pub year: i32,
    pub last_assigned: i32,
    pub padding: NumberPadding,
}

impl SeriesNumbering {
    pub fn next_number(&self) -> i32 {
        self.last_assigned.saturating_add(1)
    }

    pub fn next_reference(&self) -> String {
        format_number(&self.prefix, self.year, self.next_number(), self.padding)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum NumberingError {
    #[error("the next number must be above the last assigned one ({last_assigned})")]
    NotForward { last_assigned: i32 },
}

pub fn forward_target(requested_next: i64, last_assigned: i32) -> Result<i32, NumberingError> {
    let last = i64::from(last_assigned);
    if requested_next <= last {
        return Err(NumberingError::NotForward { last_assigned });
    }
    i32::try_from(requested_next - 1).map_err(|_| NumberingError::NotForward { last_assigned })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn padding(digits: i64) -> NumberPadding {
        NumberPadding::try_from(digits).unwrap()
    }

    #[test]
    fn a_number_is_prefix_year_and_padded_sequence() {
        assert_eq!(format_number("AV", 2026, 7, padding(4)), "AV-2026-0007");
        assert_eq!(format_number("DEV", 2026, 7, padding(3)), "DEV-2026-007");
        assert_eq!(
            format_number("FAC", 2026, 1234567, padding(2)),
            "FAC-2026-1234567"
        );
    }

    #[test]
    fn a_series_parses_from_its_wire_name() {
        for series in NumberSeries::ALL {
            assert_eq!(series.as_str().parse::<NumberSeries>(), Ok(series));
        }
        assert!("credit_note".parse::<NumberSeries>().is_err());
    }

    #[test]
    fn an_empty_series_starts_at_one() {
        let numbering = SeriesNumbering {
            series: NumberSeries::CreditNote,
            prefix: CREDIT_NOTE_NUMBER_PREFIX.to_owned(),
            year: 2027,
            last_assigned: 0,
            padding: padding(4),
        };

        assert_eq!(numbering.next_reference(), "AV-2027-0001");
    }

    #[test]
    fn resuming_at_or_below_the_last_assigned_number_is_refused() {
        assert_eq!(
            forward_target(10, 10),
            Err(NumberingError::NotForward { last_assigned: 10 })
        );
        assert_eq!(
            forward_target(3, 10),
            Err(NumberingError::NotForward { last_assigned: 10 })
        );
        assert_eq!(
            forward_target(0, 0),
            Err(NumberingError::NotForward { last_assigned: 0 })
        );
    }

    #[test]
    fn resuming_above_the_last_assigned_number_returns_the_counter_to_store() {
        assert_eq!(forward_target(11, 10), Ok(10));
        assert_eq!(forward_target(120, 10), Ok(119));
    }

    #[test]
    fn a_number_beyond_the_counter_range_is_refused() {
        assert!(forward_target(i64::from(i32::MAX) + 5, 10).is_err());
    }
}
