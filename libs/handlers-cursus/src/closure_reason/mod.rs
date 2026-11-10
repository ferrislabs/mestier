use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::AppState;

pub mod archive;
pub mod create;
pub mod list;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_patch(update::handler)
        .typed_post(archive::handler)
}
