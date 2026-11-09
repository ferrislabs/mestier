use common::{OrganizationId, UserId, generate_uuid_v7};
use discord::{
    ChannelId, ChannelPlacement, ChannelType, CreateCategoryCommand, CreateChannelCommand,
};
use sqlx::PgPool;

use crate::application::test_support::{dev_pool, purge};
use crate::application::{MestierUseCase, default_authorizer};
use crate::infrastructure::realtime::EventHub;

async fn seed_org(pool: &PgPool, label: &str) -> (OrganizationId, UserId) {
    let owner = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(owner)
    .bind(format!("{label}-{owner}@example.com"))
    .bind(format!("{label}-{owner}"))
    .bind(label)
    .bind(format!("sub-{label}-{owner}"))
    .execute(pool)
    .await
    .unwrap();
    let org = generate_uuid_v7();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(org)
        .bind("Test Org")
        .bind(format!("test-org-{org}"))
        .bind(owner)
        .execute(pool)
        .await
        .unwrap();
    (OrganizationId(org), UserId(owner))
}

async fn cleanup(pool: &PgPool, orgs: &[(OrganizationId, UserId)]) {
    for (org, owner) in orgs {
        purge(pool, "DELETE FROM organizations WHERE id = $1", org.0).await;
        purge(pool, "DELETE FROM users WHERE id = $1", owner.0).await;
    }
}

fn usecase(pool: &PgPool) -> MestierUseCase {
    MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new())
}

async fn channel(usecase: &MestierUseCase, org: OrganizationId, name: &str) -> ChannelId {
    usecase
        .create_channel(CreateChannelCommand {
            organization_id: org,
            channel_type: ChannelType::Text,
            category_id: None,
            name: name.to_owned(),
            topic: None,
            position: 0,
        })
        .await
        .unwrap()
        .id
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn reordering_moves_channels_into_a_category_and_ranks_them() {
    let pool = dev_pool().await;
    let (org, owner) = seed_org(&pool, "reorder").await;
    let usecase = usecase(&pool);
    let a = channel(&usecase, org, "a").await;
    let b = channel(&usecase, org, "b").await;
    let category = usecase
        .create_category(CreateCategoryCommand {
            organization_id: org,
            name: "Chantiers".to_owned(),
            position: 0,
        })
        .await
        .unwrap()
        .id;

    let moved = usecase
        .reorder_channels(
            org,
            vec![
                ChannelPlacement {
                    channel_id: b,
                    category_id: Some(category),
                    position: 0,
                },
                ChannelPlacement {
                    channel_id: a,
                    category_id: Some(category),
                    position: 1,
                },
            ],
        )
        .await
        .unwrap();

    assert_eq!(moved.len(), 2);
    let listed = usecase.list_channels(org).await.unwrap();
    let rank = |id: ChannelId| {
        let found = listed.iter().find(|channel| channel.id == id).unwrap();
        (found.category_id, found.position)
    };
    assert_eq!(rank(b), (Some(category), 0));
    assert_eq!(rank(a), (Some(category), 1));

    cleanup(&pool, &[(org, owner)]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn reordering_refuses_a_category_of_another_organization() {
    let pool = dev_pool().await;
    let (org, owner) = seed_org(&pool, "reorder-mine").await;
    let (other, other_owner) = seed_org(&pool, "reorder-theirs").await;
    let usecase = usecase(&pool);
    let mine = channel(&usecase, org, "mine").await;
    let foreign = usecase
        .create_category(CreateCategoryCommand {
            organization_id: other,
            name: "Theirs".to_owned(),
            position: 0,
        })
        .await
        .unwrap()
        .id;

    let result = usecase
        .reorder_channels(
            org,
            vec![ChannelPlacement {
                channel_id: mine,
                category_id: Some(foreign),
                position: 0,
            }],
        )
        .await;

    assert!(result.is_err(), "a foreign category must be refused");

    cleanup(&pool, &[(org, owner), (other, other_owner)]).await;
}
