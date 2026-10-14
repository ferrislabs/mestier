//! The two read views and the plan action against a real database:
//! @spec-ent-5 and @spec-ent-6.

use chrono::NaiveDate;
use serde_json::json;

use crate::{
    DueStatus, InterventionStatus, IsoWeekNumber,
    application::maintenance_intervention::tests::{World, pool, rolling},
};

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn week(n: u8) -> IsoWeekNumber {
    IsoWeekNumber::try_from(n).unwrap()
}

/// Mondays of 2026: S43 is 19 October, S46 is 9 November. Every line below
/// rolls every 14 days, so its due date is its last visit plus 14.
#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_ent_5_the_thursday_recap() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let vichy = world.seed_site("GAUME", Some("Vichy")).await;
    let sioule = world
        .seed_site("MAIRIE", Some("Saint-Pourçain-sur-Sioule"))
        .await;
    let season = (2026, 12, 48);
    let at_vichy = world
        .seed_contract(
            &vichy,
            season,
            &[
                ("Due mardi", rolling(14)),
                ("En retard", rolling(14)),
                ("Déjà planifiée", rolling(14)),
            ],
        )
        .await;
    let at_sioule = world
        .seed_contract(
            &sioule,
            season,
            &[
                ("Due jeudi", rolling(14)),
                ("Due samedi", rolling(14)),
                ("Due en S46", rolling(14)),
                ("Due aussi en S46", rolling(14)),
            ],
        )
        .await;
    for (line, last) in [
        (at_vichy.line_ids[0], date(2026, 10, 6)),
        (at_vichy.line_ids[1], date(2026, 9, 21)),
        (at_vichy.line_ids[2], date(2026, 10, 7)),
        (at_sioule.line_ids[0], date(2026, 10, 8)),
        (at_sioule.line_ids[1], date(2026, 10, 10)),
        (at_sioule.line_ids[2], date(2026, 10, 27)),
        (at_sioule.line_ids[3], date(2026, 10, 29)),
    ] {
        world
            .seed_visit(line, last, InterventionStatus::Validated)
            .await;
    }
    world.plan(&[at_vichy.line_ids[2]], 2026, 43).await.unwrap();

    let recap = world
        .usecase
        .maintenance_recap(
            world.manager.subject(),
            world.organization_id,
            2026,
            week(43),
        )
        .await
        .unwrap();

    let shown: Vec<(Option<&str>, Vec<&str>)> = recap
        .groups
        .iter()
        .map(|group| {
            (
                group.town.as_deref(),
                group.lines.iter().map(|l| l.label.as_str()).collect(),
            )
        })
        .collect();
    assert_eq!(
        shown,
        [
            (Some("Vichy"), vec!["En retard", "Due mardi"]),
            (
                Some("Saint-Pourçain-sur-Sioule"),
                vec!["Due jeudi", "Due samedi"]
            ),
        ]
    );
    assert_eq!(recap.groups[0].lines[0].due.status, DueStatus::Overdue);
    assert_eq!(recap.groups[0].lines[0].site.customer_name, "GAUME");
    assert_eq!(
        recap.groups[0].lines[0].last_done_on,
        Some(date(2026, 9, 21))
    );

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_ent_6_planning_from_the_recap_creates_tasks_and_empties_the_recap() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let site = world.seed_site("GAUME", Some("Vichy")).await;
    let contract = world
        .seed_contract(
            &site,
            (2026, 12, 48),
            &[
                ("Tonte", rolling(14)),
                ("Taille", rolling(14)),
                ("Plus tard", rolling(14)),
            ],
        )
        .await;
    for (line, last) in [
        (contract.line_ids[0], date(2026, 10, 6)),
        (contract.line_ids[1], date(2026, 10, 8)),
        (contract.line_ids[2], date(2026, 10, 27)),
    ] {
        world
            .seed_visit(line, last, InterventionStatus::Validated)
            .await;
    }
    let before = world.recap(2026, 43).await;
    assert_eq!(before.groups[0].lines.len(), 2);

    let tasks = world.plan(&contract.line_ids[..2], 2026, 43).await.unwrap();

    assert_eq!(tasks.len(), 2);
    for (task, line) in tasks.iter().zip(&contract.line_ids) {
        assert_eq!(task.maintenance_line_id, Some(*line));
        assert_eq!(task.project_id.map(|id| id.0), Some(contract.project_id));
        assert_eq!(task.customer_id.map(|id| id.0), Some(site.customer_id));
        assert_eq!(
            task.customer_context_id.map(|id| id.0),
            Some(site.context_id)
        );
        assert_eq!((task.starts_at, task.ends_at), (None, None));
        assert_eq!(task.status, crate::TaskStatus::Backlog);
    }
    assert!(world.recap(2026, 43).await.groups.is_empty());
    let planning_again = world.plan(&contract.line_ids[..1], 2026, 43).await;
    assert!(
        matches!(planning_again, Err(common::CoreError::Conflict(_))),
        "{planning_again:?}"
    );

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn a_finished_task_frees_the_line_for_the_recap_again() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let site = world.seed_site("GAUME", Some("Vichy")).await;
    let contract = world
        .seed_contract(&site, (2026, 12, 48), &[("Tonte", rolling(14))])
        .await;
    world
        .seed_visit(
            contract.line_ids[0],
            date(2026, 10, 6),
            InterventionStatus::Validated,
        )
        .await;
    let tasks = world.plan(&contract.line_ids, 2026, 43).await.unwrap();

    sqlx::query("UPDATE tasks SET status = 'DONE' WHERE id = $1")
        .bind(tasks[0].id.0)
        .execute(&pool)
        .await
        .unwrap();
    let recap = world
        .usecase
        .maintenance_recap(
            world.manager.subject(),
            world.organization_id,
            2026,
            week(43),
        )
        .await
        .unwrap();

    assert_eq!(recap.groups[0].lines[0].label, "Tonte");

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn the_grid_lists_active_contracts_with_their_site_and_leaves_archived_ones_out() {
    let pool = pool().await;
    let world = World::new(&pool).await;
    let site = world.seed_site("GAUME", Some("Vichy")).await;
    let kept = world
        .seed_contract(&site, (2026, 12, 48), &[("Tonte", rolling(14))])
        .await;
    let archived = world
        .seed_contract(&site, (2026, 12, 48), &[("Haie", rolling(14))])
        .await;
    let other_year = world
        .seed_contract(&site, (2027, 12, 48), &[("Tonte", rolling(14))])
        .await;
    sqlx::query("UPDATE maintenance_contracts SET archived_at = now() WHERE id = $1")
        .bind(archived.id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO customer_contacts (id, customer_id, first_name, last_name, phone, is_primary) \
         VALUES (gen_random_uuid(), $1, 'Odette', 'Gaume', '04 70 00 00 00', true)",
    )
    .bind(site.customer_id)
    .execute(&pool)
    .await
    .unwrap();
    world
        .seed_visit(
            kept.line_ids[0],
            date(2026, 10, 6),
            InterventionStatus::Proposed,
        )
        .await;

    let grid = world
        .usecase
        .maintenance_grid(world.manager.subject(), world.organization_id, 2026)
        .await
        .unwrap();

    assert_eq!(grid.contracts.len(), 1);
    let contract = &grid.contracts[0];
    assert_eq!(contract.id.0, kept.id);
    assert_eq!(contract.site.customer_name, "GAUME");
    assert_eq!(contract.site.city.as_deref(), Some("Vichy"));
    assert_eq!(
        contract.site.contact.as_ref().map(|c| c.name.as_str()),
        Some("Odette Gaume")
    );
    assert_eq!(contract.lines[0].visits.len(), 1);
    assert_eq!(contract.lines[0].visits[0].week.get(), 41);
    let _ = (other_year, json!(null));

    world.cleanup().await;
}
