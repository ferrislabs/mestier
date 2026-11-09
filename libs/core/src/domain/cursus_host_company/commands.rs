use crate::{HostCompanyId, OrganizationId};

#[derive(Debug, Clone)]
pub struct CreateHostCompanyCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub siret: Option<String>,
    pub address: Option<String>,
    pub legal_form: Option<String>,
    pub naf_code: Option<String>,
    pub employee_count: Option<i32>,
    pub director_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateHostCompanyCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: HostCompanyId,
    pub name: String,
    pub siret: Option<String>,
    pub address: Option<String>,
    pub legal_form: Option<String>,
    pub naf_code: Option<String>,
    pub employee_count: Option<i32>,
    pub director_name: Option<String>,
}
