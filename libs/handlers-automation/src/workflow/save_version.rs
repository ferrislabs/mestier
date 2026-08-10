//! `PUT .../workflows/{workflow_id}/versions`: validates the graph and
//! persists it as a new immutable version.
//!
//! `mestier_core::save_workflow_version` — the use case — already validates
//! the graph (#199) before persisting, but it flattens every
//! `mestier_core::GraphError` into one joined string
//! (`CoreError::Conflict`), because `#[transactional]` fixes the use case's
//! error type. A joined string cannot put a message on the field that is
//! actually wrong. So this handler calls `mestier_core::validate_graph`
//! itself first, to build a structured `{ connector_id, field, message }[]`
//! response, and only then calls the use case — which re-validates as an
//! authoritative guard (no I/O in `validate_graph`, so running it twice
//! costs nothing, and a use case must never trust that its caller already
//! checked). Because the two paths can disagree in principle (a race, a
//! catalogue changed between calls), the use case's own refusal still has
//! to produce *a* response — [`SaveVersionError::Api`] is that fallback,
//! deliberately duller than the structured one this handler exists to give.

use auth::Identity;
use axum::{
    Extension, Json,
    extract::State,
    response::{IntoResponse, Response as AxumResponse},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use http::StatusCode;
use mestier_core::{SaveWorkflowVersionCommand, connector_catalogue, validate_graph};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    paths::WorkflowVersionsPath,
    response::{GraphDto, GraphErrorResponse, WorkflowVersionResponse},
    workflow::require_workflow,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct SaveWorkflowVersionRequest {
    pub graph: GraphDto,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GraphInvalidBody {
    pub code: &'static str,
    pub message: String,
    pub status: u16,
    pub details: GraphInvalidDetails,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GraphInvalidDetails {
    pub errors: Vec<GraphErrorResponse>,
}

pub enum SaveVersionError {
    Api(ApiError),
    /// The graph is invalid: one entry per `mestier_core::GraphError`,
    /// naming the connector and (when there is one) the field at fault.
    GraphInvalid(Vec<GraphErrorResponse>),
}

impl From<ApiError> for SaveVersionError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<common::CoreError> for SaveVersionError {
    fn from(error: common::CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl IntoResponse for SaveVersionError {
    fn into_response(self) -> AxumResponse {
        match self {
            Self::Api(error) => error.into_response(),
            Self::GraphInvalid(errors) => {
                let body = GraphInvalidBody {
                    code: "E_GRAPH_INVALID",
                    message: "the workflow graph is invalid".to_owned(),
                    status: StatusCode::UNPROCESSABLE_ENTITY.as_u16(),
                    details: GraphInvalidDetails { errors },
                };
                (StatusCode::UNPROCESSABLE_ENTITY, Json(body)).into_response()
            }
        }
    }
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/automation/workflows/{workflow_id}/versions",
    operation_id = "saveWorkflowVersion",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("workflow_id" = uuid::Uuid, Path, description = "Workflow identifier"),
    ),
    request_body = SaveWorkflowVersionRequest,
    responses(
        (status = 201, description = "New immutable version saved, current_version_id moved to it", body = inline(DataEnvelope<WorkflowVersionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Workflow not found"),
        (status = 422, description = "The graph is invalid — errors name the connector and field at fault", body = inline(GraphInvalidBody)),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    WorkflowVersionsPath {
        organization_id,
        workflow_id,
    }: WorkflowVersionsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SaveWorkflowVersionRequest>,
) -> Result<Response<WorkflowVersionResponse>, SaveVersionError> {
    require_workflow(&state, &identity, organization_id, workflow_id).await?;
    let actor = handlers::resolve_user_id(&state, &identity).await?;

    let graph: mestier_core::Graph = payload.graph.into();
    let catalogue = connector_catalogue();
    let credentials = state.usecase.list_credentials(organization_id).await?;

    if let Err(errors) = validate_graph(&graph, &catalogue, &credentials) {
        let response: Vec<GraphErrorResponse> =
            errors.iter().map(GraphErrorResponse::from).collect();
        return Err(SaveVersionError::GraphInvalid(response));
    }

    let version = state
        .usecase
        .acting_as(actor)
        .save_workflow_version(SaveWorkflowVersionCommand {
            org_id: organization_id,
            workflow_id,
            graph,
            created_by: Some(actor.0),
        })
        .await?;

    Ok(Response::Created(WorkflowVersionResponse::from(version)))
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;
    use mestier_core::GraphError;

    use super::*;

    /// The acceptance criterion (#203): an invalid graph is refused by
    /// naming the connector and field at fault, not by a generic 400 — and
    /// specifically not through `handlers::ApiError`, whose body never
    /// carries `details` (see `handlers::errors::ApiError::into_response`).
    #[tokio::test]
    async fn graph_invalid_responds_422_naming_the_connector_and_field() {
        let errors = [
            GraphError::MissingRequiredField {
                connector_id: "c1".to_string(),
                field: "predicate".to_string(),
            },
            GraphError::UnknownCredential {
                connector_id: "c2".to_string(),
                credential_id: uuid::Uuid::from_u128(1),
            },
        ];
        let response_errors: Vec<GraphErrorResponse> =
            errors.iter().map(GraphErrorResponse::from).collect();

        let response = SaveVersionError::GraphInvalid(response_errors).into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(json["code"], "E_GRAPH_INVALID");
        assert_eq!(json["status"], 422);
        let errors = json["details"]["errors"].as_array().unwrap();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0]["connector_id"], "c1");
        assert_eq!(errors[0]["field"], "predicate");
        assert!(errors[0]["message"].as_str().unwrap().contains("predicate"));
        assert_eq!(errors[1]["connector_id"], "c2");
        assert!(errors[1]["field"].is_null());
    }

    #[test]
    fn an_api_error_still_produces_the_ordinary_error_body() {
        let response = SaveVersionError::from(ApiError::NotFound).into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
