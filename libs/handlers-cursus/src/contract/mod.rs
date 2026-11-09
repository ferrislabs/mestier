use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::AppState;

pub mod close;
pub mod create;
pub mod file_url;
pub mod get_one;
pub mod list;
pub mod set_file;
pub mod set_tutors;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_patch(update::handler)
        .typed_post(close::handler)
        .typed_put(set_tutors::handler)
        .typed_put(set_file::handler)
        .typed_get(file_url::handler)
}
