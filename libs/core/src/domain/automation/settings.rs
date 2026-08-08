use std::time::Duration;

use common::CoreError;

/// What an organization may change about how its events are delivered and how
/// long they are kept.
///
/// Every value is validated against [`SettingsBounds`] in the domain, never
/// only in the form: an organization that could set a one-millisecond retry
/// would be denial-of-servicing the third party it calls, and the database it
/// runs on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationSettings {
    pub event_retention: Duration,
    pub succeeded_delivery_retention: Duration,
    /// How long to wait before each retry. **The number of attempts is the
    /// length of this list** — one entry fewer, one attempt fewer. A single
    /// concept, and no way to configure the two into disagreeing.
    pub retry_schedule: Vec<Duration>,
    /// Consecutive failures after which a target is disabled. `None` never
    /// disables — legitimate for a self-hosted instance calling its own LAN.
    pub disable_target_after: Option<u32>,
}

impl Default for AutomationSettings {
    fn default() -> Self {
        Self {
            event_retention: Duration::from_secs(90 * 24 * 3600),
            succeeded_delivery_retention: Duration::from_secs(30 * 24 * 3600),
            retry_schedule: vec![
                Duration::from_secs(5),
                Duration::from_secs(30),
                Duration::from_secs(2 * 60),
                Duration::from_secs(10 * 60),
                Duration::from_secs(3600),
                Duration::from_secs(6 * 3600),
            ],
            disable_target_after: Some(20),
        }
    }
}

/// The limits an organization cannot cross. These belong to the deployment,
/// not the product: self-hosted, the artisan decides on their own machine;
/// hosted, the operator keeps the ceiling. Same code, two configurations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingsBounds {
    pub event_retention: (Duration, Duration),
    pub succeeded_delivery_retention: (Duration, Duration),
    pub retry_interval: (Duration, Duration),
    pub max_retry_entries: usize,
    pub disable_target_after: (u32, u32),
}

impl Default for SettingsBounds {
    fn default() -> Self {
        Self {
            event_retention: (
                Duration::from_secs(24 * 3600),
                Duration::from_secs(730 * 24 * 3600),
            ),
            succeeded_delivery_retention: (
                Duration::from_secs(24 * 3600),
                Duration::from_secs(90 * 24 * 3600),
            ),
            retry_interval: (Duration::from_secs(1), Duration::from_secs(24 * 3600)),
            max_retry_entries: 10,
            disable_target_after: (1, 1000),
        }
    }
}

impl AutomationSettings {
    pub fn validate(&self, bounds: &SettingsBounds) -> Result<(), CoreError> {
        check_range(
            "event retention",
            self.event_retention,
            bounds.event_retention,
        )?;
        check_range(
            "succeeded delivery retention",
            self.succeeded_delivery_retention,
            bounds.succeeded_delivery_retention,
        )?;

        if self.retry_schedule.is_empty() {
            return Err(CoreError::Conflict(
                "the retry schedule needs at least one entry, or nothing is ever retried"
                    .to_owned(),
            ));
        }
        if self.retry_schedule.len() > bounds.max_retry_entries {
            return Err(CoreError::Conflict(format!(
                "the retry schedule accepts at most {} entries",
                bounds.max_retry_entries
            )));
        }
        for interval in &self.retry_schedule {
            check_range("retry interval", *interval, bounds.retry_interval)?;
        }

        if let Some(threshold) = self.disable_target_after {
            let (min, max) = bounds.disable_target_after;
            if threshold < min || threshold > max {
                return Err(CoreError::Conflict(format!(
                    "the auto-disable threshold must be between {min} and {max}"
                )));
            }
        }

        Ok(())
    }

    /// When the next attempt is due after `attempts` failures, or `None` when
    /// the schedule is exhausted — which is what makes a delivery dead.
    pub fn backoff_after(&self, attempts: u32) -> Option<Duration> {
        self.retry_schedule.get(attempts as usize).copied()
    }

    pub fn max_attempts(&self) -> u32 {
        self.retry_schedule.len() as u32
    }
}

fn check_range(
    what: &str,
    value: Duration,
    (min, max): (Duration, Duration),
) -> Result<(), CoreError> {
    if value < min || value > max {
        return Err(CoreError::Conflict(format!(
            "{what} must be between {}s and {}s",
            min.as_secs(),
            max.as_secs()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_satisfy_the_default_bounds() {
        AutomationSettings::default()
            .validate(&SettingsBounds::default())
            .expect("shipping defaults that the shipping bounds reject would be a bug");
    }

    #[test]
    fn a_retry_below_the_instance_floor_is_refused() {
        let settings = AutomationSettings {
            retry_schedule: vec![Duration::from_millis(100)],
            ..AutomationSettings::default()
        };

        assert!(settings.validate(&SettingsBounds::default()).is_err());
    }

    #[test]
    fn a_retention_above_the_instance_ceiling_is_refused() {
        let settings = AutomationSettings {
            event_retention: Duration::from_secs(1000 * 24 * 3600),
            ..AutomationSettings::default()
        };

        assert!(settings.validate(&SettingsBounds::default()).is_err());
    }

    /// A schedule with no entry is not "no retries", it is a delivery that
    /// fails once and is never looked at again — silently.
    #[test]
    fn an_empty_retry_schedule_is_refused() {
        let settings = AutomationSettings {
            retry_schedule: vec![],
            ..AutomationSettings::default()
        };

        assert!(settings.validate(&SettingsBounds::default()).is_err());
    }

    #[test]
    fn a_schedule_longer_than_the_instance_allows_is_refused() {
        let settings = AutomationSettings {
            retry_schedule: vec![Duration::from_secs(60); 11],
            ..AutomationSettings::default()
        };

        assert!(settings.validate(&SettingsBounds::default()).is_err());
    }

    #[test]
    fn never_disabling_a_target_is_allowed() {
        let settings = AutomationSettings {
            disable_target_after: None,
            ..AutomationSettings::default()
        };

        assert!(settings.validate(&SettingsBounds::default()).is_ok());
    }

    /// The attempt count is not a separate setting: it is the length of the
    /// schedule, so the two cannot disagree.
    #[test]
    fn attempts_follow_the_schedule_and_run_out_with_it() {
        let settings = AutomationSettings {
            retry_schedule: vec![Duration::from_secs(5), Duration::from_secs(30)],
            ..AutomationSettings::default()
        };

        assert_eq!(settings.max_attempts(), 2);
        assert_eq!(settings.backoff_after(0), Some(Duration::from_secs(5)));
        assert_eq!(settings.backoff_after(1), Some(Duration::from_secs(30)));
        assert_eq!(
            settings.backoff_after(2),
            None,
            "an exhausted schedule is what makes a delivery dead"
        );
    }
}
