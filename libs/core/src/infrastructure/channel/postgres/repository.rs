use common::{CoreError, ProjectId};
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
            INSERT INTO chat.channels (id, org_id, channel_type, name, topic, position, category_id, parent_id, origin_message_id, archived, project_id, created_at, updated_at)
            VALUES ($1, $2, CAST($3 AS text)::chat.channel_type, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
            RETURNING id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, project_id, created_at, updated_at
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
			ch.project_id.map(|x| x.0),
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
            SELECT id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, project_id, created_at, updated_at
            FROM chat.channels WHERE id = $1
            "#,
			id.0,
		)
		.fetch_optional(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;
        row.map(TryInto::try_into).transpose()
    }

    /// Not filtered on `archived`: see the port's own doc comment — an
    /// archived project's channel must still resolve here so restoring the
    /// project can find it again.
    async fn find_by_project_id(
        &mut self,
        project_id: ProjectId,
    ) -> Result<Option<Channel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
			ChannelRow,
			r#"
            SELECT id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, project_id, created_at, updated_at
            FROM chat.channels WHERE project_id = $1
            "#,
			project_id.0,
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
            SELECT id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, project_id, created_at, updated_at
            FROM chat.channels WHERE org_id = $1 AND channel_type = 'TEXT'
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
            SELECT id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, project_id, created_at, updated_at
            FROM chat.channels WHERE parent_id = $1 AND channel_type = 'THREAD'
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
        // `channel_type`, `parent_id` and `project_id` are immutable after
        // creation — no domain command exposes changing them — so they are
        // intentionally not in the SET. `project_id`'s own archive toggle
        // goes through `set_archived` instead.
        let row = sqlx::query_as!(
			ChannelRow,
			r#"
            UPDATE chat.channels
            SET name = $2, topic = $3, position = $4, category_id = $5, archived = $6, updated_at = $7
            WHERE id = $1
            RETURNING id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, project_id, created_at, updated_at
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

    async fn set_archived(&mut self, id: ChannelId, archived: bool) -> Result<Channel, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
			ChannelRow,
			r#"
            UPDATE chat.channels
            SET archived = $2, updated_at = $3
            WHERE id = $1
            RETURNING id, org_id, channel_type::text AS "channel_type!", name, topic, position, category_id, parent_id, origin_message_id, archived, project_id, created_at, updated_at
            "#,
			id.0,
			archived,
			chrono::Utc::now(),
		)
		.fetch_optional(&mut ***tx)
		.await
		.map_err(map_sqlx_error)?;
        row.ok_or(CoreError::NotFound)?.try_into()
    }

    async fn delete(&mut self, id: ChannelId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM chat.channels WHERE id = $1", id.0)
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
    use crate::application::test_support::dev_pool;
    use chrono::Utc;
    use common::{OrganizationId, generate_uuid_v7};
    use discord::{Channel, ChannelId, ChannelType};
    use sqlx::PgPool;

    async fn make_pool() -> PgPool {
        dev_pool().await
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

    /// Seeds a throwaway project inside the current transaction so the
    /// composite FK `fk_channels_project → projects(id, org_id)` is satisfied.
    async fn seed_project(
        tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
        org_id: OrganizationId,
    ) -> ProjectId {
        let project_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO projects (id, org_id, name) VALUES ($1, $2, $3)"#,
            project_id,
            org_id.0,
            "Test Project",
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        ProjectId(project_id)
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
                project_id: None,
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
                project_id: None,
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
                project_id: None,
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

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn insert_project_channel_and_find_it_by_project_id() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let (org_id, project_id) = {
                let mut guard = tx.lock().await;
                let org_id = seed_org(*guard).await;
                let project_id = seed_project(*guard, org_id).await;
                (org_id, project_id)
            };

            let ch = Channel {
                id: ChannelId(generate_uuid_v7()),
                organization_id: org_id,
                channel_type: ChannelType::Text,
                name: "chantier-dupont".into(),
                topic: None,
                position: 0,
                category_id: None,
                parent_id: None,
                origin_message_id: None,
                archived: false,
                project_id: Some(project_id),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let mut repo = PgChannelRepository::new(&tx);
            repo.insert(&ch).await.unwrap();

            let found = repo.find_by_project_id(project_id).await.unwrap().unwrap();
            assert_eq!(found.id, ch.id);
            assert_eq!(found.project_id, Some(project_id));

            Err::<(), _>(common::CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }

    /// `uq_channels_project_id` is the actual enforcement (see the domain
    /// service's own check, which only exists for a cleaner error) — this
    /// proves the database itself refuses a second channel for one project.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_second_channel_for_the_same_project_violates_the_unique_constraint() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let (org_id, project_id) = {
                let mut guard = tx.lock().await;
                let org_id = seed_org(*guard).await;
                let project_id = seed_project(*guard, org_id).await;
                (org_id, project_id)
            };

            let make_channel = |name: &str| Channel {
                id: ChannelId(generate_uuid_v7()),
                organization_id: org_id,
                channel_type: ChannelType::Text,
                name: name.into(),
                topic: None,
                position: 0,
                category_id: None,
                parent_id: None,
                origin_message_id: None,
                archived: false,
                project_id: Some(project_id),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let mut repo = PgChannelRepository::new(&tx);
            repo.insert(&make_channel("first")).await.unwrap();
            let second = repo.insert(&make_channel("second")).await;

            assert!(matches!(second, Err(common::CoreError::Conflict(_))));

            Err::<(), _>(common::CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn set_archived_flips_the_flag_and_returns_the_channel() {
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
                project_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let mut repo = PgChannelRepository::new(&tx);
            repo.insert(&ch).await.unwrap();

            let archived = repo.set_archived(ch.id, true).await.unwrap();
            assert!(archived.archived);

            let restored = repo.set_archived(ch.id, false).await.unwrap();
            assert!(!restored.archived);

            Err::<(), _>(common::CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }

    /// `chk_channels_thread_no_project`: a THREAD can never carry a
    /// `project_id`, enforced at the database regardless of what the domain
    /// service does (the service never sets one on a thread in the first
    /// place — see `ChannelService::create_thread` — so this is the
    /// belt-and-suspenders the issue asked for).
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_thread_cannot_carry_a_project_id() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let (org_id, project_id) = {
                let mut guard = tx.lock().await;
                let org_id = seed_org(*guard).await;
                let project_id = seed_project(*guard, org_id).await;
                (org_id, project_id)
            };

            let mut repo = PgChannelRepository::new(&tx);

            let parent = Channel {
                id: ChannelId(generate_uuid_v7()),
                organization_id: org_id,
                channel_type: ChannelType::Text,
                name: "announcements".into(),
                topic: None,
                position: 0,
                category_id: None,
                parent_id: None,
                origin_message_id: None,
                archived: false,
                project_id: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };
            repo.insert(&parent).await.unwrap();

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
                project_id: Some(project_id),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            // A real, valid parent — the only thing wrong with this insert is
            // `project_id` on a THREAD, which is exactly what this test means
            // to isolate.
            let result = repo.insert(&thread).await;

            assert!(matches!(result, Err(common::CoreError::Database(_))));

            Err::<(), _>(common::CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }
}
