use chrono::Utc;
use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::transactional;
use uuid::Uuid;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        collection::{
            Collection, CollectionDetail, CollectionError, CollectionSummary,
            ImportCollectionCommand, MAX_COLLECTIONS_PER_ORGANIZATION, ReplaceCollectionCommand,
            slug_base, unique_slug,
        },
        ports::CollectionRepository,
    },
};

impl MestierUseCase {
    #[transactional(collection)]
    pub async fn import_collection(
        &self,
        command: ImportCollectionCommand,
    ) -> Result<Collection, CoreError> {
        let mut collections = collection_repository;
        collections.lock_slugs(command.org_id).await?;
        let held = collections.count_by_organization(command.org_id).await?;
        if usize::try_from(held).unwrap_or(usize::MAX) >= MAX_COLLECTIONS_PER_ORGANIZATION {
            return Err(CollectionError::TooMany.into());
        }
        let base = slug_base(&command.draft.title);
        let taken = collections.slugs_matching(command.org_id, &base).await?;
        let slug = unique_slug(&base, &taken);

        collections
            .insert(&Collection::from_draft(
                command.draft,
                generate_uuid_v7(),
                command.org_id,
                slug,
                Utc::now(),
            ))
            .await
    }

    #[transactional(collection)]
    pub async fn replace_collection(
        &self,
        command: ReplaceCollectionCommand,
    ) -> Result<CollectionDetail, CoreError> {
        let mut collections = collection_repository;
        let current = collections
            .find_by_id(command.org_id, command.id)
            .await?
            .ok_or(CollectionError::NotFound)?;

        let collection = collections
            .replace(&current.replaced_by(command.draft, Utc::now()))
            .await?;
        let history = collections.history(command.org_id, command.id).await?;

        Ok(CollectionDetail {
            collection,
            history,
        })
    }

    #[transactional(collection)]
    pub async fn list_collections(
        &self,
        org_id: OrganizationId,
    ) -> Result<Vec<CollectionSummary>, CoreError> {
        let mut collections = collection_repository;
        collections.list_by_organization(org_id).await
    }

    #[transactional(collection)]
    pub async fn find_collection(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<Collection>, CoreError> {
        let mut collections = collection_repository;
        collections.find_by_id(org_id, id).await
    }

    #[transactional(collection)]
    pub async fn find_collections(
        &self,
        org_id: OrganizationId,
        ids: &[Uuid],
    ) -> Result<Vec<Collection>, CoreError> {
        let mut collections = collection_repository;
        collections.find_many(org_id, ids).await
    }

    #[transactional(collection)]
    pub async fn find_collection_detail(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<CollectionDetail>, CoreError> {
        let mut collections = collection_repository;
        collections.find_detail(org_id, id).await
    }

    #[transactional(collection)]
    pub async fn delete_collection(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let mut collections = collection_repository;
        collections.delete(org_id, id).await
    }
}

#[cfg(test)]
mod tests {
    use common::{CoreError, OrganizationId, generate_uuid_v7};
    use serde_json::{Map, json};
    use sqlx::PgPool;

    use crate::application::MestierUseCase;
    use crate::application::default_authorizer;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::collection::{
        CollectionDraft, CollectionDraftInput, CollectionSource, HttpMethod,
        ImportCollectionCommand, Operation, ReplaceCollectionCommand,
    };
    use crate::infrastructure::realtime::EventHub;

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();

        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    fn use_case(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    fn draft(title: &str, version: &str) -> CollectionDraft {
        CollectionDraft::new(CollectionDraftInput {
            title: title.to_owned(),
            version: version.to_owned(),
            description: String::new(),
            base_url: "https://api.example.com".to_owned(),
            source: CollectionSource::Url {
                url: "https://api.example.com/openapi.json".to_owned(),
            },
            security: Vec::new(),
            operations: vec![Operation {
                id: "list".to_owned(),
                method: HttpMethod::Get,
                path: "/things".to_owned(),
                summary: "List".to_owned(),
                tag: String::new(),
                parameters: Vec::new(),
                request_body: None,
                response: Some(json!({ "type": "array" })),
                response_status: Some("200".to_owned()),
            }],
            schemas: Map::new(),
        })
        .unwrap()
    }

    fn import(org_id: OrganizationId, title: &str, version: &str) -> ImportCollectionCommand {
        ImportCollectionCommand {
            org_id,
            draft: draft(title, version),
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_imported_collection_is_stored_for_its_organization_and_no_other() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "collection-import").await;
        let stranger = seed_organization(&pool, "collection-import-stranger").await;
        let usecase = use_case(pool);

        let imported = usecase
            .import_collection(import(org_id, "Annuaire des entreprises", "1.0.0"))
            .await
            .unwrap();

        assert_eq!(imported.slug, "annuaire-des-entreprises");
        assert_eq!(imported.org_id, org_id);
        let mine = usecase.list_collections(org_id).await.unwrap();
        let theirs = usecase.list_collections(stranger).await.unwrap();
        assert_eq!(mine.len(), 1);
        assert_eq!(mine[0].id, imported.id);
        assert!(theirs.is_empty());
        assert!(
            usecase
                .find_collection(stranger, imported.id)
                .await
                .unwrap()
                .is_none()
        );
        assert!(
            usecase
                .find_collection_detail(stranger, imported.id)
                .await
                .unwrap()
                .is_none()
        );
        let detail = usecase
            .find_collection_detail(org_id, imported.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(detail.collection, imported);
        assert!(detail.history.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_clashing_title_gets_a_suffixed_slug_within_the_organization_only() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "collection-slug").await;
        let other = seed_organization(&pool, "collection-slug-other").await;
        let usecase = use_case(pool);

        let first = usecase
            .import_collection(import(org_id, "API Adresse", "1"))
            .await
            .unwrap();
        let second = usecase
            .import_collection(import(org_id, "api adresse", "1"))
            .await
            .unwrap();
        let third = usecase
            .import_collection(import(org_id, "API  Adresse!", "1"))
            .await
            .unwrap();
        let elsewhere = usecase
            .import_collection(import(other, "API Adresse", "1"))
            .await
            .unwrap();

        assert_eq!(first.slug, "api-adresse");
        assert_eq!(second.slug, "api-adresse-2");
        assert_eq!(third.slug, "api-adresse-3");
        assert_eq!(elsewhere.slug, "api-adresse");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn two_simultaneous_imports_of_the_same_title_both_succeed_with_distinct_slugs() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "collection-race").await;
        let usecase = use_case(pool);

        let (left, right) = tokio::join!(
            usecase.import_collection(import(org_id, "Race", "1")),
            usecase.import_collection(import(org_id, "Race", "1")),
        );

        let mut slugs = vec![left.unwrap().slug, right.unwrap().slug];
        slugs.sort();
        assert_eq!(slugs, ["race", "race-2"]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replacing_keeps_the_previous_version_in_the_history_and_the_slug() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "collection-replace").await;
        let usecase = use_case(pool);
        let first = usecase
            .import_collection(import(org_id, "Annuaire", "1.0.0"))
            .await
            .unwrap();

        let second_answer = usecase
            .replace_collection(ReplaceCollectionCommand {
                org_id,
                id: first.id,
                draft: draft("Annuaire renamed", "2.0.0"),
            })
            .await
            .unwrap();
        let second = second_answer.collection.clone();
        assert_eq!(second_answer.history, vec![first.version_entry()]);
        let third = usecase
            .replace_collection(ReplaceCollectionCommand {
                org_id,
                id: first.id,
                draft: draft("Annuaire renamed", "3.0.0"),
            })
            .await
            .unwrap()
            .collection;

        assert_eq!(second.id, first.id);
        assert_eq!(second.slug, "annuaire");
        assert_eq!(second.title, "Annuaire renamed");
        assert_eq!(third.version, "3.0.0");
        let detail = usecase
            .find_collection_detail(org_id, first.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(detail.collection, third);
        assert_eq!(
            detail.history,
            vec![second.version_entry(), first.version_entry()]
        );
        assert_eq!(usecase.list_collections(org_id).await.unwrap().len(), 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn replacing_a_collection_of_another_organization_is_not_found() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "collection-replace-owner").await;
        let stranger = seed_organization(&pool, "collection-replace-stranger").await;
        let usecase = use_case(pool);
        let first = usecase
            .import_collection(import(org_id, "Annuaire", "1.0.0"))
            .await
            .unwrap();

        let outcome = usecase
            .replace_collection(ReplaceCollectionCommand {
                org_id: stranger,
                id: first.id,
                draft: draft("Hijacked", "9.9.9"),
            })
            .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)), "{outcome:?}");
        let unchanged = usecase.find_collection(org_id, first.id).await.unwrap();
        assert_eq!(unchanged, Some(first));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_removes_the_collection_and_its_history() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "collection-delete").await;
        let usecase = use_case(pool.clone());
        let first = usecase
            .import_collection(import(org_id, "Annuaire", "1.0.0"))
            .await
            .unwrap();
        usecase
            .replace_collection(ReplaceCollectionCommand {
                org_id,
                id: first.id,
                draft: draft("Annuaire", "2.0.0"),
            })
            .await
            .unwrap();

        usecase.delete_collection(org_id, first.id).await.unwrap();

        assert!(
            usecase
                .find_collection(org_id, first.id)
                .await
                .unwrap()
                .is_none()
        );
        let history_rows: i64 = sqlx::query_scalar(
            "SELECT count(*) FROM automation.collection_version WHERE collection_id = $1",
        )
        .bind(first.id)
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(history_rows, 0);
        let again = usecase.delete_collection(org_id, first.id).await;
        assert!(matches!(again, Err(CoreError::NotFound)), "{again:?}");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn several_collections_are_found_at_once_inside_their_organization() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "collection-many").await;
        let stranger = seed_organization(&pool, "collection-many-stranger").await;
        let usecase = use_case(pool);
        let first = usecase
            .import_collection(import(org_id, "One", "1"))
            .await
            .unwrap();
        let second = usecase
            .import_collection(import(org_id, "Two", "1"))
            .await
            .unwrap();
        usecase
            .import_collection(import(org_id, "Three", "1"))
            .await
            .unwrap();

        let mine = usecase
            .find_collections(org_id, &[first.id, second.id])
            .await
            .unwrap();
        let theirs = usecase
            .find_collections(stranger, &[first.id, second.id])
            .await
            .unwrap();

        assert_eq!(mine.len(), 2);
        assert!(mine.contains(&first) && mine.contains(&second));
        assert!(theirs.is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_organization_holds_at_most_two_hundred_collections() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "collection-cap").await;
        let other = seed_organization(&pool, "collection-cap-other").await;
        let usecase = use_case(pool);
        let mut last = None;
        for n in 0..200 {
            last = Some(
                usecase
                    .import_collection(import(org_id, &format!("Api {n}"), "1"))
                    .await
                    .unwrap(),
            );
        }

        let refused = usecase
            .import_collection(import(org_id, "One too many", "1"))
            .await;

        let Err(CoreError::Conflict(message)) = refused else {
            panic!("expected a conflict, got {refused:?}");
        };
        assert_eq!(message, "an organization holds at most 200 collections");
        assert!(
            usecase
                .import_collection(import(other, "Elsewhere", "1"))
                .await
                .is_ok()
        );
        usecase
            .delete_collection(org_id, last.unwrap().id)
            .await
            .unwrap();
        assert!(
            usecase
                .import_collection(import(org_id, "Room again", "1"))
                .await
                .is_ok()
        );
        assert_eq!(usecase.list_collections(org_id).await.unwrap().len(), 200);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_replace_answer_and_the_read_carry_at_most_fifty_history_entries() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "collection-history-cap").await;
        let usecase = use_case(pool);
        let first = usecase
            .import_collection(import(org_id, "Annuaire", "v0"))
            .await
            .unwrap();
        let mut answer = None;
        for n in 1..=52 {
            answer = Some(
                usecase
                    .replace_collection(ReplaceCollectionCommand {
                        org_id,
                        id: first.id,
                        draft: draft("Annuaire", &format!("v{n}")),
                    })
                    .await
                    .unwrap(),
            );
        }

        let answer = answer.unwrap();
        let read = usecase
            .find_collection_detail(org_id, first.id)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(answer.history.len(), 50);
        assert_eq!(answer.history[0].version, "v51");
        assert_eq!(read.history, answer.history);
        assert_eq!(read.collection, answer.collection);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn the_import_date_is_stamped_by_the_server() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "collection-stamp").await;
        let usecase = use_case(pool);
        let before = chrono::Utc::now() - chrono::Duration::seconds(1);

        let imported = usecase
            .import_collection(import(org_id, "Stamped", "1"))
            .await
            .unwrap();

        assert!(imported.imported_at >= before);
        assert!(imported.imported_at <= chrono::Utc::now());
    }
}
