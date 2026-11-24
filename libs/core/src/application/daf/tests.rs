use common::{CoreError, FileStorageConfig, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    CreateDafCommand, DafDesignation, DafId, DafNumber, DafStatus, DeleteDafCommand,
    LibraryPhotoId, PlaceDafPhotoCommand, ProductId, ProjectId, UpdateDafCommand,
    UpdateDafDossierCommand,
    application::{
        MestierFileStorageService, MestierUseCase, default_authorizer,
        test_support::{purge, scratch_pool},
    },
    domain::file_storage::commands::UploadFileCommand,
    infrastructure::{file_storage::S3FileStorage, realtime::EventHub},
};

static DAF_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_daf_test_", &DAF_SCRATCH).await
}

fn usecase(pool: &PgPool) -> MestierUseCase {
    MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new())
}

struct Fixture {
    user_id: Uuid,
    organization_id: OrganizationId,
    project_id: ProjectId,
}

async fn seed_user(pool: &PgPool) -> Uuid {
    let user_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("user-{user_id}@example.com"))
    .bind(format!("user-{user_id}"))
    .bind("Test User")
    .bind(format!("sub-{user_id}"))
    .execute(pool)
    .await
    .unwrap();
    user_id
}

async fn seed_fixture(pool: &PgPool) -> Fixture {
    let user_id = seed_user(pool).await;
    let organization_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organizations (id, name, slug, owner_id, address_line1, address_postal_code, address_city) \
         VALUES ($1, 'Treyve Paysages', $2, $3, '3 chemin des Vignes', '03120', 'Lapalisse')",
    )
    .bind(organization_id)
    .bind(format!("org-{organization_id}"))
    .bind(user_id)
    .execute(pool)
    .await
    .unwrap();

    let customer_id = generate_uuid_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, 'Commune de Cébazat')")
        .bind(customer_id)
        .bind(organization_id)
        .execute(pool)
        .await
        .unwrap();

    let project_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO projects (id, org_id, name, customer_id) VALUES ($1, $2, 'Parc Montgroux', $3)",
    )
    .bind(project_id)
    .bind(organization_id)
    .bind(customer_id)
    .execute(pool)
    .await
    .unwrap();

    Fixture {
        user_id,
        organization_id: OrganizationId(organization_id),
        project_id: ProjectId(project_id),
    }
}

async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[Uuid]) {
    for statement in [
        "DELETE FROM dafs WHERE org_id = $1",
        "DELETE FROM daf_dossiers WHERE org_id = $1",
        "DELETE FROM library_photos WHERE org_id = $1",
        "DELETE FROM projects WHERE org_id = $1",
        "DELETE FROM products WHERE org_id = $1",
        "DELETE FROM suppliers WHERE org_id = $1",
        "DELETE FROM customers WHERE org_id = $1",
        "DELETE FROM organization_members WHERE organization_id = $1",
        "DELETE FROM organizations WHERE id = $1",
    ] {
        purge(pool, statement, organization_id.0).await;
    }
    for user_id in user_ids {
        purge(pool, "DELETE FROM users WHERE id = $1", *user_id).await;
    }
}

fn create_command(fixture: &Fixture, designation: &str) -> CreateDafCommand {
    CreateDafCommand {
        actor: authz::Subject::system(),
        organization_id: fixture.organization_id,
        project_id: fixture.project_id,
        designation: Some(DafDesignation::try_from(designation.to_owned()).unwrap()),
        product_id: None,
    }
}

fn update_from(daf: &crate::Daf, number: u16) -> UpdateDafCommand {
    UpdateDafCommand {
        actor: authz::Subject::system(),
        id: daf.id,
        number: DafNumber::try_from(number).unwrap(),
        designation: daf.designation.clone(),
        price_ref: daf.price_ref.clone(),
        supplier: daf.supplier.clone(),
        installer: daf.installer.clone(),
        specs: daf.specs.clone(),
        observations: daf.observations.clone(),
        photo_key: daf.photo_key.clone(),
        attachments: daf.attachments.clone(),
        project_owner: daf.project_owner.clone(),
        project_manager: daf.project_manager.clone(),
        status: daf.status,
        sent_on: daf.sent_on,
        issued_on: daf.issued_on,
        returned_on: daf.returned_on,
        review_note: daf.review_note.clone(),
        expected_updated_at: None,
    }
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_daf_1_the_dossier_is_prefilled_and_not_saved() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let fixture = seed_fixture(&pool).await;

    let (dossier, dafs) = usecase
        .project_dafs(fixture.organization_id, fixture.project_id)
        .await
        .unwrap();

    assert_eq!(dossier.title, "Parc Montgroux");
    assert_eq!(dossier.project_owner.as_deref(), Some("Commune de Cébazat"));
    assert_eq!(dossier.issuer, "Treyve Paysages");
    assert_eq!(dossier.project_manager, None);
    assert_eq!(dossier.updated_at, None);
    assert!(dafs.is_empty());
    let stored: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM daf_dossiers WHERE project_id = $1")
        .bind(fixture.project_id.0)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(stored, 0);

    cleanup(&pool, fixture.organization_id, &[fixture.user_id]).await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_daf_2_numbering() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let fixture = seed_fixture(&pool).await;
    usecase
        .create_daf(create_command(&fixture, "Jeux"))
        .await
        .unwrap();
    usecase
        .create_daf(create_command(&fixture, "Mobilier"))
        .await
        .unwrap();

    let daf = usecase
        .create_daf(create_command(&fixture, "Ganivelle"))
        .await
        .unwrap();

    assert_eq!(daf.number.get(), 3);
    assert_eq!(daf.status, DafStatus::Pending);
    assert_eq!(daf.issued_on, chrono::Utc::now().date_naive());
    let err = usecase.update_daf(update_from(&daf, 1)).await.unwrap_err();
    assert!(matches!(err, CoreError::Conflict(_)), "{err:?}");

    let (dossier, summaries) = usecase
        .project_dafs(fixture.organization_id, fixture.project_id)
        .await
        .unwrap();
    assert!(dossier.updated_at.is_some());
    assert_eq!(
        summaries
            .iter()
            .map(|summary| summary.number.get())
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );

    cleanup(&pool, fixture.organization_id, &[fixture.user_id]).await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_daf_3_a_daf_made_from_a_product_is_a_copy() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let fixture = seed_fixture(&pool).await;
    let supplier_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO suppliers (id, org_id, name, address, phone) VALUES ($1, $2, 'KOMPAN', '1 rue du Parc', '0102030405')",
    )
    .bind(supplier_id)
    .bind(fixture.organization_id.0)
    .execute(&pool)
    .await
    .unwrap();
    let product_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO products (id, org_id, name, unit, unit_price_cents, supplier_id, technical_specs, photo_keys, datasheet_keys) \
         VALUES ($1, $2, 'Ensemble table bois', 'u', 1000, $3, 'Bois lamellé', ARRAY['photo-1'], ARRAY['sheet-1'])",
    )
    .bind(product_id)
    .bind(fixture.organization_id.0)
    .bind(supplier_id)
    .execute(&pool)
    .await
    .unwrap();

    let created = usecase
        .create_daf(CreateDafCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            project_id: fixture.project_id,
            designation: None,
            product_id: Some(ProductId(product_id)),
        })
        .await
        .unwrap();
    sqlx::query("UPDATE suppliers SET address = '9 avenue Neuve' WHERE id = $1")
        .bind(supplier_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE products SET name = 'Autre', photo_keys = ARRAY['photo-9'] WHERE id = $1")
        .bind(product_id)
        .execute(&pool)
        .await
        .unwrap();

    let daf = usecase
        .get_daf(fixture.organization_id, created.id)
        .await
        .unwrap();
    assert_eq!(daf.designation.as_str(), "Ensemble table bois");
    assert_eq!(daf.source_product_id, Some(ProductId(product_id)));
    let supplier = daf.supplier.unwrap();
    assert_eq!(supplier.name, "KOMPAN");
    assert_eq!(supplier.address.as_deref(), Some("1 rue du Parc"));
    assert_eq!(daf.photo_key.as_deref(), Some("photo-1"));
    assert_eq!(daf.specs.as_deref(), Some("Bois lamellé"));
    assert_eq!(daf.attachments.len(), 1);
    assert_eq!(daf.attachments[0].key, "sheet-1");
    assert_eq!(daf.attachments[0].name, "Fiche technique");

    cleanup(&pool, fixture.organization_id, &[fixture.user_id]).await;
}

async fn s3_storage_service() -> MestierFileStorageService {
    let env =
        |name: &str, default: &str| std::env::var(name).unwrap_or_else(|_| default.to_owned());
    let storage = S3FileStorage::from_config(&FileStorageConfig {
        bucket: env("DAF_TEST_S3_BUCKET", "mestier-files"),
        endpoint: env("DAF_TEST_S3_ENDPOINT", "http://localhost:9100"),
        public_endpoint: None,
        region: "us-east-1".to_owned(),
        access_key_id: env("DAF_TEST_S3_ACCESS_KEY", "rustfsadmin"),
        secret_access_key: env("DAF_TEST_S3_SECRET_KEY", "rustfsadmin"),
        force_path_style: true,
        auto_create_bucket: false,
        key_prefix: "uploads".to_owned(),
        max_upload_bytes: 10 * 1024 * 1024,
    });
    storage.ensure_bucket().await.unwrap();
    MestierFileStorageService::new(storage, "uploads")
}

#[tokio::test]
#[ignore = "needs DATABASE_URL and an S3 store (RustFS on DAF_TEST_S3_ENDPOINT, default http://localhost:9100)"]
async fn spec_daf_4_a_placed_photo_outlives_its_library_entry() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let fixture = seed_fixture(&pool).await;
    let storage = s3_storage_service().await;
    let original = storage
        .upload(UploadFileCommand {
            mime_type: "image/jpeg".to_owned(),
            bytes: vec![1, 2, 3, 4],
            folder: Some("library".to_owned()),
        })
        .await
        .unwrap();
    let photo_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO library_photos (id, org_id, file_key, thumbnail_key, mime_type, width, height, size_bytes, content_hash) \
         VALUES ($1, $2, $3, $3, 'image/jpeg', 10, 10, 4, $4)",
    )
    .bind(photo_id)
    .bind(fixture.organization_id.0)
    .bind(&original.key)
    .bind(format!("hash-{photo_id}"))
    .execute(&pool)
    .await
    .unwrap();
    let daf = usecase
        .create_daf(create_command(&fixture, "Ganivelle"))
        .await
        .unwrap();

    let placed = usecase
        .place_daf_photo(
            PlaceDafPhotoCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: daf.id,
                photo_id: LibraryPhotoId(photo_id),
            },
            &storage,
        )
        .await
        .unwrap();
    sqlx::query("DELETE FROM library_photos WHERE id = $1")
        .bind(photo_id)
        .execute(&pool)
        .await
        .unwrap();
    storage.delete(&original.key).await.unwrap();

    let key = placed.photo_key.expect("the DAF has an image");
    assert_ne!(key, original.key);
    let object = storage.get(&key).await.unwrap();
    assert_eq!(object.bytes, vec![1, 2, 3, 4]);
    let reloaded = usecase
        .get_daf(fixture.organization_id, daf.id)
        .await
        .unwrap();
    assert_eq!(reloaded.photo_key.as_deref(), Some(key.as_str()));

    storage.delete(&key).await.unwrap();
    cleanup(&pool, fixture.organization_id, &[fixture.user_id]).await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_daf_7_another_organization_cannot_reach_a_daf() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let mine = seed_fixture(&pool).await;
    let theirs = seed_fixture(&pool).await;
    let daf = usecase
        .create_daf(create_command(&mine, "Ganivelle"))
        .await
        .unwrap();

    let err = usecase
        .get_daf(theirs.organization_id, daf.id)
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::NotFound), "{err:?}");
    let err = usecase
        .daf_document(theirs.organization_id, daf.id)
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::NotFound), "{err:?}");
    let err = usecase
        .project_dafs(theirs.organization_id, mine.project_id)
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::NotFound), "{err:?}");
    let err = usecase
        .daf_dossier_document(theirs.organization_id, mine.project_id)
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::NotFound), "{err:?}");
    let err = usecase
        .create_daf(CreateDafCommand {
            actor: authz::Subject::system(),
            organization_id: theirs.organization_id,
            project_id: mine.project_id,
            designation: Some(DafDesignation::try_from("Intrus".to_owned()).unwrap()),
            product_id: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::NotFound), "{err:?}");
    let err = usecase
        .save_daf_dossier(UpdateDafDossierCommand {
            actor: authz::Subject::system(),
            organization_id: theirs.organization_id,
            project_id: mine.project_id,
            title: "Intrus".to_owned(),
            issuer: "Intrus".to_owned(),
            project_owner: None,
            project_manager: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::NotFound), "{err:?}");
    assert!(matches!(
        usecase
            .get_daf(mine.organization_id, DafId(Uuid::new_v4()))
            .await,
        Err(CoreError::NotFound)
    ));
    assert_eq!(
        usecase
            .get_daf(mine.organization_id, daf.id)
            .await
            .unwrap()
            .designation
            .as_str(),
        "Ganivelle"
    );

    cleanup(&pool, mine.organization_id, &[mine.user_id]).await;
    cleanup(&pool, theirs.organization_id, &[theirs.user_id]).await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_daf_8_a_member_without_planning_manage_reads_but_cannot_write() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let fixture = seed_fixture(&pool).await;
    let daf = usecase
        .create_daf(create_command(&fixture, "Ganivelle"))
        .await
        .unwrap();

    let member_user = seed_user(&pool).await;
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name, joined_at) \
         VALUES ($1, $2, $3, 'Membre', now())",
    )
    .bind(generate_uuid_v7())
    .bind(fixture.organization_id.0)
    .bind(member_user)
    .execute(&pool)
    .await
    .unwrap();
    let actor = crate::application::policy::user_subject(UserId(member_user), Vec::new());

    let err = usecase
        .create_daf(CreateDafCommand {
            actor: actor.clone(),
            ..create_command(&fixture, "Autre")
        })
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::Forbidden { .. }), "{err:?}");
    let mut update = update_from(&daf, 1);
    update.actor = actor.clone();
    let err = usecase.update_daf(update).await.unwrap_err();
    assert!(matches!(err, CoreError::Forbidden { .. }), "{err:?}");
    let err = usecase
        .save_daf_dossier(UpdateDafDossierCommand {
            actor: actor.clone(),
            organization_id: fixture.organization_id,
            project_id: fixture.project_id,
            title: "x".to_owned(),
            issuer: "y".to_owned(),
            project_owner: None,
            project_manager: None,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::Forbidden { .. }), "{err:?}");
    let err = usecase
        .delete_daf(DeleteDafCommand { actor, id: daf.id })
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::Forbidden { .. }), "{err:?}");

    usecase
        .project_dafs(fixture.organization_id, fixture.project_id)
        .await
        .unwrap();
    usecase
        .get_daf(fixture.organization_id, daf.id)
        .await
        .unwrap();
    usecase
        .daf_document(fixture.organization_id, daf.id)
        .await
        .unwrap();
    let document = usecase
        .daf_dossier_document(fixture.organization_id, fixture.project_id)
        .await
        .unwrap();
    assert!(document.summary.is_some());
    assert_eq!(document.sheets.len(), 1);

    cleanup(
        &pool,
        fixture.organization_id,
        &[fixture.user_id, member_user],
    )
    .await;
}
