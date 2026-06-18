use axum_extra::routing::TypedPath;
use mestier_core::{OrganizationId, QuoteId};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/quotes")]
pub struct QuotesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/quotes/{quote_id}")]
pub struct QuotePath {
    pub quote_id: QuoteId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/quotes/{quote_id}/status")]
pub struct QuoteStatusPath {
    pub quote_id: QuoteId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/quotes/{quote_id}/pdf")]
pub struct QuotePdfPath {
    pub quote_id: QuoteId,
}
