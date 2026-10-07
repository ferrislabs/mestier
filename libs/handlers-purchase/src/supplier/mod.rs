//! Suppliers as a referential. See docs/chantiers/doe.md.

use axum::Router;
use handlers::AppState;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
}
