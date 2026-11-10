use common::CoreError;

use crate::{
    KnowledgeArticle, KnowledgeArticleId, KnowledgeFilter, KnowledgeLink, KnowledgeResource,
    OrganizationId,
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
}
