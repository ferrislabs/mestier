use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use uuid::Uuid;

use super::{
    AbsenceType, AbsenceTypeError, AbsenceTypeId, AbsenceTypeInput, AbsenceTypeOutcome,
    commands::AbsenceTypeDraft, ports::AbsenceTypeRepository,
};
use crate::{
    AbsenceKind, LeaveAccountId, OrganizationId,
    domain::leave_account::{ports::LeaveAccountRepository, service::ensure_defaults_for_types},
};

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

pub struct AbsenceTypeService<T, A>
where
    T: AbsenceTypeRepository,
    A: LeaveAccountRepository,
{
    types: T,
    accounts: A,
}

impl<T, A> AbsenceTypeService<T, A>
where
    T: AbsenceTypeRepository,
    A: LeaveAccountRepository,
{
    pub fn new(types: T, accounts: A) -> Self {
        Self { types, accounts }
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<AbsenceType>, CoreError> {
        ensure_defaults_for_types(&mut self.types, &mut self.accounts, organization_id).await?;
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
        if let Err(refusal) = self
            .check_account(organization_id, draft.account_id, None)
            .await?
        {
            return Ok(Err(refusal));
        }
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
        if let Err(refusal) = self
            .check_account(organization_id, draft.account_id, absence_type.account_id)
            .await?
        {
            return Ok(Err(refusal));
        }
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

    async fn check_account(
        &mut self,
        organization_id: OrganizationId,
        account_id: Option<Uuid>,
        current: Option<Uuid>,
    ) -> Result<AbsenceTypeOutcome<()>, CoreError> {
        let Some(account_id) = account_id else {
            return Ok(Ok(()));
        };
        let Some(account) = self
            .accounts
            .find(organization_id, LeaveAccountId(account_id))
            .await?
        else {
            return Ok(Err(AbsenceTypeError::AccountUnknown));
        };
        if account.is_archived() && current != Some(account_id) {
            return Ok(Err(AbsenceTypeError::AccountArchived));
        }
        Ok(Ok(()))
    }
}
