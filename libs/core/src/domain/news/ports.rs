use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{News, NewsId, UserId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait NewsRepository: Send {
    fn insert(&mut self, news: &News) -> impl Future<Output = Result<News, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: NewsId,
    ) -> impl Future<Output = Result<Option<News>, CoreError>> + Send;

    /// Replaces content, publication date and `updated_at`. `None` when the
    /// news does not exist.
    fn update(
        &mut self,
        news: &News,
    ) -> impl Future<Output = Result<Option<News>, CoreError>> + Send;

    /// Whether a news was removed. Its views go with it, by cascade.
    fn delete(&mut self, id: NewsId) -> impl Future<Output = Result<bool, CoreError>> + Send;

    /// Every news, drafts and scheduled ones included, most recently updated first.
    fn list_all(&mut self) -> impl Future<Output = Result<Vec<News>, CoreError>> + Send;

    /// News published at or before `now`, most recently published first.
    fn list_published(
        &mut self,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<News>, CoreError>> + Send;

    /// Published news `user_id` has not seen, most recently published first.
    fn list_unseen(
        &mut self,
        user_id: UserId,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<News>, CoreError>> + Send;

    /// Idempotent. Ids that are unknown or not published at `now` are ignored.
    fn mark_seen(
        &mut self,
        user_id: UserId,
        ids: &[NewsId],
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
