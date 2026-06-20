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
        let rows = sqlx::query_as!(
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
			  AND ($3::uuid IS NULL OR id > $3)
			ORDER BY id DESC
			LIMIT $4
			"#,
            channel.0,
            before.map(|m| m.0),
            after.map(|m| m.0),
            limit as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

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

        let row = sqlx::query_as!(
            MessageRow,
            r#"
			UPDATE messages
			SET content = $2, components = $3, edited_at = $4
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
