use common::CoreError;
use discord::domain::notification::ports::NotificationRepository;
use discord::{CreateNotification, Notification, NotificationId, OrganizationId, UserId};
use mestier_macros::repository;

use super::model::NotificationRow;
use crate::infrastructure::postgres::{SharedTx, error::map_sqlx_error};

#[repository(domain = Notification, backend = Postgres)]
pub struct PgNotificationRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgNotificationRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> NotificationRepository for PgNotificationRepository<'tx> {
    async fn create(&self, command: CreateNotification) -> Result<Notification, CoreError> {
        use common::generate_uuid_v7;

        let id = generate_uuid_v7();
        let kind = command.kind.to_string();
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            NotificationRow,
            r#"
            INSERT INTO chat.notification (id, org_id, user_id, channel_id, message_id, kind, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, now())
            RETURNING id, org_id, user_id, channel_id, message_id, kind, read_at, created_at
            "#,
            id,
            command.organization_id.0,
            command.user_id.0,
            command.channel_id.0,
            command.message_id.0,
            kind,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Notification::try_from(row)
    }

    async fn list(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
        unread_only: bool,
        before: Option<NotificationId>,
        limit: i64,
    ) -> Result<Vec<Notification>, CoreError> {
        let before_uuid = before.map(|n| n.0);
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            NotificationRow,
            r#"
            SELECT id, org_id, user_id, channel_id, message_id, kind, read_at, created_at
            FROM chat.notification
            WHERE user_id = $1
              AND org_id = $2
              AND ($3::bool = false OR read_at IS NULL)
              AND ($4::uuid IS NULL OR id < $4)
            ORDER BY id DESC
            LIMIT $5
            "#,
            user_id.0,
            organization_id.0,
            unread_only,
            before_uuid,
            limit,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter()
            .map(Notification::try_from)
            .collect::<Result<Vec<_>, _>>()
    }

    async fn mark_read(
        &self,
        notification_id: NotificationId,
        user_id: UserId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            UPDATE chat.notification
            SET read_at = now()
            WHERE id = $1 AND user_id = $2 AND read_at IS NULL
            "#,
            notification_id.0,
            user_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn mark_all_read(
        &self,
        user_id: UserId,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            UPDATE chat.notification
            SET read_at = now()
            WHERE user_id = $1 AND org_id = $2 AND read_at IS NULL
            "#,
            user_id.0,
            organization_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::generate_uuid_v7;
    use discord::{ChannelId, MessageId, NotificationKind};
    use sqlx::PgPool;

    async fn make_pool() -> PgPool {
        PgPool::connect("postgres://ferriskey:ferriskey@localhost:5433/mestier")
            .await
            .unwrap()
    }

    /// Seeds a throwaway user + organization + channel + message.
    /// Returns (OrganizationId, UserId, ChannelId, MessageId) for the author.
    async fn seed_org_user_channel_message(
        tx: &mut sqlx::Transaction<'static, sqlx::Postgres>,
    ) -> (OrganizationId, UserId, ChannelId, MessageId) {
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

        let message_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO chat.messages (id, org_id, channel_id, author_type, author_user_id, content, created_at)
               VALUES ($1, $2, $3, 'USER'::chat.author_type, $4, 'hello @recipient', now())"#,
            message_id,
            org_id,
            channel_id,
            user_id,
        )
        .execute(&mut **tx)
        .await
        .unwrap();

        (
            OrganizationId(org_id),
            UserId(user_id),
            ChannelId(channel_id),
            MessageId(message_id),
        )
    }

    /// Seeds a second user in the same org; returns UserId of the recipient.
    async fn seed_recipient(tx: &mut sqlx::Transaction<'static, sqlx::Postgres>) -> UserId {
        let user_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            user_id,
            format!("recipient-{}@example.com", user_id),
            format!("recipient-{}", user_id),
            "Recipient",
            format!("sub-r-{}", user_id),
        )
        .execute(&mut **tx)
        .await
        .unwrap();
        UserId(user_id)
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn notification_create_list_mark_read_mark_all_read() {
        let pool = make_pool().await;

        crate::infrastructure::postgres::with_tx(&pool, async |tx| {
            let (org_id, _author_id, channel_id, message_id) = {
                let mut guard = tx.lock().await;
                seed_org_user_channel_message(&mut guard).await
            };
            let recipient_id = {
                let mut guard = tx.lock().await;
                seed_recipient(&mut guard).await
            };

            let repo = PgNotificationRepository::new(&tx);

            // ── create: one MENTION notification for the recipient ────────────────
            let cmd = CreateNotification {
                organization_id: org_id,
                user_id: recipient_id,
                channel_id,
                message_id,
                kind: NotificationKind::Mention,
            };
            let notif = repo.create(cmd).await?;
            assert_eq!(notif.user_id, recipient_id);
            assert_eq!(notif.organization_id, org_id);
            assert_eq!(notif.channel_id, channel_id);
            assert_eq!(notif.message_id, message_id);
            assert!(matches!(notif.kind, NotificationKind::Mention));
            assert!(
                notif.read_at.is_none(),
                "newly created notification must be unread"
            );

            let notif_id = notif.id;

            // ── list unread_only=true: must contain the new notification ──────────
            let unread = repo.list(recipient_id, org_id, true, None, 50).await?;
            assert_eq!(unread.len(), 1, "must have exactly 1 unread notification");
            assert_eq!(unread[0].id, notif_id);

            // ── list unread_only=false: also contains it ──────────────────────────
            let all = repo.list(recipient_id, org_id, false, None, 50).await?;
            assert_eq!(all.len(), 1);

            // ── cursor: before=notif_id → empty (nothing older) ──────────────────
            let before_cursor = repo
                .list(recipient_id, org_id, false, Some(notif_id), 50)
                .await?;
            assert!(
                before_cursor.is_empty(),
                "no notifications older than notif_id"
            );

            // ── mark_read only affects the owner: wrong user is a no-op ──────────
            let other_user = {
                let mut guard = tx.lock().await;
                seed_recipient(&mut guard).await
            };
            repo.mark_read(notif_id, other_user).await?;
            // notification must still be unread
            let still_unread = repo.list(recipient_id, org_id, true, None, 50).await?;
            assert_eq!(
                still_unread.len(),
                1,
                "mark_read by wrong user must be a no-op"
            );

            // ── mark_read by the owner marks it read ──────────────────────────────
            repo.mark_read(notif_id, recipient_id).await?;
            let now_read = repo.list(recipient_id, org_id, true, None, 50).await?;
            assert!(
                now_read.is_empty(),
                "notification must be read after mark_read"
            );

            // ── create a second notification then mark_all_read ───────────────────
            let cmd2 = CreateNotification {
                organization_id: org_id,
                user_id: recipient_id,
                channel_id,
                message_id,
                kind: NotificationKind::Mention,
            };
            repo.create(cmd2).await?;
            repo.mark_all_read(recipient_id, org_id).await?;
            let after_all = repo.list(recipient_id, org_id, true, None, 50).await?;
            assert!(
                after_all.is_empty(),
                "no unread notifications after mark_all_read"
            );

            Err::<(), _>(CoreError::Internal("rollback".into()))
        })
        .await
        .unwrap_err();
    }
}
