use common::CoreError;

use crate::{
    Daf, DafDossier, DafId, DafSummary, DoeRenderError, FileBundle, ProjectId,
    domain::daf::render::DafDocument,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait DafRepository: Send {
    fn find_dossier(
        &mut self,
        project_id: ProjectId,
    ) -> impl Future<Output = Result<Option<DafDossier>, CoreError>> + Send;

    /// Inserts or rewrites the dossier of `dossier.project_id`.
    fn upsert_dossier(
        &mut self,
        dossier: &DafDossier,
    ) -> impl Future<Output = Result<DafDossier, CoreError>> + Send;

    /// Fails with `CoreError::Conflict` when the number is taken.
    fn insert(&mut self, daf: &Daf) -> impl Future<Output = Result<Daf, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: DafId,
    ) -> impl Future<Output = Result<Option<Daf>, CoreError>> + Send;

    /// Every DAF of the project, by number.
    fn list_by_project(
        &mut self,
        project_id: ProjectId,
    ) -> impl Future<Output = Result<Vec<Daf>, CoreError>> + Send;

    /// The summary rows of the project, by number.
    fn list_summaries(
        &mut self,
        project_id: ProjectId,
    ) -> impl Future<Output = Result<Vec<DafSummary>, CoreError>> + Send;

    /// Rewrites every field but `id`, `organization_id`, `project_id` and
    /// `created_at`. Fails with `CoreError::Conflict` when the number is
    /// taken.
    fn update(&mut self, daf: &Daf) -> impl Future<Output = Result<Daf, CoreError>> + Send;

    /// Fails with `CoreError::NotFound` when it is gone.
    fn delete(&mut self, id: DafId) -> impl Future<Output = Result<(), CoreError>> + Send;
}

/// Lays a DAF document out as a PDF. CPU-bound and synchronous: callers run
/// it off the async executor.
pub trait DafRenderer: Send + Sync {
    fn render(&self, document: &DafDocument, files: &FileBundle)
    -> Result<Vec<u8>, DoeRenderError>;
}
