use common::{CoreError, UserId, generate_uuid_v7};
use serde_json::json;
use uuid::Uuid;

use super::tests::{run, seed_organization};
use crate::{
    KnowledgeArticleId, KnowledgeFilter, KnowledgeKind, KnowledgeResource, KnowledgeResourceType,
    OrganizationId,
    domain::knowledge::{
        documents::{
            content::DocumentContent,
            markdown::{from_markdown, render},
            ports::{
                DocumentReferenceRepository, KnowledgeDocumentRepository, LockAttempt,
                ReplaceAttempt, SaveAttempt,
            },
            references::DocumentReference,
            service::KnowledgeDocumentService,
        },
        ports::KnowledgeArticleRepository,
        service::KnowledgeService,
        tables::{KnowledgeTableId, TableViewId},
    },
    infrastructure::{
        knowledge::postgres::{
            PgDocumentReferenceRepository, PgKnowledgeArticleRepository,
            PgKnowledgeDocumentRepository,
        },
        postgres::SharedTx,
    },
};

async fn seed_user(tx: &SharedTx<'_>) -> UserId {
    let mut guard = tx.lock().await;
    let id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO users (id, email, username, display_name, sub)
           VALUES ($1, $2, $3, $4, $5)"#,
        id,
        format!("doc-{id}@example.com"),
        format!("doc-{id}"),
        "Writer",
        format!("sub-doc-{id}"),
    )
    .execute(&mut ***guard)
    .await
    .unwrap();
    UserId(id)
}

async fn new_document(tx: &SharedTx<'_>, org: OrganizationId, title: &str) -> KnowledgeArticleId {
    KnowledgeService::new(PgKnowledgeArticleRepository::new(tx))
        .create_document(
            org,
            None,
            title,
            &[],
            DocumentContent::empty(),
            String::new(),
        )
        .await
        .unwrap()
        .unwrap()
        .id
}

async fn expire_lock(tx: &SharedTx<'_>, id: KnowledgeArticleId) {
    let mut guard = tx.lock().await;
    sqlx::query!(
        "UPDATE knowledge_articles SET lock_expires_at = now() - interval '1 second' WHERE id = $1",
        id.0
    )
    .execute(&mut ***guard)
    .await
    .unwrap();
}

fn content(markdown: &str) -> DocumentContent {
    from_markdown(markdown).unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_new_document_is_empty_at_revision_zero_in_its_folder() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let id = new_document(&tx, org, "Compte rendu").await;
        let mut articles = PgKnowledgeArticleRepository::new(&tx);
        let article = articles.find_by_id(org, id).await?.unwrap();
        assert_eq!(article.kind, KnowledgeKind::Document);
        assert_eq!(article.revision, 0);
        assert_eq!(article.body.as_deref(), Some(""));
        assert_eq!(article.content, Some(DocumentContent::empty().to_value()));
        assert_eq!(article.lock, None);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn one_person_holds_the_lock_and_an_expired_lock_is_free() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let id = new_document(&tx, org, "Verrou").await;
        let (alice, bob) = (seed_user(&tx).await, seed_user(&tx).await);
        let mut documents = PgKnowledgeDocumentRepository::new(&tx);

        let LockAttempt::Granted { lock, revision } =
            documents.acquire_lock(org, id, alice).await?
        else {
            panic!("the first lock is granted");
        };
        assert_eq!((lock.holder, revision), (alice, 0));
        let LockAttempt::Held(held) = documents.acquire_lock(org, id, bob).await? else {
            panic!("a live lock refuses another holder");
        };
        assert_eq!(held.holder, alice);
        assert!(matches!(
            documents.acquire_lock(org, id, alice).await?,
            LockAttempt::Granted { .. }
        ));

        documents.release_lock(org, id, bob).await?;
        assert_eq!(
            documents
                .state(org, id)
                .await?
                .unwrap()
                .lock
                .unwrap()
                .holder,
            alice
        );

        expire_lock(&tx, id).await;
        assert!(matches!(
            documents.acquire_lock(org, id, bob).await?,
            LockAttempt::Granted { lock, .. } if lock.holder == bob
        ));
        documents.release_lock(org, id, bob).await?;
        assert_eq!(documents.state(org, id).await?.unwrap().lock, None);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_lock_of_an_unknown_or_foreign_document_is_missing() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other_org = seed_organization(&tx).await;
        let id = new_document(&tx, org, "Privé").await;
        let alice = seed_user(&tx).await;
        let mut documents = PgKnowledgeDocumentRepository::new(&tx);
        assert_eq!(
            documents.acquire_lock(other_org, id, alice).await?,
            LockAttempt::Missing
        );
        assert_eq!(
            documents
                .acquire_lock(org, KnowledgeArticleId(Uuid::now_v7()), alice)
                .await?,
            LockAttempt::Missing
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_save_needs_the_live_lock_and_the_current_revision() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let id = new_document(&tx, org, "Chantier").await;
        let (alice, bob) = (seed_user(&tx).await, seed_user(&tx).await);
        let mut documents = PgKnowledgeDocumentRepository::new(&tx);
        let written = content("# Plan\n\nPosepergola et gazon");
        let body = render(&written);

        assert_eq!(
            documents.save(org, id, alice, 0, &written, &body).await?,
            SaveAttempt::LockLost
        );
        documents.acquire_lock(org, id, alice).await?;
        assert_eq!(
            documents.save(org, id, bob, 0, &written, &body).await?,
            SaveAttempt::LockLost
        );
        assert!(matches!(
            documents.save(org, id, alice, 0, &written, &body).await?,
            SaveAttempt::Saved { revision: 1, .. }
        ));
        assert_eq!(
            documents.save(org, id, alice, 0, &written, &body).await?,
            SaveAttempt::RevisionConflict
        );
        assert!(matches!(
            documents.save(org, id, alice, 1, &written, &body).await?,
            SaveAttempt::Saved { revision: 2, .. }
        ));

        expire_lock(&tx, id).await;
        assert_eq!(
            documents.save(org, id, alice, 2, &written, &body).await?,
            SaveAttempt::LockLost
        );
        let state = documents.state(org, id).await?.unwrap();
        assert_eq!(state.revision, 2);
        assert_eq!(state.content, written.to_value());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_saved_document_is_found_by_a_word_of_its_body() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let id = new_document(&tx, org, "Notes").await;
        let alice = seed_user(&tx).await;
        let mut documents = PgKnowledgeDocumentRepository::new(&tx);
        documents.acquire_lock(org, id, alice).await?;
        let written = content("# Notes\n\nLivraison de rosiers mardi");
        documents
            .save(org, id, alice, 0, &written, &render(&written))
            .await?;

        let mut articles = PgKnowledgeArticleRepository::new(&tx);
        let filter = KnowledgeFilter {
            query: Some("rosier".to_owned()),
            ..Default::default()
        };
        let (found, _) = articles.list(org, &filter, 10, 0).await?;
        assert_eq!(found.iter().map(|a| a.id).collect::<Vec<_>>(), vec![id]);
        let article = articles.find_by_id(org, id).await?.unwrap();
        assert!(article.body.unwrap().contains("rosiers"));
        assert_eq!(article.revision, 1);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_markdown_write_is_refused_while_someone_else_holds_the_lock() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let id = new_document(&tx, org, "Partagé").await;
        let (alice, bob) = (seed_user(&tx).await, seed_user(&tx).await);
        let mut documents = PgKnowledgeDocumentRepository::new(&tx);
        let written = content("texte");
        let body = render(&written);

        assert!(matches!(
            documents.replace(org, id, bob, &written, &body).await?,
            ReplaceAttempt::Replaced { revision: 1, .. }
        ));
        documents.acquire_lock(org, id, alice).await?;
        assert!(matches!(
            documents.replace(org, id, bob, &written, &body).await?,
            ReplaceAttempt::Held(lock) if lock.holder == alice
        ));
        assert!(matches!(
            documents.replace(org, id, alice, &written, &body).await?,
            ReplaceAttempt::Replaced { revision: 2, .. }
        ));
        expire_lock(&tx, id).await;
        assert!(matches!(
            documents.replace(org, id, bob, &written, &body).await?,
            ReplaceAttempt::Replaced { revision: 3, .. }
        ));
        assert_eq!(
            documents
                .replace(
                    org,
                    KnowledgeArticleId(Uuid::now_v7()),
                    bob,
                    &written,
                    &body
                )
                .await?,
            ReplaceAttempt::Missing
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_markdown_write_through_the_service_stores_blocks_and_bumps_the_revision() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let id = new_document(&tx, org, "Service").await;
        let alice = seed_user(&tx).await;
        let outcome = KnowledgeDocumentService::new(PgKnowledgeDocumentRepository::new(&tx))
            .replace_from_markdown(
                &mut PgDocumentReferenceRepository::new(&tx),
                org,
                id,
                alice,
                "# Titre\n\n- [x] fait\n\n> citation\n\n```rust\nlet a = 1;\n```",
            )
            .await?
            .unwrap();
        assert_eq!(outcome.revision, 1);
        let state = PgKnowledgeDocumentRepository::new(&tx)
            .state(org, id)
            .await?
            .unwrap();
        let blocks = state.content["blocks"].as_array().unwrap();
        let types: Vec<_> = blocks.iter().map(|b| b["type"].as_str().unwrap()).collect();
        assert_eq!(types, ["heading", "check_item", "quote", "code"]);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn references_are_found_only_inside_their_organization() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other_org = seed_organization(&tx).await;
        let (customer, table, view, file) = (
            generate_uuid_v7(),
            generate_uuid_v7(),
            generate_uuid_v7(),
            generate_uuid_v7(),
        );
        {
            let mut guard = tx.lock().await;
            sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, 'Dupont')")
                .bind(customer)
                .bind(org.0)
                .execute(&mut ***guard)
                .await
                .unwrap();
            sqlx::query("INSERT INTO knowledge_tables (id, org_id, name) VALUES ($1, $2, 'Suivi')")
                .bind(table)
                .bind(org.0)
                .execute(&mut ***guard)
                .await
                .unwrap();
            sqlx::query(
                "INSERT INTO knowledge_table_views (id, table_id, org_id, name, kind, position)
                 VALUES ($1, $2, $3, 'Grille', 'grid', 0)",
            )
            .bind(view)
            .bind(table)
            .bind(org.0)
            .execute(&mut ***guard)
            .await
            .unwrap();
            sqlx::query(
                "INSERT INTO knowledge_articles (id, org_id, kind, title, file_key, mime_type, size_bytes)
                 VALUES ($1, $2, 'file', 'Devis', 'k', 'application/pdf', 10)",
            )
            .bind(file)
            .bind(org.0)
            .execute(&mut ***guard)
            .await
            .unwrap();
        }
        let document = new_document(&tx, org, "Pas un fichier").await;
        let mut references = PgDocumentReferenceRepository::new(&tx);
        let customer_ref = DocumentReference::Resource(KnowledgeResource {
            resource_type: KnowledgeResourceType::Customer,
            id: customer,
        });
        let view_ref = DocumentReference::TableView {
            table_id: KnowledgeTableId(table),
            view_id: TableViewId(view),
        };
        let file_ref = DocumentReference::DriveFile(KnowledgeArticleId(file));

        for reference in [customer_ref, view_ref, file_ref] {
            assert!(references.exists(org, reference).await?, "{reference:?}");
            assert!(!references.exists(other_org, reference).await?, "{reference:?}");
        }
        assert!(
            !references
                .exists(org, DocumentReference::DriveFile(document))
                .await?
        );
        assert!(
            !references
                .exists(
                    org,
                    DocumentReference::TableView {
                        table_id: KnowledgeTableId(table),
                        view_id: TableViewId(Uuid::now_v7()),
                    }
                )
                .await?
        );
        assert!(!references.exists(org, DocumentReference::Photo(Uuid::now_v7())).await?);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn saving_a_document_whose_table_was_deleted_still_works() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let id = new_document(&tx, org, "Orphelin").await;
        let alice = seed_user(&tx).await;
        let (table, view) = (Uuid::now_v7(), Uuid::now_v7());
        let with_view = DocumentContent::parse(&json!({ "version": 1, "blocks": [
            { "id": "a", "type": "table_view", "props": { "table_id": table, "view_id": view },
              "content": [], "children": [] }
        ] }))
        .unwrap();
        let mut documents = PgKnowledgeDocumentRepository::new(&tx);
        documents.acquire_lock(org, id, alice).await?;
        documents
            .save(org, id, alice, 0, &with_view, &render(&with_view))
            .await?;

        let again = json!({ "version": 1, "blocks": [
            { "id": "a", "type": "table_view", "props": { "table_id": table, "view_id": view },
              "content": [], "children": [] },
            { "id": "b", "type": "paragraph", "props": {},
              "content": [{ "type": "text", "text": "ajout", "styles": {} }], "children": [] }
        ] });
        let outcome = KnowledgeDocumentService::new(PgKnowledgeDocumentRepository::new(&tx))
            .save(
                &mut PgDocumentReferenceRepository::new(&tx),
                org,
                id,
                alice,
                1,
                &again,
            )
            .await?;
        assert_eq!(outcome.unwrap().revision, 2);

        let foreign = json!({ "version": 1, "blocks": [
            { "id": "a", "type": "table_view",
              "props": { "table_id": Uuid::now_v7(), "view_id": Uuid::now_v7() },
              "content": [], "children": [] }
        ] });
        let refused = KnowledgeDocumentService::new(PgKnowledgeDocumentRepository::new(&tx))
            .save(
                &mut PgDocumentReferenceRepository::new(&tx),
                org,
                id,
                alice,
                2,
                &foreign,
            )
            .await;
        assert!(matches!(refused, Err(CoreError::NotFound)));
        Ok(())
    })
    .await;
}

static MIGRATION_SCRATCH: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_documents_migration_turns_notes_into_documents_and_back() {
    const PREVIOUS: i64 = 20261119000001;
    let pool =
        crate::application::test_support::scratch_pool("mestier_docmig_test_", &MIGRATION_SCRATCH)
            .await;
    let migrator = sqlx::migrate::Migrator::new(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
    )
    .await
    .unwrap();
    migrator.undo(&pool, PREVIOUS).await.unwrap();

    let owner = Uuid::now_v7();
    let org = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, 'Owner', $4)",
    )
    .bind(owner)
    .bind(format!("owner-{owner}@example.com"))
    .bind(format!("owner-{owner}"))
    .bind(format!("sub-{owner}"))
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, 'Org', $2, $3)")
        .bind(org)
        .bind(format!("org-{org}"))
        .bind(owner)
        .execute(&pool)
        .await
        .unwrap();
    let (note, file) = (Uuid::now_v7(), Uuid::now_v7());
    let text = "premier devis\ndeuxième \"ligne\" avec # et *\n\ntroisième";
    sqlx::query(
        "INSERT INTO knowledge_articles (id, org_id, kind, title, body) VALUES ($1, $2, 'note', 'Trois lignes', $3)",
    )
    .bind(note)
    .bind(org)
    .bind(text)
    .execute(&pool)
    .await
    .unwrap();
    let long_note = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO knowledge_articles (id, org_id, kind, title, body)
         SELECT $1, $2, 'note', 'Six mille lignes', string_agg('ligne ' || g, E'\n' ORDER BY g)
         FROM generate_series(1, 6000) AS g",
    )
    .bind(long_note)
    .bind(org)
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO knowledge_articles (id, org_id, kind, title, file_key, mime_type, size_bytes)
         VALUES ($1, $2, 'file', 'Tarifs', 'k', 'application/pdf', 10)",
    )
    .bind(file)
    .bind(org)
    .execute(&pool)
    .await
    .unwrap();

    migrator.run(&pool).await.unwrap();
    let row: (String, i32, serde_json::Value, String) = sqlx::query_as(
        "SELECT kind, revision, content, body FROM knowledge_articles WHERE id = $1",
    )
    .bind(note)
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((row.0.as_str(), row.1), ("document", 0));
    let parsed = DocumentContent::parse(&row.2).expect("the converted content is valid");
    let paragraphs: Vec<String> = parsed
        .blocks()
        .iter()
        .map(|block| match block.kind() {
            crate::domain::knowledge::documents::content::BlockKind::Paragraph(content) => {
                match &content[0] {
                    crate::domain::knowledge::documents::content::Inline::Text(run) => {
                        run.text.clone()
                    }
                    other => panic!("{other:?}"),
                }
            }
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!(
        paragraphs,
        [
            "premier devis",
            "deuxième \"ligne\" avec # et *",
            "troisième"
        ]
    );
    assert!(row.3.contains("troisième"));
    let long_content: serde_json::Value =
        sqlx::query_scalar("SELECT content FROM knowledge_articles WHERE id = $1")
            .bind(long_note)
            .fetch_one(&pool)
            .await
            .unwrap();
    let long_parsed =
        DocumentContent::parse(&long_content).expect("a 6000 line note stays a valid document");
    assert!(long_parsed.blocks().len() <= 5_000);
    let long_text: String = sqlx::query_scalar(
        "SELECT string_agg(b -> 'content' -> 0 ->> 'text', E'\n' ORDER BY n)
         FROM knowledge_articles a, jsonb_array_elements(a.content -> 'blocks') WITH ORDINALITY AS t(b, n)
         WHERE a.id = $1",
    )
    .bind(long_note)
    .fetch_one(&pool)
    .await
    .unwrap();
    let expected: Vec<String> = (1..=6000).map(|i| format!("ligne {i}")).collect();
    assert_eq!(long_text, expected.join("\n"));

    let file_kind: String = sqlx::query_scalar("SELECT kind FROM knowledge_articles WHERE id = $1")
        .bind(file)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(file_kind, "file");

    migrator.undo(&pool, PREVIOUS).await.unwrap();
    let back: (String, String) =
        sqlx::query_as("SELECT kind, body FROM knowledge_articles WHERE id = $1")
            .bind(note)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(back.0, "note");
    assert_eq!(
        back.1,
        "premier devis\ndeuxième \"ligne\" avec # et *\ntroisième"
    );

    migrator.run(&pool).await.unwrap();
    let again: String = sqlx::query_scalar("SELECT kind FROM knowledge_articles WHERE id = $1")
        .bind(note)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(again, "document");
}
