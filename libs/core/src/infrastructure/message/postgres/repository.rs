use common::CoreError;
use discord::domain::message::ports::MessageRepository;
use discord::{ChannelId, Message, MessageId, Reaction, ReactionCount, UserId};
use mestier_macros::repository;
use sqlx::PgConnection;

use super::model::{MessageAttachmentRow, MessageRow, ReactionAggRow, ReactionRow};
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
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| CoreError::Internal(format!("components serialization: {e}")))?;

        let mention_user_ids: Vec<uuid::Uuid> = m.mention_user_ids.iter().map(|u| u.0).collect();
        let mention_role_ids: Vec<uuid::Uuid> = m.mention_role_ids.iter().map(|r| r.0).collect();
        let mention_channel_ids: Vec<uuid::Uuid> =
            m.mention_channel_ids.iter().map(|c| c.0).collect();

        let row = sqlx::query_as!(
            MessageRow,
            r#"
			INSERT INTO chat.messages (
				id, org_id, channel_id, author_type, author_user_id, author_webhook_id,
				content, components, mention_user_ids, mention_role_ids, mention_channel_ids,
				mention_everyone, edited_at, created_at
			)
			VALUES (
				$1, $2, $3, CAST($4 AS text)::chat.author_type, $5, $6,
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

        // Persist attachment rows for this message (ordered by their position in the input list).
        for (position, attachment) in m.attachments.iter().enumerate() {
            sqlx::query!(
                r#"
                INSERT INTO chat.message_attachments
                    (id, org_id, message_id, storage_key, filename, mime_type,
                     size_bytes, position, created_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
                attachment.id.0,
                m.organization_id.0,
                m.id.0,
                attachment.storage_key,
                attachment.filename,
                attachment.mime_type,
                attachment.size_bytes,
                position as i32,
                attachment.created_at,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        row.into_message(vec![], m.attachments.clone())
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
			FROM chat.messages WHERE id = $1
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
                let attachments = fetch_attachments(&mut **tx, id).await?;
                Ok(Some(r.into_message(reactions, attachments)?))
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

        let rows = if let (Some(after_msg), None) = (after, before) {
            // Forward pagination: fetch the n messages CLOSEST to (just after) the
            // cursor by ordering ASC, then reverse so callers always receive newest-first.
            let after_id = after_msg.0;
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
				FROM chat.messages
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
				FROM chat.messages
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
            let attachments = fetch_attachments(&mut **tx, id).await?;
            messages.push(row.into_message(reactions, attachments)?);
        }
        Ok(messages)
    }

    async fn update(&mut self, m: &Message) -> Result<Message, CoreError> {
        let mut tx = self.tx.lock().await;
        let components_json = m
            .components
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|e| CoreError::Internal(format!("components serialization: {e}")))?;

        let mention_user_ids: Vec<uuid::Uuid> = m.mention_user_ids.iter().map(|u| u.0).collect();
        let mention_role_ids: Vec<uuid::Uuid> = m.mention_role_ids.iter().map(|r| r.0).collect();
        let mention_channel_ids: Vec<uuid::Uuid> =
            m.mention_channel_ids.iter().map(|c| c.0).collect();

        let row = sqlx::query_as!(
            MessageRow,
            r#"
			UPDATE chat.messages
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
                let attachments = fetch_attachments(&mut **tx, m.id).await?;
                r.into_message(reactions, attachments)
            }
        }
    }

    async fn delete(&mut self, id: MessageId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!("DELETE FROM chat.messages WHERE id = $1", id.0)
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
			INSERT INTO chat.message_reactions (message_id, emoji, user_id, created_at)
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
            "DELETE FROM chat.message_reactions WHERE message_id = $1 AND emoji = $2 AND user_id = $3",
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
			FROM chat.message_reactions WHERE message_id = $1
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
		FROM chat.message_reactions
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

/// Load attachments for a single message, ordered by position then id (stable tie-break).
/// Called by find_by_id / list_by_channel / update RETURNING paths — same pattern as
/// fetch_reaction_counts; the caller already holds the tx lock so we receive a bare
/// &mut PgConnection.
pub(super) async fn fetch_attachments(
    conn: &mut sqlx::PgConnection,
    message_id: MessageId,
) -> Result<Vec<discord::Attachment>, CoreError> {
    let rows = sqlx::query_as!(
        MessageAttachmentRow,
        r#"
        SELECT id, message_id, storage_key, filename, mime_type,
               size_bytes, position, created_at
        FROM chat.message_attachments
        WHERE message_id = $1
        ORDER BY position ASC, id ASC
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
			r#"INSERT INTO chat.channels (id, org_id, channel_type, name, position, archived, created_at, updated_at)
			   VALUES ($1, $2, 'TEXT'::chat.channel_type, $3, 0, false, now(), now())"#,
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
                attachments: vec![], // shim — field added by Plan 1; Task 3 adds the real INSERT/load
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

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn insert_message_with_attachments_and_reload() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let (org_id, author_id, channel_id) = {
                let mut guard = tx.lock().await;
                seed_org_user_channel(*guard).await
            };

            let mut repo = PgMessageRepository::new(&tx);

            let msg_id = MessageId(generate_uuid_v7());
            let now = Utc::now();

            // Build two attachments (order matters: position 0 and 1)
            let att_0 = discord::Attachment {
                id: discord::AttachmentId(generate_uuid_v7()),
                storage_key: "uploads/attachments/file-a".to_string(),
                filename: "photo.png".to_string(),
                mime_type: "image/png".to_string(),
                size_bytes: 2048,
                created_at: now,
            };
            let att_1 = discord::Attachment {
                id: discord::AttachmentId(generate_uuid_v7()),
                storage_key: "uploads/attachments/file-b".to_string(),
                filename: "doc.pdf".to_string(),
                mime_type: "application/pdf".to_string(),
                size_bytes: 51200,
                created_at: now,
            };

            let original = discord::Message {
                id: msg_id,
                organization_id: org_id,
                channel_id,
                author_type: discord::AuthorType::User,
                author_user_id: Some(author_id),
                author_webhook_id: None,
                content: "see attached".to_string(),
                components: None,
                mention_user_ids: vec![],
                mention_role_ids: vec![],
                mention_channel_ids: vec![],
                mention_everyone: false,
                reactions: vec![],
                attachments: vec![att_0.clone(), att_1.clone()],
                edited_at: None,
                created_at: now,
            };

            // insert
            repo.insert(&original).await.unwrap();

            // reload via find_by_id and assert round-trip
            let fetched = repo
                .find_by_id(msg_id)
                .await
                .unwrap()
                .expect("message must exist");
            assert_eq!(fetched.attachments.len(), 2, "expected 2 attachments");
            // position 0 comes first
            assert_eq!(fetched.attachments[0].filename, "photo.png");
            assert_eq!(
                fetched.attachments[0].storage_key,
                "uploads/attachments/file-a"
            );
            assert_eq!(fetched.attachments[0].size_bytes, 2048);
            // position 1 comes second
            assert_eq!(fetched.attachments[1].filename, "doc.pdf");
            assert_eq!(fetched.attachments[1].size_bytes, 51200);

            // also verify list_by_channel surfaces the attachments
            let listed = repo
                .list_by_channel(channel_id, None, None, 10)
                .await
                .unwrap();
            assert_eq!(listed.len(), 1);
            assert_eq!(listed[0].attachments.len(), 2);
            assert_eq!(listed[0].attachments[0].filename, "photo.png");

            Err::<(), _>(CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }
}
