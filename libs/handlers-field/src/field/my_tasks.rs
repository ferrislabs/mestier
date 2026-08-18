use auth::Identity;
use axum::{Extension, extract::State};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::OrganizationId;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{paths::FieldTasksPath, resolve_field_actor};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct MyTasksQuery {
    /// The day to list, in the organization's timezone. Defaults to today.
    pub work_date: Option<NaiveDate>,
}

/// The caller's own jobs for a day.
///
/// Scoped to the connected account by construction: the member is resolved
/// from the token, never taken from the request, so this cannot be pointed at
/// a colleague.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/field/tasks",
    operation_id = "listMyFieldTasks",
    tag = crate::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        MyTasksQuery,
    ),
    responses(
        (status = 200, description = "Jobs assigned to the caller", body = inline(DataEnvelope<Vec<crate::response::FieldTaskResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Caller is not an employee of this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldTasksPath { organization_id }: FieldTasksPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    axum::extract::Query(query): axum::extract::Query<MyTasksQuery>,
) -> Result<Response<Vec<crate::response::FieldTaskResponse>>, ApiError> {
    let actor = resolve_field_actor(&state, &identity, organization_id).await?;

    // The default day is resolved in the use case, which knows the
    // organization's timezone. Defaulting here would use the UTC date and hand
    // a late-shift worker yesterday's jobs.
    let tasks = state
        .usecase
        .list_tasks_assigned_to_member_on(organization_id, actor.member_id, query.work_date)
        .await?;

    Ok(Response::OK(tasks.into_iter().map(Into::into).collect()))
}
