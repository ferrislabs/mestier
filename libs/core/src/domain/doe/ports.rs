use common::CoreError;

use crate::{Doe, DoeId, DoeSummary, OrganizationId, ProjectId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DoeRepository: Send {
    /// Fails with `CoreError::Conflict` when the project already has a DOE.
    fn insert(&mut self, doe: &Doe) -> impl Future<Output = Result<Doe, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: DoeId,
    ) -> impl Future<Output = Result<Option<Doe>, CoreError>> + Send;

    fn find_by_project(
        &mut self,
        project_id: ProjectId,
    ) -> impl Future<Output = Result<Option<Doe>, CoreError>> + Send;

    /// Most recently edited first.
    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<DoeSummary>, u64), CoreError>> + Send;

    /// Rewrites every field but `id`, `organization_id`, `project_id` and
    /// `created_at`.
    fn update(&mut self, doe: &Doe) -> impl Future<Output = Result<Doe, CoreError>> + Send;
}
