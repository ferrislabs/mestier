//! Settings and the retention purge (#203): reading and writing an
//! organization's `AutomationSettings`, and turning `automation.event` and
//! `automation.run` retention into deleted rows.
//!
//! `AutomationSettings` already carried `event_retention` and
//! `succeeded_run_retention`, validated against `SettingsBounds` — until
//! this module nothing ever applied them. `run_retention_pass` is what does:
//! one pass over every organization with automation data, each purged under
//! its own settings (falling back to the default the same way a plain read
//! already does), in bounded batches so a busy organization's months of log
//! never cost a business transaction a long-held lock.

use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use mestier_macros::transactional;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        ports::{AutomationSettingsRepository, EventLogRepository, RunRepository},
        settings::{AutomationSettings, SettingsBounds},
    },
};

/// What one retention pass purged, across every organization it looked at.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RetentionOutcome {
    pub events_purged: u64,
    pub succeeded_runs_purged: u64,
}

/// How the periodic retention loop bounds itself — the retention analogue of
/// `infrastructure::automation::worker::WorkerSchedule`. There is no
/// `interval` here: that belongs to
/// `infrastructure::automation::retention::run_retention_worker`, the only
/// caller that ticks.
#[derive(Debug, Clone, Copy)]
pub struct RetentionSchedule {
    pub event_batch: i64,
    pub run_batch: i64,
}

impl Default for RetentionSchedule {
    fn default() -> Self {
        Self {
            event_batch: 1000,
            run_batch: 1000,
        }
    }
}

/// `chrono` and `std` disagree on duration types — mirrors
/// `infrastructure::automation::worker::chrono_from`, and falls back the same
/// way: no `AutomationSettings::validate`-passing retention can overflow a
/// `chrono::Duration`; this is only ever reached if that invariant is
/// somehow broken, and a hundred-year fallback purges nothing rather than
/// panicking.
fn chrono_duration(duration: std::time::Duration) -> chrono::Duration {
    chrono::Duration::from_std(duration).unwrap_or_else(|_| chrono::Duration::days(36_500))
}

impl MestierUseCase {
    #[transactional(automation_settings)]
    pub async fn get_automation_settings(
        &self,
        org_id: OrganizationId,
    ) -> Result<AutomationSettings, CoreError> {
        let mut repo = automation_settings_repository;
        repo.settings_for(org_id).await
    }

    /// Validates against the instance's `SettingsBounds` before writing —
    /// never silently clamped into range, refused instead, naming the bound
    /// (`AutomationSettings::validate`'s own `CoreError::Conflict` message).
    #[transactional(automation_settings)]
    pub async fn update_automation_settings(
        &self,
        org_id: OrganizationId,
        settings: AutomationSettings,
    ) -> Result<AutomationSettings, CoreError> {
        settings.validate(&SettingsBounds::default())?;

        let mut repo = automation_settings_repository;
        repo.upsert(org_id, &settings).await
    }

    /// Purges one organization's events and succeeded runs in a single
    /// transaction — combined on purpose. `run_retention_pass` calls this
    /// once per organization it sweeps, and a separate transaction per
    /// table would double the round trips on every pass without buying
    /// anything: the two purges share no state and neither depends on the
    /// other's outcome, so nothing is lost by writing them together.
    #[transactional(event_log, run)]
    async fn purge_org(
        &self,
        org_id: OrganizationId,
        events_cutoff: DateTime<Utc>,
        runs_cutoff: DateTime<Utc>,
        schedule: RetentionSchedule,
    ) -> Result<RetentionOutcome, CoreError> {
        let mut events = event_log_repository;
        let mut runs = run_repository;

        let events_purged = events
            .purge_expired(org_id, events_cutoff, schedule.event_batch)
            .await?;
        let succeeded_runs_purged = runs
            .purge_succeeded(org_id, runs_cutoff, schedule.run_batch)
            .await?;

        Ok(RetentionOutcome {
            events_purged,
            succeeded_runs_purged,
        })
    }

    #[transactional(event_log)]
    async fn organizations_with_automation_data(&self) -> Result<Vec<OrganizationId>, CoreError> {
        let mut repo = event_log_repository;
        repo.organizations_with_automation_data().await
    }

    /// Every organization that has ever configured its own settings, in one
    /// query — `run_retention_pass` reads this once per pass instead of
    /// calling `get_automation_settings` once per organization it sweeps,
    /// which is what kept an instance-wide pass to O(organizations)
    /// transactions total rather than O(organizations) *per table*.
    #[transactional(automation_settings)]
    async fn all_automation_settings(
        &self,
    ) -> Result<std::collections::HashMap<OrganizationId, AutomationSettings>, CoreError> {
        let mut repo = automation_settings_repository;
        Ok(repo.all_settings().await?.into_iter().collect())
    }

    /// One retention pass: every organization with automation data, purged
    /// under its own settings — one combined transaction per organization
    /// ([`Self::purge_org`]), the settings for all of them read in a single
    /// upfront query ([`Self::all_automation_settings`]) rather than one
    /// query per organization. Periodic, and much less frequent than the run
    /// worker — see `infrastructure::automation::retention::run_retention_worker`,
    /// the only caller outside tests.
    pub async fn run_retention_pass(
        &self,
        schedule: RetentionSchedule,
    ) -> Result<RetentionOutcome, CoreError> {
        let org_ids = self.organizations_with_automation_data().await?;
        let configured_settings = self.all_automation_settings().await?;
        let now = Utc::now();
        let mut outcome = RetentionOutcome::default();

        for org_id in org_ids {
            let settings = configured_settings
                .get(&org_id)
                .cloned()
                .unwrap_or_default();
            let events_cutoff = now - chrono_duration(settings.event_retention);
            let runs_cutoff = now - chrono_duration(settings.succeeded_run_retention);

            let purged = self
                .purge_org(org_id, events_cutoff, runs_cutoff, schedule)
                .await?;
            outcome.events_purged += purged.events_purged;
            outcome.succeeded_runs_purged += purged.succeeded_runs_purged;
        }

        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::default_authorizer;
    use crate::infrastructure::realtime::EventHub;

    async fn make_pool() -> PgPool {
        let url = std::env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set to run retention use case integration tests");
        PgPool::connect(&url).await.unwrap()
    }

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    fn use_case(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    // --- settings ----------------------------------------------------------

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_fresh_organization_reads_the_default_settings() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "settings-default").await;
        let usecase = use_case(pool);

        let settings = usecase.get_automation_settings(org_id).await.unwrap();

        assert_eq!(settings, AutomationSettings::default());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_within_bounds_is_read_back() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "settings-update").await;
        let usecase = use_case(pool);

        let desired = AutomationSettings {
            event_retention: Duration::from_secs(10 * 24 * 3600),
            ..AutomationSettings::default()
        };
        let updated = usecase
            .update_automation_settings(org_id, desired.clone())
            .await
            .unwrap();
        assert_eq!(updated, desired);

        let reread = usecase.get_automation_settings(org_id).await.unwrap();
        assert_eq!(reread, desired);
    }

    /// The acceptance criterion (#203): a value outside the instance's
    /// bounds is refused — never silently clamped back into range — and the
    /// refusal names the bound it crossed.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_past_the_instance_ceiling_is_refused_and_names_the_bound() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "settings-out-of-bounds").await;
        let usecase = use_case(pool);

        let too_long = AutomationSettings {
            event_retention: Duration::from_secs(10_000 * 24 * 3600),
            ..AutomationSettings::default()
        };

        let error = usecase
            .update_automation_settings(org_id, too_long)
            .await
            .expect_err("an out-of-bounds retention must be refused");

        assert!(matches!(error, CoreError::Conflict(_)));
        assert!(
            format!("{error}").contains("event retention"),
            "the refusal must name which bound was crossed: {error}"
        );

        let unchanged = usecase.get_automation_settings(org_id).await.unwrap();
        assert_eq!(
            unchanged,
            AutomationSettings::default(),
            "a refused update must not have written anything"
        );
    }

    // --- retention pass ------------------------------------------------

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_retention_pass_purges_events_and_succeeded_runs_past_their_own_organizations_retention()
     {
        use crate::domain::automation::workflow::{CreateWorkflowCommand, Graph, PlacedConnector};
        use crate::infrastructure::automation::postgres::PgEventLogRepository;
        use crate::infrastructure::postgres::with_tx;
        use events::{Actor, DomainEvent, EmissionContext, EventEnvelope, EventSubject};
        use serde_json::json;

        struct Probe;
        impl DomainEvent for Probe {
            fn name(&self) -> &'static str {
                "retention.probe"
            }
            fn version(&self) -> u16 {
                1
            }
            fn subject(&self) -> EventSubject {
                EventSubject {
                    kind: "probe",
                    id: None,
                }
            }
            fn payload(&self) -> serde_json::Value {
                json!({})
            }
        }

        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "pass-org").await;
        let usecase = use_case(pool.clone());

        // A one-day retention on both axes, so "past retention" is cheap to
        // construct without waiting on the (90-day) default.
        usecase
            .update_automation_settings(
                org_id,
                AutomationSettings {
                    event_retention: Duration::from_secs(24 * 3600),
                    succeeded_run_retention: Duration::from_secs(24 * 3600),
                    ..AutomationSettings::default()
                },
            )
            .await
            .unwrap();

        let expired_event = EventEnvelope {
            occurred_at: Utc::now() - chrono::Duration::days(10),
            ..EventEnvelope::from_event(
                &Probe,
                &EmissionContext {
                    org_id,
                    actor: Actor::system(),
                    correlation_id: None,
                },
            )
        };
        with_tx(&pool, async |tx| {
            let mut repo = PgEventLogRepository::new(&tx);
            repo.append(std::slice::from_ref(&expired_event)).await
        })
        .await
        .unwrap();

        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Retention probe".to_string(),
                description: None,
            })
            .await
            .unwrap();
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), json!("{{ true }}"));
        usecase
            .save_workflow_version(
                crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id: workflow.id,
                    graph: Graph {
                        connectors: vec![PlacedConnector {
                            id: "c1".to_string(),
                            kind: "flow.condition".to_string(),
                            version: 1,
                            credential_id: None,
                            config,
                        }],
                        edges: Vec::new(),
                    },
                    created_by: None,
                },
            )
            .await
            .unwrap();
        let run_id = usecase
            .start_run(org_id, workflow.id, json!({}))
            .await
            .unwrap();
        sqlx::query!(
            r#"UPDATE automation.run
               SET status = 'succeeded', finished_at = $2
               WHERE id = $1"#,
            run_id,
            Utc::now() - chrono::Duration::days(10),
        )
        .execute(&pool)
        .await
        .unwrap();

        // `run_retention_pass` sweeps every organization that has automation
        // data, system-wide — not only this test's own. Under parallel test
        // execution another test's own concurrent call can legitimately
        // purge this test's rows first (or vice versa), so asserting a
        // lower bound on *this* call's returned counters is exactly the
        // cross-test contamination #210 rules out: it would pass or fail
        // depending on scheduling, not on this test's own behavior. The
        // outcome is not asserted on; only the rows this test created are
        // (`event_count`/`run_count` below), which is safe regardless of
        // which call actually did the deleting.
        usecase
            .run_retention_pass(RetentionSchedule::default())
            .await
            .unwrap();

        let event_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM automation.event WHERE id = $1",
            expired_event.id,
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(event_count, Some(0));

        let run_count =
            sqlx::query_scalar!("SELECT COUNT(*) FROM automation.run WHERE id = $1", run_id,)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(run_count, Some(0));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_run_within_its_retention_is_intact_even_when_a_neighboring_organization_has_a_shorter_retention()
     {
        use crate::domain::automation::workflow::{CreateWorkflowCommand, Graph, PlacedConnector};
        use crate::infrastructure::automation::postgres::PgRunRepository;
        use crate::infrastructure::postgres::with_tx;
        use serde_json::json;

        let pool: PgPool = make_pool().await;
        let short_org = seed_organization(&pool, "neighbor-short").await;
        let long_org = seed_organization(&pool, "neighbor-long").await;
        let usecase = use_case(pool.clone());

        usecase
            .update_automation_settings(
                short_org,
                AutomationSettings {
                    // The instance floor (one day) — the shortest a
                    // succeeded run's retention can legally be.
                    succeeded_run_retention: Duration::from_secs(24 * 3600),
                    ..AutomationSettings::default()
                },
            )
            .await
            .unwrap();
        // `long_org` keeps the (thirty-day) default.

        async fn finished_run_in(
            usecase: &MestierUseCase,
            pool: &PgPool,
            org_id: OrganizationId,
            finished_at: chrono::DateTime<Utc>,
        ) -> uuid::Uuid {
            let workflow = usecase
                .create_workflow(CreateWorkflowCommand {
                    org_id,
                    name: "Probe".to_string(),
                    description: None,
                })
                .await
                .unwrap();
            let mut config = serde_json::Map::new();
            config.insert("predicate".to_string(), json!("{{ true }}"));
            usecase
                .save_workflow_version(
                    crate::domain::automation::workflow::SaveWorkflowVersionCommand {
                        org_id,
                        workflow_id: workflow.id,
                        graph: Graph {
                            connectors: vec![PlacedConnector {
                                id: "c1".to_string(),
                                kind: "flow.condition".to_string(),
                                version: 1,
                                credential_id: None,
                                config,
                            }],
                            edges: Vec::new(),
                        },
                        created_by: None,
                    },
                )
                .await
                .unwrap();
            let run_id = usecase
                .start_run(org_id, workflow.id, json!({}))
                .await
                .unwrap();
            sqlx::query!(
                r#"UPDATE automation.run SET status = 'succeeded', finished_at = $2 WHERE id = $1"#,
                run_id,
                finished_at,
            )
            .execute(pool)
            .await
            .unwrap();
            run_id
        }

        // Ten days old: past the short organization's one-day retention,
        // well within the thirty-day default the other organization relies
        // on.
        let ten_days_ago = Utc::now() - chrono::Duration::days(10);
        let short_org_run = finished_run_in(&usecase, &pool, short_org, ten_days_ago).await;
        let long_org_run = finished_run_in(&usecase, &pool, long_org, ten_days_ago).await;

        usecase
            .run_retention_pass(RetentionSchedule::default())
            .await
            .unwrap();

        let short_org_repo_view = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.find_by_id(short_org, short_org_run).await
        })
        .await
        .unwrap();
        assert_eq!(
            short_org_repo_view, None,
            "purged under its own one-day retention"
        );

        let long_org_repo_view = with_tx(&pool, async |tx| {
            let mut repo = PgRunRepository::new(&tx);
            repo.find_by_id(long_org, long_org_run).await
        })
        .await
        .unwrap();
        assert!(
            long_org_repo_view.is_some(),
            "intact: still within the default ninety-day retention"
        );
    }
}
