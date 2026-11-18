use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware::from_fn_with_state,
    response::{IntoResponse, Response},
    routing::{any, post},
};
use axum_extra::routing::{RouterExt, TypedPath};
use common::CoreError;
use handlers::{ApiError, AppState, rate_limit::rate_limit_middleware};
use mestier_core::{InProgress, PublicSlug, ResponseId, ResumeToken, SurveyResponse};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::response::{ResponseStatus, photo::body_limit};

pub mod complete;
pub mod limit;
pub mod open;
pub mod photo;
pub mod resume;
pub mod save;
pub mod start;

use limit::{PublicLimit, public_limit};

pub const PUBLIC_SURVEYS_PREFIX: &str = "/api/v1/public/surveys";

pub fn router(state: &AppState) -> Router<AppState> {
    let limited = |limit: PublicLimit| from_fn_with_state((state.clone(), limit), public_limit);
    let reads = Router::new()
        .typed_get(open::handler)
        .typed_get(resume::handler)
        .typed_put(save::handler)
        .typed_post(complete::handler)
        .route(PUBLIC_SURVEYS_PREFIX, any(unknown))
        .route("/api/v1/public/surveys/", any(unknown))
        .route_layer(limited(PublicLimit::Read));
    let starts = Router::new()
        .typed_post(start::handler)
        .route_layer(limited(PublicLimit::Start));
    let photos = Router::new()
        .route(
            PublicResponsePhotosPath::PATH,
            post(photo::handler).layer(DefaultBodyLimit::max(body_limit(
                state.args.file_storage.max_upload_bytes,
            ))),
        )
        .route_layer(limited(PublicLimit::Photo));

    Router::new()
        .merge(reads)
        .merge(starts)
        .merge(photos)
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
}

async fn unknown() -> Response {
    ApiError::NotFound.into_response()
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/public/surveys/{slug}")]
pub struct PublicSurveyPath {
    pub slug: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/public/surveys/{slug}/responses")]
pub struct PublicSurveyResponsesPath {
    pub slug: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/public/surveys/{slug}/responses/{resume_token}")]
pub struct PublicResponsePath {
    pub slug: String,
    pub resume_token: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/public/surveys/{slug}/responses/{resume_token}/complete")]
pub struct PublicResponseCompletionPath {
    pub slug: String,
    pub resume_token: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/public/surveys/{slug}/responses/{resume_token}/photos")]
pub struct PublicResponsePhotosPath {
    pub slug: String,
    pub resume_token: String,
}

pub(crate) fn slug(value: String) -> Result<PublicSlug, ApiError> {
    PublicSlug::new(value).map_err(|_| ApiError::NotFound)
}

pub(crate) fn locate(
    slug_value: String,
    token: String,
) -> Result<(PublicSlug, ResumeToken), ApiError> {
    let slug = slug(slug_value)?;
    let token = ResumeToken::new(token).map_err(|_| ApiError::NotFound)?;
    Ok((slug, token))
}

pub(crate) fn hidden(error: CoreError) -> ApiError {
    match error {
        CoreError::Refused(refusal)
            if matches!(
                refusal.reason,
                "public_slug_invalid" | "resume_token_empty" | "survey_not_found"
            ) =>
        {
            ApiError::NotFound
        }
        CoreError::Forbidden { .. } => ApiError::NotFound,
        other => other.into(),
    }
}

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct StartedPublicResponseView<'a> {
    pub id: ResponseId,
    pub status: ResponseStatus,
    #[schema(value_type = BTreeMap<String, crate::schema::SurveyAnswerSchema>)]
    pub answers: &'a mestier_core::Answers,
    pub resume_token: &'a str,
}

impl<'a> StartedPublicResponseView<'a> {
    fn of(response: &'a SurveyResponse<InProgress>) -> Result<Self, ApiError> {
        let resume_token = response.resume_token().ok_or(ApiError::Internal)?;
        Ok(Self {
            id: response.id(),
            status: ResponseStatus::InProgress,
            answers: response.answers(),
            resume_token: resume_token.as_str(),
        })
    }
}

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, response::IntoResponse};
    use common::{Refusal, RefusalKind};

    use super::*;

    async fn body(error: ApiError) -> (u16, Vec<u8>) {
        let response = error.into_response();
        let status = response.status().as_u16();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        (status, bytes.to_vec())
    }

    #[tokio::test]
    async fn an_invalid_slug_or_token_answers_the_not_found_of_an_unknown_survey() {
        let unknown = body(ApiError::NotFound).await;
        let Err(short) = slug("abc".to_owned()) else {
            panic!("a short slug is not a slug");
        };
        let Err(spaced) = slug("pas un slug valide".to_owned()) else {
            panic!("a slug with spaces is not a slug");
        };
        let Err(empty_token) = locate("abcdefgh".to_owned(), String::new()) else {
            panic!("an empty token is not a token");
        };

        assert_eq!(unknown.0, 404);
        assert_eq!(body(short).await, unknown);
        assert_eq!(body(spaced).await, unknown);
        assert_eq!(body(empty_token).await, unknown);
    }

    #[tokio::test]
    async fn a_not_found_from_the_use_case_reveals_nothing_more() {
        let unknown = body(ApiError::NotFound).await;
        let refused_slug = CoreError::Refused(Refusal {
            kind: RefusalKind::Validation,
            reason: "public_slug_invalid",
            question_id: None,
            message: "public_slug: invalid".to_owned(),
        });

        assert_eq!(body(hidden(CoreError::NotFound)).await, unknown);
        assert_eq!(body(hidden(refused_slug)).await, unknown);
    }

    #[tokio::test]
    async fn a_closed_survey_keeps_its_reason() {
        let closed = CoreError::from(mestier_core::SurveyError::SurveyClosed);
        let (status, bytes) = body(hidden(closed)).await;
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();

        assert_eq!(status, 409);
        assert_eq!(json["details"]["reason"], "survey_closed");
    }
}
