use axum::{
    Json,
    response::{IntoResponse, Response},
};
use common::CoreError;
use handlers::ApiError;
use http::StatusCode;
use mestier_core::{FolderRefusal, WorkflowRefusal};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    WorkflowArchived,
    FolderNotEmpty,
}

impl Refusal {
    pub fn code(self) -> &'static str {
        match self {
            Self::WorkflowArchived => "E_WORKFLOW_ARCHIVED",
            Self::FolderNotEmpty => "E_FOLDER_NOT_EMPTY",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::WorkflowArchived => WorkflowRefusal::Archived.message(),
            Self::FolderNotEmpty => FolderRefusal::NotEmpty.message(),
        }
    }

    fn from_conflict(message: &str) -> Option<Self> {
        if WorkflowRefusal::from_conflict(message).is_some() {
            return Some(Self::WorkflowArchived);
        }
        match FolderRefusal::from_conflict(message) {
            Some(FolderRefusal::NotEmpty) => Some(Self::FolderNotEmpty),
            _ => None,
        }
    }
}

impl From<WorkflowRefusal> for Refusal {
    fn from(refusal: WorkflowRefusal) -> Self {
        match refusal {
            WorkflowRefusal::Archived => Self::WorkflowArchived,
        }
    }
}

#[derive(Debug)]
pub enum AutomationError {
    Api(ApiError),
    Refused(Refusal),
}

impl From<ApiError> for AutomationError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<WorkflowRefusal> for AutomationError {
    fn from(refusal: WorkflowRefusal) -> Self {
        Self::Refused(refusal.into())
    }
}

impl From<CoreError> for AutomationError {
    fn from(error: CoreError) -> Self {
        match &error {
            CoreError::Conflict(message) => match Refusal::from_conflict(message) {
                Some(refusal) => Self::Refused(refusal),
                None => Self::Api(error.into()),
            },
            _ => Self::Api(error.into()),
        }
    }
}

#[derive(Serialize)]
struct RefusalBody {
    code: &'static str,
    message: &'static str,
    status: u16,
}

impl IntoResponse for AutomationError {
    fn into_response(self) -> Response {
        match self {
            Self::Api(error) => error.into_response(),
            Self::Refused(refusal) => {
                let body = RefusalBody {
                    code: refusal.code(),
                    message: refusal.message(),
                    status: StatusCode::CONFLICT.as_u16(),
                };
                (StatusCode::CONFLICT, Json(body)).into_response()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;

    async fn body_of(error: AutomationError) -> (StatusCode, serde_json::Value) {
        let response = error.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }

    #[tokio::test]
    async fn an_archived_workflow_answers_409_with_its_own_code() {
        let error = AutomationError::from(CoreError::from(WorkflowRefusal::Archived));

        let (status, json) = body_of(error).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(json["code"], "E_WORKFLOW_ARCHIVED");
        assert_eq!(json["status"], 409);
        assert!(json["message"].is_string());
        assert!(json.get("details").is_none());
    }

    #[tokio::test]
    async fn a_folder_that_is_not_empty_answers_409_with_its_own_code() {
        let error = AutomationError::from(CoreError::from(FolderRefusal::NotEmpty));

        let (status, json) = body_of(error).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(json["code"], "E_FOLDER_NOT_EMPTY");
        assert_eq!(json["status"], 409);
    }

    #[tokio::test]
    async fn the_other_folder_refusals_stay_ordinary_conflicts() {
        for refusal in [
            FolderRefusal::WouldCycle,
            FolderRefusal::NameTaken,
            FolderRefusal::Missing,
        ] {
            let (status, json) = body_of(AutomationError::from(CoreError::from(refusal))).await;

            assert_eq!(status, StatusCode::CONFLICT);
            assert_eq!(json["code"], "E_CONFLICT");
            assert!(
                json["message"]
                    .as_str()
                    .unwrap()
                    .contains(refusal.message())
            );
        }
    }

    #[tokio::test]
    async fn any_other_core_error_keeps_its_ordinary_mapping() {
        let (not_found, _) = body_of(AutomationError::from(CoreError::NotFound)).await;
        let (conflict, json) = body_of(AutomationError::from(CoreError::Conflict(
            "slug taken".into(),
        )))
        .await;

        assert_eq!(not_found, StatusCode::NOT_FOUND);
        assert_eq!(conflict, StatusCode::CONFLICT);
        assert_eq!(json["code"], "E_CONFLICT");
    }

    #[tokio::test]
    async fn an_api_error_passes_through_untouched() {
        let (status, json) = body_of(AutomationError::from(ApiError::Forbidden)).await;

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert_eq!(json["code"], "E_FORBIDDEN");
    }
}
