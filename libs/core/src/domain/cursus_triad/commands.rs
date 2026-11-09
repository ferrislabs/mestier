use chrono::NaiveDate;

use crate::{
    ApprenticeId, ClosureReasonId, CompanyTutorId, ContractFileKind, ContractId, HostCompanyId,
    MemberId, OrganizationId, PromotionId, TriadId, domain::cursus_triad::ContractKind,
};

#[derive(Debug, Clone)]
pub struct CreateTriadCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub promotion_id: PromotionId,
    pub apprentice_id: ApprenticeId,
    pub pedagogic_tutor_member_id: Option<MemberId>,
}

#[derive(Debug, Clone)]
pub struct UpdateTriadCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: TriadId,
    pub pedagogic_tutor_member_id: Option<MemberId>,
}

#[derive(Debug, Clone)]
pub struct CloseTriadCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: TriadId,
    pub closed_at: NaiveDate,
    pub closure_reason_id: ClosureReasonId,
    pub closure_note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateContractCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub triad_id: TriadId,
    pub host_company_id: HostCompanyId,
    pub kind: ContractKind,
    pub number: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct UpdateContractCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: ContractId,
    pub host_company_id: HostCompanyId,
    pub kind: ContractKind,
    pub number: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone)]
pub struct CloseContractCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: ContractId,
    pub closed_at: NaiveDate,
    pub closure_reason_id: ClosureReasonId,
    pub closure_note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SetContractTutorsCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub contract_id: ContractId,
    pub main: Option<CompanyTutorId>,
    pub secondary: Option<CompanyTutorId>,
}

#[derive(Debug, Clone)]
pub struct SetContractFileCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub contract_id: ContractId,
    pub kind: ContractFileKind,
    pub file_key: Option<String>,
}
