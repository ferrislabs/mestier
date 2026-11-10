use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use sqlx::PgPool;

use crate::{
    KnowledgeArticle, KnowledgeArticleId, KnowledgeFile, KnowledgeFilter, KnowledgeFolder,
    KnowledgeFolderId, KnowledgeFolderScope, KnowledgeKind, KnowledgeResource,
    KnowledgeResourceType, OrganizationId,
    application::test_support::dev_pool,
    domain::knowledge::ports::{KnowledgeArticleRepository, KnowledgeFolderRepository},
    infrastructure::{
        knowledge::postgres::{PgKnowledgeArticleRepository, PgKnowledgeFolderRepository},
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

fn base(organization_id: OrganizationId, kind: KnowledgeKind, title: &str) -> KnowledgeArticle {
    let now = Utc::now();
    KnowledgeArticle {
        id: KnowledgeArticleId(generate_uuid_v7()),
        organization_id,
        folder_id: None,
        kind,
        title: title.to_owned(),
        body: None,
        file: None,
        extracted_text: None,
        tags: Vec::new(),
        created_at: now,
        updated_at: now,
    }
}

fn note(organization_id: OrganizationId, title: &str, body: &str) -> KnowledgeArticle {
    KnowledgeArticle {
        body: Some(body.to_owned()),
        ..base(organization_id, KnowledgeKind::Note, title)
    }
}

fn file(organization_id: OrganizationId, title: &str, text: Option<&str>) -> KnowledgeArticle {
    KnowledgeArticle {
        file: Some(KnowledgeFile {
            key: format!("knowledge-x/{}", generate_uuid_v7()),
            mime_type: "application/pdf".to_owned(),
            size_bytes: 4096,
        }),
        extracted_text: text.map(str::to_owned),
        ..base(organization_id, KnowledgeKind::File, title)
    }
}

async fn found(
    repository: &mut PgKnowledgeArticleRepository<'_>,
    organization_id: OrganizationId,
    filter: KnowledgeFilter,
) -> Vec<KnowledgeArticleId> {
    repository
        .list(organization_id, &filter, 50, 0)
        .await
        .unwrap()
        .0
        .into_iter()
        .map(|article| article.id)
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
async fn a_note_and_a_file_round_trip() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repository = PgKnowledgeArticleRepository::new(&tx);
        let mut written = note(org, "Conditions de paiement", "30 jours fin de mois");
        written.tags = vec!["cgv".to_owned()];
        let stored_file = file(org, "Tarifs 2026", Some("Gazon en rouleau 6 euros"));

        repository.insert(&written).await?;
        repository.insert(&stored_file).await?;

        let note_back = repository.find_by_id(org, written.id).await?.unwrap();
        assert_eq!(note_back.body.as_deref(), Some("30 jours fin de mois"));
        assert_eq!(note_back.file, None);
        assert_eq!(note_back.tags, vec!["cgv"]);

        let file_back = repository.find_by_id(org, stored_file.id).await?.unwrap();
        assert_eq!(file_back.file, stored_file.file);
        assert_eq!(
            file_back.extracted_text.as_deref(),
            Some("Gazon en rouleau 6 euros")
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_refuses_an_article_of_the_wrong_shape() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repository = PgKnowledgeArticleRepository::new(&tx);
        let mut empty_note = base(org, KnowledgeKind::Note, "Vide");
        empty_note.body = Some("   ".to_owned());
        assert!(repository.insert(&empty_note).await.is_err());
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn search_finds_by_title_tag_note_and_file_text_in_french() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repository = PgKnowledgeArticleRepository::new(&tx);
        let by_title = note(org, "Arrosage automatique", "pose et réglage");
        let mut by_tag = note(org, "Mémo", "à relire");
        by_tag.tags = vec!["terrasse".to_owned()];
        let by_file_text = file(org, "Devis fournisseur", Some("Livraison de rosiers"));
        for article in [&by_title, &by_tag, &by_file_text] {
            repository.insert(article).await?;
        }

        let query = |text: &str| KnowledgeFilter {
            query: Some(text.to_owned()),
            ..Default::default()
        };
        assert_eq!(
            found(&mut repository, org, query("arrosages")).await,
            vec![by_title.id]
        );
        assert_eq!(
            found(&mut repository, org, query("terrasse")).await,
            vec![by_tag.id]
        );
        assert_eq!(
            found(&mut repository, org, query("rosier")).await,
            vec![by_file_text.id]
        );
        assert!(
            found(&mut repository, org, query("introuvable"))
                .await
                .is_empty()
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_title_match_ranks_above_a_text_match() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repository = PgKnowledgeArticleRepository::new(&tx);
        let in_text = note(org, "Mémo", "penser au gazon");
        let in_title = note(org, "Gazon", "prix au mètre carré");
        repository.insert(&in_text).await?;
        repository.insert(&in_title).await?;

        let order = found(
            &mut repository,
            org,
            KnowledgeFilter {
                query: Some("gazon".to_owned()),
                ..Default::default()
            },
        )
        .await;
        assert_eq!(order, vec![in_title.id, in_text.id]);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_list_leaves_the_extracted_text_out_but_a_get_keeps_it() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repository = PgKnowledgeArticleRepository::new(&tx);
        let stored = file(org, "Fiche", Some("texte long"));
        repository.insert(&stored).await?;

        let (listed, total) = repository
            .list(org, &KnowledgeFilter::default(), 10, 0)
            .await?;
        assert_eq!(total, 1);
        assert_eq!(listed[0].extracted_text, None);
        assert_eq!(
            repository
                .find_by_id(org, stored.id)
                .await?
                .unwrap()
                .extracted_text
                .as_deref(),
            Some("texte long")
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn tags_filter_requires_every_tag() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repository = PgKnowledgeArticleRepository::new(&tx);
        let mut both = note(org, "A", "x");
        both.tags = vec!["devis".to_owned(), "2026".to_owned()];
        let mut one = note(org, "B", "x");
        one.tags = vec!["devis".to_owned()];
        repository.insert(&both).await?;
        repository.insert(&one).await?;

        let filter = |tags: &[&str]| KnowledgeFilter {
            tags: tags.iter().map(|tag| (*tag).to_owned()).collect(),
            ..Default::default()
        };
        assert_eq!(
            found(&mut repository, org, filter(&["devis", "2026"])).await,
            vec![both.id]
        );
        assert_eq!(
            found(&mut repository, org, filter(&["devis"])).await.len(),
            2
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn update_changes_title_note_and_tags_and_the_search_follows() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repository = PgKnowledgeArticleRepository::new(&tx);
        let mut article = note(org, "Ancien titre", "ancien texte");
        repository.insert(&article).await?;

        article.title = "Nouveau titre".to_owned();
        article.body = Some("texte neuf".to_owned());
        article.tags = vec!["revu".to_owned()];
        repository.update(&article).await?;

        let query = |text: &str| KnowledgeFilter {
            query: Some(text.to_owned()),
            ..Default::default()
        };
        assert_eq!(
            found(&mut repository, org, query("nouveau")).await,
            vec![article.id]
        );
        assert!(
            found(&mut repository, org, query("ancien"))
                .await
                .is_empty()
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn links_are_idempotent_filterable_and_removed_with_the_article() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut repository = PgKnowledgeArticleRepository::new(&tx);
        let linked = note(org, "Lié", "x");
        let other = note(org, "Libre", "x");
        repository.insert(&linked).await?;
        repository.insert(&other).await?;
        let project = KnowledgeResource {
            resource_type: KnowledgeResourceType::Project,
            id: generate_uuid_v7(),
        };
        let customer = KnowledgeResource {
            resource_type: KnowledgeResourceType::Customer,
            id: generate_uuid_v7(),
        };

        repository.add_link(org, linked.id, project).await?;
        repository.add_link(org, linked.id, project).await?;
        repository.add_link(org, linked.id, customer).await?;
        assert_eq!(repository.links_of(org, linked.id).await?.len(), 2);

        let for_project = KnowledgeFilter {
            resource: Some(project),
            ..Default::default()
        };
        assert_eq!(
            found(&mut repository, org, for_project).await,
            vec![linked.id]
        );

        assert!(repository.remove_link(org, linked.id, customer).await?);
        assert!(!repository.remove_link(org, linked.id, customer).await?);

        repository.delete(org, linked.id).await?;
        assert!(repository.links_of(org, linked.id).await?.is_empty());
        assert!(matches!(
            repository.delete(org, linked.id).await,
            Err(CoreError::NotFound)
        ));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn another_organization_sees_and_touches_nothing() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let stranger = seed_organization(&tx).await;
        let mut repository = PgKnowledgeArticleRepository::new(&tx);
        let article = note(org, "Privé", "secret");
        repository.insert(&article).await?;
        let resource = KnowledgeResource {
            resource_type: KnowledgeResourceType::Quote,
            id: generate_uuid_v7(),
        };
        repository.add_link(org, article.id, resource).await?;

        assert_eq!(repository.find_by_id(stranger, article.id).await?, None);
        assert!(
            found(&mut repository, stranger, KnowledgeFilter::default())
                .await
                .is_empty()
        );
        assert!(matches!(
            repository.delete(stranger, article.id).await,
            Err(CoreError::NotFound)
        ));
        assert!(
            !repository
                .remove_link(stranger, article.id, resource)
                .await?
        );
        assert!(repository.links_of(stranger, article.id).await?.is_empty());
        assert!(
            repository
                .add_link(stranger, article.id, resource)
                .await
                .is_err()
        );
        Ok(())
    })
    .await;
}

fn folder(
    organization_id: OrganizationId,
    parent_id: Option<KnowledgeFolderId>,
    name: &str,
) -> KnowledgeFolder {
    let now = Utc::now();
    KnowledgeFolder {
        id: KnowledgeFolderId(generate_uuid_v7()),
        organization_id,
        parent_id,
        name: name.to_owned(),
        created_at: now,
        updated_at: now,
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_folder_knows_its_path_its_height_and_its_children() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut folders = PgKnowledgeFolderRepository::new(&tx);
        let top = folders.insert(&folder(org, None, "Chantiers")).await?;
        let middle = folders.insert(&folder(org, Some(top.id), "2026")).await?;
        let bottom = folders
            .insert(&folder(org, Some(middle.id), "Parc"))
            .await?;
        folders
            .insert(&folder(org, Some(top.id), "archives"))
            .await?;

        let path: Vec<_> = folders
            .path_to(org, bottom.id)
            .await?
            .into_iter()
            .map(|f| f.name)
            .collect();
        assert_eq!(path, vec!["Chantiers", "2026", "Parc"]);
        assert_eq!(folders.height(org, top.id).await?, 2);
        assert_eq!(folders.height(org, bottom.id).await?, 0);

        let children: Vec<_> = folders
            .children(org, Some(top.id))
            .await?
            .into_iter()
            .map(|f| f.name)
            .collect();
        assert_eq!(children, vec!["2026", "archives"]);
        assert_eq!(folders.children(org, None).await?.len(), 1);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn siblings_cannot_share_a_name_whatever_its_case() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let mut folders = PgKnowledgeFolderRepository::new(&tx);
        folders.insert(&folder(org, None, "Devis")).await?;
        let clash = folders.insert(&folder(org, None, "devis")).await;
        assert!(matches!(clash, Err(CoreError::Conflict(_))));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_parent_of_another_organization_is_refused() {
    run(async |tx| {
        let mine = seed_organization(&tx).await;
        let theirs = seed_organization(&tx).await;
        let mut folders = PgKnowledgeFolderRepository::new(&tx);
        let foreign = folders
            .insert(&folder(theirs, None, "Leur dossier"))
            .await?;
        assert!(folders.find_by_id(mine, foreign.id).await?.is_none());
        let refused = folders
            .insert(&folder(mine, Some(foreign.id), "Intrus"))
            .await;
        assert!(matches!(refused, Err(CoreError::Database(_))));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn articles_move_between_folders_and_are_listed_by_scope() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let mut folders = PgKnowledgeFolderRepository::new(&tx);
        let top = folders.insert(&folder(org, None, "Chantiers")).await?;
        let sub = folders.insert(&folder(org, Some(top.id), "Parc")).await?;
        let mut articles = PgKnowledgeArticleRepository::new(&tx);
        let at_root = articles.insert(&note(org, "Racine", "béton")).await?;
        let in_top = articles.insert(&note(org, "Haut", "béton")).await?;
        let in_sub = articles.insert(&note(org, "Bas", "béton")).await?;
        let foreign = articles.insert(&note(other, "Ailleurs", "béton")).await?;

        assert_eq!(
            articles
                .move_to_folder(org, &[in_top.id], Some(top.id))
                .await?,
            1
        );
        articles
            .move_to_folder(org, &[in_sub.id], Some(sub.id))
            .await?;
        assert_eq!(
            articles
                .move_to_folder(org, &[foreign.id], Some(top.id))
                .await?,
            0
        );
        assert!(!folders.is_empty(org, top.id).await?);

        let scoped = |folder| KnowledgeFilter {
            query: Some("béton".to_owned()),
            folder: Some(folder),
            ..KnowledgeFilter::default()
        };
        assert_eq!(
            found(&mut articles, org, scoped(KnowledgeFolderScope::Root)).await,
            vec![at_root.id]
        );
        assert_eq!(
            found(
                &mut articles,
                org,
                scoped(KnowledgeFolderScope::Directly(top.id))
            )
            .await,
            vec![in_top.id]
        );
        let mut within = found(
            &mut articles,
            org,
            scoped(KnowledgeFolderScope::Within(top.id)),
        )
        .await;
        within.sort_by_key(|id| id.0);
        let mut expected = vec![in_top.id, in_sub.id];
        expected.sort_by_key(|id| id.0);
        assert_eq!(within, expected);

        articles.move_to_folder(org, &[in_sub.id], None).await?;
        assert!(folders.is_empty(org, sub.id).await?);
        folders.delete(org, sub.id).await?;
        Ok(())
    })
    .await;
}
