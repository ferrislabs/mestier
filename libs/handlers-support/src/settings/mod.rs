use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::AppState;

pub mod get;
pub mod replace;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(get::handler)
        .typed_put(replace::handler)
}
