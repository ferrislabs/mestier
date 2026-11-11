use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    KnowledgeArticle, KnowledgeArticleId, KnowledgeError, KnowledgeFile, KnowledgeFilter,
    KnowledgeFolderId, KnowledgeKind, KnowledgeLink, KnowledgeOutcome, KnowledgeResource,
    OrganizationId, UpdateKnowledgeCommand, UploadKnowledgeFileCommand,
    domain::{
        file_storage::{
            commands::UploadFileCommand, ports::FileStorage, service::FileStorageService,
        },
        knowledge::{
            documents::DocumentContent,
            ports::KnowledgeArticleRepository,
            text::{check_file, normalize_tags, normalize_title},
        },
    },
};

pub const MAX_PAGE_SIZE: u64 = 100;

fn storage_folder(organization_id: OrganizationId) -> String {
    format!("knowledge-{}", organization_id.0)
}

pub fn validate_update(command: &UpdateKnowledgeCommand) -> KnowledgeOutcome<()> {
    if let Some(title) = &command.title {
        normalize_title(title)?;
    }
    if let Some(tags) = &command.tags {
        normalize_tags(tags)?;
    }
    Ok(())
}

pub struct KnowledgeService<R>
where
    R: KnowledgeArticleRepository,
{
    articles: R,
}

impl<R> KnowledgeService<R>
where
    R: KnowledgeArticleRepository,
{
    pub fn new(articles: R) -> Self {
        Self { articles }
    }

    pub async fn create_document(
        &mut self,
        organization_id: OrganizationId,
        folder_id: Option<KnowledgeFolderId>,
        title: &str,
        tags: &[String],
        content: DocumentContent,
        body: String,
    ) -> Result<KnowledgeOutcome<KnowledgeArticle>, CoreError> {
        let validated =
            (|| Ok::<_, KnowledgeError>((normalize_title(title)?, normalize_tags(tags)?)))();
        let (title, tags) = match validated {
            Ok(fields) => fields,
            Err(error) => return Ok(Err(error)),
        };
        let now = Utc::now();
        let article = KnowledgeArticle {
            id: KnowledgeArticleId(generate_uuid_v7()),
            organization_id,
            folder_id,
            kind: KnowledgeKind::Document,
            title,
            body: Some(body),
            file: None,
            content: Some(content.to_value()),
            revision: 0,
            lock: None,
            extracted_text: None,
            tags,
            created_at: now,
            updated_at: now,
        };
        self.articles.insert(&article).await.map(Ok)
    }

    pub async fn upload_file<S>(
        &mut self,
        command: UploadKnowledgeFileCommand,
        storage: &FileStorageService<S>,
    ) -> Result<KnowledgeOutcome<KnowledgeArticle>, CoreError>
    where
        S: FileStorage,
    {
        let validated = (|| {
            check_file(&command.mime_type, &command.bytes)?;
            Ok::<_, KnowledgeError>((
                normalize_title(&command.title)?,
                normalize_tags(&command.tags)?,
            ))
        })();
        let (title, tags) = match validated {
            Ok(fields) => fields,
            Err(error) => return Ok(Err(error)),
        };

        let stored = storage
            .upload(UploadFileCommand {
                mime_type: command.mime_type.clone(),
                bytes: command.bytes,
                folder: Some(storage_folder(command.organization_id)),
            })
            .await?;
        let now = Utc::now();
        let article = KnowledgeArticle {
            id: KnowledgeArticleId(generate_uuid_v7()),
            organization_id: command.organization_id,
            folder_id: command.folder_id,
            kind: KnowledgeKind::File,
            title,
            body: None,
            file: Some(KnowledgeFile {
                key: stored.key.clone(),
                mime_type: stored.mime_type,
                size_bytes: stored.size_bytes,
            }),
            content: None,
            revision: 0,
            lock: None,
            extracted_text: command.extracted_text,
            tags,
            created_at: now,
            updated_at: now,
        };
        match self.articles.insert(&article).await {
            Ok(article) => Ok(Ok(article)),
            Err(error) => {
                if let Err(discard) = storage.delete(&stored.key).await {
                    tracing::warn!(key = %stored.key, error = %discard, "could not remove an orphaned knowledge file");
                }
                Err(error)
            }
        }
    }

    pub async fn get(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> Result<KnowledgeArticle, CoreError> {
        let mut article = self
            .articles
            .find_by_id(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let now = Utc::now();
        article.lock = article.lock.filter(|lock| lock.is_active(now));
        Ok(article)
    }

    pub async fn require_document(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> Result<KnowledgeOutcome<KnowledgeArticle>, CoreError> {
        let article = self.get(organization_id, id).await?;
        if article.kind == KnowledgeKind::Document {
            Ok(Ok(article))
        } else {
            Ok(Err(KnowledgeError::NotADocument))
        }
    }

    pub async fn links(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> Result<Vec<KnowledgeLink>, CoreError> {
        self.articles.links_of(organization_id, id).await
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: KnowledgeFilter,
        limit: u64,
        offset: u64,
    ) -> Result<KnowledgeOutcome<(Vec<KnowledgeArticle>, u64)>, CoreError> {
        let tags = match normalize_tags(&filter.tags) {
            Ok(tags) => tags,
            Err(error) => return Ok(Err(error)),
        };
        let filter = KnowledgeFilter { tags, ..filter };
        let limit = limit.clamp(1, MAX_PAGE_SIZE);
        self.articles
            .list(organization_id, &filter, limit, offset)
            .await
            .map(Ok)
    }

    pub async fn update(
        &mut self,
        command: UpdateKnowledgeCommand,
    ) -> Result<KnowledgeOutcome<KnowledgeArticle>, CoreError> {
        let mut article = self.get(command.organization_id, command.id).await?;
        let applied = (|| {
            if let Some(title) = &command.title {
                article.title = normalize_title(title)?;
            }
            if let Some(tags) = &command.tags {
                article.tags = normalize_tags(tags)?;
            }
            Ok(())
        })();
        if let Err(error) = applied {
            return Ok(Err(error));
        }
        article.updated_at = Utc::now();
        self.articles.update(&article).await.map(Ok)
    }

    /// The removed article, so the caller can delete its stored file once the
    /// transaction has committed.
    pub async fn delete(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
    ) -> Result<KnowledgeArticle, CoreError> {
        let article = self.get(organization_id, id).await?;
        self.articles.delete(organization_id, id).await?;
        Ok(article)
    }

    pub async fn link(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        resource: KnowledgeResource,
    ) -> Result<KnowledgeLink, CoreError> {
        self.get(organization_id, id).await?;
        self.articles.add_link(organization_id, id, resource).await
    }

    pub async fn unlink(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeArticleId,
        resource: KnowledgeResource,
    ) -> Result<(), CoreError> {
        self.get(organization_id, id).await?;
        self.articles
            .remove_link(organization_id, id, resource)
            .await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        KnowledgeResourceType,
        domain::knowledge::{documents::EditLock, ports::MockKnowledgeArticleRepository},
    };

    fn org() -> OrganizationId {
        OrganizationId(generate_uuid_v7())
    }

    fn stored(kind: KnowledgeKind, organization_id: OrganizationId) -> KnowledgeArticle {
        let now = Utc::now();
        KnowledgeArticle {
            id: KnowledgeArticleId(generate_uuid_v7()),
            organization_id,
            folder_id: None,
            kind,
            title: "Titre".to_owned(),
            body: (kind == KnowledgeKind::Document).then(|| "texte".to_owned()),
            file: (kind == KnowledgeKind::File).then(|| KnowledgeFile {
                key: "k".to_owned(),
                mime_type: "application/pdf".to_owned(),
                size_bytes: 10,
            }),
            content: (kind == KnowledgeKind::Document).then(|| DocumentContent::empty().to_value()),
            revision: 0,
            lock: None,
            extracted_text: None,
            tags: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    #[tokio::test]
    async fn a_document_is_trimmed_and_stored_empty_at_revision_zero() {
        let mut repository = MockKnowledgeArticleRepository::new();
        repository
            .expect_insert()
            .withf(|article| {
                article.title == "Devis"
                    && article.body.as_deref() == Some("texte")
                    && article.tags == vec!["devis".to_owned()]
                    && article.revision == 0
                    && article.content == Some(DocumentContent::empty().to_value())
            })
            .returning(|article| {
                let article = article.clone();
                Box::pin(async move { Ok(article) })
            });

        let created = KnowledgeService::new(repository)
            .create_document(
                org(),
                None,
                "  Devis ",
                &[" Devis".to_owned()],
                DocumentContent::empty(),
                "texte".to_owned(),
            )
            .await
            .unwrap();
        assert_eq!(created.unwrap().kind, KnowledgeKind::Document);
    }

    #[tokio::test]
    async fn an_invalid_title_never_reaches_the_repository() {
        let mut service = KnowledgeService::new(MockKnowledgeArticleRepository::new());
        let outcome = service
            .create_document(
                org(),
                None,
                "   ",
                &[],
                DocumentContent::empty(),
                String::new(),
            )
            .await
            .unwrap();
        assert_eq!(outcome.unwrap_err(), KnowledgeError::TitleEmpty);
    }

    #[tokio::test]
    async fn the_text_of_a_file_cannot_be_edited() {
        let organization_id = org();
        let article = stored(KnowledgeKind::File, organization_id);
        let id = article.id;
        let mut repository = MockKnowledgeArticleRepository::new();
        repository.expect_find_by_id().returning(move |_, _| {
            let article = article.clone();
            Box::pin(async move { Ok(Some(article)) })
        });

        let outcome = KnowledgeService::new(repository)
            .require_document(organization_id, id)
            .await
            .unwrap();
        assert_eq!(outcome.unwrap_err(), KnowledgeError::NotADocument);
    }

    #[tokio::test]
    async fn an_expired_lock_is_not_reported() {
        let organization_id = org();
        let mut article = stored(KnowledgeKind::Document, organization_id);
        article.lock = Some(EditLock {
            holder: common::UserId(generate_uuid_v7()),
            expires_at: Utc::now() - chrono::Duration::seconds(1),
        });
        let id = article.id;
        let mut repository = MockKnowledgeArticleRepository::new();
        repository.expect_find_by_id().returning(move |_, _| {
            let article = article.clone();
            Box::pin(async move { Ok(Some(article)) })
        });

        let read = KnowledgeService::new(repository)
            .get(organization_id, id)
            .await
            .unwrap();
        assert_eq!(read.lock, None);
    }

    #[tokio::test]
    async fn linking_an_unknown_article_is_not_found() {
        let mut repository = MockKnowledgeArticleRepository::new();
        repository
            .expect_find_by_id()
            .returning(|_, _| Box::pin(async { Ok(None) }));

        let result = KnowledgeService::new(repository)
            .link(
                org(),
                KnowledgeArticleId(generate_uuid_v7()),
                KnowledgeResource {
                    resource_type: KnowledgeResourceType::Project,
                    id: generate_uuid_v7(),
                },
            )
            .await;
        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn a_list_page_is_capped() {
        let mut repository = MockKnowledgeArticleRepository::new();
        repository
            .expect_list()
            .withf(|_, _, limit, _| *limit == MAX_PAGE_SIZE)
            .returning(|_, _, _, _| Box::pin(async { Ok((Vec::new(), 0)) }));

        KnowledgeService::new(repository)
            .list(org(), KnowledgeFilter::default(), 10_000, 0)
            .await
            .unwrap()
            .unwrap();
    }
}
