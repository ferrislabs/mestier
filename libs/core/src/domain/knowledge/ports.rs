use common::CoreError;

use crate::{
    KnowledgeArticle, KnowledgeArticleId, KnowledgeFilter, KnowledgeFolder, KnowledgeFolderId,
    KnowledgeLink, KnowledgeResource, OrganizationId,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait KnowledgeArticleRepository: Send {
    fn insert(
        &mut self,
        article: &KnowledgeArticle,
    ) -> impl Future<Output = Result<KnowledgeArticle, CoreError>> + Send;

    /// Full article, extracted text included.
    fn find_by_id(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> impl Future<Output = Result<Option<KnowledgeArticle>, CoreError>> + Send;

    /// A page of articles and the total. The extracted text is left out of
    /// the rows: it can weigh hundreds of kilobytes each.
    fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &KnowledgeFilter,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<KnowledgeArticle>, u64), CoreError>> + Send;

    /// Replaces title, note and tags. The file and its text never change.
    fn update(
        &mut self,
        article: &KnowledgeArticle,
    ) -> impl Future<Output = Result<KnowledgeArticle, CoreError>> + Send;

    /// Removes the article and, by cascade, its links.
    fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    /// Idempotent: linking twice keeps one link.
    fn add_link(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        resource: KnowledgeResource,
    ) -> impl Future<Output = Result<KnowledgeLink, CoreError>> + Send;

    /// Whether a link was removed.
    fn remove_link(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        resource: KnowledgeResource,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    fn links_of(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> impl Future<Output = Result<Vec<KnowledgeLink>, CoreError>> + Send;

    /// How many of the articles were found in the organization and moved.
    fn move_to_folder(
        &mut self,
        organization_id: OrganizationId,
        ids: &[KnowledgeArticleId],
        folder_id: Option<KnowledgeFolderId>,
    ) -> impl Future<Output = Result<u64, CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait KnowledgeFolderRepository: Send {
    fn insert(
        &mut self,
        folder: &KnowledgeFolder,
    ) -> impl Future<Output = Result<KnowledgeFolder, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> impl Future<Output = Result<Option<KnowledgeFolder>, CoreError>> + Send;

    /// The folders directly under `parent` (the root when `None`), by name.
    fn children(
        &mut self,
        organization_id: OrganizationId,
        parent_id: Option<KnowledgeFolderId>,
    ) -> impl Future<Output = Result<Vec<KnowledgeFolder>, CoreError>> + Send;

    /// Every folder of the organization, by name, at most `limit`.
    fn all(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
    ) -> impl Future<Output = Result<Vec<KnowledgeFolder>, CoreError>> + Send;

    /// The path from the root down to the folder, the folder included. Its
    /// length is the depth of the folder.
    fn path_to(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> impl Future<Output = Result<Vec<KnowledgeFolder>, CoreError>> + Send;

    /// How many levels hang below the folder: 0 when it has no subfolder.
    fn height(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> impl Future<Output = Result<u32, CoreError>> + Send;

    /// Whether the folder holds neither a subfolder nor an article.
    fn is_empty(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Replaces the name and the parent.
    fn update(
        &mut self,
        folder: &KnowledgeFolder,
    ) -> impl Future<Output = Result<KnowledgeFolder, CoreError>> + Send;

    fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeFolderId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
