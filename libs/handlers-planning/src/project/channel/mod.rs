//! A project's one channel (#345) — creating it, and reading it back.
//!
//! Owns its own leaf path, `.../projects/{project_id}/channel`, the same way
//! `project` owns `.../restore`: a lifecycle-adjacent action gets its own
//! verb rather than a field folded into an existing route.
//!
//! This crate does not depend on `handlers-discord` (areas stay separate
//! crates — see the workspace's own `CLAUDE.md`), so the response here is
//! `super::super::response::ProjectChannelResponse`, a thin projection built
//! straight from `discord::Channel`, not that crate's own `ChannelResponse`.

use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::AppState;
use mestier_core::ProjectId;
use serde::Deserialize;

pub mod create;
pub mod get_one;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_post(create::handler)
        .typed_get(get_one::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/projects/{project_id}/channel")]
pub struct ProjectChannelPath {
    pub organization_id: mestier_core::OrganizationId,
    pub project_id: ProjectId,
}
