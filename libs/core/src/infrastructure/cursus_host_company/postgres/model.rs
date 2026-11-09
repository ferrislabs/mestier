use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{HostCompany, HostCompanyAddress, HostCompanyId, OrganizationId};

#[derive(Debug, Clone)]
pub struct HostCompanyRow {
    pub id: Uuid,
    pub org_id: Uuid,
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
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl HostCompanyRow {
    pub fn into_company(self, addresses: Vec<HostCompanyAddress>) -> HostCompany {
        HostCompany {
            id: HostCompanyId(self.id),
            organization_id: OrganizationId(self.org_id),
            name: self.name,
            group_name: self.group_name,
            siret: self.siret,
            legal_form: self.legal_form,
            naf_code: self.naf_code,
            capital: self.capital,
            employee_count: self.employee_count,
            group_employee_count: self.group_employee_count,
            director_name: self.director_name,
            phone: self.phone,
            email: self.email,
            website: self.website,
            activity_domains: self.activity_domains,
            missions: self.missions,
            addresses,
            archived_at: self.archived_at,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct HostCompanyAddressRow {
    pub host_company_id: Uuid,
    pub label: Option<String>,
    pub line1: String,
    pub line2: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub is_main: bool,
}

impl From<HostCompanyAddressRow> for HostCompanyAddress {
    fn from(row: HostCompanyAddressRow) -> Self {
        Self {
            label: row.label,
            line1: row.line1,
            line2: row.line2,
            postal_code: row.postal_code,
            city: row.city,
            is_main: row.is_main,
        }
    }
}

pub fn assemble(
    companies: Vec<HostCompanyRow>,
    addresses: Vec<HostCompanyAddressRow>,
) -> Vec<HostCompany> {
    let mut addresses = addresses;
    companies
        .into_iter()
        .map(|row| {
            let (mine, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut addresses)
                .into_iter()
                .partition(|address| address.host_company_id == row.id);
            addresses = rest;

            row.into_company(mine.into_iter().map(Into::into).collect())
        })
        .collect()
}
