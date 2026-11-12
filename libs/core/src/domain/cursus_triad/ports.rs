use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{
    CompanyTutorId, Contract, ContractId, ContractTutor, ContractView, OrganizationId,
    ParticipantId, Triad, TriadDetail, TriadFilter, TriadId, TriadSummary, TutorTriadLink,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CursusTriadRepository: Send {
    fn insert_triad(
        &mut self,
        triad: &Triad,
    ) -> impl Future<Output = Result<Triad, CoreError>> + Send;

    fn find_triad(
        &mut self,
        organization_id: OrganizationId,
        id: TriadId,
    ) -> impl Future<Output = Result<Option<Triad>, CoreError>> + Send;

    fn save_triad(
        &mut self,
        triad: &Triad,
    ) -> impl Future<Output = Result<Triad, CoreError>> + Send;

    fn find_triad_detail(
        &mut self,
        organization_id: OrganizationId,
        id: TriadId,
    ) -> impl Future<Output = Result<Option<TriadDetail>, CoreError>> + Send;

    fn list_triads(
        &mut self,
        organization_id: OrganizationId,
        filter: &TriadFilter,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<TriadSummary>, u64), CoreError>> + Send;

    fn list_triad_summaries(
        &mut self,
        organization_id: OrganizationId,
        ids: &[TriadId],
    ) -> impl Future<Output = Result<Vec<TriadSummary>, CoreError>> + Send;

    fn insert_contract(
        &mut self,
        contract: &Contract,
    ) -> impl Future<Output = Result<Contract, CoreError>> + Send;

    fn find_contract(
        &mut self,
        organization_id: OrganizationId,
        id: ContractId,
    ) -> impl Future<Output = Result<Option<Contract>, CoreError>> + Send;

    fn save_contract(
        &mut self,
        contract: &Contract,
    ) -> impl Future<Output = Result<Contract, CoreError>> + Send;

    fn find_contract_view(
        &mut self,
        organization_id: OrganizationId,
        id: ContractId,
    ) -> impl Future<Output = Result<Option<ContractView>, CoreError>> + Send;

    fn list_contracts(
        &mut self,
        organization_id: OrganizationId,
        triad_id: TriadId,
    ) -> impl Future<Output = Result<Vec<Contract>, CoreError>> + Send;

    fn list_contract_views(
        &mut self,
        organization_id: OrganizationId,
        triad_id: TriadId,
    ) -> impl Future<Output = Result<Vec<ContractView>, CoreError>> + Send;

    fn list_contract_tutors(
        &mut self,
        organization_id: OrganizationId,
        contract_id: ContractId,
    ) -> impl Future<Output = Result<Vec<ContractTutor>, CoreError>> + Send;

    fn replace_contract_tutors(
        &mut self,
        organization_id: OrganizationId,
        contract_id: ContractId,
        tutors: &[ContractTutor],
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn archive_contract_tutors(
        &mut self,
        organization_id: OrganizationId,
        contract_id: ContractId,
        archived_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn company_tutor_is_linked(
        &mut self,
        organization_id: OrganizationId,
        id: CompanyTutorId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn list_apprentice_triad_ids(
        &mut self,
        organization_id: OrganizationId,
        participant_id: ParticipantId,
    ) -> impl Future<Output = Result<Vec<TriadId>, CoreError>> + Send;

    fn list_tutor_links(
        &mut self,
        organization_id: OrganizationId,
        participant_id: ParticipantId,
    ) -> impl Future<Output = Result<Vec<TutorTriadLink>, CoreError>> + Send;
}
