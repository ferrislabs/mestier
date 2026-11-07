use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::AppState;

pub mod create;
pub mod delete;
pub mod list;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
}
