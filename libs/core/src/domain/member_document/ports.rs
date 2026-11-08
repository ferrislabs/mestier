use common::CoreError;

use super::{MemberDocument, MemberDocumentId};
use crate::OrganizationId;

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait MemberDocumentRepository: Send {
    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<MemberDocument>, CoreError>> + Send;

    fn find(
        &mut self,
        organization_id: OrganizationId,
        id: MemberDocumentId,
    ) -> impl Future<Output = Result<Option<MemberDocument>, CoreError>> + Send;

    fn insert(
        &mut self,
        document: &MemberDocument,
    ) -> impl Future<Output = Result<MemberDocument, CoreError>> + Send;

    fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: MemberDocumentId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
