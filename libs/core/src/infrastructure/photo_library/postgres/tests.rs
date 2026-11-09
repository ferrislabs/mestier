use chrono::{Duration, Utc};
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;

use crate::{
    InsertOutcome, LibraryPhoto, LibraryPhotoId, OrganizationId, PhotoFilter, ProjectId,
    application::test_support::dev_pool,
    domain::photo_library::ports::LibraryPhotoRepository,
    infrastructure::{
        photo_library::postgres::PgLibraryPhotoRepository,
        postgres::{SharedTx, with_tx},
    },
};

async fn seed_organization(tx: &SharedTx<'_>) -> OrganizationId {
    let mut guard = tx.lock().await;
    let user_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO users (id, email, username, display_name, sub)
           VALUES ($1, $2, $3, $4, $5)"#,
        user_id,
        format!("owner-{user_id}@example.com"),
        format!("owner-{user_id}"),
        "Owner",
        format!("sub-{user_id}"),
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    let org_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)"#,
        org_id,
        "Test Org",
        format!("test-org-{org_id}"),
        user_id,
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    OrganizationId(org_id)
}

async fn seed_project(tx: &SharedTx<'_>, organization_id: OrganizationId) -> ProjectId {
    let mut guard = tx.lock().await;
    let id = generate_uuid_v7();
    sqlx::query!(
        "INSERT INTO projects (id, org_id, name) VALUES ($1, $2, 'Avenue du 8 mai')",
        id,
        organization_id.0,
    )
    .execute(&mut ***guard)
    .await
    .unwrap();
    ProjectId(id)
}

fn photo(organization_id: OrganizationId, hash: &str) -> LibraryPhoto {
    let now = Utc::now();
    let id = generate_uuid_v7();
    LibraryPhoto {
        id: LibraryPhotoId(id),
        organization_id,
        file_key: format!("library-x/{id}"),
        thumbnail_key: format!("library-x/{id}-thumb"),
        mime_type: "image/jpeg".to_owned(),
        width: 2400,
        height: 1600,
        size_bytes: 812_345,
        content_hash: hash.to_owned(),
        caption: None,
        tags: Vec::new(),
        project_id: None,
        taken_at: None,
        created_at: now,
        updated_at: now,
    }
}

async fn inserted(
    repository: &mut PgLibraryPhotoRepository<'_>,
    photo: &LibraryPhoto,
) -> LibraryPhoto {
    match repository.insert_if_absent(photo).await.unwrap() {
        InsertOutcome::Inserted(stored) => stored,
        InsertOutcome::Existing(_) => panic!("the photo was expected to be new"),
    }
}

async fn listed(
    repository: &mut PgLibraryPhotoRepository<'_>,
    organization_id: OrganizationId,
    filter: PhotoFilter,
) -> Vec<LibraryPhotoId> {
    repository
        .list(organization_id, &filter, 50, 0)
        .await
        .unwrap()
        .0
        .into_iter()
        .map(|photo| photo.id)
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
async fn a_photo_round_trips_with_its_metadata() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let project_id = seed_project(&tx, org).await;
        let mut repository = PgLibraryPhotoRepository::new(&tx);
        let mut photo = photo(org, "hash-a");
        photo.caption = Some("Massif de rosiers".to_owned());
        photo.tags = vec!["rosier".to_owned(), "massif".to_owned()];
        photo.project_id = Some(project_id);
        photo.taken_at = Some(Utc::now() - Duration::days(3));

        let stored = inserted(&mut repository, &photo).await;
        let found = repository.find_by_id(org, photo.id).await?.unwrap();

        assert_eq!(stored.id, photo.id);
        assert_eq!(found.caption, photo.caption);
        assert_eq!(found.tags, photo.tags);
        assert_eq!(found.project_id, Some(project_id));
        assert_eq!(
            (found.width, found.height, found.size_bytes),
            (2400, 1600, 812_345)
        );
        assert_eq!(found.file_key, photo.file_key);
        assert_eq!(found.thumbnail_key, photo.thumbnail_key);
        assert_eq!(
            found.taken_at.map(|moment| moment.timestamp_micros()),
            photo.taken_at.map(|moment| moment.timestamp_micros())
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_same_hash_returns_the_existing_photo_without_a_second_row() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other_org = seed_organization(&tx).await;
        let mut repository = PgLibraryPhotoRepository::new(&tx);
        let first = photo(org, "same-hash");
        inserted(&mut repository, &first).await;

        let outcome = repository
            .insert_if_absent(&photo(org, "same-hash"))
            .await?;
        let elsewhere = repository
            .insert_if_absent(&photo(other_org, "same-hash"))
            .await?;

        match outcome {
            InsertOutcome::Existing(existing) => assert_eq!(existing.id, first.id),
            InsertOutcome::Inserted(_) => panic!("a duplicate must not be inserted"),
        }
        assert!(matches!(elsewhere, InsertOutcome::Inserted(_)));
        assert_eq!(
            listed(&mut repository, org, PhotoFilter::default())
                .await
                .len(),
            1
        );
        assert_eq!(
            repository.find_by_hash(org, "same-hash").await?.unwrap().id,
            first.id
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn another_organization_sees_neither_reads_nor_writes() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let intruder = seed_organization(&tx).await;
        let mut repository = PgLibraryPhotoRepository::new(&tx);
        let photo = photo(org, "hash-a");
        inserted(&mut repository, &photo).await;

        assert!(repository.find_by_id(intruder, photo.id).await?.is_none());
        assert!(
            listed(&mut repository, intruder, PhotoFilter::default())
                .await
                .is_empty()
        );
        let mut hijacked = photo.clone();
        hijacked.organization_id = intruder;
        assert!(matches!(
            repository.update(&hijacked).await,
            Err(CoreError::NotFound)
        ));
        assert!(matches!(
            repository.delete(intruder, photo.id).await,
            Err(CoreError::NotFound)
        ));
        assert!(repository.find_by_id(org, photo.id).await?.is_some());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_list_is_newest_first_and_paginated_with_a_total() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repository = PgLibraryPhotoRepository::new(&tx);
        let mut ids = Vec::new();
        for (index, minutes) in [30, 20, 10].into_iter().enumerate() {
            let mut photo = photo(org, &format!("hash-{index}"));
            photo.created_at = Utc::now() - Duration::minutes(minutes);
            inserted(&mut repository, &photo).await;
            ids.push(photo.id);
        }

        let (page, total) = repository.list(org, &PhotoFilter::default(), 2, 0).await?;
        let (rest, _) = repository.list(org, &PhotoFilter::default(), 2, 2).await?;

        assert_eq!(total, 3);
        assert_eq!(
            page.iter().map(|photo| photo.id).collect::<Vec<_>>(),
            vec![ids[2], ids[1]]
        );
        assert_eq!(
            rest.iter().map(|photo| photo.id).collect::<Vec<_>>(),
            vec![ids[0]]
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn filters_combine_text_tags_project_and_to_sort() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let project_id = seed_project(&tx, org).await;
        let mut repository = PgLibraryPhotoRepository::new(&tx);

        let mut roses = photo(org, "roses");
        roses.caption = Some("Massif de ROSIERS grimpants".to_owned());
        roses.tags = vec!["rosier".to_owned(), "terrasse".to_owned()];
        roses.project_id = Some(project_id);
        let mut terrace = photo(org, "terrace");
        terrace.caption = Some("Dalles de 50% grès".to_owned());
        terrace.tags = vec!["terrasse".to_owned()];
        let mut tagged_only = photo(org, "tagged");
        tagged_only.tags = vec!["haie".to_owned()];
        let raw = photo(org, "raw");
        for photo in [&roses, &terrace, &tagged_only, &raw] {
            inserted(&mut repository, photo).await;
        }

        let by_caption = PhotoFilter {
            query: Some("rosiers".to_owned()),
            ..PhotoFilter::default()
        };
        assert_eq!(
            listed(&mut repository, org, by_caption).await,
            vec![roses.id]
        );

        let by_tag_text = PhotoFilter {
            query: Some("HAIE".to_owned()),
            ..PhotoFilter::default()
        };
        assert_eq!(
            listed(&mut repository, org, by_tag_text).await,
            vec![tagged_only.id]
        );

        let literal_percent = PhotoFilter {
            query: Some("50%".to_owned()),
            ..PhotoFilter::default()
        };
        assert_eq!(
            listed(&mut repository, org, literal_percent).await,
            vec![terrace.id]
        );

        let wildcard_like = PhotoFilter {
            query: Some("r_siers".to_owned()),
            ..PhotoFilter::default()
        };
        assert!(listed(&mut repository, org, wildcard_like).await.is_empty());

        let one_tag = PhotoFilter {
            tags: vec!["terrasse".to_owned()],
            ..PhotoFilter::default()
        };
        assert_eq!(listed(&mut repository, org, one_tag).await.len(), 2);

        let both_tags = PhotoFilter {
            tags: vec!["terrasse".to_owned(), "rosier".to_owned()],
            ..PhotoFilter::default()
        };
        assert_eq!(
            listed(&mut repository, org, both_tags).await,
            vec![roses.id]
        );

        let by_project = PhotoFilter {
            project_id: Some(project_id),
            ..PhotoFilter::default()
        };
        assert_eq!(
            listed(&mut repository, org, by_project).await,
            vec![roses.id]
        );

        let to_sort = PhotoFilter {
            to_sort: true,
            ..PhotoFilter::default()
        };
        assert_eq!(listed(&mut repository, org, to_sort).await, vec![raw.id]);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_update_rewrites_the_editable_fields_only() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let project_id = seed_project(&tx, org).await;
        let mut repository = PgLibraryPhotoRepository::new(&tx);
        let original = photo(org, "hash-a");
        inserted(&mut repository, &original).await;

        let mut edited = original.clone();
        edited.caption = Some("Réception".to_owned());
        edited.tags = vec!["reception".to_owned()];
        edited.project_id = Some(project_id);
        edited.updated_at = Utc::now() + Duration::seconds(5);
        edited.content_hash = "ignored".to_owned();
        let stored = repository.update(&edited).await?;

        assert_eq!(stored.caption.as_deref(), Some("Réception"));
        assert_eq!(stored.tags, vec!["reception"]);
        assert_eq!(stored.project_id, Some(project_id));
        assert_eq!(stored.content_hash, "hash-a");
        assert_eq!(stored.file_key, original.file_key);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_project_of_another_organization_is_refused_by_the_schema() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let foreign_project = seed_project(&tx, other).await;
        let mut repository = PgLibraryPhotoRepository::new(&tx);
        let mut photo = photo(org, "hash-a");
        photo.project_id = Some(foreign_project);

        {
            let mut guard = tx.lock().await;
            sqlx::query("SAVEPOINT attempt")
                .execute(&mut ***guard)
                .await
                .unwrap();
        }
        let outcome = repository.insert_if_absent(&photo).await;
        {
            let mut guard = tx.lock().await;
            sqlx::query("ROLLBACK TO SAVEPOINT attempt")
                .execute(&mut ***guard)
                .await
                .unwrap();
        }

        assert!(outcome.is_err());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_deleted_photo_is_gone() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repository = PgLibraryPhotoRepository::new(&tx);
        let photo = photo(org, "hash-a");
        inserted(&mut repository, &photo).await;

        repository.delete(org, photo.id).await?;

        assert!(repository.find_by_id(org, photo.id).await?.is_none());
        assert!(matches!(
            repository.delete(org, photo.id).await,
            Err(CoreError::NotFound)
        ));
        Ok(())
    })
    .await;
}
