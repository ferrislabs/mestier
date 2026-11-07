use serde_json::json;

use super::super::clients::tests_clients::{config, failure, produced, registry, run};
use crate::application::test_support::automation_pool;
use crate::infrastructure::automation::connectors::world_support;
use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_reports_cover_the_run_organization_over_a_period() {
    let _guard = RUN_CLAIM_LOCK.lock().await;
    let pool = automation_pool().await;
    let acme = world_support::organization(&pool, "reporting").await;
    let registry = registry(&pool);
    let period = config(&[("from", json!("2026-01-01")), ("to", json!("2026-01-31"))]);

    let profitability =
        produced(run(&registry, "mestier.reporting.profitability", acme, &period).await);
    let hours = produced(run(&registry, "mestier.reporting.worked_hours", acme, &period).await);
    let reversed = run(
        &registry,
        "mestier.reporting.worked_hours",
        acme,
        &config(&[("from", json!("2026-02-01")), ("to", json!("2026-01-01"))]),
    )
    .await;

    assert_eq!(profitability["projects"], json!([]));
    assert_eq!(hours["total_planned_minutes"], 0);
    assert!(failure(reversed).contains("precede"));
}

#[tokio::test]
async fn a_malformed_period_is_refused_before_the_database() {
    let pool = sqlx::PgPool::connect_lazy("postgres://unused:unused@localhost/unused").unwrap();
    let registry = registry(&pool);
    let org = common::OrganizationId(uuid::Uuid::from_u128(1));

    let error = failure(
        run(
            &registry,
            "mestier.reporting.profitability",
            org,
            &config(&[("from", json!("01/01/2026")), ("to", json!("2026-01-31"))]),
        )
        .await,
    );

    assert_eq!(error, "`from` must be a date formatted YYYY-MM-DD");
}
