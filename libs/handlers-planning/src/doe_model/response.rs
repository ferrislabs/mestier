use chrono::{DateTime, Utc};
use mestier_core::{DoeModel, DoeModelSection, DoeSettings, DoeStyle, StarterModel};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DoeModelResponse {
    pub id: mestier_core::DoeModelId,
    pub organization_id: mestier_core::OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub sections: Vec<DoeModelSection>,
    pub style: Option<DoeStyle>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<DoeModel> for DoeModelResponse {
    fn from(model: DoeModel) -> Self {
        Self {
            id: model.id,
            organization_id: model.organization_id,
            name: model.name,
            description: model.description,
            sections: model.sections,
            style: model.style,
            archived_at: model.archived_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DoeModelStarterResponse {
    pub starter: StarterModel,
    pub name: String,
    pub description: Option<String>,
    pub sections: Vec<DoeModelSection>,
}

impl From<StarterModel> for DoeModelStarterResponse {
    fn from(starter: StarterModel) -> Self {
        Self {
            starter,
            name: starter.name().to_owned(),
            description: starter.description().map(str::to_owned),
            sections: starter.sections(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct DoeSettingsResponse {
    pub style: DoeStyle,
}

impl From<DoeSettings> for DoeSettingsResponse {
    fn from(settings: DoeSettings) -> Self {
        Self {
            style: settings.style,
        }
    }
}
