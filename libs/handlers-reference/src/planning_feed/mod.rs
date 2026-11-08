use auth::Identity;
use axum::{
    Extension, Router,
    extract::{Query, State},
};
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    DayOverrideError, FeedMember, HrPlanningFeed, HrPlanningFeedCommand, MemberId, OrganizationId,
    PlanningPeriod,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{
    absence::AbsenceResponse,
    absence_type::AbsenceTypeResponse,
    day_override::{DayOverrideFailure, DayOverrideResponse},
    member_contract::MemberContractResponse,
};

pub const MAX_FEED_DAYS: i64 = 62;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new().typed_get(planning_feed)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/hr/planning-feed")]
pub struct PlanningFeedPath {
    pub organization_id: OrganizationId,
}

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct PlanningFeedQuery {
    #[param(value_type = String, format = Date)]
    pub from: NaiveDate,
    #[param(value_type = String, format = Date)]
    pub to: NaiveDate,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct FeedMemberResponse {
    pub id: MemberId,
    pub last_name: String,
    pub first_name: Option<String>,
    pub display_name: String,
    pub archived: bool,
}

impl From<FeedMember> for FeedMemberResponse {
    fn from(value: FeedMember) -> Self {
        Self {
            id: value.member.id,
            display_name: value.member.display_name(),
            last_name: value.member.last_name,
            first_name: value.member.first_name,
            archived: value.archived,
        }
    }
}

/// Everything the HR planning screen draws for a period, in one answer:
/// members (archived ones flagged), the contracts and absences (every status)
/// that overlap the period, the overrides inside it and the absence types.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct HrPlanningFeedResponse {
    pub members: Vec<FeedMemberResponse>,
    pub contracts: Vec<MemberContractResponse>,
    pub absences: Vec<AbsenceResponse>,
    pub overrides: Vec<DayOverrideResponse>,
    pub absence_types: Vec<AbsenceTypeResponse>,
}

impl From<HrPlanningFeed> for HrPlanningFeedResponse {
    fn from(value: HrPlanningFeed) -> Self {
        Self {
            members: value.members.into_iter().map(Into::into).collect(),
            contracts: value.contracts.into_iter().map(Into::into).collect(),
            absences: value.absences.into_iter().map(Into::into).collect(),
            overrides: value.overrides.into_iter().map(Into::into).collect(),
            absence_types: value.absence_types.into_iter().map(Into::into).collect(),
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/hr/planning-feed",
    operation_id = "getHrPlanningFeed",
    tag = super::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        PlanningFeedQuery,
    ),
    responses(
        (status = 200, description = "The planning data of the period (dates inclusive, at most 62 days)", body = inline(DataEnvelope<HrPlanningFeedResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Needs planning management"),
        (status = 422, description = "The period is inverted or longer than 62 days"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn planning_feed(
    path: PlanningFeedPath,
    Query(query): Query<PlanningFeedQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<HrPlanningFeedResponse>, DayOverrideFailure> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let period =
        PlanningPeriod::new(query.from, query.to, MAX_FEED_DAYS).map_err(DayOverrideError::from)?;
    let feed = state
        .usecase
        .acting_as(user_id)
        .hr_planning_feed(HrPlanningFeedCommand {
            actor,
            organization_id: path.organization_id,
            period,
        })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(feed.into()))
}
