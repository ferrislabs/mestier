use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::AppState;

pub mod upsert;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new().typed_put(upsert::handler)
}
