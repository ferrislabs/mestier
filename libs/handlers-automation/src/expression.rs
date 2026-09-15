use std::collections::BTreeMap;

use auth::Identity;
use axum::{Extension, Json, Router, extract::State};
use axum_extra::routing::RouterExt;
use chrono::Utc;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{ExpressionContext, LoopFrame, parse_template};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::{paths::EvaluateExpressionPath, require_view_automation};

pub const MAX_CONTEXT_CONNECTORS: usize = 256;
pub const MAX_CONTEXT_BYTES: usize = 256 * 1024;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new().typed_post(evaluate)
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct LoopFrameBody {
    pub item: Value,
    pub index: usize,
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct EvaluateContextBody {
    #[serde(default)]
    pub trigger: Option<Value>,
    #[serde(default)]
    pub connectors: BTreeMap<String, Value>,
    #[serde(default)]
    pub r#loop: Option<LoopFrameBody>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EvaluateExpressionRequest {
    pub template: Value,
    pub context: EvaluateContextBody,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct EvaluateExpressionResponse {
    pub value: Value,
}

fn bounds_error(context: &EvaluateContextBody) -> Option<ApiError> {
    if context.connectors.len() > MAX_CONTEXT_CONNECTORS {
        return Some(ApiError::UnprocessableEntity(format!(
            "the context holds at most {MAX_CONTEXT_CONNECTORS} connector outputs, got {}",
            context.connectors.len()
        )));
    }

    let size = serde_json::to_vec(context)
        .map(|bytes| bytes.len())
        .unwrap_or(usize::MAX);
    if size > MAX_CONTEXT_BYTES {
        return Some(ApiError::UnprocessableEntity(format!(
            "the serialized context holds at most {MAX_CONTEXT_BYTES} bytes, got {size}"
        )));
    }

    None
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/expressions/evaluate",
    operation_id = "evaluateExpression",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = EvaluateExpressionRequest,
    responses(
        (status = 200, description = "The template resolved against the supplied context", body = inline(DataEnvelope<EvaluateExpressionResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "The template failed to parse or evaluate, or the context exceeded a bound — the message names why"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn evaluate(
    EvaluateExpressionPath { organization_id }: EvaluateExpressionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<EvaluateExpressionRequest>,
) -> Result<Response<EvaluateExpressionResponse>, ApiError> {
    require_view_automation(&state, &identity, organization_id).await?;

    if let Some(error) = bounds_error(&payload.context) {
        return Err(error);
    }

    let template = parse_template(&payload.template)
        .map_err(|error| ApiError::UnprocessableEntity(error.to_string()))?;

    let loop_frame = payload.context.r#loop.as_ref().map(|frame| LoopFrame {
        item: &frame.item,
        index: frame.index,
    });
    let ctx = ExpressionContext {
        trigger: payload.context.trigger.as_ref(),
        connectors: &payload.context.connectors,
        loop_frame,
        now: Utc::now(),
    };

    let value = template
        .evaluate(&ctx)
        .map_err(|error| ApiError::UnprocessableEntity(error.to_string()))?;

    Ok(Response::OK(EvaluateExpressionResponse { value }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context_with_connectors(count: usize) -> EvaluateContextBody {
        let mut connectors = BTreeMap::new();
        for index in 0..count {
            connectors.insert(format!("c{index}"), Value::Bool(true));
        }
        EvaluateContextBody {
            trigger: None,
            connectors,
            r#loop: None,
        }
    }

    #[test]
    fn a_context_within_bounds_is_accepted() {
        let context = context_with_connectors(1);

        assert!(bounds_error(&context).is_none());
    }

    #[test]
    fn a_context_over_the_connector_cap_names_that_bound() {
        let context = context_with_connectors(MAX_CONTEXT_CONNECTORS + 1);

        let error = bounds_error(&context).expect("must be refused");
        assert!(matches!(error, ApiError::UnprocessableEntity(_)));
        assert!(error.to_string().contains("connector"));
    }

    #[test]
    fn a_context_over_the_byte_cap_names_that_bound() {
        let mut context = context_with_connectors(1);
        context.trigger = Some(Value::String("x".repeat(MAX_CONTEXT_BYTES)));

        let error = bounds_error(&context).expect("must be refused");
        assert!(matches!(error, ApiError::UnprocessableEntity(_)));
        assert!(error.to_string().contains("bytes"));
    }
}
