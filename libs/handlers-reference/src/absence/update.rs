use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{AbsenceKind, PatchAbsenceCommand};
use serde::{Deserialize, Deserializer};
use utoipa::ToSchema;

use crate::absence::{AbsencePath, AbsenceResponse, require_absence};

/// Distinguishes "the key is absent" (leave the field unchanged) from "the
/// key is present" (apply it, `null` included) — plain `Option<Option<T>>`
/// cannot make that distinction on its own because serde treats a missing
/// key and an explicit `null` the same way by default.
fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

/// Every field is optional and, when present, replaces the current value —
/// `note` additionally distinguishes "absent" from "present but `null`"
/// (see [`deserialize_present`]). `employee_id` is not patchable: an
/// absence's owner is fixed at creation.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateAbsenceRequest {
    #[serde(default)]
    pub kind: Option<AbsenceKind>,
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub all_day: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<String>, nullable)]
    pub note: Option<Option<String>>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/absences/{absence_id}",
    operation_id = "patchAbsence",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("absence_id" = mestier_core::EmployeeAbsenceId, Path, description = "Absence identifier"),
    ),
    request_body = UpdateAbsenceRequest,
    responses(
        (status = 200, description = "Absence updated", body = inline(DataEnvelope<AbsenceResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Absence not found"),
        (status = 409, description = "Absence conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    AbsencePath {
        organization_id,
        absence_id,
    }: AbsencePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateAbsenceRequest>,
) -> Result<Response<AbsenceResponse>, ApiError> {
    require_absence(&state, &identity, organization_id, absence_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let mut command = PatchAbsenceCommand::new(absence_id);
    command.kind = payload.kind;
    command.starts_at = payload.starts_at;
    command.ends_at = payload.ends_at;
    command.all_day = payload.all_day;
    command.note = payload.note;

    let absence = state
        .usecase
        .acting_as(actor)
        .patch_absence(command)
        .await?;

    Ok(Response::OK(absence.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn parse(value: serde_json::Value) -> UpdateAbsenceRequest {
        serde_json::from_value(value).expect("payload must deserialize")
    }

    #[test]
    fn absent_note_leaves_it_unset() {
        let request = parse(json!({}));

        assert_eq!(
            request.note, None,
            "an absent `note` key must not be mistaken for a clearing `null`"
        );
    }

    #[test]
    fn null_note_clears_it() {
        let request = parse(json!({ "note": null }));

        assert_eq!(
            request.note,
            Some(None),
            "an explicit `null` must be distinguishable from an absent key: it clears the note"
        );
    }

    #[test]
    fn present_note_sets_the_new_value() {
        let request = parse(json!({ "note": "Arrêt maladie" }));

        assert_eq!(request.note, Some(Some("Arrêt maladie".to_owned())));
    }

    #[test]
    fn kind_deserializes_from_screaming_snake_case() {
        let request = parse(json!({ "kind": "SICK" }));

        assert_eq!(request.kind, Some(AbsenceKind::Sick));
    }

    #[test]
    fn unknown_kind_is_rejected_not_ignored() {
        let result: Result<UpdateAbsenceRequest, _> =
            serde_json::from_value(json!({ "kind": "HOLIDAY" }));

        assert!(
            result.is_err(),
            "an unrecognized `kind` must fail to deserialize rather than being silently dropped"
        );
    }

    #[test]
    fn absent_fields_all_default_to_none() {
        let request = parse(json!({}));

        assert_eq!(request.kind, None);
        assert_eq!(request.starts_at, None);
        assert_eq!(request.ends_at, None);
        assert_eq!(request.all_day, None);
    }
}
