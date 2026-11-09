use common::CoreError;

use crate::{
    LibraryPhoto, LibraryPhotoId, OrganizationId, PhotoFilter, domain::photo_library::InsertOutcome,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait LibraryPhotoRepository: Send {
    fn insert_if_absent(
        &mut self,
        photo: &LibraryPhoto,
    ) -> impl Future<Output = Result<InsertOutcome, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        organization_id: OrganizationId,
        id: LibraryPhotoId,
    ) -> impl Future<Output = Result<Option<LibraryPhoto>, CoreError>> + Send;

    fn find_by_hash(
        &mut self,
        organization_id: OrganizationId,
        content_hash: &str,
    ) -> impl Future<Output = Result<Option<LibraryPhoto>, CoreError>> + Send;

    fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &PhotoFilter,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<LibraryPhoto>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        photo: &LibraryPhoto,
    ) -> impl Future<Output = Result<LibraryPhoto, CoreError>> + Send;

    fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: LibraryPhotoId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
