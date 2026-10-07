use axum::Router;
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState};
use mestier_core::{OrganizationId, Supplier, SupplierId};

pub mod create;
pub mod get_one;
pub mod list;
pub mod paths;
pub mod response;
pub mod update;

async fn load_in_organization(
    state: &AppState,
    organization_id: OrganizationId,
    supplier_id: SupplierId,
) -> Result<Supplier, ApiError> {
    let supplier = state.usecase.get_supplier(supplier_id).await?;
    if supplier.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(supplier)
}

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_put(update::handler)
}
