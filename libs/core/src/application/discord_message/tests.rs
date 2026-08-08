#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {

    use common::{OrganizationId, UserId, generate_uuid_v7};
    use discord::domain::message::commands::CreateMessageCommand;
    use discord::{ChannelId, MessageAuthor};
    use sqlx::PgPool;

    use crate::application::{MestierUseCase, default_authorizer};
    use crate::infrastructure::realtime::EventHub;

    async fn make_pool() -> PgPool {
        PgPool::connect("postgres://ferriskey:ferriskey@localhost:5433/mestier")
            .await
            .unwrap()
    }

    /// Seeds an author user, an organization, and a channel directly on `pool`
    /// (each INSERT is auto-committed). Returns `(OrganizationId, UserId, ChannelId)`.
    async fn seed_author_org_channel(pool: &PgPool) -> (OrganizationId, UserId, ChannelId) {
        let user_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            user_id,
            format!("author-{}@example.com", user_id),
            format!("author-{}", user_id),
            "Author User",
            format!("sub-author-{}", user_id),
        )
        .execute(pool)
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
        .execute(pool)
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
        .execute(pool)
        .await
        .unwrap();

        (
            OrganizationId(org_id),
            UserId(user_id),
            ChannelId(channel_id),
        )
    }

    /// Seeds a second user (the mention recipient) directly on `pool`.
    async fn seed_recipient(pool: &PgPool) -> UserId {
        let user_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            user_id,
            format!("recipient-{}@example.com", user_id),
            format!("recipient-{}", user_id),
            "Recipient User",
            format!("sub-recipient-{}", user_id),
        )
        .execute(pool)
        .await
        .unwrap();
        UserId(user_id)
    }

    /// Removes all rows seeded by a single test run, keyed by org_id (cascade
    /// deletes channels + messages + notifications) and the loose user rows.
    async fn cleanup(pool: &PgPool, org_id: OrganizationId, user_ids: &[UserId]) {
        sqlx::query!("DELETE FROM organizations WHERE id = $1", org_id.0)
            .execute(pool)
            .await
            .ok();
        for uid in user_ids {
            sqlx::query!("DELETE FROM users WHERE id = $1", uid.0)
                .execute(pool)
                .await
                .ok();
        }
    }

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    // ── Case 1: mention persists a MENTION notification ──────────────────────

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_message_mention_persists_notification() {
        let pool = make_pool().await;
        let (org_id, author_id, channel_id) = seed_author_org_channel(&pool).await;
        let recipient_id = seed_recipient(&pool).await;

        let usecase = make_usecase(pool.clone());

        let content = format!("hello <@{}>", recipient_id.0);
        let cmd = CreateMessageCommand {
            organization_id: org_id,
            channel_id,
            author: MessageAuthor::User(author_id),
            content,
            components: None,
            attachments: vec![],
        };

        let message = usecase
            .create_message(cmd)
            .await
            .expect("create_message must succeed");

        // Assert exactly one notification row for the recipient with kind=MENTION and read_at=NULL.
        struct NotifRow {
            kind: String,
            read_at: Option<chrono::DateTime<chrono::Utc>>,
        }
        let rows = sqlx::query_as!(
            NotifRow,
            r#"SELECT kind, read_at FROM chat.notification
               WHERE user_id = $1 AND message_id = $2"#,
            recipient_id.0,
            message.id.0,
        )
        .fetch_all(&pool)
        .await
        .expect("notification query must succeed");

        assert_eq!(
            rows.len(),
            1,
            "expected exactly one notification for the recipient"
        );
        assert_eq!(rows[0].kind, "MENTION");
        assert!(
            rows[0].read_at.is_none(),
            "newly created notification must be unread"
        );

        cleanup(&pool, org_id, &[author_id, recipient_id]).await;
    }

    // ── Case 2: self-mention creates no notification ──────────────────────────

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn create_message_self_mention_creates_no_notification() {
        let pool = make_pool().await;
        let (org_id, author_id, channel_id) = seed_author_org_channel(&pool).await;

        let usecase = make_usecase(pool.clone());

        // Content mentions only the author themselves.
        let content = format!("hello <@{}>", author_id.0);
        let cmd = CreateMessageCommand {
            organization_id: org_id,
            channel_id,
            author: MessageAuthor::User(author_id),
            content,
            components: None,
            attachments: vec![],
        };

        let message = usecase
            .create_message(cmd)
            .await
            .expect("create_message must succeed");

        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) FROM chat.notification WHERE user_id = $1 AND message_id = $2"#,
            author_id.0,
            message.id.0,
        )
        .fetch_one(&pool)
        .await
        .expect("notification count query must succeed");

        assert_eq!(
            count.unwrap_or(0),
            0,
            "self-mention must create no notification"
        );

        cleanup(&pool, org_id, &[author_id]).await;
    }
}
