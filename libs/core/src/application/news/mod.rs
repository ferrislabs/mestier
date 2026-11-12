use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use mestier_macros::transactional;

use crate::{
    CreateNewsCommand, News, NewsContent, NewsId, NewsOutcome, PlatformAdmin, UpdateNewsCommand,
    UserId, application::MestierUseCase, domain::news::ports::NewsRepository,
};

impl MestierUseCase {
    #[transactional(news)]
    pub async fn list_news_for_admin(&self, _: PlatformAdmin) -> Result<Vec<News>, CoreError> {
        let mut news_repository = news_repository;

        news_repository.list_all().await
    }

    #[transactional(news)]
    pub async fn create_news(
        &self,
        command: CreateNewsCommand,
    ) -> Result<NewsOutcome<News>, CoreError> {
        let mut news_repository = news_repository;
        let content = match NewsContent::try_from(command.content) {
            Ok(content) => content,
            Err(error) => return Ok(Err(error)),
        };
        let now = Utc::now();
        let news = News {
            id: NewsId(generate_uuid_v7()),
            content,
            published_at: command.published_at,
            created_at: now,
            updated_at: now,
        };
        news_repository.insert(&news).await.map(Ok)
    }

    #[transactional(news)]
    pub async fn update_news(
        &self,
        command: UpdateNewsCommand,
    ) -> Result<NewsOutcome<News>, CoreError> {
        let mut news_repository = news_repository;
        let content = match NewsContent::try_from(command.content) {
            Ok(content) => content,
            Err(error) => return Ok(Err(error)),
        };
        let existing = news_repository
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let news = News {
            content,
            published_at: command.published_at,
            updated_at: Utc::now(),
            ..existing
        };
        news_repository
            .update(&news)
            .await?
            .ok_or(CoreError::NotFound)
            .map(Ok)
    }

    #[transactional(news)]
    pub async fn delete_news(&self, _: PlatformAdmin, id: NewsId) -> Result<(), CoreError> {
        let mut news_repository = news_repository;

        if news_repository.delete(id).await? {
            Ok(())
        } else {
            Err(CoreError::NotFound)
        }
    }

    #[transactional(news)]
    pub async fn list_published_news(&self) -> Result<Vec<News>, CoreError> {
        let mut news_repository = news_repository;
        news_repository.list_published(Utc::now()).await
    }

    #[transactional(news)]
    pub async fn list_unseen_news(&self, user_id: UserId) -> Result<Vec<News>, CoreError> {
        let mut news_repository = news_repository;
        news_repository.list_unseen(user_id, Utc::now()).await
    }

    #[transactional(news)]
    pub async fn mark_news_seen(&self, user_id: UserId, ids: Vec<NewsId>) -> Result<(), CoreError> {
        let mut news_repository = news_repository;
        news_repository.mark_seen(user_id, &ids, Utc::now()).await
    }
}
