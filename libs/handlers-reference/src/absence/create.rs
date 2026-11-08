use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, NaiveDate, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    AbsenceKind, AbsenceSchedule, AbsenceTypeId, CreateAbsenceCommand, DayPart, MemberId,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    absence::{AbsenceFailure, AbsenceResponse, AbsencesPath, require_member_target},
    require_org_membership,
};

/// Two shapes. The current one names a `type_id`, `start_date`, `end_date`
/// and optional parts (morning to afternoon by default). The original one,
/// still accepted, gives `kind` with `starts_at` and `ends_at`: the absence
/// takes the default type of that kind and is approved at once.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateAbsenceRequest {
    pub member_id: MemberId,
    #[serde(default)]
    pub type_id: Option<AbsenceTypeId>,
    #[serde(default)]
    pub kind: Option<AbsenceKind>,
    #[serde(default)]
    pub start_date: Option<NaiveDate>,
    #[serde(default)]
    pub start_part: Option<DayPart>,
    #[serde(default)]
    pub end_date: Option<NaiveDate>,
    #[serde(default)]
    pub end_part: Option<DayPart>,
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub note: Option<String>,
    #[serde(default)]
    pub proof_file_key: Option<String>,
}

impl CreateAbsenceRequest {
    fn schedule(&self) -> Result<AbsenceSchedule, ApiError> {
        let days = self.start_date.is_some()
            || self.end_date.is_some()
            || self.start_part.is_some()
            || self.end_part.is_some();
        let window = self.starts_at.is_some() || self.ends_at.is_some();

        match (days, window) {
            (true, true) => Err(ApiError::Validation(
                "give either start_date and end_date or starts_at and ends_at, not both".to_owned(),
            )),
            (true, false) => match (self.start_date, self.end_date) {
                (Some(start_date), Some(end_date)) => Ok(AbsenceSchedule::Days {
                    start_date,
                    start_part: self.start_part.unwrap_or(DayPart::Morning),
                    end_date,
                    end_part: self.end_part.unwrap_or(DayPart::Afternoon),
                }),
                _ => Err(ApiError::Validation(
                    "start_date and end_date are both required".to_owned(),
                )),
            },
            (false, true) => match (self.starts_at, self.ends_at) {
                (Some(starts_at), Some(ends_at)) => Ok(AbsenceSchedule::Window {
                    starts_at,
                    ends_at,
                    all_day: self.all_day,
                }),
                _ => Err(ApiError::Validation(
                    "starts_at and ends_at are both required".to_owned(),
                )),
            },
            (false, false) => Err(ApiError::Validation(
                "start_date and end_date are required".to_owned(),
            )),
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/absences",
    operation_id = "createAbsence",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateAbsenceRequest,
    responses(
        (status = 201, description = "Absence created, PENDING when its type needs approval", body = inline(DataEnvelope<AbsenceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Member or absence type not found in the organization"),
        (status = 409, description = "Absence conflict"),
        (status = 422, description = "The absence is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: AbsencesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateAbsenceRequest>,
) -> Result<Response<AbsenceResponse>, AbsenceFailure> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    require_member_target(&state, path.organization_id, payload.member_id).await?;
    let schedule = payload.schedule()?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let absence = state
        .usecase
        .acting_as(user_id)
        .create_absence(CreateAbsenceCommand {
            actor,
            organization_id: path.organization_id,
            member_id: payload.member_id,
            kind: payload.kind,
            type_id: payload.type_id,
            schedule,
            note: payload.note,
            proof_file_key: payload.proof_file_key,
        })
        .await
        .map_err(ApiError::from)??;

    Ok(Response::Created(absence.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(value: serde_json::Value) -> CreateAbsenceRequest {
        serde_json::from_value(value).expect("payload must deserialize")
    }

    const MEMBER: &str = "33333333-3333-3333-3333-333333333333";

    #[test]
    fn the_original_shape_maps_to_a_window() {
        let request = parse(json!({
            "member_id": MEMBER,
            "kind": "LEAVE",
            "starts_at": "2026-07-06T08:00:00Z",
            "ends_at": "2026-07-06T10:00:00Z",
            "all_day": true,
        }));

        assert!(matches!(
            request.schedule().unwrap(),
            AbsenceSchedule::Window { all_day: true, .. }
        ));
    }

    #[test]
    fn the_new_shape_defaults_to_whole_days() {
        let request = parse(json!({
            "member_id": MEMBER,
            "type_id": "44444444-4444-4444-4444-444444444444",
            "start_date": "2026-07-06",
            "end_date": "2026-07-07",
        }));

        assert!(matches!(
            request.schedule().unwrap(),
            AbsenceSchedule::Days {
                start_part: DayPart::Morning,
                end_part: DayPart::Afternoon,
                ..
            }
        ));
    }

    #[test]
    fn mixing_both_shapes_or_giving_half_of_one_is_refused() {
        for body in [
            json!({"member_id": MEMBER, "start_date": "2026-07-06", "end_date": "2026-07-06",
                   "starts_at": "2026-07-06T08:00:00Z", "ends_at": "2026-07-06T10:00:00Z"}),
            json!({"member_id": MEMBER, "start_date": "2026-07-06"}),
            json!({"member_id": MEMBER, "starts_at": "2026-07-06T08:00:00Z"}),
            json!({"member_id": MEMBER}),
        ] {
            assert!(matches!(
                parse(body).schedule(),
                Err(ApiError::Validation(_))
            ));
        }
    }
}
