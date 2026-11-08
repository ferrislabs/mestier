use common::CoreError;
use mestier_macros::transactional;

use crate::{
    MemberContract, MemberContractOutcome,
    application::MestierUseCase,
    domain::member_contract::{
        MemberContractService,
        commands::{
            CreateMemberContractCommand, DeleteMemberContractCommand, ListMemberContractsCommand,
            UpdateMemberContractCommand,
        },
    },
};

impl MestierUseCase {
    #[transactional(member_contract, member, role, authz)]
    pub async fn list_member_contracts(
        &self,
        command: ListMemberContractsCommand,
    ) -> Result<Vec<MemberContract>, CoreError> {
        let mut service = MemberContractService::new(
            member_contract_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.list(command).await
    }

    #[transactional(member_contract, member, role, authz)]
    pub async fn create_member_contract(
        &self,
        command: CreateMemberContractCommand,
    ) -> Result<MemberContractOutcome<MemberContract>, CoreError> {
        let mut service = MemberContractService::new(
            member_contract_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.create(command).await
    }

    #[transactional(member_contract, member, role, authz)]
    pub async fn update_member_contract(
        &self,
        command: UpdateMemberContractCommand,
    ) -> Result<MemberContractOutcome<MemberContract>, CoreError> {
        let mut service = MemberContractService::new(
            member_contract_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.update(command).await
    }

    #[transactional(member_contract, member, role, authz)]
    pub async fn delete_member_contract(
        &self,
        command: DeleteMemberContractCommand,
    ) -> Result<(), CoreError> {
        let mut service = MemberContractService::new(
            member_contract_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.delete(command).await
    }
}
