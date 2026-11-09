use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::AppState;

pub mod spaces;
pub mod triad;
pub mod triads;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(spaces::handler)
        .typed_get(triads::handler)
        .typed_get(triad::handler)
}
