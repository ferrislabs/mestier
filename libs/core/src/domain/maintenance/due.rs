//! When a line must next be done. Pure: `today` is a parameter, never the
//! clock, so week and year boundaries are tested without one.

use chrono::{Days, NaiveDate, Weekday};
use serde::Serialize;
use utoipa::ToSchema;

use crate::domain::maintenance::{Cadence, IsoWeekNumber, Season};

/// Ordered by priority: the first that applies wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DueStatus {
    /// An open task (backlog, planned or in progress) already covers it.
    Planned,
    /// Due before the Monday of the current week.
    Overdue,
    ThisWeek,
    NextWeek,
    Later,
    /// Season over, every window done, or on demand.
    Done,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
pub struct Due {
    /// `None` when `status` is `Done`.
    pub on: Option<NaiveDate>,
    pub status: DueStatus,
}

/// What `due_for` needs to know about one line.
#[derive(Debug, Clone, Copy)]
pub struct LineHistory<'a> {
    /// Dates of the line's validated interventions, any order.
    pub validated_on: &'a [NaiveDate],
    /// Whether a non-cancelled, not-done task carries this line.
    pub has_open_task: bool,
}

/// Rules, in `docs/chantiers/entretien.md`, section "Calcul de l'échéance":
/// - `Rolling`: last validated + `every_days`; without one, the season's
///   first day.
/// - `Windows`: Monday of the first window holding no validated
///   intervention; overdue once that window has ended.
/// - `OnDemand`: always `Done`.
/// - Nothing is due after `season.last_day()`.
pub fn due_for(
    season: Season,
    cadence: &Cadence,
    history: LineHistory<'_>,
    today: NaiveDate,
) -> Due {
    let Some(next) = next_due(season, cadence, history.validated_on) else {
        return DONE;
    };
    if next.on > season.last_day() || today > season.last_day() {
        return DONE;
    }
    let status = if history.has_open_task {
        DueStatus::Planned
    } else {
        status_of(next, today)
    };
    Due {
        on: Some(next.on),
        status,
    }
}

const DONE: Due = Due {
    on: None,
    status: DueStatus::Done,
};

#[derive(Clone, Copy)]
struct NextDue {
    on: NaiveDate,
    /// Last day of the window the date opens; `None` for a rolling cadence.
    window_end: Option<NaiveDate>,
}

fn next_due(season: Season, cadence: &Cadence, validated_on: &[NaiveDate]) -> Option<NextDue> {
    match cadence {
        Cadence::OnDemand => None,
        Cadence::Rolling { every_days } => {
            let on = match validated_on.iter().max() {
                Some(last) => last
                    .checked_add_days(Days::new(u64::from(every_days.get())))?
                    .max(season.first_day()),
                None => season.first_day(),
            };
            Some(NextDue {
                on,
                window_end: None,
            })
        }
        Cadence::Windows { windows } => windows.as_slice().iter().find_map(|window| {
            let start = season.monday_of(window.from_week());
            let end = sunday_of(season, window.to_week());
            let visited = validated_on.iter().any(|day| (start..=end).contains(day));
            (!visited).then_some(NextDue {
                on: start,
                window_end: Some(end),
            })
        }),
    }
}

fn sunday_of(season: Season, week: IsoWeekNumber) -> NaiveDate {
    NaiveDate::from_isoywd_opt(season.year(), u32::from(week.get()), Weekday::Sun)
        .unwrap_or_else(|| season.last_day())
}

fn status_of(next: NextDue, today: NaiveDate) -> DueStatus {
    if next
        .window_end
        .is_some_and(|end| next.on <= today && today <= end)
    {
        return DueStatus::ThisWeek;
    }
    let this_monday = today.week(Weekday::Mon).first_day();
    match (next.on - this_monday).num_days() {
        ..0 => DueStatus::Overdue,
        0..7 => DueStatus::ThisWeek,
        7..14 => DueStatus::NextWeek,
        _ => DueStatus::Later,
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;
    use crate::domain::maintenance::{IntervalDays, IsoWeekNumber, Window, Windows};

    fn week(n: u8) -> IsoWeekNumber {
        IsoWeekNumber::try_from(n).unwrap()
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn season_2026() -> Season {
        Season::new(2026, week(12), week(48)).unwrap()
    }

    fn rolling(days: u16) -> Cadence {
        Cadence::Rolling {
            every_days: IntervalDays::try_from(days).unwrap(),
        }
    }

    fn windows(spans: &[(u8, u8)]) -> Cadence {
        let windows = spans
            .iter()
            .map(|&(from, to)| Window::new(week(from), week(to)).unwrap())
            .collect::<Vec<_>>();
        Cadence::Windows {
            windows: Windows::try_from(windows).unwrap(),
        }
    }

    fn history(validated_on: &[NaiveDate]) -> LineHistory<'_> {
        LineHistory {
            validated_on,
            has_open_task: false,
        }
    }

    fn due(on: NaiveDate, status: DueStatus) -> Due {
        Due {
            on: Some(on),
            status,
        }
    }

    #[test]
    fn spec_ent_1_a_rolling_line_is_due_the_interval_after_the_last_validated_visit() {
        let visits = [date(2026, 10, 10)];

        let result = due_for(
            season_2026(),
            &rolling(14),
            history(&visits),
            date(2026, 10, 12),
        );

        assert_eq!(result, due(date(2026, 10, 24), DueStatus::NextWeek));
    }

    #[test]
    fn spec_ent_1_the_last_visit_counts_whatever_the_order_of_the_list() {
        let visits = [date(2026, 10, 10), date(2026, 9, 1), date(2026, 9, 26)];

        let result = due_for(
            season_2026(),
            &rolling(14),
            history(&visits),
            date(2026, 10, 12),
        );

        assert_eq!(result.on, Some(date(2026, 10, 24)));
    }

    #[test]
    fn spec_ent_2_the_history_only_holds_validated_visits_so_a_proposal_changes_nothing() {
        let visits = [date(2026, 10, 10)];
        let before = due_for(
            season_2026(),
            &rolling(14),
            history(&visits),
            date(2026, 10, 12),
        );

        let after = due_for(
            season_2026(),
            &rolling(14),
            history(&visits),
            date(2026, 10, 21),
        );

        assert_eq!(before.on, Some(date(2026, 10, 24)));
        assert_eq!(after.on, before.on);
    }

    #[test]
    fn a_rolling_line_without_a_visit_is_due_on_the_first_day_of_the_season() {
        let result = due_for(season_2026(), &rolling(14), history(&[]), date(2026, 3, 10));

        assert_eq!(result, due(date(2026, 3, 16), DueStatus::NextWeek));
    }

    #[test]
    fn a_rolling_line_never_started_is_overdue_once_the_season_is_under_way() {
        let result = due_for(season_2026(), &rolling(14), history(&[]), date(2026, 4, 1));

        assert_eq!(result, due(date(2026, 3, 16), DueStatus::Overdue));
    }

    #[test]
    fn a_rolling_due_date_in_the_past_is_overdue() {
        let visits = [date(2026, 9, 1)];

        let result = due_for(
            season_2026(),
            &rolling(14),
            history(&visits),
            date(2026, 10, 12),
        );

        assert_eq!(result, due(date(2026, 9, 15), DueStatus::Overdue));
    }

    #[test]
    fn status_follows_the_iso_week_of_today() {
        let visits = [date(2026, 10, 4)];
        let monday = date(2026, 10, 12);
        let cases = [
            (date(2026, 10, 11), DueStatus::Overdue),
            (date(2026, 10, 12), DueStatus::ThisWeek),
            (date(2026, 10, 18), DueStatus::ThisWeek),
            (date(2026, 10, 19), DueStatus::NextWeek),
            (date(2026, 10, 25), DueStatus::NextWeek),
            (date(2026, 10, 26), DueStatus::Later),
        ];

        for (on, expected) in cases {
            let days = (on - visits[0]).num_days() as u16;
            let result = due_for(season_2026(), &rolling(days), history(&visits), monday);
            assert_eq!(result, due(on, expected), "due on {on}");
        }
    }

    #[test]
    fn spec_ent_3_the_first_window_without_a_validated_visit_gives_the_due_date() {
        let cadence = windows(&[(14, 17), (27, 30), (44, 47)]);
        let visits = [date(2026, 4, 8), date(2026, 7, 8)];

        let result = due_for(season_2026(), &cadence, history(&visits), date(2026, 7, 29));

        assert_eq!(result, due(date(2026, 10, 26), DueStatus::Later));
    }

    #[test]
    fn spec_ent_3_a_window_that_ended_without_a_visit_is_overdue() {
        let cadence = windows(&[(14, 17), (27, 30), (44, 47)]);
        let visits = [date(2026, 4, 8), date(2026, 7, 8)];

        let result = due_for(
            season_2026(),
            &cadence,
            history(&visits),
            date(2026, 11, 23),
        );

        assert_eq!(result, due(date(2026, 10, 26), DueStatus::Overdue));
    }

    #[test]
    fn an_open_window_is_this_week_not_overdue() {
        let cadence = windows(&[(44, 47)]);

        let first = due_for(season_2026(), &cadence, history(&[]), date(2026, 10, 26));
        let middle = due_for(season_2026(), &cadence, history(&[]), date(2026, 11, 4));
        let last = due_for(season_2026(), &cadence, history(&[]), date(2026, 11, 22));

        assert_eq!(first.status, DueStatus::ThisWeek);
        assert_eq!(middle, due(date(2026, 10, 26), DueStatus::ThisWeek));
        assert_eq!(last.status, DueStatus::ThisWeek);
    }

    #[test]
    fn a_window_starting_next_week_is_next_week() {
        let cadence = windows(&[(44, 47)]);

        let result = due_for(season_2026(), &cadence, history(&[]), date(2026, 10, 19));

        assert_eq!(result, due(date(2026, 10, 26), DueStatus::NextWeek));
    }

    #[test]
    fn a_missed_window_stays_overdue_when_a_later_one_is_done() {
        let cadence = windows(&[(14, 17), (27, 30)]);
        let visits = [date(2026, 7, 8)];

        let result = due_for(season_2026(), &cadence, history(&visits), date(2026, 8, 3));

        assert_eq!(result, due(date(2026, 3, 30), DueStatus::Overdue));
    }

    #[test]
    fn a_visit_on_the_edges_of_a_window_counts_for_it() {
        let cadence = windows(&[(14, 17), (27, 30)]);
        let visits = [date(2026, 3, 30), date(2026, 7, 26)];

        let result = due_for(season_2026(), &cadence, history(&visits), date(2026, 8, 3));

        assert_eq!(result, DONE);
    }

    #[test]
    fn every_window_done_means_done() {
        let cadence = windows(&[(14, 17), (27, 30)]);
        let visits = [date(2026, 4, 8), date(2026, 7, 8)];

        let result = due_for(season_2026(), &cadence, history(&visits), date(2026, 8, 3));

        assert_eq!(result, DONE);
    }

    #[test]
    fn spec_ent_4_nothing_is_due_after_the_last_day_of_the_season() {
        let visits = [date(2026, 11, 18)];

        let result = due_for(
            season_2026(),
            &rolling(14),
            history(&visits),
            date(2026, 11, 19),
        );

        assert_eq!(result, DONE);
    }

    #[test]
    fn a_due_date_on_the_last_day_of_the_season_is_still_due() {
        let visits = [date(2026, 11, 15)];

        let result = due_for(
            season_2026(),
            &rolling(14),
            history(&visits),
            date(2026, 11, 23),
        );

        assert_eq!(result, due(date(2026, 11, 29), DueStatus::ThisWeek));
    }

    #[test]
    fn once_the_season_is_over_a_missed_line_is_done_not_overdue() {
        let rolling_missed = due_for(
            season_2026(),
            &rolling(14),
            history(&[]),
            date(2026, 11, 30),
        );
        let window_missed = due_for(
            season_2026(),
            &windows(&[(44, 47)]),
            history(&[]),
            date(2026, 11, 30),
        );

        assert_eq!(rolling_missed, DONE);
        assert_eq!(window_missed, DONE);
    }

    #[test]
    fn spec_ent_14_an_on_demand_line_is_always_done() {
        let visits = [date(2026, 10, 10)];

        assert_eq!(
            due_for(
                season_2026(),
                &Cadence::OnDemand,
                history(&[]),
                date(2026, 10, 12)
            ),
            DONE
        );
        assert_eq!(
            due_for(
                season_2026(),
                &Cadence::OnDemand,
                history(&visits),
                date(2026, 10, 12)
            ),
            DONE
        );
    }

    #[test]
    fn an_open_task_makes_a_due_line_planned() {
        let visits = [date(2026, 10, 10)];
        let carried = LineHistory {
            validated_on: &visits,
            has_open_task: true,
        };

        let result = due_for(season_2026(), &rolling(14), carried, date(2026, 10, 12));

        assert_eq!(result, due(date(2026, 10, 24), DueStatus::Planned));
    }

    #[test]
    fn an_open_task_does_not_make_a_line_without_a_due_date_planned() {
        let carried = LineHistory {
            validated_on: &[],
            has_open_task: true,
        };

        let on_demand = due_for(
            season_2026(),
            &Cadence::OnDemand,
            carried,
            date(2026, 10, 12),
        );
        let over = due_for(season_2026(), &rolling(14), carried, date(2026, 12, 1));

        assert_eq!(on_demand, DONE);
        assert_eq!(over, DONE);
    }

    #[test]
    fn a_due_date_on_a_sunday_belongs_to_the_week_that_ends_with_it() {
        let visits = [date(2026, 10, 4)];

        let on_sunday = due_for(
            season_2026(),
            &rolling(14),
            history(&visits),
            date(2026, 10, 12),
        );
        let after = due_for(
            season_2026(),
            &rolling(14),
            history(&visits),
            date(2026, 10, 19),
        );

        assert_eq!(on_sunday, due(date(2026, 10, 18), DueStatus::ThisWeek));
        assert_eq!(after, due(date(2026, 10, 18), DueStatus::Overdue));
    }

    #[test]
    fn week_fifty_three_runs_into_the_new_year() {
        let season = Season::new(2026, week(40), week(53)).unwrap();
        let visits = [date(2026, 12, 20)];

        let in_week_53 = due_for(season, &rolling(14), history(&visits), date(2026, 12, 28));
        let in_week_1 = due_for(season, &rolling(14), history(&visits), date(2027, 1, 4));

        assert_eq!(in_week_53, due(date(2027, 1, 3), DueStatus::ThisWeek));
        assert_eq!(in_week_1, DONE);
    }

    #[test]
    fn a_due_date_across_new_year_past_the_season_is_done() {
        let season = Season::new(2026, week(40), week(53)).unwrap();
        let visits = [date(2026, 12, 21)];

        let result = due_for(season, &rolling(14), history(&visits), date(2026, 12, 28));

        assert_eq!(result, DONE);
    }

    #[test]
    fn a_window_in_week_fifty_three_ends_on_the_sunday_after_new_year() {
        let season = Season::new(2026, week(40), week(53)).unwrap();
        let cadence = windows(&[(52, 53)]);

        let inside = due_for(season, &cadence, history(&[]), date(2027, 1, 3));

        assert_eq!(inside, due(date(2026, 12, 21), DueStatus::ThisWeek));
    }

    #[test]
    fn a_window_that_ended_before_new_year_is_overdue_in_week_fifty_two() {
        let season = Season::new(2026, week(40), week(53)).unwrap();
        let cadence = windows(&[(50, 51)]);

        let result = due_for(season, &cadence, history(&[]), date(2026, 12, 21));

        assert_eq!(result, due(date(2026, 12, 7), DueStatus::Overdue));
    }

    fn rank(due: Due) -> Option<NaiveDate> {
        due.on
    }

    proptest! {
        #[test]
        fn adding_a_validated_visit_never_moves_a_rolling_due_date_earlier(
            offsets in prop::collection::vec(0i64..258, 0..8),
            extra in 0i64..258,
            every in 1u16..=365,
            today_offset in 0i64..400,
        ) {
            let season = season_2026();
            let day = |offset: i64| season.first_day() + chrono::Duration::days(offset);
            let mut visits = offsets.iter().map(|&o| day(o)).collect::<Vec<_>>();
            let today = day(today_offset);
            let cadence = rolling(every);

            let before = due_for(season, &cadence, history(&visits), today);
            visits.push(day(extra));
            let after = due_for(season, &cadence, history(&visits), today);

            match (rank(before), rank(after)) {
                (Some(b), Some(a)) => prop_assert!(a >= b, "{b} then {a}"),
                (None, Some(a)) => prop_assert!(false, "done became due on {a}"),
                _ => {}
            }
        }
    }
}
