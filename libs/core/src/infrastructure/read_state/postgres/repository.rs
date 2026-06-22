use common::CoreError;
use discord::domain::read_state::ports::ReadStateRepository;
use discord::{ChannelId, ChannelReadState, MarkChannelReadCommand, OrganizationId, UserId};
use mestier_macros::repository;

use super::model::ChannelReadStateRow;
use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = ReadState, backend = Postgres)]
pub struct PgReadStateRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgReadStateRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> ReadStateRepository for PgReadStateRepository<'tx> {
    async fn upsert(
        &self,
        command: MarkChannelReadCommand,
    ) -> Result<Option<ChannelReadState>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ChannelReadStateRow,
            r#"
            INSERT INTO chat.channel_read_state
                (user_id, channel_id, org_id, last_read_message_id, updated_at)
            VALUES ($1, $2, $3, $4, now())
            ON CONFLICT (user_id, channel_id) DO UPDATE
                SET last_read_message_id = EXCLUDED.last_read_message_id,
                    updated_at           = now()
            WHERE EXCLUDED.last_read_message_id > chat.channel_read_state.last_read_message_id
            RETURNING user_id, channel_id, org_id, last_read_message_id, updated_at
            "#,
            command.user_id.0,
            command.channel_id.0,
            command.organization_id.0,
            command.message_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn get(
        &self,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> Result<Option<ChannelReadState>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ChannelReadStateRow,
            r#"
            SELECT user_id, channel_id, org_id, last_read_message_id, updated_at
            FROM chat.channel_read_state
            WHERE user_id = $1 AND channel_id = $2
            "#,
            user_id.0,
            channel_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn unread_channels(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<Vec<ChannelId>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query!(
            r#"
            SELECT c.id AS "id!"
            FROM chat.channels c
            WHERE c.org_id = $1
              AND EXISTS (
                  SELECT 1 FROM chat.messages m
                  WHERE m.channel_id = c.id
                    AND m.id > COALESCE(
                        (SELECT rs.last_read_message_id
                         FROM chat.channel_read_state rs
                         WHERE rs.user_id = $2 AND rs.channel_id = c.id),
                        '00000000-0000-0000-0000-000000000000'::uuid)
              )
            "#,
            organization_id.0,
            user_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(|r| ChannelId(r.id)).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::generate_uuid_v7;
    use discord::MessageId;
    use sqlx::PgPool;

    async fn make_pool() -> PgPool {
        PgPool::connect("postgres://ferriskey:ferriskey@localhost:5433/mestier")
            .await
            .unwrap()
    }

    /// Seeds a throwaway user + organization + channel, returns (OrganizationId, UserId, ChannelId).
    /// Mirrors the helper in message/postgres/repository.rs exactly.
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

    /// Inserts two messages into the channel and returns their IDs in insertion order.
    /// Uses UUIDv7 timestamps 1 second apart to guarantee msg2 > msg1 under
    /// the second-precision `generate_uuid_v7` implementation.
    async fn seed_two_messages(
        tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
        org_id: OrganizationId,
        user_id: UserId,
        channel_id: ChannelId,
    ) -> (MessageId, MessageId) {
        use uuid::{NoContext, Timestamp, Uuid};
        // Anchor at a fixed epoch + 1 s / 2 s so msg1 < msg2 regardless of wall clock.
        let msg1 = Uuid::new_v7(Timestamp::from_unix(NoContext, 1_000_000_000, 0));
        let msg2 = Uuid::new_v7(Timestamp::from_unix(NoContext, 1_000_000_001, 0));

        sqlx::query!(
            r#"INSERT INTO chat.messages (id, org_id, channel_id, author_type, author_user_id, content, created_at)
               VALUES ($1, $2, $3, 'USER'::chat.author_type, $4, 'first', now())"#,
            msg1,
            org_id.0,
            channel_id.0,
            user_id.0,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        sqlx::query!(
            r#"INSERT INTO chat.messages (id, org_id, channel_id, author_type, author_user_id, content, created_at)
               VALUES ($1, $2, $3, 'USER'::chat.author_type, $4, 'second', now())"#,
            msg2,
            org_id.0,
            channel_id.0,
            user_id.0,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        (MessageId(msg1), MessageId(msg2))
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn read_state_marker_advance_and_unread_query() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let (org_id, user_id, channel_id) = {
                let mut guard = tx.lock().await;
                seed_org_user_channel(&mut *guard).await
            };
            let (msg1, msg2) = {
                let mut guard = tx.lock().await;
                seed_two_messages(&mut *guard, org_id, user_id, channel_id).await
            };

            let repo = PgReadStateRepository::new(&tx);

            // ── Before any mark: channel is unread (has messages, no marker) ──────
            let unread = repo.unread_channels(user_id, org_id).await?;
            assert!(
                unread.contains(&channel_id),
                "channel must be unread before any mark"
            );

            // ── Mark read at msg1 (first insert — always stores, returns Some) ────
            let cmd1 = MarkChannelReadCommand {
                organization_id: org_id,
                channel_id,
                user_id,
                message_id: msg1,
            };
            let state1 = repo.upsert(cmd1).await?;
            assert!(
                state1.is_some(),
                "first ack must return Some (marker moved)"
            );
            assert_eq!(
                state1.unwrap().last_read_message_id,
                Some(msg1),
                "marker must point to msg1"
            );

            // ── Channel is still unread (msg2 > msg1 exists) ─────────────────────
            let unread = repo.unread_channels(user_id, org_id).await?;
            assert!(
                unread.contains(&channel_id),
                "channel still unread after marking msg1 (msg2 is newer)"
            );

            // ── Advance to msg2 ───────────────────────────────────────────────────
            let cmd2 = MarkChannelReadCommand {
                organization_id: org_id,
                channel_id,
                user_id,
                message_id: msg2,
            };
            let state2 = repo.upsert(cmd2).await?;
            assert!(state2.is_some(), "advance to msg2 must return Some");
            assert_eq!(state2.unwrap().last_read_message_id, Some(msg2));

            // ── Channel now read (no message newer than msg2) ─────────────────────
            let unread = repo.unread_channels(user_id, org_id).await?;
            assert!(
                !unread.contains(&channel_id),
                "channel must be read after marking the latest message"
            );

            // ── Re-ack at msg1 (older) → no-op, returns None ─────────────────────
            let cmd_old = MarkChannelReadCommand {
                organization_id: org_id,
                channel_id,
                user_id,
                message_id: msg1,
            };
            let state_old = repo.upsert(cmd_old).await?;
            assert!(
                state_old.is_none(),
                "re-ack at older message must return None (no-op)"
            );

            // ── Marker unchanged (still msg2) ─────────────────────────────────────
            let fetched = repo.get(user_id, channel_id).await?;
            assert!(fetched.is_some());
            assert_eq!(
                fetched.unwrap().last_read_message_id,
                Some(msg2),
                "marker must remain msg2 after no-op re-ack"
            );

            Err::<(), _>(CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }
}
