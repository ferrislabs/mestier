use axum::{Router, extract::DefaultBodyLimit};
use axum_extra::routing::RouterExt;
use handlers::AppState;
use mestier_core::MAX_COLLECTION_BYTES;

pub mod create;
pub mod delete;
mod dto;
pub mod get_one;
pub mod list;
pub mod replace;

pub use dto::{
    ApiKeyLocationDto, CollectionRequest, CollectionResponse, CollectionSourceDto,
    CollectionSummaryResponse, CollectionVersionResponse, HttpMethodDto, OperationDto,
    ParameterDto, ParameterLocationDto, RequestBodyDto, SecurityKindDto, SecuritySchemeDto,
};

pub const REQUEST_LIMIT_BYTES: usize = MAX_COLLECTION_BYTES + 256 * 1024;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_put(replace::handler)
        .typed_delete(delete::handler)
        .layer(DefaultBodyLimit::max(REQUEST_LIMIT_BYTES))
}
