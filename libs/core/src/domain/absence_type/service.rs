use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use super::{
    AbsenceType, AbsenceTypeId, AbsenceTypeInput, AbsenceTypeOutcome, commands::AbsenceTypeDraft,
    default_absence_types, ports::AbsenceTypeRepository, staggered,
};
use crate::{AbsenceKind, OrganizationId};

impl AbsenceTypeDraft {
    pub fn as_input(&self) -> AbsenceTypeInput<'_> {
        AbsenceTypeInput {
            name: &self.name,
            hue: self.hue,
            count_mode: self.count_mode,
            account_id: self.account_id,
            paid: self.paid,
            needs_approval: self.needs_approval,
            needs_proof: self.needs_proof,
            kind: self.kind,
        }
    }
}

pub struct AbsenceTypeService<T>
where
    T: AbsenceTypeRepository,
{
    types: T,
}

impl<T> AbsenceTypeService<T>
where
    T: AbsenceTypeRepository,
{
    pub fn new(types: T) -> Self {
        Self { types }
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<AbsenceType>, CoreError> {
        self.types.lock_organization(organization_id).await?;
        let existing = self.types.list_by_organization(organization_id).await?;
        if !existing.is_empty() {
            return Ok(existing);
        }

        let base = Utc::now();
        for (index, default) in default_absence_types().iter().enumerate() {
            let absence_type = AbsenceType::new(
                AbsenceTypeId(generate_uuid_v7()),
                organization_id,
                AbsenceTypeInput::from(default),
                staggered(base, index),
            )
            .map_err(|error| {
                CoreError::Internal(format!("default absence type is invalid: {error}"))
            })?;
            self.types.insert(&absence_type).await?;
        }

        self.types.list_by_organization(organization_id).await
    }

    pub async fn default_for_kind(
        &mut self,
        organization_id: OrganizationId,
        kind: AbsenceKind,
    ) -> Result<Option<AbsenceType>, CoreError> {
        Ok(self
            .list(organization_id)
            .await?
            .into_iter()
            .find(|absence_type| absence_type.kind == kind && !absence_type.is_archived()))
    }

    pub async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: AbsenceTypeId,
    ) -> Result<AbsenceType, CoreError> {
        self.types
            .find(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn create(
        &mut self,
        organization_id: OrganizationId,
        draft: &AbsenceTypeDraft,
    ) -> Result<AbsenceTypeOutcome<AbsenceType>, CoreError> {
        self.list(organization_id).await?;
        let absence_type = match AbsenceType::new(
            AbsenceTypeId(generate_uuid_v7()),
            organization_id,
            draft.as_input(),
            Utc::now(),
        ) {
            Ok(absence_type) => absence_type,
            Err(refusal) => return Ok(Err(refusal)),
        };
        self.types.insert(&absence_type).await.map(Ok)
    }

    pub async fn update(
        &mut self,
        organization_id: OrganizationId,
        id: AbsenceTypeId,
        draft: &AbsenceTypeDraft,
    ) -> Result<AbsenceTypeOutcome<AbsenceType>, CoreError> {
        let mut absence_type = self.find(organization_id, id).await?;
        if let Err(refusal) = absence_type.revise(draft.as_input()) {
            return Ok(Err(refusal));
        }
        self.types.update(&absence_type).await.map(Ok)
    }

    pub async fn set_archived(
        &mut self,
        organization_id: OrganizationId,
        id: AbsenceTypeId,
        archived: bool,
    ) -> Result<AbsenceType, CoreError> {
        let mut absence_type = self.find(organization_id, id).await?;
        if archived {
            absence_type.archive(Utc::now());
        } else {
            absence_type.restore();
        }
        self.types.update(&absence_type).await
    }
}
