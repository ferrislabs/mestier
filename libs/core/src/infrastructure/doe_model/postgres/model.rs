use chrono::{DateTime, Utc};
use common::CoreError;
use sqlx::types::Json;
use uuid::Uuid;

use crate::{DoeModel, DoeModelId, DoeModelSection, DoeSettings, HexColor, OrganizationId};

#[derive(Debug, Clone)]
pub struct DoeModelRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub sections: Json<Vec<DoeModelSection>>,
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
            archived_at: row.archived_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(Debug, Clone)]
pub struct DoeSettingsRow {
    pub org_id: Uuid,
    pub accent: String,
    pub logo_key: Option<String>,
}

impl TryFrom<DoeSettingsRow> for DoeSettings {
    type Error = CoreError;

    fn try_from(row: DoeSettingsRow) -> Result<Self, Self::Error> {
        Ok(Self {
            organization_id: OrganizationId(row.org_id),
            accent: HexColor::try_from(row.accent)?,
            logo_key: row.logo_key,
        })
    }
}
