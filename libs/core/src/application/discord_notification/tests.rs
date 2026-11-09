use common::{OrganizationId, UserId, generate_uuid_v7};
use discord::{
    ChannelId, ChannelType, CreateChannelCommand, CreateMessageCommand, NotificationLevel,
    domain::message::commands::MessageAuthor,
};
use sqlx::PgPool;

use super::{ActivityEntry, ActivityKind};
use crate::application::test_support::{dev_pool, purge};
use crate::application::{MestierUseCase, default_authorizer};
use crate::infrastructure::realtime::EventHub;

async fn seed_user(pool: &PgPool, label: &str) -> UserId {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(id)
    .bind(format!("{label}-{id}@example.com"))
    .bind(format!("{label}-{id}"))
    .bind(label)
    .bind(format!("sub-{label}-{id}"))
    .execute(pool)
    .await
    .unwrap();
    UserId(id)
}

/// An organization with three seated members: `(org, alice, bob, carol)`.
async fn seed_org(pool: &PgPool) -> (OrganizationId, UserId, UserId, UserId) {
    let alice = seed_user(pool, "alice").await;
    let bob = seed_user(pool, "bob").await;
    let carol = seed_user(pool, "carol").await;
    let org_id = generate_uuid_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(org_id)
        .bind("Test Org")
        .bind(format!("test-org-{org_id}"))
        .bind(alice.0)
        .execute(pool)
        .await
        .unwrap();
    for user in [alice, bob, carol] {
        sqlx::query(
            "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
        )
        .bind(generate_uuid_v7())
        .bind(org_id)
        .bind(user.0)
        .bind("Dupont")
        .execute(pool)
        .await
        .unwrap();
    }
    (OrganizationId(org_id), alice, bob, carol)
}

async fn cleanup(pool: &PgPool, org: OrganizationId, users: &[UserId]) {
    purge(pool, "DELETE FROM organizations WHERE id = $1", org.0).await;
    for user in users {
        purge(pool, "DELETE FROM users WHERE id = $1", user.0).await;
    }
}

fn usecase(pool: &PgPool) -> MestierUseCase {
    MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new())
}

async fn channel(usecase: &MestierUseCase, org: OrganizationId) -> ChannelId {
    usecase
        .create_channel(CreateChannelCommand {
            organization_id: org,
            channel_type: ChannelType::Text,
            category_id: None,
            name: "general".to_owned(),
            topic: None,
            position: 0,
        })
        .await
        .unwrap()
        .id
}

async fn say(
    usecase: &MestierUseCase,
    org: OrganizationId,
    channel: ChannelId,
    author: UserId,
    content: String,
) {
    usecase
        .create_message(CreateMessageCommand {
            organization_id: org,
            channel_id: channel,
            author: MessageAuthor::User(author),
            content,
            components: None,
            attachments: vec![],
        })
        .await
        .unwrap();
}

async fn activity(
    usecase: &MestierUseCase,
    org: OrganizationId,
    user: UserId,
) -> Vec<ActivityEntry> {
    let visible = usecase.list_visible_channels(user, org).await.unwrap();
    usecase.list_activity(user, org, visible, 50).await.unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_mention_shows_up_in_the_activity_of_the_mentioned_person() {
    let pool = dev_pool().await;
    let (org, alice, bob, carol) = seed_org(&pool).await;
    let usecase = usecase(&pool);
    let general = channel(&usecase, org).await;

    say(&usecase, org, general, bob, format!("salut <@{}>", alice.0)).await;

    let entries = activity(&usecase, org, alice).await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, ActivityKind::Mention);
    assert_eq!(entries[0].message.author_user_id, Some(bob));
    assert!(!entries[0].read);
    assert!(activity(&usecase, org, carol).await.is_empty());

    cleanup(&pool, org, &[alice, bob, carol]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_muted_channel_records_no_mention_and_hides_the_old_ones() {
    let pool = dev_pool().await;
    let (org, alice, bob, carol) = seed_org(&pool).await;
    let usecase = usecase(&pool);
    let general = channel(&usecase, org).await;
    say(&usecase, org, general, bob, format!("avant <@{}>", alice.0)).await;

    usecase
        .set_notification_level(alice, general, NotificationLevel::Nothing)
        .await
        .unwrap();
    say(&usecase, org, general, bob, format!("après <@{}>", alice.0)).await;

    assert!(activity(&usecase, org, alice).await.is_empty());
    let recorded: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM chat.notification WHERE user_id = $1 AND channel_id = $2",
    )
    .bind(alice.0)
    .bind(general.0)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(recorded, 1, "only the mention made before muting exists");

    cleanup(&pool, org, &[alice, bob, carol]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn unread_messages_show_up_only_for_a_channel_followed_at_the_all_level() {
    let pool = dev_pool().await;
    let (org, alice, bob, carol) = seed_org(&pool).await;
    let usecase = usecase(&pool);
    let general = channel(&usecase, org).await;
    say(&usecase, org, general, bob, "un".to_owned()).await;
    say(&usecase, org, general, bob, "deux".to_owned()).await;

    assert!(
        activity(&usecase, org, alice).await.is_empty(),
        "a channel defaults to mentions only"
    );

    usecase
        .set_notification_level(alice, general, NotificationLevel::All)
        .await
        .unwrap();
    let entries = activity(&usecase, org, alice).await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, ActivityKind::Unread);
    assert_eq!(entries[0].unread_count, Some(2));

    cleanup(&pool, org, &[alice, bob, carol]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_direct_message_is_unread_activity_by_default_and_only_for_its_two_people() {
    let pool = dev_pool().await;
    let (org, alice, bob, carol) = seed_org(&pool).await;
    let usecase = usecase(&pool);
    let direct = usecase.open_direct_channel(org, alice, bob).await.unwrap();

    say(
        &usecase,
        org,
        direct.channel.id,
        bob,
        format!("psst <@{}>", carol.0),
    )
    .await;

    let entries = activity(&usecase, org, alice).await;
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].kind, ActivityKind::Unread);
    assert_eq!(entries[0].channel_id, direct.channel.id);
    assert!(
        activity(&usecase, org, carol).await.is_empty(),
        "a mention in somebody else's direct message must not leak"
    );

    cleanup(&pool, org, &[alice, bob, carol]).await;
}
