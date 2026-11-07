use chrono::NaiveDate;
use serde_json::{Value, json};

use super::{ActionContext, failure};
use crate::ProfitabilityReport;

#[cfg(test)]
#[path = "tests_reporting.rs"]
mod tests_reporting;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.reporting.profitability" => profitability(context).await,
        "mestier.reporting.worked_hours" => worked_hours(context).await,
        _ => return None,
    })
}

fn day(context: &ActionContext<'_>, name: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(context.text(name)?.trim(), "%Y-%m-%d")
        .map_err(|_| format!("`{name}` must be a date formatted YYYY-MM-DD"))
}

async fn report(context: &ActionContext<'_>) -> Result<ProfitabilityReport, String> {
    let from = day(context, "from")?;
    let to = day(context, "to")?;

    context
        .usecase
        .profitability_report(context.organization_id, from, to)
        .await
        .map_err(failure)
}

async fn profitability(context: &ActionContext<'_>) -> Result<Value, String> {
    let report = report(context).await?;

    serde_json::to_value(&report).map_err(|error| error.to_string())
}

async fn worked_hours(context: &ActionContext<'_>) -> Result<Value, String> {
    let report = report(context).await?;
    let total_planned_minutes: i64 = report
        .members
        .iter()
        .map(|member| member.planned_minutes)
        .sum();

    Ok(json!({
        "members": report
            .members
            .iter()
            .map(|member| json!({
                "member_id": member.member_id.0,
                "planned_minutes": member.planned_minutes,
                "missing_cost": member.missing_cost,
            }))
            .collect::<Vec<_>>(),
        "total_planned_minutes": total_planned_minutes,
    }))
}
