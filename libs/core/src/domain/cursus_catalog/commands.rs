use crate::{
    ArchivedFilter, ClosureReasonId, EstablishmentId, OrganizationId, PromotionId, SearchPattern,
    TrainingId,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EstablishmentFilter {
    pub archived: ArchivedFilter,
    pub search: Option<SearchPattern>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TrainingFilter {
    pub establishment_id: Option<EstablishmentId>,
    pub archived: ArchivedFilter,
    pub search: Option<SearchPattern>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PromotionFilter {
    pub training_id: Option<TrainingId>,
    pub archived: ArchivedFilter,
    pub search: Option<SearchPattern>,
}

#[derive(Debug, Clone)]
pub struct CreateEstablishmentCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub uai: Option<String>,
    pub siret: Option<String>,
    pub address: Option<String>,
    pub is_main_site: bool,
}

#[derive(Debug, Clone)]
pub struct UpdateEstablishmentCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: EstablishmentId,
    pub name: String,
    pub uai: Option<String>,
    pub siret: Option<String>,
    pub address: Option<String>,
    pub is_main_site: bool,
}

#[derive(Debug, Clone)]
pub struct CreateTrainingCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub establishment_id: Option<EstablishmentId>,
    pub name: String,
    pub rncp_code: Option<String>,
    pub rncp_label: Option<String>,
    pub level: Option<String>,
    pub degree_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateTrainingCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: TrainingId,
    pub establishment_id: Option<EstablishmentId>,
    pub name: String,
    pub rncp_code: Option<String>,
    pub rncp_label: Option<String>,
    pub level: Option<String>,
    pub degree_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CreatePromotionCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub training_id: TrainingId,
    pub label: String,
    pub start_year: i32,
    pub end_year: i32,
}

#[derive(Debug, Clone)]
pub struct UpdatePromotionCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: PromotionId,
    pub training_id: TrainingId,
    pub label: String,
    pub start_year: i32,
    pub end_year: i32,
}

#[derive(Debug, Clone)]
pub struct CreateClosureReasonCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub label: String,
}

#[derive(Debug, Clone)]
pub struct UpdateClosureReasonCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: ClosureReasonId,
    pub label: String,
}
