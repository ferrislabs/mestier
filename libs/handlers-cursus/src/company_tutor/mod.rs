use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::AppState;

pub mod add;
pub mod remove;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_post(add::handler)
        .typed_delete(remove::handler)
}
