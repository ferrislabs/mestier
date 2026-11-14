use std::{collections::BTreeMap, num::NonZeroU32};

use auth::Identity;
use axum::{
    Extension, Json,
    extract::State,
    response::{IntoResponse, Response as HttpResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CampaignId, SurveyPatch, UpdateSurveyCommand, Visibility};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    request::{deserialize_present, graph, metadata},
    require_org_membership,
    survey::{SurveyPath, SurveyView},
};

#[derive(Debug, Default, Deserialize, ToSchema)]
pub struct UpdateSurveyRequest {
    pub title: Option<String>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<String>, nullable)]
    pub description: Option<Option<String>>,
    pub visibility: Option<Visibility>,
    pub allow_multiple_responses: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<u32>, nullable, minimum = 1)]
    pub response_cap: Option<Option<u32>>,
    pub metadata: Option<BTreeMap<String, String>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<Object>, nullable)]
    pub graph: Option<Option<serde_json::Value>>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<CampaignId>, nullable)]
    pub campaign_id: Option<Option<CampaignId>>,
}

impl UpdateSurveyRequest {
    fn patch(self) -> Result<(SurveyPatch, Option<Option<CampaignId>>), ApiError> {
        let response_cap = self
            .response_cap
            .map(|cap| cap.map(response_cap).transpose())
            .transpose()?;
        let graph = self
            .graph
            .map(|value| value.map(graph).transpose())
            .transpose()?;
        let metadata = self
            .metadata
            .map(|entries| metadata(Some(entries)))
            .transpose()?;
        Ok((
            SurveyPatch {
                title: self.title,
                description: self.description,
                visibility: self.visibility,
                allow_multiple_responses: self.allow_multiple_responses,
                response_cap,
                metadata,
                graph,
            },
            self.campaign_id,
        ))
    }
}

fn response_cap(cap: u32) -> Result<NonZeroU32, ApiError> {
    NonZeroU32::new(cap)
        .ok_or_else(|| ApiError::UnprocessableEntity("response_cap: must be at least 1".to_owned()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/surveys/{survey_id}",
    operation_id = "updateSurvey",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("survey_id" = mestier_core::SurveyId, Path, description = "Survey identifier"),
    ),
    request_body(
        content = UpdateSurveyRequest,
        description = "Every field is optional; an explicit null clears description, response_cap, graph or campaign_id. The graph is the domain form: QUESTION, CONDITION and END nodes, edges with a NEXT, THEN or ELSE branch, and the start question.",
        example = json!({
            "title": "Satisfaction des stagiaires",
            "allow_multiple_responses": false,
            "response_cap": 30,
            "metadata": {"promotion": "2026"},
            "graph": {
                "nodes": [
                    {"type": "QUESTION", "id": "0195f0a0-0000-7000-8000-000000000001", "kind": "SINGLE_CHOICE", "label": "Le stage vous a-t-il plu ?", "required": true, "options": ["Oui", "Non"]},
                    {"type": "CONDITION", "id": "0195f0a0-0000-7000-8000-000000000002", "question": "0195f0a0-0000-7000-8000-000000000001", "option": "Non", "checked": true},
                    {"type": "QUESTION", "id": "0195f0a0-0000-7000-8000-000000000003", "kind": "LONG_TEXT", "label": "Pourquoi ?", "required": false},
                    {"type": "END", "id": "0195f0a0-0000-7000-8000-000000000004", "message": "Merci !"}
                ],
                "edges": [
                    {"from": "0195f0a0-0000-7000-8000-000000000001", "to": "0195f0a0-0000-7000-8000-000000000002", "branch": "NEXT"},
                    {"from": "0195f0a0-0000-7000-8000-000000000002", "to": "0195f0a0-0000-7000-8000-000000000003", "branch": "THEN"},
                    {"from": "0195f0a0-0000-7000-8000-000000000002", "to": "0195f0a0-0000-7000-8000-000000000004", "branch": "ELSE"},
                    {"from": "0195f0a0-0000-7000-8000-000000000003", "to": "0195f0a0-0000-7000-8000-000000000004", "branch": "NEXT"}
                ],
                "start": "0195f0a0-0000-7000-8000-000000000001"
            }
        })
    ),
    responses(
        (status = 200, description = "Draft survey updated", body = inline(DataEnvelope<SurveyView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Survey or campaign not found, or surveys not enabled"),
        (status = 409, description = "The survey is not a draft, or changed meanwhile"),
        (status = 422, description = "Blank title, invalid metadata, response cap below 1, or a graph that breaks a structure rule"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    SurveyPath {
        organization_id,
        survey_id,
    }: SurveyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateSurveyRequest>,
) -> Result<HttpResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (patch, campaign_id) = payload.patch()?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let survey = state
        .usecase
        .acting_as(user_id)
        .update_survey(UpdateSurveyCommand {
            actor,
            organization_id,
            id: survey_id,
            patch,
            campaign_id,
        })
        .await?;

    Ok(Response::OK(SurveyView::draft(&survey)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(body: serde_json::Value) -> UpdateSurveyRequest {
        serde_json::from_value(body).unwrap_or_default()
    }

    #[test]
    fn an_absent_field_is_left_alone_and_a_null_clears_it() {
        let (patch, campaign_id) =
            read(serde_json::json!({"description": null, "campaign_id": null}))
                .patch()
                .ok()
                .unwrap_or_default();
        assert_eq!(patch.description, Some(None));
        assert_eq!(campaign_id, Some(None));
        assert_eq!(patch.title, None);
        assert_eq!(patch.graph, None);
        assert_eq!(patch.response_cap, None);
    }

    #[test]
    fn a_response_cap_of_zero_is_unprocessable() {
        let refused = read(serde_json::json!({"response_cap": 0})).patch();
        assert!(matches!(refused, Err(ApiError::UnprocessableEntity(_))));
    }

    #[test]
    fn a_graph_with_a_cycle_is_unprocessable() {
        let question = "0195f0a0-0000-7000-8000-000000000001";
        let other = "0195f0a0-0000-7000-8000-000000000002";
        let end = "0195f0a0-0000-7000-8000-000000000003";
        let refused = read(serde_json::json!({"graph": {
            "nodes": [
                {"type": "QUESTION", "id": question, "kind": "TEXT", "label": "A", "required": false},
                {"type": "QUESTION", "id": other, "kind": "TEXT", "label": "B", "required": false},
                {"type": "END", "id": end}
            ],
            "edges": [
                {"from": question, "to": other, "branch": "NEXT"},
                {"from": other, "to": question, "branch": "NEXT"}
            ],
            "start": question
        }}))
        .patch();
        assert!(matches!(refused, Err(ApiError::UnprocessableEntity(_))));
    }
}
