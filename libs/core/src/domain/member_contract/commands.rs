use authz::Subject;
use chrono::NaiveDate;

use super::{ContractType, MemberContractId, schedule::ScheduleInput};
use crate::{MemberId, OrganizationId};

#[derive(Debug, Clone)]
pub struct MemberContractInput {
    pub contract_type: ContractType,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub trial_end_date: Option<NaiveDate>,
    pub schedule: ScheduleInput,
}

#[derive(Debug, Clone)]
pub struct ListMemberContractsCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: Option<MemberId>,
}

#[derive(Debug, Clone)]
pub struct CreateMemberContractCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub input: MemberContractInput,
}

#[derive(Debug, Clone)]
pub struct UpdateMemberContractCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub contract_id: MemberContractId,
    pub input: MemberContractInput,
}

#[derive(Debug, Clone)]
pub struct DeleteMemberContractCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub contract_id: MemberContractId,
}
