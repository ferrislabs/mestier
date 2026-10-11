use chrono::{DateTime, Utc};
use sqlx::types::Json;
use uuid::Uuid;

use crate::{DoeModel, DoeModelId, DoeModelSection, DoeSettings, DoeStyle, OrganizationId};

#[derive(Debug, Clone)]
pub struct DoeModelRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sections: Json<Vec<DoeModelSection>>,
    pub style: Option<Json<DoeStyle>>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DoeModelRow> for DoeModel {
    fn from(row: DoeModelRow) -> Self {
        Self {
            id: DoeModelId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            description: row.description,
            sections: row.sections.0,
            style: row.style.map(|style| style.0),
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DoeSettingsRow {
    pub org_id: Uuid,
    pub style: Json<DoeStyle>,
}

impl From<DoeSettingsRow> for DoeSettings {
    fn from(row: DoeSettingsRow) -> Self {
        Self {
            organization_id: OrganizationId(row.org_id),
            style: row.style.0,
        }
    }
}
