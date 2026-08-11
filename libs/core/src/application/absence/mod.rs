use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Absence, AbsenceId, OrganizationId,
    application::MestierUseCase,
    domain::absence::{
        commands::{CreateAbsenceCommand, PatchAbsenceCommand},
        service::AbsenceService,
    },
};

mod tests;

impl MestierUseCase {
    #[transactional(absence)]
    pub async fn create_absence(
        &self,
        command: CreateAbsenceCommand,
    ) -> Result<Absence, CoreError> {
        let mut service = AbsenceService::new(absence_repository);
        service.create_absence(command).await
    }

    #[transactional(absence)]
    pub async fn get_absence(&self, id: AbsenceId) -> Result<Absence, CoreError> {
        let mut service = AbsenceService::new(absence_repository);
        service.get_absence(id).await
    }

    #[transactional(absence)]
    pub async fn list_absences(
        &self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Absence>, u64), CoreError> {
        let mut service = AbsenceService::new(absence_repository);
        service.list_absences(organization_id, limit, offset).await
    }

    #[transactional(absence)]
    pub async fn patch_absence(&self, command: PatchAbsenceCommand) -> Result<Absence, CoreError> {
        let mut service = AbsenceService::new(absence_repository);
        service.patch_absence(command).await
    }

    #[transactional(absence)]
    pub async fn soft_delete_absence(&self, id: AbsenceId) -> Result<(), CoreError> {
        let mut service = AbsenceService::new(absence_repository);
        service.soft_delete_absence(id).await
    }
}
