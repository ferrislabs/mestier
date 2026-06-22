use common::CoreError;
use discord::{Channel, ChannelId, ChannelRepository, OrganizationId};
use mestier_macros::repository;

use super::model::ChannelRow;
use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = Channel, backend = Postgres)]
pub struct PgChannelRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgChannelRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> ChannelRepository for PgChannelRepository<'tx> {
    async fn insert(&mut self, ch: &Channel) -> Result<Channel, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
			ChannelRow,
			r#"
            INSERT INTO channels (id, org_id, channel_type, name, topic, position, category_id, parent_id, origin_message_id, archived, created_at, updated_at)
            VALUES ($1, $2, CAST($3 AS text)::channel_type, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, created_at, updated_at
            "#,
			ch.id.0,
			ch.organization_id.0,
			ch.channel_type.as_str(),
			ch.name,
			ch.topic,
			ch.position,
			ch.category_id.map(|x| x.0),
			ch.parent_id.map(|x| x.0),
			ch.origin_message_id.map(|x| x.0),
			ch.archived,
			ch.created_at,
			ch.updated_at,
		)
		.fetch_one(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;
        row.try_into()
    }

    async fn find_by_id(&mut self, id: ChannelId) -> Result<Option<Channel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
			ChannelRow,
			r#"
            SELECT id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, created_at, updated_at
            FROM channels WHERE id = $1
            "#,
			id.0,
		)
		.fetch_optional(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;
        row.map(TryInto::try_into).transpose()
    }

    async fn list_by_organization(
        &mut self,
        org: OrganizationId,
    ) -> Result<Vec<Channel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
			ChannelRow,
			r#"
            SELECT id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, created_at, updated_at
            FROM channels WHERE org_id = $1 AND channel_type = 'TEXT'
            ORDER BY position ASC, id ASC
            "#,
			org.0,
		)
		.fetch_all(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn list_threads(&mut self, parent: ChannelId) -> Result<Vec<Channel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
			ChannelRow,
			r#"
            SELECT id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, created_at, updated_at
            FROM channels WHERE parent_id = $1 AND channel_type = 'THREAD'
            ORDER BY id ASC
            "#,
			parent.0,
		)
		.fetch_all(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;
        rows.into_iter().map(TryInto::try_into).collect()
    }

    async fn update(&mut self, ch: &Channel) -> Result<Channel, CoreError> {
        let mut tx = self.tx.lock().await;
        // `channel_type` and `parent_id` are immutable after creation — no domain
        // command exposes changing them — so they are intentionally not in the SET.
        let row = sqlx::query_as!(
			ChannelRow,
			r#"
            UPDATE channels
            SET name = $2, topic = $3, position = $4, category_id = $5, archived = $6, updated_at = $7
            WHERE id = $1
            RETURNING id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, created_at, updated_at
            "#,
			ch.id.0,
			ch.name,
			ch.topic,
			ch.position,
			ch.category_id.map(|x| x.0),
			ch.archived,
			ch.updated_at,
		)
		.fetch_optional(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;
        row.ok_or(CoreError::NotFound)?.try_into()
    }

    async fn delete(&mut self, id: ChannelId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM channels WHERE id = $1", id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use common::{OrganizationId, generate_uuid_v7};
    use discord::{Channel, ChannelId, ChannelType};
    use sqlx::PgPool;

    async fn make_pool() -> PgPool {
        PgPool::connect("postgres://ferriskey:ferriskey@localhost:5433/mestier")
            .await
            .unwrap()
    }

    /// Seeds a throwaway user + organization inside the current transaction so
    /// the FK `channels_org_id_fkey → organizations(id)` is satisfied.
    /// Returns the generated `OrganizationId`.
    async fn seed_org(tx: &mut sqlx::Transaction<'static, sqlx::Postgres>) -> OrganizationId {
        let user_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            user_id,
            format!("test-{}@example.com", user_id),
            format!("user-{}", user_id),
            "Test User",
            format!("sub-{}", user_id),
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            "Test Org",
            format!("test-org-{}", org_id),
            user_id,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn insert_text_channel_and_list() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let org_id = {
                let mut guard = tx.lock().await;
                seed_org(*guard).await
            };

            let ch = Channel {
                id: ChannelId(generate_uuid_v7()),
                organization_id: org_id,
                channel_type: ChannelType::Text,
                name: "general".into(),
                topic: None,
                position: 0,
                category_id: None,
                parent_id: None,
                origin_message_id: None,
                archived: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let mut repo = PgChannelRepository::new(&tx);
            let inserted = repo.insert(&ch).await.unwrap();
            assert_eq!(inserted.name, "general");
            assert_eq!(inserted.channel_type, ChannelType::Text);

            let list = repo.list_by_organization(org_id).await.unwrap();
            assert!(list.iter().any(|c| c.id == ch.id));
            assert!(list.iter().all(|c| c.channel_type == ChannelType::Text));

            Err::<(), _>(common::CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn insert_thread_and_list_threads() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let org_id = {
                let mut guard = tx.lock().await;
                seed_org(*guard).await
            };

            let parent = Channel {
                id: ChannelId(generate_uuid_v7()),
                organization_id: org_id,
                channel_type: ChannelType::Text,
                name: "announcements".into(),
                topic: None,
                position: 1,
                category_id: None,
                parent_id: None,
                origin_message_id: None,
                archived: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let thread = Channel {
                id: ChannelId(generate_uuid_v7()),
                organization_id: org_id,
                channel_type: ChannelType::Thread,
                name: "my-thread".into(),
                topic: None,
                position: 0,
                category_id: None,
                parent_id: Some(parent.id),
                origin_message_id: None,
                archived: false,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let mut repo = PgChannelRepository::new(&tx);
            repo.insert(&parent).await.unwrap();
            let inserted_thread = repo.insert(&thread).await.unwrap();
            assert_eq!(inserted_thread.channel_type, ChannelType::Thread);

            let threads = repo.list_threads(parent.id).await.unwrap();
            assert!(threads.iter().any(|t| t.id == thread.id));
            let text_channels = repo.list_by_organization(org_id).await.unwrap();
            assert!(
                text_channels
                    .iter()
                    .all(|c| c.channel_type == ChannelType::Text)
            );

            Err::<(), _>(common::CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }
}
