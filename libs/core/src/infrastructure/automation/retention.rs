//! The periodic loop that turns `application::automation::retention::run_retention_pass`
//! into a background job — the retention analogue of
//! `infrastructure::automation::worker::run_automation_worker`.
//!
//! Deliberately its own loop rather than a tick tacked onto the run worker's:
//! retention is counted in days, so ticking it on the run worker's five-second
//! cadence would mean 17,000 no-op passes for every one that finds anything to
//! purge.

use std::time::Duration;

use tracing::{error, info};

use crate::application::{MestierUseCase, automation::retention::RetentionOutcome};

pub use crate::application::automation::retention::RetentionSchedule;

/// Pacing for the retention loop: how often it ticks, layered on top of
/// [`RetentionSchedule`]'s per-pass batch bounds.
#[derive(Debug, Clone, Copy)]
pub struct RetentionWorkerSchedule {
    pub interval: Duration,
    pub pass: RetentionSchedule,
}

impl Default for RetentionWorkerSchedule {
    fn default() -> Self {
        Self {
            // Retention is counted in days (#203): running more than once an
            // hour buys nothing, and an hour is already generous against
            // that scale.
            interval: Duration::from_secs(3600),
            pass: RetentionSchedule::default(),
        }
    }
}

/// Runs the retention loop until the process ends. Mirrors
/// `run_automation_worker`'s discipline: nothing here propagates an error, a
/// failing pass must not take the process down, and it says so loudly instead.
pub async fn run_retention_worker(usecase: MestierUseCase, schedule: RetentionWorkerSchedule) {
    info!("automation retention worker started");
    let mut ticker = tokio::time::interval(schedule.interval);

    loop {
        ticker.tick().await;

        match usecase.run_retention_pass(schedule.pass).await {
            Ok(RetentionOutcome {
                events_purged,
                succeeded_runs_purged,
            }) if events_purged > 0 || succeeded_runs_purged > 0 => {
                info!(events_purged, succeeded_runs_purged, "retention pass");
            }
            Ok(_) => {}
            Err(error) => error!(%error, "the retention pass failed"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_schedule_ticks_at_most_once_an_hour() {
        let schedule = RetentionWorkerSchedule::default();

        assert!(
            schedule.interval >= Duration::from_secs(3600),
            "retention is counted in days; ticking more than hourly buys nothing"
        );
        assert!(schedule.pass.event_batch > 0);
        assert!(schedule.pass.run_batch > 0);
    }
}
