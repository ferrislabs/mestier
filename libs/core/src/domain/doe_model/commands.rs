use crate::{
    DoeModelId, OrganizationId,
    domain::doe_model::{DoeModelSection, HexColor, StarterModel},
};

#[derive(Debug, Clone)]
pub struct CreateDoeModelCommand {
    /// Authenticated actor. Built by the handler from the request `Identity`.
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub sections: Vec<DoeModelSection>,
}

#[derive(Debug, Clone)]
pub struct UpdateDoeModelCommand {
    pub actor: authz::Subject,
    pub id: DoeModelId,
    pub name: String,
    pub description: Option<String>,
    pub sections: Vec<DoeModelSection>,
}

#[derive(Debug, Clone)]
pub struct CreateDoeModelFromStarterCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub starter: StarterModel,
}

#[derive(Debug, Clone)]
pub struct UpdateDoeSettingsCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub accent: HexColor,
    pub logo_key: Option<String>,
}
