use chrono::{DateTime, Duration, SecondsFormat, Utc};
use serde_json::{Map, Value, json};

use crate::domain::automation::run::{Connector, ConnectorInput, ConnectorOutcome};

pub const MAX_WAIT_DAYS: u64 = 365;

pub struct FlowWaitConnector;

fn seconds_per(unit: &str) -> Option<u64> {
    match unit {
        "minutes" => Some(60),
        "hours" => Some(3_600),
        "days" => Some(86_400),
        _ => None,
    }
}

fn whole_number(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.trim().parse().ok(),
        _ => None,
    }
}

fn failed(error: &str) -> ConnectorOutcome {
    ConnectorOutcome::Failed {
        error: error.to_owned(),
    }
}

fn decide(config: &Map<String, Value>, now: DateTime<Utc>) -> ConnectorOutcome {
    let Some(duration) = config
        .get("duration")
        .and_then(whole_number)
        .filter(|duration| *duration > 0)
    else {
        return failed("`duration` must be a whole number of at least 1");
    };
    let Some(unit) = config.get("unit").and_then(Value::as_str) else {
        return failed("`unit` must be minutes, hours or days");
    };
    let Some(per_unit) = seconds_per(unit) else {
        return failed("`unit` must be minutes, hours or days");
    };
    let too_long = format!("a wait lasts at most {MAX_WAIT_DAYS} days");
    let Some(seconds) = duration
        .checked_mul(per_unit)
        .filter(|seconds| *seconds <= MAX_WAIT_DAYS * 86_400)
        .and_then(|seconds| i64::try_from(seconds).ok())
    else {
        return failed(&too_long);
    };

    let resume_at = now + Duration::seconds(seconds);
    ConnectorOutcome::Wait {
        resume_at,
        output: json!({
            "duration": duration,
            "unit": unit,
            "resume_at": resume_at.to_rfc3339_opts(SecondsFormat::Secs, true),
        }),
    }
}

impl Connector for FlowWaitConnector {
    async fn execute(&self, input: ConnectorInput<'_>) -> ConnectorOutcome {
        decide(input.config, Utc::now())
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use common::OrganizationId;
    use uuid::Uuid;

    use super::*;
    use crate::domain::automation::connector::connector_catalogue;
    use crate::domain::automation::run::DryRunInput;

    fn config(duration: Value, unit: &str) -> Map<String, Value> {
        let mut config = Map::new();
        config.insert("duration".to_owned(), duration);
        config.insert("unit".to_owned(), json!(unit));
        config
    }

    fn noon() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 6, 12, 0, 0).unwrap()
    }

    fn parked(outcome: ConnectorOutcome) -> (DateTime<Utc>, Value) {
        match outcome {
            ConnectorOutcome::Wait { resume_at, output } => (resume_at, output),
            other => panic!("expected a wait, got {other:?}"),
        }
    }

    fn failure(outcome: ConnectorOutcome) -> String {
        match outcome {
            ConnectorOutcome::Failed { error } => error,
            other => panic!("expected a failure, got {other:?}"),
        }
    }

    #[test]
    fn each_unit_parks_the_step_that_long_from_now() {
        for (unit, expected) in [
            ("minutes", Duration::minutes(5)),
            ("hours", Duration::hours(5)),
            ("days", Duration::days(5)),
        ] {
            let (resume_at, output) = parked(decide(&config(json!(5), unit), noon()));

            assert_eq!(resume_at, noon() + expected, "{unit}");
            assert_eq!(output["unit"], unit);
            assert_eq!(output["duration"], 5);
        }
    }

    #[test]
    fn the_output_names_the_resume_time_the_step_is_parked_until() {
        let (resume_at, output) = parked(decide(&config(json!(2), "days"), noon()));

        assert_eq!(
            resume_at,
            Utc.with_ymd_and_hms(2026, 11, 8, 12, 0, 0).unwrap()
        );
        assert_eq!(output["resume_at"], "2026-11-08T12:00:00Z");
    }

    #[test]
    fn a_duration_resolved_to_a_numeric_string_is_accepted() {
        let (resume_at, _) = parked(decide(&config(json!(" 3 "), "hours"), noon()));

        assert_eq!(resume_at, noon() + Duration::hours(3));
    }

    #[test]
    fn a_duration_that_is_not_a_positive_whole_number_fails_naming_the_field() {
        for duration in [
            json!(0),
            json!(-1),
            json!(1.5),
            json!("soon"),
            json!(null),
            json!([2]),
        ] {
            let error = failure(decide(&config(duration.clone(), "days"), noon()));

            assert_eq!(
                error, "`duration` must be a whole number of at least 1",
                "{duration}"
            );
        }
    }

    #[test]
    fn a_missing_duration_fails_naming_the_field() {
        let mut config = config(json!(1), "days");
        config.remove("duration");

        assert_eq!(
            failure(decide(&config, noon())),
            "`duration` must be a whole number of at least 1"
        );
    }

    #[test]
    fn an_unknown_or_missing_unit_fails_naming_the_field() {
        for unit in [json!("weeks"), json!(3), json!(null)] {
            let mut config = config(json!(1), "days");
            config.insert("unit".to_owned(), unit);

            assert_eq!(
                failure(decide(&config, noon())),
                "`unit` must be minutes, hours or days"
            );
        }
    }

    #[test]
    fn a_wait_longer_than_a_year_is_refused() {
        assert!(matches!(
            decide(&config(json!(365), "days"), noon()),
            ConnectorOutcome::Wait { .. }
        ));
        for (duration, unit) in [(366, "days"), (8_761, "hours"), (525_601, "minutes")] {
            assert_eq!(
                failure(decide(&config(json!(duration), unit), noon())),
                "a wait lasts at most 365 days",
                "{duration} {unit}"
            );
        }
        assert_eq!(
            failure(decide(&config(json!(u64::MAX), "days"), noon())),
            "a wait lasts at most 365 days"
        );
    }

    #[tokio::test]
    async fn executing_parks_the_step_in_the_future() {
        let config = config(json!(10), "minutes");
        let before = Utc::now();

        let outcome = FlowWaitConnector
            .execute(ConnectorInput {
                org_id: OrganizationId(Uuid::from_u128(1)),
                run_id: Uuid::from_u128(2),
                config: &config,
                credential_id: None,
                operation: None,
            })
            .await;

        let (resume_at, _) = parked(outcome);
        assert!(resume_at >= before + Duration::minutes(10));
        assert!(resume_at <= Utc::now() + Duration::minutes(10));
    }

    #[test]
    fn the_dry_run_answers_at_once_with_the_sample_and_parks_nothing() {
        let catalogue = connector_catalogue();
        let descriptor = catalogue.get("flow.wait", 1).unwrap();
        let config = config(json!(2), "days");

        let outcome = FlowWaitConnector::dry_run(
            descriptor,
            &DryRunInput {
                config: &config,
                stored: None,
            },
        );

        assert_eq!(
            outcome,
            ConnectorOutcome::Produced(descriptor.sample_output.clone())
        );
    }

    #[test]
    fn the_sample_has_the_shape_of_what_a_wait_outputs() {
        let (_, output) = parked(decide(&config(json!(2), "days"), noon()));

        super::super::test_support::assert_same_shape(&output, "flow.wait");
    }
}
