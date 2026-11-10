use chrono::{DateTime, Duration, Utc};
use common::{CoreError, UserId, generate_uuid_v7};
use sqlx::PgPool;

use crate::{
    News, NewsContent, NewsCta, NewsIcon, NewsId, NewsItem,
    application::test_support::dev_pool,
    domain::news::ports::NewsRepository,
    infrastructure::{
        news::postgres::PgNewsRepository,
        postgres::{SharedTx, with_tx},
    },
};

async fn seed_user(tx: &SharedTx<'_>) -> UserId {
    let mut guard = tx.lock().await;
    let user_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO users (id, email, username, display_name, sub)
           VALUES ($1, $2, $3, $4, $5)"#,
        user_id,
        format!("reader-{user_id}@example.com"),
        format!("reader-{user_id}"),
        "Reader",
        format!("sub-{user_id}"),
    )
    .execute(&mut ***guard)
    .await
    .unwrap();
    UserId(user_id)
}

fn news(title: &str, published_at: Option<DateTime<Utc>>) -> News {
    let now = Utc::now();
    News {
        id: NewsId(generate_uuid_v7()),
        content: NewsContent {
            title: title.to_owned(),
            image_key: None,
            items: vec![NewsItem::new(NewsIcon::Rocket, "Point", "Texte").unwrap()],
            cta: None,
        },
        published_at,
        created_at: now,
        updated_at: now,
    }
}

async fn ids(list: Vec<News>, among: &[NewsId]) -> Vec<NewsId> {
    list.into_iter()
        .map(|news| news.id)
        .filter(|id| among.contains(id))
        .collect()
}

async fn run<F>(work: F)
where
    F: AsyncFnOnce(SharedTx<'_>) -> Result<(), CoreError>,
{
    let pool: PgPool = dev_pool().await;
    let outcome = with_tx(&pool, async |tx| {
        work(tx).await?;
        Err::<(), _>(CoreError::Internal("rollback".into()))
    })
    .await;
    assert!(matches!(outcome, Err(CoreError::Internal(message)) if message == "rollback"));
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_news_round_trips_with_its_items_and_button() {
    run(async |tx| {
        let mut repository = PgNewsRepository::new(&tx);
        let mut written = news("Nouveautés", None);
        written.content.image_key = Some("news/a.png".to_owned());
        written.content.cta = Some(NewsCta::new("Voir", "https://mestier.app/news").unwrap());

        repository.insert(&written).await?;
        let read = repository.find_by_id(written.id).await?.unwrap();

        assert_eq!(read.content, written.content);
        assert_eq!(read.published_at, None);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn published_excludes_drafts_and_future_dates_and_sorts_newest_first() {
    run(async |tx| {
        let mut repository = PgNewsRepository::new(&tx);
        let now = Utc::now();
        let older = news("older", Some(now - Duration::days(2)));
        let newer = news("newer", Some(now - Duration::days(1)));
        let draft = news("draft", None);
        let scheduled = news("scheduled", Some(now + Duration::days(1)));
        for item in [&older, &draft, &newer, &scheduled] {
            repository.insert(item).await?;
        }
        let mine = [older.id, newer.id, draft.id, scheduled.id];

        let published = ids(repository.list_published(now).await?, &mine).await;

        assert_eq!(published, vec![newer.id, older.id]);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn unseen_excludes_what_that_user_saw_and_only_that_user() {
    run(async |tx| {
        let reader = seed_user(&tx).await;
        let other = seed_user(&tx).await;
        let mut repository = PgNewsRepository::new(&tx);
        let now = Utc::now();
        let first = news("first", Some(now - Duration::days(2)));
        let second = news("second", Some(now - Duration::days(1)));
        let draft = news("draft", None);
        for item in [&first, &second, &draft] {
            repository.insert(item).await?;
        }
        let mine = [first.id, second.id, draft.id];

        repository.mark_seen(reader, &[first.id], now).await?;

        let for_reader = ids(repository.list_unseen(reader, now).await?, &mine).await;
        let for_other = ids(repository.list_unseen(other, now).await?, &mine).await;
        assert_eq!(for_reader, vec![second.id]);
        assert_eq!(for_other, vec![second.id, first.id]);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn marking_seen_is_idempotent_and_ignores_unknown_and_unpublished_ids() {
    run(async |tx| {
        let reader = seed_user(&tx).await;
        let mut repository = PgNewsRepository::new(&tx);
        let now = Utc::now();
        let published = news("published", Some(now - Duration::hours(1)));
        let draft = news("draft", None);
        let scheduled = news("scheduled", Some(now + Duration::days(1)));
        for item in [&published, &draft, &scheduled] {
            repository.insert(item).await?;
        }
        let unknown = NewsId(generate_uuid_v7());

        let all = [published.id, draft.id, scheduled.id, unknown];
        repository.mark_seen(reader, &all, now).await?;
        repository.mark_seen(reader, &all, now).await?;

        let later = now + Duration::days(2);
        let unseen = ids(
            repository.list_unseen(reader, later).await?,
            &[published.id, draft.id, scheduled.id],
        )
        .await;
        assert_eq!(unseen, vec![scheduled.id]);

        let mut guard = tx.lock().await;
        let views = sqlx::query_scalar!(
            "SELECT count(*) FROM news_views WHERE user_id = $1",
            reader.0
        )
        .fetch_one(&mut ***guard)
        .await
        .unwrap();
        assert_eq!(views, Some(1));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn deleting_a_news_deletes_its_views() {
    run(async |tx| {
        let reader = seed_user(&tx).await;
        let mut repository = PgNewsRepository::new(&tx);
        let now = Utc::now();
        let item = news("gone", Some(now - Duration::hours(1)));
        repository.insert(&item).await?;
        repository.mark_seen(reader, &[item.id], now).await?;

        assert!(repository.delete(item.id).await?);
        assert!(!repository.delete(item.id).await?);

        let mut guard = tx.lock().await;
        let views = sqlx::query_scalar!(
            "SELECT count(*) FROM news_views WHERE news_id = $1",
            item.id.0
        )
        .fetch_one(&mut ***guard)
        .await
        .unwrap();
        assert_eq!(views, Some(0));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn update_replaces_the_content_and_reports_a_missing_news() {
    run(async |tx| {
        let mut repository = PgNewsRepository::new(&tx);
        let mut item = news("before", None);
        repository.insert(&item).await?;

        item.content.title = "after".to_owned();
        item.published_at = Some(Utc::now());
        let updated = repository.update(&item).await?.unwrap();
        assert_eq!(updated.content.title, "after");
        assert!(updated.published_at.is_some());

        let missing = news("missing", None);
        assert!(repository.update(&missing).await?.is_none());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn list_all_includes_drafts_and_scheduled_most_recently_updated_first() {
    run(async |tx| {
        let mut repository = PgNewsRepository::new(&tx);
        let now = Utc::now();
        let mut old = news("old", None);
        old.updated_at = now - Duration::days(1);
        let mut recent = news("recent", Some(now + Duration::days(1)));
        recent.updated_at = now;
        repository.insert(&old).await?;
        repository.insert(&recent).await?;

        let listed = ids(repository.list_all().await?, &[old.id, recent.id]).await;

        assert_eq!(listed, vec![recent.id, old.id]);
        Ok(())
    })
    .await;
}
