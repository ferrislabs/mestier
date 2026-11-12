use chrono::NaiveDate;

use crate::{
    CompanyTutorId, HostCompanyId, OrganizationId, ParticipantId, ParticipantKind,
    ParticipantStatus, SearchPattern,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ParticipantFilter {
    pub kind: Option<ParticipantKind>,
    pub status: Option<ParticipantStatus>,
    pub host_company_id: Option<HostCompanyId>,
    pub search: Option<SearchPattern>,
}

#[derive(Debug, Clone)]
pub struct ApprenticeInput {
    pub birth_date: Option<NaiveDate>,
    pub ine_code: Option<String>,
    pub address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CompanyTutorInput {
    pub host_company_id: HostCompanyId,
    pub job_title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreateParticipantCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub last_name: String,
    pub first_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
    pub apprentice: Option<ApprenticeInput>,
    pub company_tutor: Option<CompanyTutorInput>,
}

#[derive(Debug, Clone)]
pub struct UpdateParticipantCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: ParticipantId,
    pub last_name: String,
    pub first_name: Option<String>,
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpsertApprenticeCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub participant_id: ParticipantId,
    pub apprentice: ApprenticeInput,
}

#[derive(Debug, Clone)]
pub struct AddCompanyTutorCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub participant_id: ParticipantId,
    pub company_tutor: CompanyTutorInput,
}

#[derive(Debug, Clone)]
pub struct RemoveCompanyTutorCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: CompanyTutorId,
}

#[derive(Debug, Clone)]
pub struct AcceptParticipantInvitationCommand {
    pub token: String,
    pub user_id: crate::UserId,
}
