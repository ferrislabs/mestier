use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    News, NewsId, UserId,
    domain::news::ports::NewsRepository,
    infrastructure::{
        news::postgres::model::NewsRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = News, backend = Postgres)]
pub struct PgNewsRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgNewsRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

fn json<T: serde::Serialize>(value: &T) -> Result<serde_json::Value, CoreError> {
    serde_json::to_value(value).map_err(|e| CoreError::Internal(format!("news json: {e}")))
}

impl<'tx> NewsRepository for PgNewsRepository<'tx> {
    async fn insert(&mut self, news: &News) -> Result<News, CoreError> {
        let items = json(&news.content.items)?;
        let cta = news.content.cta.as_ref().map(json).transpose()?;

        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            NewsRow,
            r#"
            INSERT INTO news (id, title, image_key, items, cta, published_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, title, image_key, items, cta, published_at, created_at, updated_at
            "#,
            news.id.0,
            news.content.title,
            news.content.image_key,
            items,
            cta,
            news.published_at,
            news.created_at,
            news.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_by_id(&mut self, id: NewsId) -> Result<Option<News>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            NewsRow,
            r#"
            SELECT id, title, image_key, items, cta, published_at, created_at, updated_at
            FROM news
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(News::try_from).transpose()
    }

    async fn update(&mut self, news: &News) -> Result<Option<News>, CoreError> {
        let items = json(&news.content.items)?;
        let cta = news.content.cta.as_ref().map(json).transpose()?;

        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            NewsRow,
            r#"
            UPDATE news
            SET title = $2, image_key = $3, items = $4, cta = $5, published_at = $6, updated_at = $7
            WHERE id = $1
            RETURNING id, title, image_key, items, cta, published_at, created_at, updated_at
            "#,
            news.id.0,
            news.content.title,
            news.content.image_key,
            items,
            cta,
            news.published_at,
            news.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(News::try_from).transpose()
    }

    async fn delete(&mut self, id: NewsId) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM news WHERE id = $1", id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_all(&mut self) -> Result<Vec<News>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            NewsRow,
            r#"
            SELECT id, title, image_key, items, cta, published_at, created_at, updated_at
            FROM news
            ORDER BY updated_at DESC, id
            "#,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(News::try_from).collect()
    }

    async fn list_published(&mut self, now: DateTime<Utc>) -> Result<Vec<News>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            NewsRow,
            r#"
            SELECT id, title, image_key, items, cta, published_at, created_at, updated_at
            FROM news
            WHERE published_at IS NOT NULL AND published_at <= $1
            ORDER BY published_at DESC, id
            "#,
            now,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(News::try_from).collect()
    }

    async fn list_unseen(
        &mut self,
        user_id: UserId,
        now: DateTime<Utc>,
    ) -> Result<Vec<News>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            NewsRow,
            r#"
            SELECT n.id, n.title, n.image_key, n.items, n.cta, n.published_at, n.created_at, n.updated_at
            FROM news n
            WHERE n.published_at IS NOT NULL AND n.published_at <= $1
              AND NOT EXISTS (
                    SELECT 1 FROM news_views v WHERE v.news_id = n.id AND v.user_id = $2)
            ORDER BY n.published_at DESC, n.id
            "#,
            now,
            user_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(News::try_from).collect()
    }

    async fn mark_seen(
        &mut self,
        user_id: UserId,
        ids: &[NewsId],
        now: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let ids: Vec<uuid::Uuid> = ids.iter().map(|id| id.0).collect();

        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            INSERT INTO news_views (news_id, user_id, seen_at)
            SELECT id, $2, $3 FROM news
            WHERE id = ANY($1) AND published_at IS NOT NULL AND published_at <= $3
            ON CONFLICT (news_id, user_id) DO NOTHING
            "#,
            &ids,
            user_id.0,
            now,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }
}
