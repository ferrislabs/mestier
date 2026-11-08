use authz::{Authorizer, Resource, Subject};
use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use super::{
    MemberContract, MemberContractError, MemberContractId, MemberContractOutcome,
    commands::{
        CreateMemberContractCommand, DeleteMemberContractCommand, ListMemberContractsCommand,
        UpdateMemberContractCommand,
    },
    ports::MemberContractRepository,
};
use crate::{
    MemberId, OrganizationId,
    application::policy,
    domain::{member::ports::MemberRepository, role::ports::RoleRepository},
};

pub struct MemberContractService<C, M, Ro, A>
where
    C: MemberContractRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    contracts: C,
    members: M,
    roles: Ro,
    authz: A,
}

impl<C, M, Ro, A> MemberContractService<C, M, Ro, A>
where
    C: MemberContractRepository,
    M: MemberRepository,
    Ro: RoleRepository,
    A: Authorizer,
{
    pub fn new(contracts: C, members: M, roles: Ro, authz: A) -> Self {
        Self {
            contracts,
            members,
            roles,
            authz,
        }
    }

    pub async fn list(
        &mut self,
        command: ListMemberContractsCommand,
    ) -> Result<Vec<MemberContract>, CoreError> {
        self.authorize(command.actor, command.organization_id)
            .await?;
        match command.member_id {
            Some(member_id) => {
                self.require_member(command.organization_id, member_id)
                    .await?;
                self.contracts
                    .list_by_member(command.organization_id, member_id)
                    .await
            }
            None => {
                self.contracts
                    .list_by_organization(command.organization_id)
                    .await
            }
        }
    }

    pub async fn create(
        &mut self,
        command: CreateMemberContractCommand,
    ) -> Result<MemberContractOutcome<MemberContract>, CoreError> {
        self.authorize(command.actor, command.organization_id)
            .await?;
        self.require_member(command.organization_id, command.member_id)
            .await?;
        let draft = match MemberContract::draft(
            MemberContractId(generate_uuid_v7()),
            command.organization_id,
            command.member_id,
            command.input,
            Utc::now(),
        ) {
            Ok(draft) => draft,
            Err(refusal) => return Ok(Err(refusal)),
        };
        if self.overlaps_a_sibling(&draft).await? {
            return Ok(Err(MemberContractError::Overlap));
        }
        self.contracts.insert(&draft).await.map(Ok)
    }

    pub async fn update(
        &mut self,
        command: UpdateMemberContractCommand,
    ) -> Result<MemberContractOutcome<MemberContract>, CoreError> {
        self.authorize(command.actor, command.organization_id)
            .await?;
        let current = self
            .owned_contract(
                command.organization_id,
                command.member_id,
                command.contract_id,
            )
            .await?;
        let replaced = match current.replaced_with(command.input, Utc::now()) {
            Ok(replaced) => replaced,
            Err(refusal) => return Ok(Err(refusal)),
        };
        if self.overlaps_a_sibling(&replaced).await? {
            return Ok(Err(MemberContractError::Overlap));
        }
        self.contracts.update(&replaced).await.map(Ok)
    }

    pub async fn delete(&mut self, command: DeleteMemberContractCommand) -> Result<(), CoreError> {
        self.authorize(command.actor, command.organization_id)
            .await?;
        self.owned_contract(
            command.organization_id,
            command.member_id,
            command.contract_id,
        )
        .await?;
        self.contracts
            .delete(command.organization_id, command.contract_id)
            .await
    }

    async fn authorize(
        &mut self,
        actor: Subject,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        let actor = policy::enrich_for_organization(
            actor,
            organization_id,
            &mut self.members,
            &mut self.roles,
        )
        .await?;
        policy::require(
            &self.authz,
            &actor,
            "member.manage",
            Resource::new("organization", organization_id.0.to_string()),
        )
        .await
    }

    async fn require_member(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
    ) -> Result<(), CoreError> {
        self.members
            .find_by_id(member_id)
            .await?
            .filter(|member| member.organization_id == organization_id)
            .map(|_| ())
            .ok_or(CoreError::NotFound)
    }

    async fn owned_contract(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
        contract_id: MemberContractId,
    ) -> Result<MemberContract, CoreError> {
        self.require_member(organization_id, member_id).await?;
        self.contracts
            .find(organization_id, contract_id)
            .await?
            .filter(|contract| contract.member_id == member_id)
            .ok_or(CoreError::NotFound)
    }

    async fn overlaps_a_sibling(&mut self, contract: &MemberContract) -> Result<bool, CoreError> {
        Ok(self
            .contracts
            .list_by_member(contract.organization_id, contract.member_id)
            .await?
            .iter()
            .any(|other| other.id != contract.id && other.period.overlaps(&contract.period)))
    }
}
