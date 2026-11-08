use authz::Subject;
use chrono::NaiveDate;

use crate::{MemberId, OrganizationId};

#[derive(Debug, Clone, Default)]
pub struct MemberProfileInput {
    pub work_email: Option<String>,
    pub personal_email: Option<String>,
    pub mobile: Option<String>,
    pub landline: Option<String>,
    pub street: Option<String>,
    pub complement: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub job_title: Option<String>,
    pub manager_member_id: Option<MemberId>,
    pub work_permit_end_date: Option<NaiveDate>,
}

#[derive(Debug, Clone)]
pub struct GetMemberProfileCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
}

#[derive(Debug, Clone)]
pub struct ListMemberProfilesCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
}

#[derive(Debug, Clone)]
pub struct PutMemberProfileCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub input: MemberProfileInput,
}

#[derive(Debug, Clone)]
pub struct ArchiveMemberCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
}

#[derive(Debug, Clone)]
pub struct RestoreMemberCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
}
