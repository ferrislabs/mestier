use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{
    MemberId, OrganizationId, SchoolTutor, SchoolTutorFilter, SchoolTutorId, SchoolTutorView,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait CursusSchoolTutorRepository: Send {
    fn insert(
        &mut self,
        tutor: &SchoolTutor,
    ) -> impl Future<Output = Result<SchoolTutor, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: SchoolTutorId,
    ) -> impl Future<Output = Result<Option<SchoolTutorView>, CoreError>> + Send;

    fn find_by_member(
        &mut self,
        organization_id: OrganizationId,
        member_id: MemberId,
    ) -> impl Future<Output = Result<Option<SchoolTutor>, CoreError>> + Send;

    fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &SchoolTutorFilter,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<SchoolTutorView>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        tutor: &SchoolTutor,
    ) -> impl Future<Output = Result<SchoolTutor, CoreError>> + Send;

    fn set_archived(
        &mut self,
        organization_id: OrganizationId,
        id: SchoolTutorId,
        archived_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<SchoolTutor, CoreError>> + Send;
}
