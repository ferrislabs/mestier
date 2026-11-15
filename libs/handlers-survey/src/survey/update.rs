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
#[serde(deny_unknown_fields)]
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
    #[schema(value_type = Option<crate::schema::SurveyGraphSchema>, nullable)]
    pub graph: Option<Option<serde_json::Value>>,
    #[schema(nullable)]
    pub campaign_ids: Option<Vec<CampaignId>>,
}

impl UpdateSurveyRequest {
    fn patch(self) -> Result<(SurveyPatch, Option<Vec<CampaignId>>), ApiError> {
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
            self.campaign_ids,
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
    description = "Updates a survey in any state: an opened or closed survey stays editable and keeps its state. Its visibility is frozen once opened (409 survey_visibility_frozen) and its graph cannot be cleared. A graph version is frozen once a response was completed on it: a graph changed on a frozen version gets a new graph_version, otherwise it is replaced in place. Unknown body fields are refused with 422. The body shape (graph, metadata, response cap) is checked before the permission, so a caller without survey.manage who sends an invalid body gets 422 rather than 403.",
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("survey_id" = mestier_core::SurveyId, Path, description = "Survey identifier"),
    ),
    request_body(
        content = UpdateSurveyRequest,
        description = "Every field is optional; an explicit null clears description, response_cap or graph (a draft only). campaign_ids absent or null leaves the campaigns unchanged; an array, even empty, replaces the whole set, in attachment order. An unknown field (such as the former campaign_id) is refused with 422. The graph is the domain form: QUESTION, CONDITION and END nodes, edges with a NEXT, THEN or ELSE branch, and the start question. A QUESTION node may carry a block (uuid, null or absent outside a block): the questions sharing a block are shown on one screen, so they form a chain of NEXT edges entered only through its first question, a block has at least two questions, and no condition sits inside it or depends on a question of the block it guards (422 graph_block_too_small, graph_block_not_contiguous, graph_block_member_has_condition, with the question_id). A QUESTION node may carry a description (free text, null or absent when none) shown under its label: it is trimmed, a blank one is stored as null, and it holds at most 500 characters (422 graph_question_description_too_long, with the question_id).",
        example = json!({
            "title": "Satisfaction des stagiaires",
            "allow_multiple_responses": false,
            "response_cap": 30,
            "metadata": {"promotion": "2026"},
            "graph": {
                "nodes": [
                    {"type": "QUESTION", "id": "0195f0a0-0000-7000-8000-000000000001", "kind": "SINGLE_CHOICE", "label": "Le stage vous a-t-il plu ?", "required": true, "options": ["Oui", "Non"], "block": null, "description": "Pensez à l'accueil, aux missions et à l'encadrement."},
                    {"type": "CONDITION", "id": "0195f0a0-0000-7000-8000-000000000002", "question": "0195f0a0-0000-7000-8000-000000000001", "option": "Non", "checked": true},
                    {"type": "QUESTION", "id": "0195f0a0-0000-7000-8000-000000000003", "kind": "LONG_TEXT", "label": "Pourquoi ?", "required": false, "block": "0195f0a0-0000-7000-8000-0000000000b1"},
                    {"type": "QUESTION", "id": "0195f0a0-0000-7000-8000-000000000005", "kind": "LONG_TEXT", "label": "Que faudrait-il changer ?", "required": false, "block": "0195f0a0-0000-7000-8000-0000000000b1"},
                    {"type": "END", "id": "0195f0a0-0000-7000-8000-000000000004", "message": "Merci !"}
                ],
                "edges": [
                    {"from": "0195f0a0-0000-7000-8000-000000000001", "to": "0195f0a0-0000-7000-8000-000000000002", "branch": "NEXT"},
                    {"from": "0195f0a0-0000-7000-8000-000000000002", "to": "0195f0a0-0000-7000-8000-000000000003", "branch": "THEN"},
                    {"from": "0195f0a0-0000-7000-8000-000000000002", "to": "0195f0a0-0000-7000-8000-000000000004", "branch": "ELSE"},
                    {"from": "0195f0a0-0000-7000-8000-000000000003", "to": "0195f0a0-0000-7000-8000-000000000005", "branch": "NEXT"},
                    {"from": "0195f0a0-0000-7000-8000-000000000005", "to": "0195f0a0-0000-7000-8000-000000000004", "branch": "NEXT"}
                ],
                "start": "0195f0a0-0000-7000-8000-000000000001"
            }
        })
    ),
    responses(
        (status = 200, description = "Survey updated", body = inline(DataEnvelope<SurveyView>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Requires survey.manage"),
        (status = 404, description = "Survey or campaign not found, or surveys not enabled"),
        (status = 409, description = "The visibility of an opened survey cannot change, an opened survey cannot lose its graph, or the survey changed meanwhile"),
        (status = 422, description = "Blank title, invalid metadata, response cap below 1, a graph that breaks a structure rule, or an unknown field"),
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
    let (patch, campaign_ids) = payload.patch()?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let survey = state
        .usecase
        .acting_as(user_id)
        .update_survey(UpdateSurveyCommand {
            actor,
            organization_id,
            id: survey_id,
            patch,
            campaign_ids,
        })
        .await?;

    Ok(Response::OK(SurveyView::from(&survey)).into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(body: serde_json::Value) -> UpdateSurveyRequest {
        serde_json::from_value(body).unwrap_or_default()
    }

    #[test]
    fn an_absent_field_is_left_alone_and_a_null_clears_it() {
        let (patch, campaign_ids) = read(serde_json::json!({"description": null}))
            .patch()
            .ok()
            .unwrap_or_default();
        assert_eq!(patch.description, Some(None));
        assert_eq!(campaign_ids, None);
        assert_eq!(patch.title, None);
        assert_eq!(patch.graph, None);
        assert_eq!(patch.response_cap, None);
    }

    #[test]
    fn campaign_ids_replace_the_campaigns_in_order() {
        let first = "0195f0a0-0000-7000-8000-000000000001";
        let second = "0195f0a0-0000-7000-8000-000000000002";
        let (_, campaign_ids) = read(serde_json::json!({"campaign_ids": [second, first]}))
            .patch()
            .ok()
            .unwrap_or_default();
        assert_eq!(
            campaign_ids,
            Some(vec![
                CampaignId(second.parse().unwrap_or_default()),
                CampaignId(first.parse().unwrap_or_default()),
            ])
        );
    }

    #[test]
    fn the_former_single_campaign_field_is_refused() {
        let refused = serde_json::from_value::<UpdateSurveyRequest>(serde_json::json!({
            "campaign_id": "0195f0a0-0000-7000-8000-000000000001"
        }));
        let Err(error) = refused else {
            panic!("an unknown field must be refused");
        };
        assert!(error.to_string().contains("campaign_id"), "{error}");
    }

    #[test]
    fn a_null_campaign_set_is_left_alone() {
        let (_, campaign_ids) = read(serde_json::json!({"campaign_ids": null}))
            .patch()
            .ok()
            .unwrap_or_default();
        assert_eq!(campaign_ids, None);
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
        assert!(matches!(
            refused,
            Err(ApiError::Refused(common::Refusal {
                kind: common::RefusalKind::Validation,
                reason: "graph_cycle",
                ..
            }))
        ));
    }

    #[test]
    fn a_description_over_five_hundred_characters_is_refused_with_its_question() {
        let question = "0195f0a0-0000-7000-8000-000000000001";
        let end = "0195f0a0-0000-7000-8000-000000000003";
        let refused = read(serde_json::json!({"graph": {
            "nodes": [
                {"type": "QUESTION", "id": question, "kind": "TEXT", "label": "A", "required": false, "description": "x".repeat(501)},
                {"type": "END", "id": end}
            ],
            "edges": [{"from": question, "to": end, "branch": "NEXT"}],
            "start": question
        }}))
        .patch();
        let Err(ApiError::Refused(refusal)) = refused else {
            panic!("a description over 500 characters must be refused");
        };
        assert_eq!(refusal.reason, "graph_question_description_too_long");
        assert_eq!(refusal.kind, common::RefusalKind::Validation);
        assert_eq!(refusal.question_id, question.parse().ok());
    }

    #[test]
    fn a_block_of_one_question_is_unprocessable_and_a_block_of_two_is_read() {
        let question = "0195f0a0-0000-7000-8000-000000000001";
        let other = "0195f0a0-0000-7000-8000-000000000002";
        let end = "0195f0a0-0000-7000-8000-000000000003";
        let block = "0195f0a0-0000-7000-8000-0000000000b1";
        let graph = |second: serde_json::Value| {
            serde_json::json!({"graph": {
                "nodes": [
                    {"type": "QUESTION", "id": question, "kind": "TEXT", "label": "A", "required": false, "block": block},
                    {"type": "QUESTION", "id": other, "kind": "TEXT", "label": "B", "required": false, "block": second},
                    {"type": "END", "id": end}
                ],
                "edges": [
                    {"from": question, "to": other, "branch": "NEXT"},
                    {"from": other, "to": end, "branch": "NEXT"}
                ],
                "start": question
            }})
        };
        let refused = read(graph(serde_json::Value::Null)).patch();
        let Err(ApiError::Refused(refusal)) = refused else {
            panic!("a block of one question must be refused");
        };
        assert_eq!(refusal.reason, "graph_block_too_small");
        assert_eq!(refusal.kind, common::RefusalKind::Validation);
        assert_eq!(refusal.question_id, question.parse().ok());
        let accepted = read(graph(serde_json::json!(block))).patch();
        assert!(matches!(
            accepted,
            Ok((
                SurveyPatch {
                    graph: Some(Some(_)),
                    ..
                },
                _
            ))
        ));
    }
}
