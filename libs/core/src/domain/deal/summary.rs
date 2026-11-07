use chrono::{DateTime, Days, Utc};

use super::DealStageName;

pub const SUMMARY_WINDOW_DAYS: u64 = 90;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StageTotal {
    pub stage: DealStageName,
    pub count: i64,
    pub amount_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineSummary {
    pub open_count: i64,
    pub open_amount_cents: i64,
    pub stages: Vec<StageTotal>,
    pub won_count: i64,
    pub lost_count: i64,
    pub won_amount_cents: i64,
    pub win_rate: Option<u8>,
    pub conversions: i64,
}

pub fn window_start(now: DateTime<Utc>) -> DateTime<Utc> {
    let midnight = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("midnight is a valid time")
        .and_utc();
    midnight
        .checked_sub_days(Days::new(SUMMARY_WINDOW_DAYS))
        .expect("ninety days back is in range")
}

fn total_of(totals: &[StageTotal], stage: DealStageName) -> StageTotal {
    totals
        .iter()
        .find(|total| total.stage == stage)
        .copied()
        .unwrap_or(StageTotal {
            stage,
            count: 0,
            amount_cents: 0,
        })
}

pub fn win_rate(won: i64, lost: i64) -> Option<u8> {
    let decided = won + lost;
    if decided == 0 {
        return None;
    }
    let rate = (won * 100 + decided / 2) / decided;
    Some(rate as u8)
}

pub fn summarise(
    by_stage: &[StageTotal],
    closed_in_window: &[StageTotal],
    conversions: i64,
) -> PipelineSummary {
    let stages: Vec<StageTotal> = DealStageName::ALL
        .into_iter()
        .map(|stage| total_of(by_stage, stage))
        .collect();
    let open = stages.iter().filter(|total| total.stage.is_open());
    let (open_count, open_amount_cents) = open.fold((0, 0), |(count, amount), total| {
        (count + total.count, amount + total.amount_cents)
    });
    let won = total_of(closed_in_window, DealStageName::Won);
    let lost = total_of(closed_in_window, DealStageName::Lost);

    PipelineSummary {
        open_count,
        open_amount_cents,
        stages,
        won_count: won.count,
        lost_count: lost.count,
        won_amount_cents: won.amount_cents,
        win_rate: win_rate(won.count, lost.count),
        conversions,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn total(stage: DealStageName, count: i64, amount_cents: i64) -> StageTotal {
        StageTotal {
            stage,
            count,
            amount_cents,
        }
    }

    #[test]
    fn the_rate_is_null_when_nothing_was_decided() {
        assert_eq!(win_rate(0, 0), None);
    }

    #[test]
    fn the_rate_rounds_half_up() {
        assert_eq!(win_rate(1, 2), Some(33));
        assert_eq!(win_rate(2, 1), Some(67));
        assert_eq!(win_rate(1, 8), Some(11));
        assert_eq!(win_rate(1, 7), Some(13));
        assert_eq!(win_rate(3, 0), Some(100));
        assert_eq!(win_rate(0, 4), Some(0));
    }

    #[test]
    fn open_totals_cover_the_four_open_stages_only() {
        let summary = summarise(
            &[
                total(DealStageName::New, 2, 100),
                total(DealStageName::Negotiation, 1, 50),
                total(DealStageName::Won, 9, 9_000),
            ],
            &[],
            0,
        );

        assert_eq!(summary.open_count, 3);
        assert_eq!(summary.open_amount_cents, 150);
        assert_eq!(summary.stages.len(), 6);
        assert_eq!(summary.stages[1], total(DealStageName::Visit, 0, 0));
    }

    #[test]
    fn won_and_lost_come_from_the_window() {
        let summary = summarise(
            &[],
            &[
                total(DealStageName::Won, 3, 3_000),
                total(DealStageName::Lost, 1, 400),
            ],
            2,
        );

        assert_eq!(summary.won_count, 3);
        assert_eq!(summary.lost_count, 1);
        assert_eq!(summary.won_amount_cents, 3_000);
        assert_eq!(summary.win_rate, Some(75));
        assert_eq!(summary.conversions, 2);
    }

    #[test]
    fn the_window_starts_ninety_days_before_midnight() {
        let now = chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 11, 7, 15, 30, 0).unwrap();

        let expected = chrono::TimeZone::with_ymd_and_hms(&Utc, 2026, 8, 9, 0, 0, 0).unwrap();
        assert_eq!(window_start(now), expected);
    }
}
