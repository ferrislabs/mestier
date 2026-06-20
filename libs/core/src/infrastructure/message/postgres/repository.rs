use common::CoreError;
use discord::domain::message::ports::MessageRepository;
use discord::{ChannelId, Message, MessageId, Reaction, ReactionCount, UserId};
use mestier_macros::repository;
use sqlx::PgConnection;

use super::model::{MessageRow, ReactionAggRow, ReactionRow};
use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = Message, backend = Postgres)]
pub struct PgMessageRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgMessageRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> MessageRepository for PgMessageRepository<'tx> {
    async fn insert(&mut self, m: &Message) -> Result<Message, CoreError> {
        let mut tx = self.tx.lock().await;
        let components_json = m
            .components
            .as_ref()
            .map(|c| serde_json::to_value(c))
            .transpose()
            .map_err(|e| CoreError::Internal(format!("components serialization: {e}")))?;

        let mention_user_ids: Vec<uuid::Uuid> = m.mention_user_ids.iter().map(|u| u.0).collect();
        let mention_role_ids: Vec<uuid::Uuid> = m.mention_role_ids.iter().map(|r| r.0).collect();
        let mention_channel_ids: Vec<uuid::Uuid> =
            m.mention_channel_ids.iter().map(|c| c.0).collect();

        let row = sqlx::query_as!(
            MessageRow,
            r#"
			INSERT INTO messages (
				id, org_id, channel_id, author_type, author_user_id, author_webhook_id,
				content, components, mention_user_ids, mention_role_ids, mention_channel_ids,
				mention_everyone, edited_at, created_at
			)
			VALUES (
				$1, $2, $3, CAST($4 AS text)::author_type, $5, $6,
				$7, $8, $9, $10, $11, $12, $13, $14
			)
			RETURNING
				id, org_id, channel_id,
				author_type::text AS "author_type!",
				author_user_id, author_webhook_id,
				content, components,
				mention_user_ids AS "mention_user_ids!: Vec<uuid::Uuid>",
				mention_role_ids AS "mention_role_ids!: Vec<uuid::Uuid>",
				mention_channel_ids AS "mention_channel_ids!: Vec<uuid::Uuid>",
				mention_everyone, edited_at, created_at
			"#,
            m.id.0,
            m.organization_id.0,
            m.channel_id.0,
            m.author_type.as_str(),
            m.author_user_id.map(|u| u.0),
            m.author_webhook_id.map(|w| w.0),
            m.content,
            components_json,
            mention_user_ids.as_slice(),
            mention_role_ids.as_slice(),
            mention_channel_ids.as_slice(),
            m.mention_everyone,
            m.edited_at,
            m.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.into_message(vec![])
    }

    async fn find_by_id(&mut self, id: MessageId) -> Result<Option<Message>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            MessageRow,
            r#"
			SELECT id, org_id, channel_id,
			       author_type::text AS "author_type!",
			       author_user_id, author_webhook_id,
			       content, components,
			       mention_user_ids AS "mention_user_ids!: Vec<uuid::Uuid>",
			       mention_role_ids AS "mention_role_ids!: Vec<uuid::Uuid>",
			       mention_channel_ids AS "mention_channel_ids!: Vec<uuid::Uuid>",
			       mention_everyone, edited_at, created_at
			FROM messages WHERE id = $1
			"#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            None => Ok(None),
            Some(r) => {
                let reactions = fetch_reaction_counts(&mut tx, id).await?;
                Ok(Some(r.into_message(reactions)?))
            }
        }
    }

    async fn list_by_channel(
        &mut self,
        channel: ChannelId,
        before: Option<MessageId>,
        after: Option<MessageId>,
        limit: u64,
    ) -> Result<Vec<Message>, CoreError> {
        let mut tx = self.tx.lock().await;

        let rows = if after.is_some() && before.is_none() {
            // Forward pagination: fetch the n messages CLOSEST to (just after) the
            // cursor by ordering ASC, then reverse so callers always receive newest-first.
            let after_id = after.unwrap().0;
            let mut rows = sqlx::query_as!(
                MessageRow,
                r#"
				SELECT id, org_id, channel_id,
				       author_type::text AS "author_type!",
				       author_user_id, author_webhook_id,
				       content, components,
				       mention_user_ids AS "mention_user_ids!: Vec<uuid::Uuid>",
				       mention_role_ids AS "mention_role_ids!: Vec<uuid::Uuid>",
				       mention_channel_ids AS "mention_channel_ids!: Vec<uuid::Uuid>",
				       mention_everyone, edited_at, created_at
				FROM messages
				WHERE channel_id = $1
				  AND id > $2
				ORDER BY id ASC
				LIMIT $3
				"#,
                channel.0,
                after_id,
                limit as i64,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
            // Reverse to restore newest-first order, consistent with other pagination paths.
            rows.reverse();
            rows
        } else {
            sqlx::query_as!(
                MessageRow,
                r#"
				SELECT id, org_id, channel_id,
				       author_type::text AS "author_type!",
				       author_user_id, author_webhook_id,
				       content, components,
				       mention_user_ids AS "mention_user_ids!: Vec<uuid::Uuid>",
				       mention_role_ids AS "mention_role_ids!: Vec<uuid::Uuid>",
				       mention_channel_ids AS "mention_channel_ids!: Vec<uuid::Uuid>",
				       mention_everyone, edited_at, created_at
				FROM messages
				WHERE channel_id = $1
				  AND ($2::uuid IS NULL OR id < $2)
				ORDER BY id DESC
				LIMIT $3
				"#,
                channel.0,
                before.map(|m| m.0),
                limit as i64,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
        };

        let mut messages = Vec::with_capacity(rows.len());
        for row in rows {
            let id = MessageId(row.id);
            let reactions = fetch_reaction_counts(&mut tx, id).await?;
            messages.push(row.into_message(reactions)?);
        }
        Ok(messages)
    }

    async fn update(&mut self, m: &Message) -> Result<Message, CoreError> {
        let mut tx = self.tx.lock().await;
        let components_json = m
            .components
            .as_ref()
            .map(|c| serde_json::to_value(c))
            .transpose()
            .map_err(|e| CoreError::Internal(format!("components serialization: {e}")))?;

        let mention_user_ids: Vec<uuid::Uuid> = m.mention_user_ids.iter().map(|u| u.0).collect();
        let mention_role_ids: Vec<uuid::Uuid> = m.mention_role_ids.iter().map(|r| r.0).collect();
        let mention_channel_ids: Vec<uuid::Uuid> =
            m.mention_channel_ids.iter().map(|c| c.0).collect();

        let row = sqlx::query_as!(
            MessageRow,
            r#"
			UPDATE messages
			SET content = $2,
			    components = $3,
			    edited_at = $4,
			    mention_user_ids = $5,
			    mention_role_ids = $6,
			    mention_channel_ids = $7,
			    mention_everyone = $8
			WHERE id = $1
			RETURNING
				id, org_id, channel_id,
				author_type::text AS "author_type!",
				author_user_id, author_webhook_id,
				content, components,
				mention_user_ids AS "mention_user_ids!: Vec<uuid::Uuid>",
				mention_role_ids AS "mention_role_ids!: Vec<uuid::Uuid>",
				mention_channel_ids AS "mention_channel_ids!: Vec<uuid::Uuid>",
				mention_everyone, edited_at, created_at
			"#,
            m.id.0,
            m.content,
            components_json,
            m.edited_at,
            mention_user_ids.as_slice(),
            mention_role_ids.as_slice(),
            mention_channel_ids.as_slice(),
            m.mention_everyone,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            None => Err(CoreError::NotFound),
            Some(r) => {
                let reactions = fetch_reaction_counts(&mut tx, m.id).await?;
                r.into_message(reactions)
            }
        }
    }

    async fn delete(&mut self, id: MessageId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM messages WHERE id = $1", id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    async fn add_reaction(&mut self, r: &Reaction) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
			INSERT INTO message_reactions (message_id, emoji, user_id, created_at)
			VALUES ($1, $2, $3, $4)
			ON CONFLICT (message_id, emoji, user_id) DO NOTHING
			"#,
            r.message_id.0,
            r.emoji,
            r.user_id.0,
            r.created_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn remove_reaction(
        &mut self,
        message_id: MessageId,
        emoji: &str,
        user_id: UserId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            "DELETE FROM message_reactions WHERE message_id = $1 AND emoji = $2 AND user_id = $3",
            message_id.0,
            emoji,
            user_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    async fn list_reactions(&mut self, message_id: MessageId) -> Result<Vec<Reaction>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ReactionRow,
            r#"
			SELECT message_id, emoji, user_id, created_at
			FROM message_reactions WHERE message_id = $1
			ORDER BY created_at ASC
			"#,
            message_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }
}

/// Aggregate reactions per emoji for a given message (called by find_by_id / list_by_channel).
async fn fetch_reaction_counts(
    conn: &mut PgConnection,
    message_id: MessageId,
) -> Result<Vec<ReactionCount>, CoreError> {
    let rows = sqlx::query_as!(
        ReactionAggRow,
        r#"
		SELECT emoji,
		       COUNT(*)                                    AS "count!: i64",
		       array_agg(user_id ORDER BY created_at)     AS "user_ids!: Vec<uuid::Uuid>"
		FROM message_reactions
		WHERE message_id = $1
		GROUP BY emoji
		ORDER BY emoji
		"#,
        message_id.0,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)?;
    Ok(rows.into_iter().map(Into::into).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use common::{OrganizationId, generate_uuid_v7};
    use discord::{AuthorType, ChannelId, Message, MessageId, UserId};
    use sqlx::PgPool;

    async fn make_pool() -> PgPool {
        PgPool::connect("postgres://ferriskey:ferriskey@localhost:5433/mestier")
            .await
            .unwrap()
    }

    /// Seeds a throwaway user + organization + channel in the current transaction.
    /// Returns `(OrganizationId, UserId, ChannelId)`.
    async fn seed_org_user_channel(
        tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    ) -> (OrganizationId, UserId, ChannelId) {
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

        let channel_id = generate_uuid_v7();
        sqlx::query!(
			r#"INSERT INTO channels (id, org_id, channel_type, name, position, archived, created_at, updated_at)
			   VALUES ($1, $2, 'TEXT'::channel_type, $3, 0, false, now(), now())"#,
			channel_id,
			org_id,
			format!("test-channel-{}", channel_id),
		)
		.execute(&mut **tx)
		.await
		.unwrap();

        (
            OrganizationId(org_id),
            UserId(user_id),
            ChannelId(channel_id),
        )
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn update_persists_reparsed_mentions() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let (org_id, author_id, channel_id) = {
                let mut guard = tx.lock().await;
                seed_org_user_channel(*guard).await
            };

            // Seed a second user to mention later
            let other_user_id = UserId(generate_uuid_v7());
            {
                let mut guard = tx.lock().await;
                sqlx::query!(
                    r#"INSERT INTO users (id, email, username, display_name, sub)
					   VALUES ($1, $2, $3, $4, $5)"#,
                    other_user_id.0,
                    format!("test-{}@example.com", other_user_id.0),
                    format!("user-{}", other_user_id.0),
                    "Other User",
                    format!("sub-{}", other_user_id.0),
                )
                .execute(&mut ***guard)
                .await
                .unwrap();
            }

            let mut repo = PgMessageRepository::new(&tx);

            // Insert an initial message mentioning author_id
            let msg_id = MessageId(generate_uuid_v7());
            let now = Utc::now();
            let original = Message {
                id: msg_id,
                organization_id: org_id,
                channel_id,
                author_type: AuthorType::User,
                author_user_id: Some(author_id),
                author_webhook_id: None,
                content: "hello".into(),
                components: None,
                mention_user_ids: vec![author_id],
                mention_role_ids: vec![],
                mention_channel_ids: vec![],
                mention_everyone: false,
                reactions: vec![],
                edited_at: None,
                created_at: now,
            };
            repo.insert(&original).await.unwrap();

            // Update: different mention arrays
            let updated_msg = Message {
                mention_user_ids: vec![other_user_id],
                mention_role_ids: vec![],
                mention_channel_ids: vec![channel_id],
                mention_everyone: true,
                content: "hello everyone".into(),
                edited_at: Some(Utc::now()),
                ..original.clone()
            };
            repo.update(&updated_msg).await.unwrap();

            // Re-fetch and assert stored mentions equal the new ones
            let fetched = repo
                .find_by_id(msg_id)
                .await
                .unwrap()
                .expect("message must exist");
            assert_eq!(fetched.mention_user_ids, vec![other_user_id]);
            assert_eq!(fetched.mention_channel_ids, vec![channel_id]);
            assert!(fetched.mention_everyone);

            Err::<(), _>(CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }
}
