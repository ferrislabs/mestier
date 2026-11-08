use authz::Subject;
use uuid::Uuid;

use super::{AbsenceTypeId, CountMode};
use crate::{AbsenceKind, OrganizationId};

#[derive(Debug, Clone)]
pub struct ListAbsenceTypesCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct AbsenceTypeDraft {
    pub name: String,
    pub hue: u16,
    pub count_mode: CountMode,
    pub account_id: Option<Uuid>,
    pub paid: bool,
    pub needs_approval: bool,
    pub needs_proof: bool,
    pub kind: AbsenceKind,
}

#[derive(Debug, Clone)]
pub struct CreateAbsenceTypeCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub draft: AbsenceTypeDraft,
}

#[derive(Debug, Clone)]
pub struct UpdateAbsenceTypeCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: AbsenceTypeId,
    pub draft: AbsenceTypeDraft,
}

#[derive(Debug, Clone)]
pub struct ArchiveAbsenceTypeCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub id: AbsenceTypeId,
    pub archived: bool,
}
