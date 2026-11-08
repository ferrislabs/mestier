use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use super::{
    LeaveAccount, LeaveAccountId, LeaveAccountInput, LeaveAccountOutcome,
    commands::LeaveAccountDraft, default_leave_accounts, ports::LeaveAccountRepository,
};
use crate::{
    AbsenceType, AbsenceTypeId, AbsenceTypeInput, OrganizationId,
    domain::absence_type::{default_absence_types, ports::AbsenceTypeRepository, staggered},
};

impl LeaveAccountDraft {
    pub fn as_input(&self) -> LeaveAccountInput<'_> {
        LeaveAccountInput {
            name: &self.name,
            acquisition: self.acquisition,
            period_start_month: self.period_start_month,
            carry_over_max: self.carry_over_max,
        }
    }
}

async fn seed_defaults<T, A>(
    types: &mut T,
    accounts: &mut A,
    organization_id: OrganizationId,
) -> Result<(), CoreError>
where
    T: AbsenceTypeRepository,
    A: LeaveAccountRepository,
{
    let base = Utc::now();
    let mut stored = accounts.list_by_organization(organization_id).await?;
    if stored.is_empty() {
        for (index, default) in default_leave_accounts().iter().enumerate() {
            let account = LeaveAccount::new(
                LeaveAccountId(generate_uuid_v7()),
                organization_id,
                LeaveAccountInput::from(default),
                staggered(base, index),
            )
            .map_err(|error| {
                CoreError::Internal(format!("default leave account is invalid: {error}"))
            })?;
            accounts.insert(&account).await?;
        }
        stored = accounts.list_by_organization(organization_id).await?;
    }

    if !types
        .list_by_organization(organization_id)
        .await?
        .is_empty()
    {
        return Ok(());
    }
    for (index, default) in default_absence_types().iter().enumerate() {
        let mut input = AbsenceTypeInput::from(default);
        input.account_id = default.account_name.and_then(|name| {
            stored
                .iter()
                .find(|account| account.name.as_str() == name)
                .map(|account| account.id.0)
        });
        let absence_type = AbsenceType::new(
            AbsenceTypeId(generate_uuid_v7()),
            organization_id,
            input,
            staggered(base, index),
        )
        .map_err(|error| {
            CoreError::Internal(format!("default absence type is invalid: {error}"))
        })?;
        types.insert(&absence_type).await?;
    }
    Ok(())
}

pub async fn ensure_defaults_for_types<T, A>(
    types: &mut T,
    accounts: &mut A,
    organization_id: OrganizationId,
) -> Result<(), CoreError>
where
    T: AbsenceTypeRepository,
    A: LeaveAccountRepository,
{
    types.lock_organization(organization_id).await?;
    if !types
        .list_by_organization(organization_id)
        .await?
        .is_empty()
    {
        return Ok(());
    }
    seed_defaults(types, accounts, organization_id).await
}

async fn ensure_defaults_for_accounts<T, A>(
    types: &mut T,
    accounts: &mut A,
    organization_id: OrganizationId,
) -> Result<(), CoreError>
where
    T: AbsenceTypeRepository,
    A: LeaveAccountRepository,
{
    types.lock_organization(organization_id).await?;
    if !accounts
        .list_by_organization(organization_id)
        .await?
        .is_empty()
    {
        return Ok(());
    }
    seed_defaults(types, accounts, organization_id).await
}

pub struct LeaveAccountService<A, T>
where
    A: LeaveAccountRepository,
    T: AbsenceTypeRepository,
{
    accounts: A,
    types: T,
}

impl<A, T> LeaveAccountService<A, T>
where
    A: LeaveAccountRepository,
    T: AbsenceTypeRepository,
{
    pub fn new(accounts: A, types: T) -> Self {
        Self { accounts, types }
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<LeaveAccount>, CoreError> {
        ensure_defaults_for_accounts(&mut self.types, &mut self.accounts, organization_id).await?;
        self.accounts.list_by_organization(organization_id).await
    }

    pub async fn find(
        &mut self,
        organization_id: OrganizationId,
        id: LeaveAccountId,
    ) -> Result<LeaveAccount, CoreError> {
        self.accounts
            .find(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn create(
        &mut self,
        organization_id: OrganizationId,
        draft: &LeaveAccountDraft,
    ) -> Result<LeaveAccountOutcome<LeaveAccount>, CoreError> {
        self.list(organization_id).await?;
        let account = match LeaveAccount::new(
            LeaveAccountId(generate_uuid_v7()),
            organization_id,
            draft.as_input(),
            Utc::now(),
        ) {
            Ok(account) => account,
            Err(refusal) => return Ok(Err(refusal)),
        };
        self.accounts.insert(&account).await.map(Ok)
    }

    pub async fn update(
        &mut self,
        organization_id: OrganizationId,
        id: LeaveAccountId,
        draft: &LeaveAccountDraft,
    ) -> Result<LeaveAccountOutcome<LeaveAccount>, CoreError> {
        let mut account = self.find(organization_id, id).await?;
        if let Err(refusal) = account.revise(draft.as_input()) {
            return Ok(Err(refusal));
        }
        self.accounts.update(&account).await.map(Ok)
    }

    pub async fn set_archived(
        &mut self,
        organization_id: OrganizationId,
        id: LeaveAccountId,
        archived: bool,
    ) -> Result<LeaveAccount, CoreError> {
        let mut account = self.find(organization_id, id).await?;
        if archived {
            account.archive(Utc::now());
        } else {
            account.restore();
        }
        self.accounts.update(&account).await
    }
}
