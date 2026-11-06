use chrono::{DateTime, Duration, SecondsFormat, Utc};
use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::transactional;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        ports::{RunRepository, ScheduleRepository, WebhookRouteRepository, WorkflowRepository},
        run::{Run, RunStatus},
        workflow::{Graph, ResolvedWebhook, TriggerKind, WebhookMethod},
    },
    infrastructure::automation::{
        connectors::http_client::{HttpAuth, open_http_auth},
        webhook::inbound::presents,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScheduleOutcome {
    pub fired: usize,
    pub unreadable: usize,
}

fn triggered_run(
    org_id: OrganizationId,
    workflow_id: Uuid,
    workflow_version_id: Uuid,
    trigger_id: &str,
    payload: Value,
    now: DateTime<Utc>,
) -> Run {
    Run {
        id: generate_uuid_v7(),
        org_id,
        workflow_id,
        workflow_version_id,
        trigger_event_id: None,
        trigger_payload: Some(payload),
        trigger_id: Some(trigger_id.to_owned()),
        status: RunStatus::Pending,
        error: None,
        next_attempt_at: Some(now),
        locked_at: None,
        locked_by: None,
        started_at: None,
        finished_at: None,
        created_at: now,
    }
}

impl MestierUseCase {
    #[transactional(webhook_route)]
    pub async fn taken_webhook_triggers(
        &self,
        workflow_id: Uuid,
        graph: &Graph,
    ) -> Result<Vec<String>, CoreError> {
        let mut routes = webhook_route_repository;
        let wanted: Vec<(&str, &str)> = graph
            .triggers
            .iter()
            .filter_map(|trigger| match &trigger.kind {
                TriggerKind::Webhook { path, .. } => Some((trigger.id.as_str(), path.as_str())),
                _ => None,
            })
            .collect();
        let paths: Vec<String> = wanted.iter().map(|(_, path)| (*path).to_owned()).collect();
        let held = routes
            .paths_held_by_other_workflows(workflow_id, &paths)
            .await?;

        Ok(wanted
            .into_iter()
            .filter(|(_, path)| held.iter().any(|taken| taken == path))
            .map(|(trigger_id, _)| trigger_id.to_owned())
            .collect())
    }

    #[transactional(webhook_route)]
    pub async fn resolve_webhook(
        &self,
        path: &str,
        method: WebhookMethod,
    ) -> Result<Option<ResolvedWebhook>, CoreError> {
        let mut routes = webhook_route_repository;
        routes.resolve(path, method).await
    }

    pub async fn verify_webhook_credential<'a>(
        &self,
        org_id: OrganizationId,
        credential_id: Uuid,
        header: impl Fn(&str) -> Option<&'a str>,
    ) -> bool {
        match open_http_auth(self, org_id, credential_id).await {
            Ok(expected) => presents(&expected, header),
            Err(error) => {
                tracing::warn!(
                    org_id = %org_id.0,
                    %credential_id,
                    %error,
                    "a webhook credential could not be opened, the request is refused"
                );
                let unmatchable = HttpAuth::Header {
                    name: "authorization".to_owned(),
                    value: String::new(),
                };
                let _ = presents(&unmatchable, header);
                false
            }
        }
    }

    #[transactional(run, workflow)]
    pub async fn start_trigger_run(
        &self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        trigger_id: &str,
        payload: Value,
    ) -> Result<Uuid, CoreError> {
        let mut workflows = workflow_repository;
        let mut runs = run_repository;

        let workflow = workflows
            .find_by_id_for_update(org_id, workflow_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        if workflow.is_archived() || !workflow.enabled {
            return Err(CoreError::NotFound);
        }
        let workflow_version_id = workflow.current_version_id.ok_or(CoreError::NotFound)?;

        let run = triggered_run(
            org_id,
            workflow_id,
            workflow_version_id,
            trigger_id,
            payload,
            Utc::now(),
        );
        Ok(runs.create_run(&run).await?.id)
    }

    #[transactional(schedule, run)]
    pub async fn fire_due_schedules(
        &self,
        now: DateTime<Utc>,
        batch: i64,
    ) -> Result<ScheduleOutcome, CoreError> {
        let mut schedules = schedule_repository;
        let mut runs = run_repository;
        let mut outcome = ScheduleOutcome::default();

        for due in schedules.claim_due(now, batch).await? {
            let Some(next) = due.next_after(now) else {
                tracing::error!(
                    workflow_id = %due.workflow_id,
                    trigger_id = %due.trigger_id,
                    "a schedule has no readable time zone, so its next occurrence cannot be computed"
                );
                schedules
                    .advance(
                        due.workflow_id,
                        &due.trigger_id,
                        None,
                        now + Duration::hours(1),
                    )
                    .await?;
                outcome.unreadable += 1;
                continue;
            };

            let payload = json!({
                "scheduled_for": due.scheduled_for.to_rfc3339_opts(SecondsFormat::Secs, true)
            });
            runs.create_run(&triggered_run(
                due.org_id,
                due.workflow_id,
                due.workflow_version_id,
                &due.trigger_id,
                payload,
                now,
            ))
            .await?;
            schedules
                .advance(due.workflow_id, &due.trigger_id, Some(now), next)
                .await?;
            outcome.fired += 1;
        }

        Ok(outcome)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Duration, Utc};
    use common::{CoreError, OrganizationId};
    use sqlx::PgPool;
    use std::sync::Arc;

    use super::*;
    use crate::application::automation::trigger_fixtures::{
        daily, graph_of, seed_organization, set_organization_timezone, webhook, webhook_with,
        weekly,
    };
    use crate::application::default_authorizer;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::credential::{CreateCredentialCommand, CredentialOrigin};
    use crate::domain::automation::workflow::{
        CreateWorkflowCommand, OnInvalid, SaveWorkflowVersionCommand, TriggerKind,
        UpdateWorkflowCommand, WebhookMethod, Weekday, WorkflowRefusal, schedule_seeds,
    };
    use crate::infrastructure::automation::webhook::secret::SecretCipher;
    use crate::infrastructure::realtime::EventHub;
    use serde_json::{Value, json};

    fn use_case(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
            .with_cipher(Some(Arc::new(SecretCipher::new(&[7u8; 32]).unwrap())))
    }

    fn unique(label: &str) -> String {
        format!("{label}-{}", Uuid::now_v7().simple())
    }

    async fn new_workflow(usecase: &MestierUseCase, org_id: OrganizationId) -> Uuid {
        usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Trigger workflow".to_string(),
                description: None,
            })
            .await
            .unwrap()
            .id
    }

    async fn save(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        workflow_id: Uuid,
        graph: Graph,
    ) -> Result<(), CoreError> {
        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id,
                graph,
                layout: None,
                created_by: None,
            })
            .await
            .map(|_| ())
    }

    async fn route_rows(pool: &PgPool, workflow_id: Uuid) -> Vec<(String, String, String)> {
        sqlx::query!(
            r#"SELECT path, trigger_id, method FROM automation.webhook_route
               WHERE workflow_id = $1 ORDER BY trigger_id"#,
            workflow_id,
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| (row.path, row.trigger_id, row.method))
        .collect()
    }

    async fn schedule_rows(pool: &PgPool, workflow_id: Uuid) -> Vec<(String, DateTime<Utc>)> {
        sqlx::query!(
            r#"SELECT trigger_id, next_run_at FROM automation.schedule_state
               WHERE workflow_id = $1 ORDER BY trigger_id"#,
            workflow_id,
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| (row.trigger_id, row.next_run_at))
        .collect()
    }

    fn assert_close_to(actual: DateTime<Utc>, expected: DateTime<Utc>) {
        assert!(
            (actual - expected).abs() < Duration::seconds(5),
            "{actual} is not near {expected}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_webhook_trigger_derives_its_route_and_saving_without_it_removes_it() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "derive-route").await;
        let usecase = use_case(pool.clone());
        let workflow_id = new_workflow(&usecase, org_id).await;
        let path = unique("derive");

        save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![
                ("t1", webhook(WebhookMethod::Put, &path)),
                ("t2", TriggerKind::Manual),
            ]),
        )
        .await
        .unwrap();
        assert_eq!(
            route_rows(&pool, workflow_id).await,
            vec![(path.clone(), "t1".to_string(), "PUT".to_string())]
        );

        save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![("t2", TriggerKind::Manual)]),
        )
        .await
        .unwrap();
        assert!(route_rows(&pool, workflow_id).await.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_keeps_its_own_path_when_it_saves_again() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "derive-resave").await;
        let usecase = use_case(pool.clone());
        let workflow_id = new_workflow(&usecase, org_id).await;
        let path = unique("resave");

        for _ in 0..2 {
            save(
                &usecase,
                org_id,
                workflow_id,
                graph_of(vec![("t1", webhook(WebhookMethod::Post, &path))]),
            )
            .await
            .unwrap();
        }

        assert_eq!(route_rows(&pool, workflow_id).await.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_path_another_workflow_holds_is_refused_and_nothing_of_the_save_survives() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "derive-taken").await;
        let other_org = seed_organization(&pool, "derive-taken-other").await;
        let usecase = use_case(pool.clone());
        let path = unique("taken");
        let first = new_workflow(&usecase, org_id).await;
        let second = new_workflow(&usecase, other_org).await;
        save(
            &usecase,
            org_id,
            first,
            graph_of(vec![("t1", webhook(WebhookMethod::Post, &path))]),
        )
        .await
        .unwrap();

        let refused = save(
            &usecase,
            other_org,
            second,
            graph_of(vec![("t1", webhook(WebhookMethod::Post, &path))]),
        )
        .await;

        let Err(CoreError::Conflict(message)) = refused else {
            panic!("expected a conflict, got {refused:?}");
        };
        assert_eq!(
            WorkflowRefusal::from_conflict(&message),
            Some(WorkflowRefusal::WebhookPathTaken)
        );
        assert!(route_rows(&pool, second).await.is_empty());
        assert_eq!(route_rows(&pool, first).await.len(), 1);
        assert!(
            usecase
                .list_workflow_versions(other_org, second)
                .await
                .unwrap()
                .is_empty(),
            "the refused version must be rolled back with the route"
        );
        let reloaded = usecase
            .find_workflow(other_org, second)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(reloaded.current_version_id, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_triggers_whose_paths_other_workflows_hold_are_named() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "derive-named").await;
        let usecase = use_case(pool.clone());
        let (taken, free) = (unique("named-taken"), unique("named-free"));
        let holder = new_workflow(&usecase, org_id).await;
        let candidate = new_workflow(&usecase, org_id).await;
        save(
            &usecase,
            org_id,
            holder,
            graph_of(vec![("t1", webhook(WebhookMethod::Post, &taken))]),
        )
        .await
        .unwrap();
        let graph = graph_of(vec![
            ("a", webhook(WebhookMethod::Post, &free)),
            ("b", webhook(WebhookMethod::Get, &taken)),
            ("c", TriggerKind::Manual),
        ]);

        let named = usecase
            .taken_webhook_triggers(candidate, &graph)
            .await
            .unwrap();
        let own = usecase
            .taken_webhook_triggers(holder, &graph)
            .await
            .unwrap();

        assert_eq!(named, vec!["b".to_string()]);
        assert!(own.is_empty(), "a workflow's own path is not taken from it");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_schedule_derives_the_next_run_in_the_trigger_zone_or_the_organizations() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "derive-schedule").await;
        set_organization_timezone(&pool, org_id, "Asia/Tokyo").await;
        let usecase = use_case(pool.clone());
        let workflow_id = new_workflow(&usecase, org_id).await;

        save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![
                ("t1", daily("09:00", None)),
                ("t2", daily("09:00", Some("America/New_York"))),
                ("t3", weekly(Weekday::Monday, "09:00", Some("UTC"))),
                ("t4", TriggerKind::Manual),
            ]),
        )
        .await
        .unwrap();

        let rows = schedule_rows(&pool, workflow_id).await;
        assert_eq!(
            rows.iter().map(|(id, _)| id.as_str()).collect::<Vec<_>>(),
            vec!["t1", "t2", "t3"]
        );
        let now = Utc::now();
        for (id, next) in &rows {
            assert!(*next > now, "{id} is due in the future");
            assert!(
                *next <= now + Duration::days(8),
                "{id} is due within a week"
            );
        }
        let local = |name: &str, instant: DateTime<Utc>| {
            use chrono::Timelike;
            instant
                .with_timezone(&name.parse::<chrono_tz::Tz>().unwrap())
                .time()
                .hour()
        };
        assert_eq!(local("Asia/Tokyo", rows[0].1), 9);
        assert_eq!(local("America/New_York", rows[1].1), 9);
        assert_eq!(local("UTC", rows[2].1), 9);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_without_the_schedule_removes_its_row() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "derive-schedule-gone").await;
        let usecase = use_case(pool.clone());
        let workflow_id = new_workflow(&usecase, org_id).await;
        save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![("t1", daily("09:00", None))]),
        )
        .await
        .unwrap();
        assert_eq!(schedule_rows(&pool, workflow_id).await.len(), 1);

        save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![("t1", TriggerKind::Manual)]),
        )
        .await
        .unwrap();

        assert!(schedule_rows(&pool, workflow_id).await.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn re_enabling_a_workflow_recomputes_its_schedules_from_now() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "derive-reenable").await;
        let usecase = use_case(pool.clone());
        let workflow_id = new_workflow(&usecase, org_id).await;
        save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![("t1", daily("09:00", Some("UTC")))]),
        )
        .await
        .unwrap();
        let disable = UpdateWorkflowCommand {
            org_id,
            id: workflow_id,
            name: None,
            description: None,
            enabled: Some(false),
            folder_id: None,
            tags: None,
            archived: None,
        };
        usecase.update_workflow(disable.clone()).await.unwrap();
        let stale = Utc::now() - Duration::days(30);
        sqlx::query!(
            "UPDATE automation.schedule_state SET next_run_at = $2 WHERE workflow_id = $1",
            workflow_id,
            stale,
        )
        .execute(&pool)
        .await
        .unwrap();

        usecase
            .update_workflow(UpdateWorkflowCommand {
                enabled: Some(true),
                ..disable
            })
            .await
            .unwrap();

        let rows = schedule_rows(&pool, workflow_id).await;
        assert_eq!(rows.len(), 1);
        assert!(rows[0].1 > Utc::now(), "{} is still in the past", rows[0].1);
        assert!(rows[0].1 <= Utc::now() + Duration::days(1) + Duration::minutes(1));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_a_workflow_that_stays_enabled_or_stays_disabled_leaves_its_schedules_alone() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "derive-untouched").await;
        let usecase = use_case(pool.clone());
        let workflow_id = new_workflow(&usecase, org_id).await;
        save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![("t1", daily("09:00", Some("UTC")))]),
        )
        .await
        .unwrap();
        let marker = Utc::now() - Duration::days(3);
        sqlx::query!(
            "UPDATE automation.schedule_state SET next_run_at = $2 WHERE workflow_id = $1",
            workflow_id,
            marker,
        )
        .execute(&pool)
        .await
        .unwrap();
        let command = |enabled: Option<bool>, name: Option<&str>| UpdateWorkflowCommand {
            org_id,
            id: workflow_id,
            name: name.map(|name| name.to_string().try_into().unwrap()),
            description: None,
            enabled,
            folder_id: None,
            tags: None,
            archived: None,
        };

        usecase
            .update_workflow(command(None, Some("Renamed")))
            .await
            .unwrap();
        usecase
            .update_workflow(command(Some(true), None))
            .await
            .unwrap();
        assert_close_to(schedule_rows(&pool, workflow_id).await[0].1, marker);

        usecase
            .update_workflow(command(Some(false), None))
            .await
            .unwrap();
        usecase
            .update_workflow(command(Some(false), None))
            .await
            .unwrap();
        assert_close_to(schedule_rows(&pool, workflow_id).await[0].1, marker);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_credential_a_webhook_trigger_uses_cannot_be_deleted() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "derive-credential").await;
        let usecase = use_case(pool.clone());
        let (credential, _) = usecase
            .create_credential(CreateCredentialCommand {
                org_id,
                kind: "bearer_token".to_string(),
                name: "Webhook secret".to_string(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "s3cret" })),
            })
            .await
            .unwrap();
        let workflow_id = new_workflow(&usecase, org_id).await;
        save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![(
                "t1",
                webhook_with(
                    &unique("credential"),
                    Some(credential.id),
                    Vec::new(),
                    OnInvalid::Reject,
                ),
            )]),
        )
        .await
        .unwrap();

        let refused = usecase.delete_credential(org_id, credential.id).await;

        assert!(
            matches!(&refused, Err(CoreError::Conflict(message)) if message.contains("Trigger workflow")),
            "{refused:?}"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_webhook_naming_a_credential_of_another_organization_is_refused() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "derive-foreign").await;
        let other_org = seed_organization(&pool, "derive-foreign-other").await;
        let usecase = use_case(pool.clone());
        let (credential, _) = usecase
            .create_credential(CreateCredentialCommand {
                org_id: other_org,
                kind: "bearer_token".to_string(),
                name: "Not yours".to_string(),
                origin: CredentialOrigin::Supplied,
                data: Some(serde_json::json!({ "token": "s3cret" })),
            })
            .await
            .unwrap();
        let workflow_id = new_workflow(&usecase, org_id).await;

        let refused = save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![(
                "t1",
                webhook_with(
                    &unique("foreign"),
                    Some(credential.id),
                    Vec::new(),
                    OnInvalid::Reject,
                ),
            )]),
        )
        .await;

        assert!(
            matches!(refused, Err(CoreError::Conflict(_))),
            "{refused:?}"
        );
        assert!(route_rows(&pool, workflow_id).await.is_empty());
    }

    struct RunRow {
        trigger_id: Option<String>,
        payload: Option<Value>,
        status: String,
        version_id: Uuid,
    }

    async fn runs_of(pool: &PgPool, workflow_id: Uuid) -> Vec<RunRow> {
        sqlx::query!(
            r#"SELECT trigger_id, trigger_payload, status, workflow_version_id
               FROM automation.run WHERE workflow_id = $1 ORDER BY run_number"#,
            workflow_id,
        )
        .fetch_all(pool)
        .await
        .unwrap()
        .into_iter()
        .map(|row| RunRow {
            trigger_id: row.trigger_id,
            payload: row.trigger_payload,
            status: row.status,
            version_id: row.workflow_version_id,
        })
        .collect()
    }

    async fn scheduled_for_of(pool: &PgPool, workflow_id: Uuid) -> Vec<String> {
        runs_of(pool, workflow_id)
            .await
            .into_iter()
            .map(|run| {
                run.payload.expect("a payload")["scheduled_for"]
                    .as_str()
                    .expect("an instant")
                    .to_owned()
            })
            .collect()
    }

    async fn schedule_state(
        pool: &PgPool,
        workflow_id: Uuid,
    ) -> (DateTime<Utc>, Option<DateTime<Utc>>) {
        let row = sqlx::query!(
            "SELECT next_run_at, last_run_at FROM automation.schedule_state WHERE workflow_id = $1",
            workflow_id,
        )
        .fetch_one(pool)
        .await
        .unwrap();
        (row.next_run_at, row.last_run_at)
    }

    async fn set_next_run(pool: &PgPool, workflow_id: Uuid, next_run_at: DateTime<Utc>) {
        sqlx::query!(
            "UPDATE automation.schedule_state SET next_run_at = $2, last_run_at = NULL WHERE workflow_id = $1",
            workflow_id,
            next_run_at,
        )
        .execute(pool)
        .await
        .unwrap();
    }

    fn at(text: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(text)
            .unwrap()
            .with_timezone(&Utc)
    }

    async fn scheduled(
        usecase: &MestierUseCase,
        pool: &PgPool,
        org_id: OrganizationId,
        kind: TriggerKind,
        due: DateTime<Utc>,
    ) -> Uuid {
        let workflow_id = new_workflow(usecase, org_id).await;
        save(usecase, org_id, workflow_id, graph_of(vec![("t1", kind)]))
            .await
            .unwrap();
        set_next_run(pool, workflow_id, due).await;
        workflow_id
    }

    async fn simulate(
        pool: &PgPool,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
        step: Duration,
    ) -> usize {
        let mut fired = 0;
        let mut tick = from;
        while tick < to {
            let (one, other) = (use_case(pool.clone()), use_case(pool.clone()));
            let (first, second) = tokio::join!(
                one.fire_due_schedules(tick, 100),
                other.fire_due_schedules(tick, 100),
            );
            fired += first.unwrap().fired + second.unwrap().fired;
            tick += step;
        }
        fired
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_due_schedule_starts_one_run_with_its_scheduled_instant_and_moves_on() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-one").await;
        let usecase = use_case(pool.clone());
        let due = at("2026-11-06T08:00:00Z");
        let workflow_id = scheduled(&usecase, &pool, org_id, daily("09:00", None), due).await;
        let now = due + Duration::seconds(5);

        let outcome = usecase.fire_due_schedules(now, 100).await.unwrap();

        assert_eq!(
            outcome,
            ScheduleOutcome {
                fired: 1,
                unreadable: 0
            }
        );
        let runs = runs_of(&pool, workflow_id).await;
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].trigger_id.as_deref(), Some("t1"));
        assert_eq!(
            runs[0].payload,
            Some(json!({ "scheduled_for": "2026-11-06T08:00:00Z" }))
        );
        assert_eq!(runs[0].status, "pending");
        let current = usecase
            .find_workflow(org_id, workflow_id)
            .await
            .unwrap()
            .unwrap()
            .current_version_id;
        assert_eq!(Some(runs[0].version_id), current);
        assert_eq!(
            schedule_state(&pool, workflow_id).await,
            (at("2026-11-07T08:00:00Z"), Some(now))
        );
        assert_eq!(usecase.fire_due_schedules(now, 100).await.unwrap().fired, 0);
        assert_eq!(runs_of(&pool, workflow_id).await.len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_schedule_that_is_not_due_yet_does_not_fire() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-early").await;
        let usecase = use_case(pool.clone());
        let due = at("2026-11-06T08:00:00Z");
        let workflow_id = scheduled(&usecase, &pool, org_id, daily("09:00", None), due).await;

        let outcome = usecase
            .fire_due_schedules(due - Duration::seconds(1), 100)
            .await
            .unwrap();

        assert_eq!(outcome.fired, 0);
        assert!(runs_of(&pool, workflow_id).await.is_empty());
        assert_eq!(schedule_state(&pool, workflow_id).await, (due, None));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 4)]
    #[ignore = "requires live postgres"]
    async fn several_workers_claiming_at_once_start_each_due_schedule_once() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-race").await;
        let usecase = use_case(pool.clone());
        let due = at("2026-11-06T08:00:00Z");
        let mut workflows = Vec::new();
        for _ in 0..6 {
            workflows.push(scheduled(&usecase, &pool, org_id, daily("09:00", None), due).await);
        }
        let now = due + Duration::seconds(1);

        let mut workers = tokio::task::JoinSet::new();
        for _ in 0..8 {
            let pool = pool.clone();
            workers.spawn(async move { use_case(pool).fire_due_schedules(now, 100).await });
        }
        let mut fired = 0;
        while let Some(finished) = workers.join_next().await {
            fired += finished.unwrap().unwrap().fired;
        }

        assert_eq!(fired, 6);
        for workflow_id in workflows {
            assert_eq!(runs_of(&pool, workflow_id).await.len(), 1);
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_restart_does_not_fire_a_schedule_again_and_the_next_day_fires_once_more() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-restart").await;
        let due = at("2026-11-06T08:00:00Z");
        let workflow_id = scheduled(
            &use_case(pool.clone()),
            &pool,
            org_id,
            daily("09:00", None),
            due,
        )
        .await;

        let before = use_case(pool.clone());
        assert_eq!(
            before
                .fire_due_schedules(due + Duration::seconds(1), 100)
                .await
                .unwrap()
                .fired,
            1
        );
        drop(before);

        let after = use_case(pool.clone());
        for offset in [
            Duration::seconds(2),
            Duration::hours(12),
            Duration::hours(24) - Duration::seconds(1),
        ] {
            assert_eq!(
                after
                    .fire_due_schedules(due + offset, 100)
                    .await
                    .unwrap()
                    .fired,
                0,
                "{offset}"
            );
        }
        assert_eq!(
            after
                .fire_due_schedules(due + Duration::hours(24), 100)
                .await
                .unwrap()
                .fired,
            1
        );
        assert_eq!(
            scheduled_for_of(&pool, workflow_id).await,
            vec!["2026-11-06T08:00:00Z", "2026-11-07T08:00:00Z"]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_long_stop_fires_one_run_and_waits_for_the_next_future_occurrence() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-backlog").await;
        let usecase = use_case(pool.clone());
        let missed = at("2026-11-01T08:00:00Z");
        let workflow_id = scheduled(&usecase, &pool, org_id, daily("09:00", None), missed).await;
        let back = at("2026-11-06T12:00:00Z");

        assert_eq!(
            usecase.fire_due_schedules(back, 100).await.unwrap().fired,
            1
        );
        assert_eq!(
            usecase
                .fire_due_schedules(back + Duration::seconds(1), 100)
                .await
                .unwrap()
                .fired,
            0
        );
        assert_eq!(
            schedule_state(&pool, workflow_id).await.0,
            at("2026-11-07T08:00:00Z")
        );
        assert_eq!(
            usecase
                .fire_due_schedules(at("2026-11-07T08:00:00Z"), 100)
                .await
                .unwrap()
                .fired,
            1
        );
        assert_eq!(
            scheduled_for_of(&pool, workflow_id).await,
            vec!["2026-11-01T08:00:00Z", "2026-11-07T08:00:00Z"]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_daily_schedule_at_nine_paris_fires_at_nine_local_across_the_spring_clock_change() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-spring").await;
        let from = at("2026-03-26T00:00:00Z");
        let workflow_id = {
            let usecase = use_case(pool.clone());
            let seed = schedule_seeds(
                &graph_of(vec![("t1", daily("09:00", None))]).triggers,
                chrono_tz::Europe::Paris,
                from,
            )[0]
            .next_run_at;
            scheduled(&usecase, &pool, org_id, daily("09:00", None), seed).await
        };

        let fired = simulate(
            &pool,
            from,
            at("2026-04-01T00:00:00Z"),
            Duration::minutes(15),
        )
        .await;

        assert_eq!(fired, 6);
        assert_eq!(
            scheduled_for_of(&pool, workflow_id).await,
            vec![
                "2026-03-26T08:00:00Z",
                "2026-03-27T08:00:00Z",
                "2026-03-28T08:00:00Z",
                "2026-03-29T07:00:00Z",
                "2026-03-30T07:00:00Z",
                "2026-03-31T07:00:00Z",
            ]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_daily_schedule_at_nine_paris_fires_at_nine_local_across_the_autumn_clock_change() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-autumn").await;
        let from = at("2026-10-23T00:00:00Z");
        let workflow_id = {
            let usecase = use_case(pool.clone());
            let seed = schedule_seeds(
                &graph_of(vec![("t1", daily("09:00", None))]).triggers,
                chrono_tz::Europe::Paris,
                from,
            )[0]
            .next_run_at;
            scheduled(&usecase, &pool, org_id, daily("09:00", None), seed).await
        };

        let fired = simulate(
            &pool,
            from,
            at("2026-10-28T00:00:00Z"),
            Duration::minutes(15),
        )
        .await;

        assert_eq!(fired, 5);
        assert_eq!(
            scheduled_for_of(&pool, workflow_id).await,
            vec![
                "2026-10-23T07:00:00Z",
                "2026-10-24T07:00:00Z",
                "2026-10-25T08:00:00Z",
                "2026-10-26T08:00:00Z",
                "2026-10-27T08:00:00Z",
            ]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_time_inside_the_spring_gap_fires_once_when_the_clocks_resume_and_the_repeated_hour_once()
     {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-gap").await;
        let from = at("2026-03-27T00:00:00Z");
        let workflow_id = {
            let usecase = use_case(pool.clone());
            let seed = schedule_seeds(
                &graph_of(vec![("t1", daily("02:30", None))]).triggers,
                chrono_tz::Europe::Paris,
                from,
            )[0]
            .next_run_at;
            scheduled(&usecase, &pool, org_id, daily("02:30", None), seed).await
        };

        simulate(
            &pool,
            from,
            at("2026-03-31T00:00:00Z"),
            Duration::minutes(15),
        )
        .await;

        assert_eq!(
            scheduled_for_of(&pool, workflow_id).await,
            vec![
                "2026-03-27T01:30:00Z",
                "2026-03-28T01:30:00Z",
                "2026-03-29T01:00:00Z",
                "2026-03-30T00:30:00Z",
            ]
        );

        let from = at("2026-10-24T00:00:00Z");
        let repeated = {
            let usecase = use_case(pool.clone());
            let seed = schedule_seeds(
                &graph_of(vec![("t1", daily("02:30", None))]).triggers,
                chrono_tz::Europe::Paris,
                from,
            )[0]
            .next_run_at;
            scheduled(&usecase, &pool, org_id, daily("02:30", None), seed).await
        };

        simulate(
            &pool,
            from,
            at("2026-10-27T00:00:00Z"),
            Duration::minutes(15),
        )
        .await;

        assert_eq!(
            scheduled_for_of(&pool, repeated).await,
            vec![
                "2026-10-24T00:30:00Z",
                "2026-10-25T00:30:00Z",
                "2026-10-26T01:30:00Z",
            ]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_weekly_and_an_hourly_schedule_fire_on_their_own_rhythm() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-rhythm").await;
        let usecase = use_case(pool.clone());
        let from = at("2026-11-02T00:00:00Z");
        let seeds_of = |kind: &TriggerKind| {
            schedule_seeds(
                &graph_of(vec![("t1", kind.clone())]).triggers,
                chrono_tz::Europe::Paris,
                from,
            )[0]
            .next_run_at
        };
        let weekly_kind = weekly(Weekday::Monday, "09:00", None);
        let hourly_kind = TriggerKind::Schedule {
            frequency: crate::domain::automation::workflow::Frequency::Hourly,
            time: "00:15".to_string(),
            weekday: None,
            timezone: None,
        };
        let weekly_workflow = scheduled(
            &usecase,
            &pool,
            org_id,
            weekly_kind.clone(),
            seeds_of(&weekly_kind),
        )
        .await;
        let hourly_workflow = scheduled(
            &usecase,
            &pool,
            org_id,
            hourly_kind.clone(),
            seeds_of(&hourly_kind),
        )
        .await;

        simulate(
            &pool,
            from,
            at("2026-11-17T00:00:00Z"),
            Duration::minutes(30),
        )
        .await;

        assert_eq!(
            scheduled_for_of(&pool, weekly_workflow).await,
            vec![
                "2026-11-02T08:00:00Z",
                "2026-11-09T08:00:00Z",
                "2026-11-16T08:00:00Z"
            ]
        );
        assert_eq!(runs_of(&pool, hourly_workflow).await.len(), 15 * 24);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_workflow_does_not_fire_and_re_enabling_does_not_replay_what_it_missed() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-disabled").await;
        let usecase = use_case(pool.clone());
        let due = at("2020-01-01T08:00:00Z");
        let workflow_id = scheduled(&usecase, &pool, org_id, daily("09:00", None), due).await;
        let command = |enabled: Option<bool>, archived: Option<bool>| UpdateWorkflowCommand {
            org_id,
            id: workflow_id,
            name: None,
            description: None,
            enabled,
            folder_id: None,
            tags: None,
            archived,
        };
        usecase
            .update_workflow(command(Some(false), None))
            .await
            .unwrap();
        set_next_run(&pool, workflow_id, due).await;

        let outcome = usecase.fire_due_schedules(Utc::now(), 100).await.unwrap();

        assert_eq!(outcome.fired, 0);
        assert!(runs_of(&pool, workflow_id).await.is_empty());
        assert_eq!(schedule_state(&pool, workflow_id).await, (due, None));

        usecase
            .update_workflow(command(Some(true), None))
            .await
            .unwrap();
        assert_eq!(
            usecase
                .fire_due_schedules(Utc::now(), 100)
                .await
                .unwrap()
                .fired,
            0
        );
        assert!(runs_of(&pool, workflow_id).await.is_empty());
        assert!(schedule_state(&pool, workflow_id).await.0 > Utc::now());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_archived_workflow_does_not_fire() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-archived").await;
        let usecase = use_case(pool.clone());
        let due = at("2020-01-01T08:00:00Z");
        let workflow_id = scheduled(&usecase, &pool, org_id, daily("09:00", None), due).await;
        usecase
            .update_workflow(UpdateWorkflowCommand {
                org_id,
                id: workflow_id,
                name: None,
                description: None,
                enabled: None,
                folder_id: None,
                tags: None,
                archived: Some(true),
            })
            .await
            .unwrap();
        set_next_run(&pool, workflow_id, due).await;

        let outcome = usecase.fire_due_schedules(Utc::now(), 100).await.unwrap();

        assert_eq!(outcome.fired, 0);
        assert!(runs_of(&pool, workflow_id).await.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_schedule_without_a_usable_zone_fires_nothing_and_retries_in_an_hour() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-zone").await;
        let usecase = use_case(pool.clone());
        let due = at("2026-11-06T08:00:00Z");
        let workflow_id = scheduled(&usecase, &pool, org_id, daily("09:00", None), due).await;
        set_organization_timezone(&pool, org_id, "Mars/Olympus").await;
        let now = due + Duration::seconds(1);

        let outcome = usecase.fire_due_schedules(now, 100).await.unwrap();

        assert_eq!(
            outcome,
            ScheduleOutcome {
                fired: 0,
                unreadable: 1
            }
        );
        assert!(runs_of(&pool, workflow_id).await.is_empty());
        assert_eq!(
            schedule_state(&pool, workflow_id).await,
            (now + Duration::hours(1), None)
        );
        assert_eq!(
            usecase.fire_due_schedules(now, 100).await.unwrap(),
            ScheduleOutcome::default()
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_batch_bounds_how_many_schedules_fire_in_one_pass() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-batch").await;
        let usecase = use_case(pool.clone());
        let due = at("2026-11-06T08:00:00Z");
        for _ in 0..3 {
            scheduled(&usecase, &pool, org_id, daily("09:00", None), due).await;
        }
        let now = due + Duration::seconds(1);

        assert_eq!(usecase.fire_due_schedules(now, 2).await.unwrap().fired, 2);
        assert_eq!(usecase.fire_due_schedules(now, 2).await.unwrap().fired, 1);
        assert_eq!(usecase.fire_due_schedules(now, 2).await.unwrap().fired, 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_run_a_schedule_starts_is_executed_by_the_engine() {
        use crate::infrastructure::automation::connectors::ConnectorRegistry;
        use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
        use crate::infrastructure::automation::worker::WorkerSchedule;

        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "fire-engine").await;
        let usecase = use_case(pool.clone());
        let now = Utc::now();
        let workflow_id = scheduled(
            &usecase,
            &pool,
            org_id,
            daily("09:00", None),
            now - Duration::minutes(1),
        )
        .await;
        usecase.fire_due_schedules(now, 100).await.unwrap();
        sqlx::query!(
            "UPDATE automation.run SET next_attempt_at = now() WHERE workflow_id = $1",
            workflow_id,
        )
        .execute(&pool)
        .await
        .unwrap();

        let connectors = ConnectorRegistry::new(usecase.clone());
        usecase
            .run_engine_pass(&connectors, "worker-1", WorkerSchedule::default())
            .await
            .unwrap();

        assert_eq!(runs_of(&pool, workflow_id).await[0].status, "succeeded");
    }

    fn presented<'a>(
        pairs: &'a [(&'static str, &'static str)],
    ) -> impl Fn(&str) -> Option<&'a str> + 'a {
        move |name| {
            pairs
                .iter()
                .find(|(candidate, _)| candidate.eq_ignore_ascii_case(name))
                .map(|(_, value)| *value)
        }
    }

    async fn new_credential(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        kind: &str,
        data: Value,
    ) -> Uuid {
        usecase
            .create_credential(CreateCredentialCommand {
                org_id,
                kind: kind.to_string(),
                name: "Inbound".to_string(),
                origin: CredentialOrigin::Supplied,
                data: Some(data),
            })
            .await
            .unwrap()
            .0
            .id
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_credential_is_verified_against_the_headers_the_request_presents() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "verify-kinds").await;
        let usecase = use_case(pool.clone());
        let bearer = new_credential(
            &usecase,
            org_id,
            "bearer_token",
            json!({ "token": "s3cret" }),
        )
        .await;
        let basic = new_credential(
            &usecase,
            org_id,
            "http_basic",
            json!({ "username": "alice", "password": "p4ss" }),
        )
        .await;
        let header = new_credential(
            &usecase,
            org_id,
            "http_header",
            json!({ "header_name": "X-Signature", "header_value": "sig-123" }),
        )
        .await;

        let right_bearer = [("authorization", "Bearer s3cret")];
        let wrong_bearer = [("authorization", "Bearer s3cret!")];
        let right_basic = [("authorization", "Basic YWxpY2U6cDRzcw==")];
        let wrong_basic = [("authorization", "Basic YWxpY2U6cDRzcg==")];
        let right_header = [("x-signature", "sig-123")];
        let wrong_header = [("x-signature", "sig-124")];

        assert!(
            usecase
                .verify_webhook_credential(org_id, bearer, presented(&right_bearer))
                .await
        );
        assert!(
            !usecase
                .verify_webhook_credential(org_id, bearer, presented(&wrong_bearer))
                .await
        );
        assert!(
            !usecase
                .verify_webhook_credential(org_id, bearer, presented(&[]))
                .await
        );
        assert!(
            usecase
                .verify_webhook_credential(org_id, basic, presented(&right_basic))
                .await
        );
        assert!(
            !usecase
                .verify_webhook_credential(org_id, basic, presented(&wrong_basic))
                .await
        );
        assert!(
            usecase
                .verify_webhook_credential(org_id, header, presented(&right_header))
                .await
        );
        assert!(
            !usecase
                .verify_webhook_credential(org_id, header, presented(&wrong_header))
                .await
        );
        assert!(
            !usecase
                .verify_webhook_credential(org_id, header, presented(&right_bearer))
                .await
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_credential_that_cannot_be_opened_never_authorizes() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "verify-closed").await;
        let other_org = seed_organization(&pool, "verify-closed-other").await;
        let usecase = use_case(pool.clone());
        let theirs = new_credential(
            &usecase,
            other_org,
            "bearer_token",
            json!({ "token": "s3cret" }),
        )
        .await;
        let odoo = new_credential(
            &usecase,
            org_id,
            "odoo_api",
            json!({ "base_url": "https://odoo.example.com", "database": "d", "username": "u", "api_key": "k" }),
        )
        .await;
        let mine = new_credential(
            &usecase,
            org_id,
            "bearer_token",
            json!({ "token": "s3cret" }),
        )
        .await;
        let right = [("authorization", "Bearer s3cret")];
        let without_key = MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new());

        assert!(
            usecase
                .verify_webhook_credential(org_id, mine, presented(&right))
                .await
        );
        assert!(
            !usecase
                .verify_webhook_credential(org_id, Uuid::now_v7(), presented(&right))
                .await
        );
        assert!(
            !usecase
                .verify_webhook_credential(org_id, theirs, presented(&right))
                .await
        );
        assert!(
            !usecase
                .verify_webhook_credential(org_id, odoo, presented(&right))
                .await
        );
        assert!(
            !without_key
                .verify_webhook_credential(org_id, mine, presented(&right))
                .await
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_trigger_run_pins_the_current_version_and_carries_the_payload() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "trigger-run").await;
        let usecase = use_case(pool.clone());
        let workflow_id = new_workflow(&usecase, org_id).await;
        save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![("t1", webhook(WebhookMethod::Post, &unique("run")))]),
        )
        .await
        .unwrap();
        let payload = json!({ "montant": 12, "nested": { "ok": [1, 2, 3] } });

        let run_id = usecase
            .start_trigger_run(org_id, workflow_id, "t1", payload.clone())
            .await
            .unwrap();

        let runs = runs_of(&pool, workflow_id).await;
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].trigger_id.as_deref(), Some("t1"));
        assert_eq!(runs[0].payload, Some(payload));
        assert_eq!(runs[0].status, "pending");
        let current = usecase
            .find_workflow(org_id, workflow_id)
            .await
            .unwrap()
            .unwrap()
            .current_version_id;
        assert_eq!(Some(runs[0].version_id), current);
        let stored = usecase.find_run(org_id, run_id).await.unwrap().unwrap();
        assert_eq!(stored.workflow_id, workflow_id);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_disabled_an_archived_an_unsaved_or_a_foreign_workflow_never_starts() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "trigger-run-never").await;
        let other_org = seed_organization(&pool, "trigger-run-never-other").await;
        let usecase = use_case(pool.clone());
        let saved = |label: &str| {
            let usecase = &usecase;
            let path = unique(label);
            async move {
                let workflow_id = new_workflow(usecase, org_id).await;
                save(
                    usecase,
                    org_id,
                    workflow_id,
                    graph_of(vec![("t1", webhook(WebhookMethod::Post, &path))]),
                )
                .await
                .unwrap();
                workflow_id
            }
        };
        let command =
            |id: Uuid, enabled: Option<bool>, archived: Option<bool>| UpdateWorkflowCommand {
                org_id,
                id,
                name: None,
                description: None,
                enabled,
                folder_id: None,
                tags: None,
                archived,
            };
        let disabled = saved("never-disabled").await;
        usecase
            .update_workflow(command(disabled, Some(false), None))
            .await
            .unwrap();
        let archived = saved("never-archived").await;
        usecase
            .update_workflow(command(archived, None, Some(true)))
            .await
            .unwrap();
        let unsaved = new_workflow(&usecase, org_id).await;
        let foreign = saved("never-foreign").await;

        for (workflow_id, org) in [
            (disabled, org_id),
            (archived, org_id),
            (unsaved, org_id),
            (foreign, other_org),
            (Uuid::now_v7(), org_id),
        ] {
            let refused = usecase
                .start_trigger_run(org, workflow_id, "t1", json!({}))
                .await;

            assert!(
                matches!(refused, Err(CoreError::NotFound)),
                "{workflow_id}: {refused:?}"
            );
            assert!(runs_of(&pool, workflow_id).await.is_empty());
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_webhook_route_resolves_while_its_workflow_is_enabled() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "resolve-use-case").await;
        let usecase = use_case(pool.clone());
        let workflow_id = new_workflow(&usecase, org_id).await;
        let path = unique("resolve");
        save(
            &usecase,
            org_id,
            workflow_id,
            graph_of(vec![("t1", webhook(WebhookMethod::Put, &path))]),
        )
        .await
        .unwrap();

        let found = usecase
            .resolve_webhook(&path, WebhookMethod::Put)
            .await
            .unwrap();
        let wrong_method = usecase
            .resolve_webhook(&path, WebhookMethod::Post)
            .await
            .unwrap();
        usecase
            .update_workflow(UpdateWorkflowCommand {
                org_id,
                id: workflow_id,
                name: None,
                description: None,
                enabled: Some(false),
                folder_id: None,
                tags: None,
                archived: None,
            })
            .await
            .unwrap();
        let disabled = usecase
            .resolve_webhook(&path, WebhookMethod::Put)
            .await
            .unwrap();

        assert_eq!(found.map(|route| route.workflow_id), Some(workflow_id));
        assert!(wrong_method.is_none());
        assert!(disabled.is_none());
    }
}
