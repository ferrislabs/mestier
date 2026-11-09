use chrono::NaiveDate;
use mestier_core::{ApprenticeInput, CompanyTutorInput, HostCompanyId};
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
