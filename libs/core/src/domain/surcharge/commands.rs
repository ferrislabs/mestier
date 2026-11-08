use authz::Subject;

use super::{SurchargeId, SurchargeKind};
use crate::OrganizationId;

#[derive(Debug, Clone)]
pub struct SurchargeDraft {
    pub name: String,
    pub rate_bp: i32,
    pub kind: SurchargeKind,
}

#[derive(Debug, Clone)]
pub struct ListSurchargesCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct CreateSurchargeCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub draft: SurchargeDraft,
}

#[derive(Debug, Clone)]
pub struct UpdateSurchargeCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: SurchargeId,
    pub draft: SurchargeDraft,
}

#[derive(Debug, Clone)]
pub struct ArchiveSurchargeCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: SurchargeId,
    pub archived: bool,
}
