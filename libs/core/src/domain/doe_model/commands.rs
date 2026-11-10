use crate::{
    DoeModelId, OrganizationId, ProjectId,
    domain::doe_model::{DoeModelSection, DoeStyle, StarterModel},
};

#[derive(Debug, Clone)]
pub struct CreateDoeModelCommand {
    /// Authenticated actor. Built by the handler from the request `Identity`.
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub sections: Vec<DoeModelSection>,
    pub style: Option<DoeStyle>,
}

#[derive(Debug, Clone)]
pub struct UpdateDoeModelCommand {
    pub actor: authz::Subject,
    pub id: DoeModelId,
    pub name: String,
    pub description: Option<String>,
    pub sections: Vec<DoeModelSection>,
    pub style: Option<DoeStyle>,
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
    pub style: DoeStyle,
}

#[derive(Debug, Clone)]
pub struct PreviewDoeStyleCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub style: DoeStyle,
}

#[derive(Debug, Clone)]
pub struct ProjectDoeStyleCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
}

#[derive(Debug, Clone)]
pub struct UpdateProjectDoeStyleCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
    pub style: DoeStyle,
}
