use axum::{Router, extract::DefaultBodyLimit, middleware::from_fn_with_state, routing::post};
use axum_extra::routing::TypedPath;
use handlers::{AppState, auth::auth_middleware, rate_limit::rate_limit_middleware};

use crate::paths::FilesPath;

pub mod paths;
pub mod response;
pub mod upload;

pub const TAG: &str = "files";

pub fn router(state: &AppState) -> Router<AppState> {
    let max_upload_bytes =
        usize::try_from(state.args.file_storage.max_upload_bytes).unwrap_or(usize::MAX);

    Router::new()
        .route(FilesPath::PATH, post(upload::handler))
        .layer(DefaultBodyLimit::max(max_upload_bytes))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(from_fn_with_state(state.clone(), auth_middleware))
}
