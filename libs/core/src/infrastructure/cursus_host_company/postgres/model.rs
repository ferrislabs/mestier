use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{HostCompany, HostCompanyId, OrganizationId};

#[derive(Debug, Clone)]
pub struct HostCompanyRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub siret: Option<String>,
    pub address: Option<String>,
    pub legal_form: Option<String>,
    pub naf_code: Option<String>,
    pub employee_count: Option<i32>,
    pub director_name: Option<String>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<HostCompanyRow> for HostCompany {
    fn from(row: HostCompanyRow) -> Self {
        Self {
            id: HostCompanyId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            siret: row.siret,
            address: row.address,
            legal_form: row.legal_form,
            naf_code: row.naf_code,
            employee_count: row.employee_count,
            director_name: row.director_name,
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
