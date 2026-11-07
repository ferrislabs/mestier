use auth::Identity;
use axum::{Extension, body::Bytes, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{DryRunError, DryRunStatus, DryRunStep};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;

use crate::{
    paths::WorkflowDryRunPath,
    require_manage_automation,
    response::{GraphDto, GraphErrorResponse},
    workflow::save_version::{GraphInvalidBody, SaveVersionError},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct DryRunRequest {
    pub graph: GraphDto,
    pub trigger_id: Option<String>,
    #[schema(value_type = Option<Object>)]
    pub trigger_payload: Option<Map<String, Value>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DryRunStepStatus {
    Succeeded,
    Failed,
    Skipped,
}

impl From<DryRunStatus> for DryRunStepStatus {
    fn from(status: DryRunStatus) -> Self {
        match status {
            DryRunStatus::Succeeded => Self::Succeeded,
            DryRunStatus::Failed => Self::Failed,
            DryRunStatus::Skipped => Self::Skipped,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DryRunStepResponse {
    pub connector_id: String,
    pub iteration_path: String,
    pub status: DryRunStepStatus,
    pub input: Option<Value>,
    pub output: Option<Value>,
    pub error: Option<String>,
}

impl From<DryRunStep> for DryRunStepResponse {
    fn from(step: DryRunStep) -> Self {
        Self {
            connector_id: step.connector_id,
            iteration_path: step.iteration_path,
            status: step.status.into(),
            input: step.input,
            output: step.output,
            error: step.error,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DryRunResponse {
    pub steps: Vec<DryRunStepResponse>,
}

fn refusal(error: DryRunError) -> SaveVersionError {
    match error {
        DryRunError::InvalidGraph(errors) => {
            SaveVersionError::GraphInvalid(errors.iter().map(GraphErrorResponse::from).collect())
        }
        DryRunError::Core(error) => error.into(),
        error @ (DryRunError::UnknownTrigger { .. }
        | DryRunError::Oversized(_)
        | DryRunError::TooManySteps { .. }
        | DryRunError::TooLarge { .. }
        | DryRunError::TimedOut) => ApiError::UnprocessableEntity(error.to_string()).into(),
    }
}

fn parse(body: &[u8]) -> Result<DryRunRequest, SaveVersionError> {
    serde_json::from_slice(body).map_err(|error| {
        let message = format!("the request body is not a dry run request: {error}");
        if error.is_data() {
            ApiError::UnprocessableEntity(message).into()
        } else {
            ApiError::BadRequest(message).into()
        }
    })
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/workflows/dry-run",
    operation_id = "dryRunWorkflow",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = DryRunRequest,
    responses(
        (status = 200, description = "One step per reachable node, in walk order; nothing was executed or stored", body = inline(DataEnvelope<DryRunResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 413, description = "The request body is over the size limit"),
        (status = 422, description = "E_GRAPH_INVALID with the node and field at fault, or an unknown trigger, or a walk too long or too heavy to run", body = inline(GraphInvalidBody)),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    WorkflowDryRunPath { organization_id }: WorkflowDryRunPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    body: Bytes,
) -> Result<Response<DryRunResponse>, SaveVersionError> {
    require_manage_automation(&state, &identity, organization_id).await?;
    let payload = parse(&body)?;

    let graph: mestier_core::Graph = payload.graph.into();
    let trigger_payload = payload.trigger_payload.map(Value::Object);
    let steps = state
        .usecase
        .dry_run_graph(
            organization_id,
            &graph,
            payload.trigger_id.as_deref(),
            trigger_payload.as_ref(),
        )
        .await
        .map_err(refusal)?;

    Ok(Response::OK(DryRunResponse {
        steps: steps.into_iter().map(DryRunStepResponse::from).collect(),
    }))
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, response::IntoResponse};
    use http::StatusCode;
    use mestier_core::GraphError;
    use serde_json::json;

    use super::*;

    #[test]
    fn a_step_answers_with_its_six_fields_and_a_snake_case_status() {
        let step = DryRunStep {
            connector_id: "c1".to_owned(),
            iteration_path: "l1[2]".to_owned(),
            status: DryRunStatus::Skipped,
            input: None,
            output: Some(json!({ "n": 1 })),
            error: None,
        };

        let wire = serde_json::to_value(DryRunStepResponse::from(step)).unwrap();

        assert_eq!(
            wire,
            json!({
                "connector_id": "c1",
                "iteration_path": "l1[2]",
                "status": "skipped",
                "input": null,
                "output": { "n": 1 },
                "error": null
            })
        );
        assert_eq!(
            serde_json::to_value(DryRunStepStatus::from(DryRunStatus::Succeeded)).unwrap(),
            "succeeded"
        );
        assert_eq!(
            serde_json::to_value(DryRunStepStatus::from(DryRunStatus::Failed)).unwrap(),
            "failed"
        );
    }

    #[test]
    fn the_request_accepts_a_graph_alone_and_refuses_a_payload_that_is_not_an_object() {
        let graph = json!({ "connectors": [], "edges": [], "triggers": [] });

        let alone = serde_json::from_value::<DryRunRequest>(json!({ "graph": graph }));
        let scalar = serde_json::from_value::<DryRunRequest>(
            json!({ "graph": graph, "trigger_payload": 42 }),
        );
        let full = serde_json::from_value::<DryRunRequest>(
            json!({ "graph": graph, "trigger_id": "t1", "trigger_payload": { "a": 1 } }),
        );

        let alone = alone.expect("a graph alone is a request");
        assert!(alone.trigger_id.is_none() && alone.trigger_payload.is_none());
        assert!(scalar.is_err());
        let full = full.expect("every field is accepted");
        assert_eq!(full.trigger_id.as_deref(), Some("t1"));
        assert_eq!(full.trigger_payload.unwrap()["a"], 1);
    }

    #[tokio::test]
    async fn a_body_that_is_not_json_is_a_400_and_one_of_the_wrong_shape_a_422() {
        let broken = parse(b"{ not json").unwrap_err().into_response();
        let truncated = parse(b"{\"graph\": ").unwrap_err().into_response();
        let wrong_shape = parse(br#"{ "graph": 42 }"#).unwrap_err().into_response();
        let missing = parse(b"{}").unwrap_err().into_response();

        assert_eq!(broken.status(), StatusCode::BAD_REQUEST);
        assert_eq!(truncated.status(), StatusCode::BAD_REQUEST);
        assert_eq!(wrong_shape.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(missing.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            parse(br#"{ "graph": { "connectors": [], "edges": [], "triggers": [] } }"#).is_ok()
        );
    }

    #[tokio::test]
    async fn an_invalid_graph_answers_the_422_body_a_save_answers() {
        let error = DryRunError::InvalidGraph(vec![GraphError::MissingRequiredField {
            connector_id: "c1".to_owned(),
            field: "predicate".to_owned(),
        }]);

        let response = refusal(error).into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let body: Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(body["code"], "E_GRAPH_INVALID");
        assert_eq!(body["details"]["errors"][0]["connector_id"], "c1");
        assert_eq!(body["details"]["errors"][0]["field"], "predicate");
    }

    #[tokio::test]
    async fn an_unknown_trigger_and_a_walk_too_long_or_too_heavy_answer_422() {
        for error in [
            DryRunError::UnknownTrigger {
                trigger_id: "ghost".to_owned(),
            },
            DryRunError::TooManySteps { limit: 1000 },
            DryRunError::TooLarge { limit: 1 << 24 },
            DryRunError::TimedOut,
            DryRunError::Oversized(mestier_core::DryRunLimit::Nodes),
        ] {
            let response = refusal(error).into_response();

            assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        }
    }

    #[tokio::test]
    async fn a_core_failure_keeps_its_ordinary_mapping() {
        let response = refusal(DryRunError::Core(common::CoreError::NotFound)).into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
