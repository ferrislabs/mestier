//! End-to-end tests for the reporting API.
//!
//! Real socket, real database, real auth. The fixture plans, and clocks nothing
//! that counts: a three-hour task with 45 € of travel on a project quoted at
//! 4 200 €, a two-hour meeting with two people on an internal project, and nine
//! hours of pointage that must not move any number.
//!
//! The two people are costed on different bases on purpose: one at 35 €/h, one
//! salaried at 3 500 € a month on a 35 h contract, which is 23,08 € an hour. A
//! salaried person used to cost 0,00 € here, so every figure below is one this
//! file states rather than merely accepts.
//!
//! The customer project also carries a confirmed supplier invoice: one line,
//! 200,00 € net at 20 % VAT, fully allocated to it (#338). The fixture's
//! organization has no VAT status on file, the same as a franchise, so the
//! real cost is the grossed-up figure — 240,00 €, not the 200,00 € net.
//!
//! ```bash
//! docker compose up -d postgres redis
//! source .env
//! cargo test -p handlers-reporting --test http_e2e -- --ignored
//! ```

mod harness;
mod issuer;

use chrono::Utc;

fn period() -> String {
    let today = Utc::now().date_naive();
    let from = today - chrono::Duration::days(2);
    format!("from={from}&to={today}")
}

fn project_named(body: &serde_json::Value, project_id: uuid::Uuid) -> Option<&serde_json::Value> {
    body["data"]["projects"]
        .as_array()
        .expect("the report carries projects")
        .iter()
        .find(|project| project["project_id"] == serde_json::json!(project_id))
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_planned_project_is_costed_and_compared_to_its_quote() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let anonymous = client
        .get(format!("{}?{}", app.url("/profitability"), period()))
        .send()
        .await
        .expect("the api answers an unauthenticated call");
    assert_eq!(
        anonymous.status(),
        401,
        "the auth middleware must be in the chain"
    );

    let raw = client
        .get(format!("{}?{}", app.url("/profitability"), period()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the profitability call")
        .text()
        .await
        .expect("the answer has a body");
    let body: serde_json::Value = serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("the profitability answer is json: {e}: {raw}"));

    let project = project_named(&body, app.project_id)
        .unwrap_or_else(|| panic!("the planned project is missing from {body}"));

    // Three hours at 35 euros an hour, and no equipment on the fixture. Nine
    // hours are clocked on this very task and change nothing.
    assert_eq!(
        project["planned_minutes"],
        serde_json::json!(180),
        "{project}"
    );
    assert_eq!(
        project["labour_cost_cents"],
        serde_json::json!(10_500),
        "{project}"
    );
    assert_eq!(
        project["equipment_cost_cents"],
        serde_json::json!(0),
        "{project}"
    );
    assert_eq!(
        project["expenses_cents"],
        serde_json::json!(4_500),
        "{project}"
    );
    // 200,00 € net at 20 % VAT, grossed up because this organization has no
    // VAT status on file and so cannot recover it (#338).
    assert_eq!(
        project["supplier_cost_cents"],
        serde_json::json!(24_000),
        "{project}"
    );
    assert_eq!(
        project["quoted_cents"],
        serde_json::json!(420_000),
        "{project}"
    );
    assert_eq!(
        project["margin_cents"],
        serde_json::json!(420_000 - 10_500 - 4_500 - 24_000),
        "the margin is the quote less labour, expenses and the grossed-up supplier cost: {project}"
    );
    assert_eq!(
        project["overlapping_minutes"],
        serde_json::json!(0),
        "nobody is double booked here: {project}"
    );

    app.cleanup().await;
}

/// The headline of the change. A meeting bills nobody, has no quote, and cost
/// six person-hours that the clocked model reported as nothing at all.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn an_internal_project_is_costed_even_though_it_bills_nobody() {
    let app = harness::start().await;

    let body: serde_json::Value = reqwest::Client::new()
        .get(format!("{}?{}", app.url("/profitability"), period()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the profitability call")
        .json()
        .await
        .expect("the answer is json");

    let meeting = project_named(&body, app.internal_project_id)
        .unwrap_or_else(|| panic!("the internal project is missing from {body}"));

    // Two hours, two people: 240 planned minutes, 120 occupied.
    assert_eq!(
        meeting["planned_minutes"],
        serde_json::json!(240),
        "{meeting}"
    );
    assert_eq!(
        meeting["occupied_minutes"],
        serde_json::json!(120),
        "{meeting}"
    );
    assert_eq!(
        meeting["labour_cost_cents"],
        // Two hours each: 70,00 € for the hourly person, 46,16 € for the salaried
        // one. That second figure was 0,00 € before the monthly cost existed.
        serde_json::json!(7_000 + 4_616),
        "a salaried attendee costs their derived rate, not nothing: {meeting}"
    );
    assert_eq!(
        meeting["members_without_rate"],
        serde_json::json!([]),
        "both cost bases are stateable, so nothing is withheld: {meeting}"
    );
    assert_eq!(
        meeting["customer_id"],
        serde_json::Value::Null,
        "an internal project has no customer: {meeting}"
    );
    assert_eq!(
        meeting["margin_cents"],
        serde_json::Value::Null,
        "no quote means no margin, not a margin of zero: {meeting}"
    );

    // A project whose work sits outside the period is absent rather than listed
    // at zero.
    assert!(
        project_named(&body, app.stale_project_id).is_none(),
        "a project with nothing planned in the period should not be reported: {body}"
    );

    // Ranked server-side, so every screen agrees on what "most profitable" is.
    assert!(
        !body["data"]["most_profitable"]
            .as_array()
            .expect("a ranking")
            .is_empty(),
        "{body}"
    );

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn worked_hours_reports_the_same_time_the_costing_used() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let body: serde_json::Value = client
        .get(format!("{}?{}", app.url("/worked-hours"), period()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the worked-hours call")
        .json()
        .await
        .expect("the answer is json");

    // 180 on the chantier plus 240 across the meeting's two attendees. The nine
    // clocked hours are not in there, and that is the point: payroll and costing
    // read the same plan.
    assert_eq!(
        body["data"]["total_planned_minutes"],
        serde_json::json!(420),
        "payroll and costing must not disagree about the same week: {body}"
    );

    app.cleanup().await;
}

/// The regression #301 exists to close: a raise entered today must not
/// change what a task planned before it already cost.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_raise_does_not_change_the_cost_of_a_task_already_planned() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let before: serde_json::Value = client
        .get(format!("{}?{}", app.url("/profitability"), period()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the profitability call")
        .json()
        .await
        .expect("the answer is json");
    let labour_before = project_named(&before, app.project_id)
        .unwrap_or_else(|| panic!("the planned project is missing from {before}"))["labour_cost_cents"]
        .clone();
    assert_eq!(
        labour_before,
        serde_json::json!(10_500),
        "the fixture's stated figure, before any raise"
    );

    harness::raise(&app.pool, app.organization_id, app.employee_id, 5_000).await;

    let after: serde_json::Value = client
        .get(format!("{}?{}", app.url("/profitability"), period()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the profitability call")
        .json()
        .await
        .expect("the answer is json");
    let labour_after = &project_named(&after, app.project_id)
        .unwrap_or_else(|| panic!("the planned project is missing from {after}"))["labour_cost_cents"];
    assert_eq!(
        labour_after, &labour_before,
        "the task was planned before the raise took effect, so its cost must not move: {after}"
    );

    app.cleanup().await;
}

/// #306: redaction, not refusal. A member with `VIEW_REPORTS` but not
/// `VIEW_COST` reads the same period as the owner, with every money field
/// null and the flag naming why, while the planning facts survive.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_view_cost_reads_the_same_period_with_costs_redacted() {
    let app = harness::start().await;
    let client = reqwest::Client::new();

    let full: serde_json::Value = client
        .get(format!("{}?{}", app.url("/profitability"), period()))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the owner's profitability call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(full["data"]["costs_redacted"], serde_json::json!(false));
    let full_project = project_named(&full, app.project_id)
        .unwrap_or_else(|| panic!("the planned project is missing from {full}"));
    assert_eq!(
        full_project["labour_cost_cents"],
        serde_json::json!(10_500),
        "the owner reads the real figure: {full_project}"
    );

    let redacted: serde_json::Value = client
        .get(format!("{}?{}", app.url("/profitability"), period()))
        .bearer_auth(&app.restricted_token)
        .send()
        .await
        .expect("the api answers the restricted member's profitability call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(
        redacted["data"]["costs_redacted"],
        serde_json::json!(true),
        "{redacted}"
    );
    let redacted_project = project_named(&redacted, app.project_id)
        .unwrap_or_else(|| panic!("the planned project is missing from {redacted}"));
    for field in [
        "labour_cost_cents",
        "equipment_cost_cents",
        "expenses_cents",
        "supplier_cost_cents",
        "quoted_cents",
        "margin_cents",
    ] {
        assert_eq!(
            redacted_project[field],
            serde_json::Value::Null,
            "{field} must be redacted: {redacted_project}"
        );
    }
    // Planning facts survive redaction — they are not payroll.
    assert_eq!(
        redacted_project["planned_minutes"],
        serde_json::json!(180),
        "{redacted_project}"
    );
    assert_eq!(
        redacted_project["overlapping_minutes"],
        serde_json::json!(0),
        "{redacted_project}"
    );

    // A margin ranking is itself a leak of relative cost — the whole list
    // is withheld, not repopulated with redacted-shape entries.
    assert_eq!(
        redacted["data"]["most_profitable"],
        serde_json::json!([]),
        "{redacted}"
    );
    assert_eq!(
        redacted["data"]["least_profitable"],
        serde_json::json!([]),
        "{redacted}"
    );

    // The worked-hours read is redacted by the same code, but has no money
    // field to null in the first place — the flag still reflects the
    // caller's own VIEW_COST status.
    let worked_hours: serde_json::Value = client
        .get(format!("{}?{}", app.url("/worked-hours"), period()))
        .bearer_auth(&app.restricted_token)
        .send()
        .await
        .expect("the api answers the restricted member's worked-hours call")
        .json()
        .await
        .expect("the answer is json");
    assert_eq!(
        worked_hours["data"]["costs_redacted"],
        serde_json::json!(true),
        "{worked_hours}"
    );
    assert_eq!(
        worked_hours["data"]["total_planned_minutes"],
        serde_json::json!(420),
        "planning minutes are not payroll and must not be withheld: {worked_hours}"
    );

    app.cleanup().await;
}

/// `VIEW_REPORTS` gates the endpoint outright — a member with no role at
/// all (so no bit whatsoever) is refused before any redaction question
/// even arises.
#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_member_without_view_reports_is_refused_outright() {
    let app = harness::start().await;

    let refused = reqwest::Client::new()
        .get(format!("{}?{}", app.url("/profitability"), period()))
        .bearer_auth(&app.no_role_token)
        .send()
        .await
        .expect("the api answers the no-role member's profitability call");
    assert_eq!(
        refused.status(),
        403,
        "membership alone is no longer enough to read a report"
    );

    let refused_worked_hours = reqwest::Client::new()
        .get(format!("{}?{}", app.url("/worked-hours"), period()))
        .bearer_auth(&app.no_role_token)
        .send()
        .await
        .expect("the api answers the no-role member's worked-hours call");
    assert_eq!(refused_worked_hours.status(), 403);

    app.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres and redis"]
async fn a_period_that_ends_before_it_starts_is_refused() {
    let app = harness::start().await;

    let answer = reqwest::Client::new()
        .get(format!(
            "{}?from=2026-08-20&to=2026-08-01",
            app.url("/profitability")
        ))
        .bearer_auth(&app.token)
        .send()
        .await
        .expect("the api answers the call");

    assert_eq!(answer.status(), 409);

    app.cleanup().await;
}
