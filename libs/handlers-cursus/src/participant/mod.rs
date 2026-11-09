use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::AppState;

pub mod create;
pub mod deactivate;
pub mod get_one;
pub mod invite;
pub mod list;
pub mod reactivate;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_patch(update::handler)
        .typed_post(invite::handler)
        .typed_post(deactivate::handler)
        .typed_post(reactivate::handler)
}
