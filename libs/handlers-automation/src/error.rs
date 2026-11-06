use axum::{
    Json,
    response::{IntoResponse, Response},
};
use common::CoreError;
use handlers::ApiError;
use http::StatusCode;
use mestier_core::{CollectionError, FolderRefusal, WorkflowRefusal};
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refusal {
    WorkflowArchived,
    FolderNotEmpty,
    WebhookPathTaken,
}

impl Refusal {
    pub fn code(self) -> &'static str {
        match self {
            Self::WorkflowArchived => "E_WORKFLOW_ARCHIVED",
            Self::FolderNotEmpty => "E_FOLDER_NOT_EMPTY",
            Self::WebhookPathTaken => "E_WEBHOOK_PATH_TAKEN",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::WorkflowArchived => WorkflowRefusal::Archived.message(),
            Self::FolderNotEmpty => FolderRefusal::NotEmpty.message(),
            Self::WebhookPathTaken => WorkflowRefusal::WebhookPathTaken.message(),
        }
    }

    fn from_conflict(message: &str) -> Option<Self> {
        match WorkflowRefusal::from_conflict(message) {
            Some(WorkflowRefusal::Archived) => return Some(Self::WorkflowArchived),
            Some(WorkflowRefusal::WebhookPathTaken) => return Some(Self::WebhookPathTaken),
            _ => {}
        }
        match FolderRefusal::from_conflict(message) {
            Some(FolderRefusal::NotEmpty) => Some(Self::FolderNotEmpty),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum AutomationError {
    Api(ApiError),
    Refused(Refusal),
    Collection(CollectionError),
}

impl From<CollectionError> for AutomationError {
    fn from(error: CollectionError) -> Self {
        Self::Collection(error)
    }
}

impl From<ApiError> for AutomationError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<WorkflowRefusal> for AutomationError {
    fn from(refusal: WorkflowRefusal) -> Self {
        match refusal {
            WorkflowRefusal::Archived => Self::Refused(Refusal::WorkflowArchived),
            WorkflowRefusal::WebhookPathTaken => Self::Refused(Refusal::WebhookPathTaken),
            WorkflowRefusal::ConflictingFlags => {
                Self::Api(ApiError::Validation(refusal.message().to_owned()))
            }
        }
    }
}

impl From<CoreError> for AutomationError {
    fn from(error: CoreError) -> Self {
        let CoreError::Conflict(message) = &error else {
            return Self::Api(error.into());
        };
        if let Some(refusal) = WorkflowRefusal::from_conflict(message) {
            return refusal.into();
        }
        match Refusal::from_conflict(message) {
            Some(refusal) => Self::Refused(refusal),
            None => Self::Api(error.into()),
        }
    }
}

#[derive(Serialize)]
struct RefusalBody {
    code: &'static str,
    message: &'static str,
    status: u16,
}

#[derive(Serialize)]
struct TooLargeBody {
    code: &'static str,
    message: String,
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
            Self::Collection(CollectionError::NotFound) => ApiError::NotFound.into_response(),
            Self::Collection(error @ CollectionError::InvalidModel(_)) => {
                ApiError::Validation(error.to_string()).into_response()
            }
            Self::Collection(error @ CollectionError::TooMany) => {
                ApiError::Conflict(error.to_string()).into_response()
            }
            Self::Collection(
                error @ (CollectionError::Oversized { .. }
                | CollectionError::RequestTooLarge { .. }),
            ) => {
                let body = TooLargeBody {
                    code: "E_COLLECTION_TOO_LARGE",
                    message: error.to_string(),
                    status: StatusCode::PAYLOAD_TOO_LARGE.as_u16(),
                };
                (StatusCode::PAYLOAD_TOO_LARGE, Json(body)).into_response()
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
    async fn archiving_and_enabling_together_is_a_validation_error_not_a_conflict() {
        let error = AutomationError::from(CoreError::from(WorkflowRefusal::ConflictingFlags));

        let (status, json) = body_of(error).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], "E_VALIDATION");
        assert_eq!(json["status"], 400);
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

    #[tokio::test]
    async fn an_oversized_collection_answers_413_with_its_own_code() {
        let error = AutomationError::from(CollectionError::Oversized {
            size: 3_000_000,
            limit: 2_097_152,
        });

        let (status, json) = body_of(error).await;

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(json["code"], "E_COLLECTION_TOO_LARGE");
        assert_eq!(json["status"], 413);
        let message = json["message"].as_str().unwrap();
        assert!(
            message.contains("3000000") && message.contains("2097152"),
            "{message}"
        );
    }

    #[tokio::test]
    async fn an_invalid_collection_answers_400_with_the_reason() {
        let error = AutomationError::from(CollectionError::InvalidModel(
            "a collection needs at least one operation".to_owned(),
        ));

        let (status, json) = body_of(error).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(json["code"], "E_VALIDATION");
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("at least one operation")
        );
    }

    #[tokio::test]
    async fn too_many_collections_answers_409_with_a_static_message() {
        let (status, json) = body_of(AutomationError::from(CollectionError::TooMany)).await;

        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(json["code"], "E_CONFLICT");
        assert!(
            json["message"]
                .as_str()
                .unwrap()
                .contains("at most 200 collections")
        );
        let through_core = AutomationError::from(CoreError::from(CollectionError::TooMany));
        let (status, json) = body_of(through_core).await;
        assert_eq!(status, StatusCode::CONFLICT);
        assert_eq!(json["code"], "E_CONFLICT");
    }

    #[tokio::test]
    async fn a_request_body_over_the_door_answers_413_with_the_collection_code() {
        let error = AutomationError::from(CollectionError::RequestTooLarge { limit: 2_359_296 });

        let (status, json) = body_of(error).await;

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert_eq!(json["code"], "E_COLLECTION_TOO_LARGE");
        assert_eq!(json["status"], 413);
        assert!(json["message"].as_str().unwrap().contains("2359296"));
    }

    #[tokio::test]
    async fn a_missing_collection_answers_404() {
        let (status, json) = body_of(AutomationError::from(CollectionError::NotFound)).await;

        assert_eq!(status, StatusCode::NOT_FOUND);
        assert_eq!(json["code"], "E_NOT_FOUND");
    }

    #[tokio::test]
    async fn a_webhook_path_another_workflow_holds_answers_409_with_its_own_code() {
        let from_core = AutomationError::from(CoreError::from(WorkflowRefusal::WebhookPathTaken));
        let direct = AutomationError::from(WorkflowRefusal::WebhookPathTaken);

        for error in [from_core, direct] {
            let (status, json) = body_of(error).await;

            assert_eq!(status, StatusCode::CONFLICT);
            assert_eq!(json["code"], "E_WEBHOOK_PATH_TAKEN");
            assert_eq!(json["status"], 409);
            assert!(json["message"].is_string());
        }
    }
}
