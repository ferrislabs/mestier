use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use discord::ChannelId;
use sqlx::PgPool;

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

async fn seat(pool: &PgPool, org: OrganizationId, user: UserId) {
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ($1, $2, $3, $4)",
    )
    .bind(generate_uuid_v7())
    .bind(org.0)
    .bind(user.0)
    .bind("Dupont")
    .execute(pool)
    .await
    .unwrap();
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
    let org = OrganizationId(org_id);
    for user in [alice, bob, carol] {
        seat(pool, org, user).await;
    }
    (org, alice, bob, carol)
}

async fn cleanup(pool: &PgPool, org: OrganizationId, users: &[UserId]) {
    purge(pool, "DELETE FROM organizations WHERE id = $1", org.0).await;
    for user in users {
        purge(pool, "DELETE FROM users WHERE id = $1", user.0).await;
    }
}

fn usecase(pool: PgPool) -> MestierUseCase {
    MestierUseCase::new(pool, default_authorizer(), EventHub::new())
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn opening_twice_returns_the_same_conversation_from_either_side() {
    let pool = dev_pool().await;
    let (org, alice, bob, carol) = seed_org(&pool).await;
    let usecase = usecase(pool.clone());

    let first = usecase.open_direct_channel(org, alice, bob).await.unwrap();
    let again = usecase.open_direct_channel(org, alice, bob).await.unwrap();
    let reversed = usecase.open_direct_channel(org, bob, alice).await.unwrap();

    assert_eq!(first.channel.id, again.channel.id);
    assert_eq!(first.channel.id, reversed.channel.id);
    assert_eq!(
        usecase
            .list_direct_channels(org, alice)
            .await
            .unwrap()
            .len(),
        1
    );
    assert!(
        usecase
            .list_direct_channels(org, carol)
            .await
            .unwrap()
            .is_empty()
    );

    cleanup(&pool, org, &[alice, bob, carol]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn opening_refuses_oneself_and_outsiders() {
    let pool = dev_pool().await;
    let (org, alice, bob, carol) = seed_org(&pool).await;
    let outsider = seed_user(&pool, "outsider").await;
    let usecase = usecase(pool.clone());

    assert!(matches!(
        usecase.open_direct_channel(org, alice, alice).await,
        Err(CoreError::Conflict(_))
    ));
    assert!(matches!(
        usecase.open_direct_channel(org, alice, outsider).await,
        Err(CoreError::NotFound)
    ));

    cleanup(&pool, org, &[alice, bob, carol, outsider]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn only_the_two_participants_can_resolve_the_channel() {
    let pool = dev_pool().await;
    let (org, alice, bob, carol) = seed_org(&pool).await;
    let usecase = usecase(pool.clone());
    let direct = usecase.open_direct_channel(org, alice, bob).await.unwrap();
    let channel: ChannelId = direct.channel.id;

    for participant in [alice, bob] {
        let permissions = usecase
            .resolve_channel_permissions(participant, channel)
            .await
            .unwrap();
        assert!(permissions.contains(crate::Permissions::VIEW_CHANNEL));
        assert!(permissions.contains(crate::Permissions::SEND_MESSAGES));
        assert!(!permissions.contains(crate::Permissions::MANAGE_CHANNELS));
    }
    assert!(matches!(
        usecase.resolve_channel_permissions(carol, channel).await,
        Err(CoreError::NotFound)
    ));

    cleanup(&pool, org, &[alice, bob, carol]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_conversation_is_neither_listed_nor_unread_for_a_third_member() {
    let pool = dev_pool().await;
    let (org, alice, bob, carol) = seed_org(&pool).await;
    let usecase = usecase(pool.clone());
    let direct = usecase.open_direct_channel(org, alice, bob).await.unwrap();
    sqlx::query(
        "INSERT INTO chat.messages (id, org_id, channel_id, author_type, author_user_id, content)
         VALUES ($1, $2, $3, 'USER'::chat.author_type, $4, 'hello')",
    )
    .bind(generate_uuid_v7())
    .bind(org.0)
    .bind(direct.channel.id.0)
    .bind(alice.0)
    .execute(&pool)
    .await
    .unwrap();

    let for_bob = usecase.list_unread_summaries(bob, org).await.unwrap();
    let for_carol = usecase.list_unread_summaries(carol, org).await.unwrap();
    let visible_to_carol = usecase.list_visible_channels(carol, org).await.unwrap();

    assert_eq!(for_bob.len(), 1);
    assert_eq!(for_bob[0].channel_id, direct.channel.id);
    assert!(for_carol.is_empty());
    assert!(visible_to_carol.iter().all(|c| c.id != direct.channel.id));

    cleanup(&pool, org, &[alice, bob, carol]).await;
}
