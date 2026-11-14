use crate::{ArchivedFilter, MemberId, OrganizationId, SchoolTutorId, SearchPattern};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SchoolTutorFilter {
    pub archived: ArchivedFilter,
    pub search: Option<SearchPattern>,
}

#[derive(Debug, Clone)]
pub struct CreateSchoolTutorCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub job_title: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateSchoolTutorCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: SchoolTutorId,
    pub job_title: Option<String>,
}
