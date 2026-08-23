use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::application::policy;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::cost_basis::{EmployeeCostBasisPath, EmployeeCostBasisResponse};

/// Corrects a version that was entered wrong — the dangerous verb, because
/// unlike the `POST` above it rewrites history on purpose rather than dating
/// a new change. Every field can be corrected, `effective_from`/
/// `effective_to` included: a typo in the date is exactly the kind of
/// mistake this exists to fix.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CorrectEmployeeCostBasisRequest {
    pub effective_from: NaiveDate,
    #[serde(default)]
    pub effective_to: Option<NaiveDate>,
    #[serde(default)]
    pub hourly_rate_cents: Option<i32>,
    pub is_salaried: bool,
    #[serde(default)]
    pub monthly_cost_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
}

#[utoipa::path(
    patch,
    path = "/api/v1/cost-bases/{cost_basis_id}",
    operation_id = "correctEmployeeCostBasis",
    tag = super::super::TAG,
    params(
        ("cost_basis_id" = mestier_core::EmployeeCostBasisId, Path, description = "Cost basis version identifier"),
    ),
    request_body = CorrectEmployeeCostBasisRequest,
    responses(
        (status = 200, description = "Cost basis version corrected", body = inline(DataEnvelope<EmployeeCostBasisResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Cost basis version not found"),
        (status = 409, description = "The correction would overlap another version"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    EmployeeCostBasisPath { cost_basis_id }: EmployeeCostBasisPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CorrectEmployeeCostBasisRequest>,
) -> Result<Response<EmployeeCostBasisResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;
    // TODO: thread JWT realm roles once Identity exposes them.
    let actor = policy::user_subject(user_id, Vec::new());

    let basis = state
        .usecase
        .acting_as(user_id)
        .correct_employee_cost_basis(
            actor,
            cost_basis_id,
            payload.effective_from,
            payload.effective_to,
            payload.is_salaried,
            payload.hourly_rate_cents,
            payload.monthly_cost_cents,
            payload.weekly_contract_minutes,
        )
        .await?;

    Ok(Response::OK(basis.into()))
}
