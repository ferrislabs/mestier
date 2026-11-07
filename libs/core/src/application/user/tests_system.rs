use common::UserId;

use crate::application::test_support::dev_pool;
use crate::application::{MestierUseCase, default_authorizer};
use crate::domain::user::commands::{CreateUserCommand, UpsertUserBySubCommand};
use crate::domain::user::system::{SYSTEM_USER_EMAIL, SYSTEM_USER_SUB};
use crate::infrastructure::realtime::EventHub;

async fn usecase() -> (MestierUseCase, sqlx::PgPool) {
    let pool = dev_pool().await;
    (
        MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new()),
        pool,
    )
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_system_user_is_created_once_however_often_and_however_concurrently() {
    let (usecase, pool) = usecase().await;

    let first = usecase.system_user_id().await.unwrap();
    let second = usecase.system_user_id().await.unwrap();
    let mut tasks = tokio::task::JoinSet::new();
    for _ in 0..8 {
        let usecase = usecase.clone();
        tasks.spawn(async move { usecase.system_user_id().await });
    }
    let mut concurrent = Vec::new();
    while let Some(joined) = tasks.join_next().await {
        concurrent.push(joined.unwrap());
    }

    assert_eq!(first, second);
    for id in concurrent {
        assert_eq!(id.unwrap(), first);
    }
    let rows: i64 = sqlx::query_scalar("SELECT count(*) FROM users WHERE sub = $1")
        .bind(SYSTEM_USER_SUB)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(rows, 1);
    let memberships: i64 =
        sqlx::query_scalar("SELECT count(*) FROM organization_members WHERE user_id = $1")
            .bind(first.0)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(memberships, 0);
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_system_user_is_hidden_from_listings_and_cannot_be_resolved_or_claimed() {
    let (usecase, pool) = usecase().await;
    let id = usecase.system_user_id().await.unwrap();

    let listed = usecase.list_users().await.unwrap();
    let by_sub = usecase.find_user_by_sub(SYSTEM_USER_SUB).await.unwrap();
    let by_email = usecase.find_user_by_email(SYSTEM_USER_EMAIL).await.unwrap();
    let by_sub_claim = usecase
        .create_user(CreateUserCommand {
            name: "Intrus".into(),
            username: "intrus-sub".into(),
            email: "intrus-sub@example.com".into(),
            sub: SYSTEM_USER_SUB.into(),
        })
        .await;
    let by_email_claim = usecase
        .create_user(CreateUserCommand {
            name: "Intrus".into(),
            username: "intrus-email".into(),
            email: SYSTEM_USER_EMAIL.into(),
            sub: "sub-intrus".into(),
        })
        .await;
    let reconcile_claim = usecase
        .reconcile_user_upsert(UpsertUserBySubCommand {
            sub: SYSTEM_USER_SUB.into(),
            email: "intrus@example.com".into(),
            username: "intrus".into(),
            name: None,
        })
        .await;
    let deletion = usecase
        .reconcile_user_deletion(SYSTEM_USER_SUB.to_owned())
        .await;

    assert!(listed.iter().all(|user| user.id != id));
    assert!(by_sub.is_none());
    assert!(by_email.is_none());
    assert!(by_sub_claim.is_err());
    assert!(by_email_claim.is_err());
    assert!(reconcile_claim.is_err());
    assert!(deletion.is_err());
    let (email, deleted): (String, bool) =
        sqlx::query_as("SELECT email, deleted_at IS NOT NULL FROM users WHERE id = $1")
            .bind(id.0)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(email, SYSTEM_USER_EMAIL);
    assert!(!deleted);
    let _: UserId = id;
}
