use crate::{ArchivedFilter, HostCompanyAddress, HostCompanyId, OrganizationId, SearchPattern};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HostCompanyFilter {
    pub archived: ArchivedFilter,
    pub search: Option<SearchPattern>,
}

#[derive(Debug, Clone)]
pub struct CreateHostCompanyCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub group_name: Option<String>,
    pub siret: Option<String>,
    pub legal_form: Option<String>,
    pub naf_code: Option<String>,
    pub capital: Option<String>,
    pub employee_count: Option<i32>,
    pub group_employee_count: Option<i32>,
    pub director_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub activity_domains: Option<String>,
    pub missions: Option<String>,
    pub addresses: Vec<HostCompanyAddress>,
}

#[derive(Debug, Clone)]
pub struct UpdateHostCompanyCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: HostCompanyId,
    pub name: String,
    pub group_name: Option<String>,
    pub siret: Option<String>,
    pub legal_form: Option<String>,
    pub naf_code: Option<String>,
    pub capital: Option<String>,
    pub employee_count: Option<i32>,
    pub group_employee_count: Option<i32>,
    pub director_name: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub website: Option<String>,
    pub activity_domains: Option<String>,
    pub missions: Option<String>,
    pub addresses: Vec<HostCompanyAddress>,
}
