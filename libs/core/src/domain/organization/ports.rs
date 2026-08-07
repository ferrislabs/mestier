use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{
    UserId,
    domain::organization::{Organization, OrganizationId},
};

#[cfg_attr(test, mockall::automock)]
pub trait OrganizationRepository: Send {
    fn insert(
        &mut self,
        organization: &Organization,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: OrganizationId,
    ) -> impl Future<Output = Result<Option<Organization>, CoreError>> + Send;

    /// The organization's IANA timezone (`organizations.timezone`), or
    /// `None` if the organization does not exist or is soft-deleted.
    /// Added additively for the planning read model, which reconciles
    /// `TIMESTAMPTZ` entries (absences, work orders) against
    /// organization-local work-time minutes and needs the zone to do it
    /// (see the planning module design doc).
    fn find_timezone(
        &mut self,
        id: OrganizationId,
    ) -> impl Future<Output = Result<Option<String>, CoreError>> + Send;

    fn list_for_user(
        &mut self,
        user_id: UserId,
    ) -> impl Future<Output = Result<Vec<Organization>, CoreError>> + Send;

    fn list_paginated(
        &mut self,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<Organization>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        organization: &Organization,
    ) -> impl Future<Output = Result<Organization, CoreError>> + Send;

    fn soft_delete(
        &mut self,
        id: OrganizationId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
