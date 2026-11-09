use chrono::NaiveDate;
use mestier_core::{
    ApprenticeId, ApprenticeInput, ClosureReasonId, CompanyTutorId, CompanyTutorInput,
    CursusContractKind, HostCompanyId, MemberId, PromotionId,
};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct ApprenticeRequest {
    pub birth_date: Option<NaiveDate>,
    pub ine_code: Option<String>,
    pub address: Option<String>,
}

impl From<ApprenticeRequest> for ApprenticeInput {
    fn from(value: ApprenticeRequest) -> Self {
        Self {
            birth_date: value.birth_date,
            ine_code: value.ine_code,
            address: value.address,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CompanyTutorRequest {
    pub host_company_id: HostCompanyId,
    pub job_title: Option<String>,
}

impl From<CompanyTutorRequest> for CompanyTutorInput {
    fn from(value: CompanyTutorRequest) -> Self {
        Self {
            host_company_id: value.host_company_id,
            job_title: value.job_title,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CloseRequest {
    pub closed_at: NaiveDate,
    pub closure_reason_id: ClosureReasonId,
    pub closure_note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTriadRequest {
    pub promotion_id: PromotionId,
    pub apprentice_id: ApprenticeId,
    pub pedagogic_tutor_member_id: Option<MemberId>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTriadRequest {
    pub pedagogic_tutor_member_id: Option<MemberId>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ContractRequest {
    pub host_company_id: HostCompanyId,
    pub kind: CursusContractKind,
    pub number: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetContractTutorsRequest {
    pub main: Option<CompanyTutorId>,
    pub secondary: Option<CompanyTutorId>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetContractFileRequest {
    pub file_key: Option<String>,
}
