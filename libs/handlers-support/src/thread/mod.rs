use axum::{Router, extract::DefaultBodyLimit};
use axum_extra::routing::RouterExt;
use handlers::AppState;

pub mod add_entry;
pub mod attach;
pub mod attachment_url;
pub mod delete_draft;
pub mod edit_draft;
pub mod list;
pub mod send_draft;

pub fn router(state: &AppState) -> Router<AppState> {
    let max_upload_bytes =
        usize::try_from(state.args.file_storage.max_upload_bytes).unwrap_or(usize::MAX);

    let uploads = Router::new()
        .typed_post(attach::handler)
        .layer(DefaultBodyLimit::max(max_upload_bytes));

    Router::new()
        .typed_get(list::handler)
        .typed_post(add_entry::handler)
        .typed_patch(edit_draft::handler)
        .typed_delete(delete_draft::handler)
        .typed_post(send_draft::handler)
        .typed_get(attachment_url::handler)
        .merge(uploads)
}
