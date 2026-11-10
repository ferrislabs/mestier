use authz::{Resource, Subject};
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    BackfillReport, DeleteLibraryPhotoCommand, DoeId, GetLibraryPhotoCommand, LibraryPhoto,
    ListLibraryPhotosCommand, OrganizationId, PhotoLibraryOutcome, PlaceLibraryPhotosCommand,
    PlacementReport, UpdateLibraryPhotoCommand, UploadLibraryPhotoCommand, UploadedPhoto,
    application::{MestierAuthorizer, MestierFileStorageService, MestierUseCase, policy},
    domain::{
        doe::ports::DoeRepository, member::ports::MemberRepository,
        photo_library::service::PhotoLibraryService, role::ports::RoleRepository,
    },
};

async fn require_planning<M, R>(
    authz: &MestierAuthorizer,
    actor: Subject,
    organization_id: OrganizationId,
    members: &mut M,
    roles: &mut R,
) -> Result<(), CoreError>
where
    M: MemberRepository,
    R: RoleRepository,
{
    let actor = policy::enrich_for_organization(actor, organization_id, members, roles).await?;
    policy::require(
        authz,
        &actor,
        "planning.manage",
        Resource::new("organization", organization_id.0.to_string()),
    )
    .await
}

impl MestierUseCase {
    #[transactional(library_photo, project, member, role, authz)]
    pub async fn upload_library_photo(
        &self,
        command: UploadLibraryPhotoCommand,
        storage: &MestierFileStorageService,
    ) -> Result<UploadedPhoto, CoreError> {
        let mut project_repository = project_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_planning(
            authz,
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        PhotoLibraryService::new(library_photo_repository)
            .upload(command, &mut project_repository, storage)
            .await
    }

    #[transactional(library_photo, member, role, authz)]
    pub async fn get_library_photo(
        &self,
        command: GetLibraryPhotoCommand,
    ) -> Result<LibraryPhoto, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_planning(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        PhotoLibraryService::new(library_photo_repository)
            .get(command.organization_id, command.id)
            .await
    }

    #[transactional(library_photo, member, role, authz)]
    pub async fn copy_library_photo(
        &self,
        command: GetLibraryPhotoCommand,
        storage: &MestierFileStorageService,
    ) -> Result<String, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_planning(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        PhotoLibraryService::new(library_photo_repository)
            .copy_file(command.organization_id, command.id, storage)
            .await
    }

    #[transactional(library_photo, member, role, authz)]
    pub async fn list_library_photos(
        &self,
        command: ListLibraryPhotosCommand,
    ) -> Result<PhotoLibraryOutcome<(Vec<LibraryPhoto>, u64)>, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_planning(
            authz,
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        PhotoLibraryService::new(library_photo_repository)
            .list(command)
            .await
    }

    #[transactional(library_photo, project, member, role, authz)]
    pub async fn update_library_photo(
        &self,
        command: UpdateLibraryPhotoCommand,
    ) -> Result<PhotoLibraryOutcome<LibraryPhoto>, CoreError> {
        let mut project_repository = project_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_planning(
            authz,
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        PhotoLibraryService::new(library_photo_repository)
            .update(command, &mut project_repository)
            .await
    }

    #[transactional(library_photo, member, role, authz)]
    pub async fn delete_library_photo(
        &self,
        command: DeleteLibraryPhotoCommand,
    ) -> Result<LibraryPhoto, CoreError> {
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_planning(
            authz,
            command.actor,
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        PhotoLibraryService::new(library_photo_repository)
            .delete(command.organization_id, command.id)
            .await
    }

    #[transactional(library_photo, doe, member, role, authz)]
    pub async fn place_library_photos(
        &self,
        command: PlaceLibraryPhotosCommand,
        storage: &MestierFileStorageService,
    ) -> Result<PhotoLibraryOutcome<PlacementReport>, CoreError> {
        let mut doe_repository = doe_repository;
        let mut member_repository = member_repository;
        let mut role_repository = role_repository;
        require_planning(
            authz,
            command.actor.clone(),
            command.organization_id,
            &mut member_repository,
            &mut role_repository,
        )
        .await?;

        PhotoLibraryService::new(library_photo_repository)
            .place(command, &mut doe_repository, storage)
            .await
    }

    /// System task, no actor: copies the photos of one existing DOE into the
    /// library of its organization. Safe to run again.
    #[transactional(library_photo, doe)]
    pub async fn backfill_doe_photos(
        &self,
        organization_id: OrganizationId,
        doe_id: DoeId,
        storage: &MestierFileStorageService,
    ) -> Result<BackfillReport, CoreError> {
        let mut doe_repository = doe_repository;
        let doe = doe_repository
            .find_by_id(doe_id)
            .await?
            .filter(|doe| doe.organization_id == organization_id)
            .ok_or(CoreError::NotFound)?;

        Ok(PhotoLibraryService::new(library_photo_repository)
            .backfill_doe(&doe, storage)
            .await)
    }

    #[transactional(doe)]
    async fn list_doe_ids(&self, organization_id: OrganizationId) -> Result<Vec<DoeId>, CoreError> {
        let mut doe_repository = doe_repository;
        let mut ids = Vec::new();
        let mut offset = 0;
        loop {
            let (page, total) = doe_repository
                .list_by_organization(organization_id, 100, offset)
                .await?;
            if page.is_empty() {
                break;
            }
            offset += page.len() as u64;
            ids.extend(page.into_iter().map(|doe| doe.id));
            if offset >= total {
                break;
            }
        }
        Ok(ids)
    }

    /// Copies the photos of every DOE of an organization into its library,
    /// one transaction per DOE. Returns the sum of the reports.
    pub async fn backfill_organization_photos(
        &self,
        organization_id: OrganizationId,
        storage: &MestierFileStorageService,
    ) -> Result<BackfillReport, CoreError> {
        let mut total = BackfillReport::default();
        for doe_id in self.list_doe_ids(organization_id).await? {
            let report = self
                .backfill_doe_photos(organization_id, doe_id, storage)
                .await?;
            total.imported += report.imported;
            total.already_present += report.already_present;
            total.skipped += report.skipped;
            total.failed += report.failed;
        }
        Ok(total)
    }
}
