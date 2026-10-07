use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use serde_json::json;
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    CreateDoeCommand, DoeId, DoeModelId, DoeSectionContent, DraftSupplyFromProductCommand,
    ProductId, ProjectId, UpdateDoeCommand,
    application::{
        MestierUseCase, default_authorizer,
        test_support::{purge, scratch_pool},
    },
    domain::doe::{DoeSection, PartyRole},
    infrastructure::realtime::EventHub,
};

static DOE_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_doe_test_", &DOE_SCRATCH).await
}

fn usecase(pool: &PgPool) -> MestierUseCase {
    MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new())
}

struct Fixture {
    user_id: Uuid,
    organization_id: OrganizationId,
    project_id: ProjectId,
    model_id: DoeModelId,
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

async fn seed_organization(pool: &PgPool, owner_id: Uuid, name: &str) -> OrganizationId {
    let id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organizations (id, name, slug, owner_id, address_line1, address_postal_code, address_city) \
         VALUES ($1, $2, $3, $4, '3 chemin des Vignes', '03120', 'Lapalisse')",
    )
    .bind(id)
    .bind(name)
    .bind(format!("org-{id}"))
    .bind(owner_id)
    .execute(pool)
    .await
    .unwrap();
    OrganizationId(id)
}

async fn seed_project(
    pool: &PgPool,
    organization_id: OrganizationId,
    customer_id: Option<Uuid>,
    name: &str,
) -> ProjectId {
    let id = generate_uuid_v7();
    sqlx::query("INSERT INTO projects (id, org_id, name, customer_id) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(organization_id.0)
        .bind(name)
        .bind(customer_id)
        .execute(pool)
        .await
        .unwrap();
    ProjectId(id)
}

async fn seed_landscaping_model(pool: &PgPool, organization_id: OrganizationId) -> DoeModelId {
    let id = generate_uuid_v7();
    let section = |title: &str, enabled: bool, kind: serde_json::Value| json!({"title": title, "enabled": enabled, "kind": kind});
    let sections = json!([
        section(
            "Végétaux",
            true,
            json!({"kind": "supplies", "maker_label": "nursery"})
        ),
        section(
            "Matériaux",
            true,
            json!({"kind": "supplies", "maker_label": "manufacturer"})
        ),
        section(
            "Mobilier",
            true,
            json!({"kind": "supplies", "maker_label": "manufacturer"})
        ),
        section("Photos", true, json!({"kind": "photos", "per_page": "two"})),
        section("Plans", true, json!({"kind": "plans"})),
        section("Notes", true, json!({"kind": "blank_pages", "count": 1})),
        section(
            "Jeux",
            false,
            json!({"kind": "supplies", "maker_label": "manufacturer"})
        ),
    ]);
    sqlx::query("INSERT INTO doe_models (id, org_id, name, sections) VALUES ($1, $2, $3, $4)")
        .bind(id)
        .bind(organization_id.0)
        .bind("Paysagiste")
        .bind(sections)
        .execute(pool)
        .await
        .unwrap();
    DoeModelId(id)
}

async fn seed_fixture(pool: &PgPool) -> Fixture {
    let user_id = seed_user(pool).await;
    let organization_id = seed_organization(pool, user_id, "Treyve Paysages").await;

    let customer_id = generate_uuid_v7();
    sqlx::query("INSERT INTO customers (id, org_id, name) VALUES ($1, $2, $3)")
        .bind(customer_id)
        .bind(organization_id.0)
        .bind("Commune de Lapalisse")
        .execute(pool)
        .await
        .unwrap();

    let project_id =
        seed_project(pool, organization_id, Some(customer_id), "Avenue du 8 mai").await;
    let model_id = seed_landscaping_model(pool, organization_id).await;

    Fixture {
        user_id,
        organization_id,
        project_id,
        model_id,
    }
}

async fn cleanup(pool: &PgPool, organization_id: OrganizationId, user_ids: &[Uuid]) {
    for statement in [
        "DELETE FROM does WHERE org_id = $1",
        "DELETE FROM projects WHERE org_id = $1",
        "DELETE FROM doe_models WHERE org_id = $1",
        "DELETE FROM products WHERE org_id = $1",
        "DELETE FROM suppliers WHERE org_id = $1",
        "DELETE FROM customer_contexts WHERE customer_id IN (SELECT id FROM customers WHERE org_id = $1)",
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

fn create_command(fixture: &Fixture) -> CreateDoeCommand {
    CreateDoeCommand {
        actor: authz::Subject::system(),
        organization_id: fixture.organization_id,
        project_id: fixture.project_id,
        model_id: fixture.model_id,
    }
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_doe_1_create_a_doe_from_a_model() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let fixture = seed_fixture(&pool).await;

    let doe = usecase.create_doe(create_command(&fixture)).await.unwrap();

    assert_eq!(doe.title, "Avenue du 8 mai");
    assert_eq!(doe.revision.as_str(), "A");
    assert_eq!(doe.sections.len(), 7);
    let titles: Vec<(&str, bool)> = doe
        .sections
        .iter()
        .map(|section| (section.title.as_str(), section.enabled))
        .collect();
    assert_eq!(
        titles,
        vec![
            ("Végétaux", true),
            ("Matériaux", true),
            ("Mobilier", true),
            ("Photos", true),
            ("Plans", true),
            ("Notes", true),
            ("Jeux", false),
        ]
    );
    assert!(matches!(
        &doe.sections[3].content,
        DoeSectionContent::Photos { photos, .. } if photos.is_empty()
    ));
    let owner = doe
        .parties
        .iter()
        .find(|party| party.role == PartyRole::ProjectOwner)
        .unwrap();
    assert_eq!(owner.name, "Commune de Lapalisse");
    let contractor = doe
        .parties
        .iter()
        .find(|party| party.role == PartyRole::Contractor)
        .unwrap();
    assert_eq!(contractor.name, "Treyve Paysages");
    assert_eq!(
        contractor.address.as_deref(),
        Some("3 chemin des Vignes\n03120 Lapalisse")
    );

    let reloaded = usecase.get_project_doe(fixture.project_id).await.unwrap();
    assert_eq!(reloaded.id, doe.id);
    assert_eq!(reloaded.sections, doe.sections);
    assert_eq!(reloaded.parties, doe.parties);
    assert_eq!(reloaded.issued_on, doe.issued_on);

    cleanup(&pool, fixture.organization_id, &[fixture.user_id]).await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_doe_2_one_doe_per_project() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let fixture = seed_fixture(&pool).await;

    usecase.create_doe(create_command(&fixture)).await.unwrap();
    let err = usecase
        .create_doe(create_command(&fixture))
        .await
        .unwrap_err();

    assert!(matches!(err, CoreError::Conflict(_)), "{err:?}");

    cleanup(&pool, fixture.organization_id, &[fixture.user_id]).await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL; the draft reads suppliers through the W2 SupplierRepository adapter"]
async fn spec_doe_3_a_supply_is_a_copy() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let fixture = seed_fixture(&pool).await;
    let doe = usecase.create_doe(create_command(&fixture)).await.unwrap();

    let supplier_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO suppliers (id, org_id, name, address, phone) VALUES ($1, $2, 'PLAS ECO', '1 rue du Parc', '0102030405')",
    )
    .bind(supplier_id)
    .bind(fixture.organization_id.0)
    .execute(&pool)
    .await
    .unwrap();
    let product_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO products (id, org_id, name, unit, unit_price_cents, supplier_id, manufacturer, technical_specs, datasheet_keys) \
         VALUES ($1, $2, 'Banc Garden', 'u', 1000, $3, 'PLAS ECO', 'Plastique recyclé', ARRAY['sheet-1', 'sheet-2'])",
    )
    .bind(product_id)
    .bind(fixture.organization_id.0)
    .bind(supplier_id)
    .execute(&pool)
    .await
    .unwrap();

    let draft = usecase
        .draft_doe_supply(DraftSupplyFromProductCommand {
            actor: authz::Subject::system(),
            doe_id: doe.id,
            product_id: ProductId(product_id),
        })
        .await
        .unwrap();
    assert_eq!(draft.designation, "Banc Garden");
    assert_eq!(draft.datasheet_keys, vec!["sheet-1", "sheet-2"]);

    let mut sections: Vec<DoeSection> = doe.sections.clone();
    let furniture = sections
        .iter_mut()
        .find(|section| section.title == "Mobilier")
        .unwrap();
    let DoeSectionContent::Supplies { supplies, .. } = &mut furniture.content else {
        panic!("expected supplies");
    };
    supplies.push(draft);
    usecase
        .update_doe(UpdateDoeCommand {
            actor: authz::Subject::system(),
            id: doe.id,
            title: doe.title.clone(),
            place: None,
            lot: None,
            issued_on: doe.issued_on,
            author: None,
            revision: doe.revision.clone(),
            parties: doe.parties.clone(),
            sections,
        })
        .await
        .unwrap();

    sqlx::query("UPDATE products SET name = 'Banc Garden V2' WHERE id = $1")
        .bind(product_id)
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("UPDATE suppliers SET address = '9 avenue Neuve' WHERE id = $1")
        .bind(supplier_id)
        .execute(&pool)
        .await
        .unwrap();

    let reloaded = usecase.get_doe(doe.id).await.unwrap();
    let furniture = reloaded
        .sections
        .iter()
        .find(|section| section.title == "Mobilier")
        .unwrap();
    let DoeSectionContent::Supplies { supplies, .. } = &furniture.content else {
        panic!("expected supplies");
    };
    assert_eq!(supplies.len(), 1);
    assert_eq!(supplies[0].designation, "Banc Garden");
    assert_eq!(
        supplies[0].supplier.as_ref().unwrap().address.as_deref(),
        Some("1 rue du Parc")
    );
    assert_eq!(supplies[0].source_product_id, Some(ProductId(product_id)));

    cleanup(&pool, fixture.organization_id, &[fixture.user_id]).await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_doe_6_another_organizations_project_and_model_are_not_found() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let mine = seed_fixture(&pool).await;
    let theirs = seed_fixture(&pool).await;

    let err = usecase
        .create_doe(CreateDoeCommand {
            actor: authz::Subject::system(),
            organization_id: mine.organization_id,
            project_id: theirs.project_id,
            model_id: mine.model_id,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::NotFound), "{err:?}");

    let err = usecase
        .create_doe(CreateDoeCommand {
            actor: authz::Subject::system(),
            organization_id: mine.organization_id,
            project_id: mine.project_id,
            model_id: theirs.model_id,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::NotFound), "{err:?}");

    let doe = usecase.create_doe(create_command(&theirs)).await.unwrap();
    assert_eq!(doe.organization_id, theirs.organization_id);
    assert_ne!(doe.organization_id, mine.organization_id);

    let forged = sqlx::query(
        "INSERT INTO does (id, org_id, project_id, title, issued_on, revision) \
         VALUES ($1, $2, $3, 'x', CURRENT_DATE, 'A')",
    )
    .bind(generate_uuid_v7())
    .bind(mine.organization_id.0)
    .bind(theirs.project_id.0)
    .execute(&pool)
    .await;
    assert!(forged.is_err());

    assert!(matches!(
        usecase.get_doe(DoeId(Uuid::new_v4())).await,
        Err(CoreError::NotFound)
    ));

    cleanup(&pool, mine.organization_id, &[mine.user_id]).await;
    cleanup(&pool, theirs.organization_id, &[theirs.user_id]).await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn spec_doe_7_a_member_without_planning_manage_cannot_write() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let fixture = seed_fixture(&pool).await;
    let doe = usecase.create_doe(create_command(&fixture)).await.unwrap();

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

    let update = UpdateDoeCommand {
        actor: actor.clone(),
        id: doe.id,
        title: "Autre titre".to_owned(),
        place: None,
        lot: None,
        issued_on: doe.issued_on,
        author: None,
        revision: doe.revision.clone(),
        parties: doe.parties.clone(),
        sections: doe.sections.clone(),
    };
    let err = usecase.update_doe(update).await.unwrap_err();
    assert!(matches!(err, CoreError::Forbidden { .. }), "{err:?}");

    let err = usecase
        .create_doe(CreateDoeCommand {
            actor: actor.clone(),
            organization_id: fixture.organization_id,
            project_id: fixture.project_id,
            model_id: fixture.model_id,
        })
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::Forbidden { .. }), "{err:?}");

    let err = usecase
        .draft_doe_supply(DraftSupplyFromProductCommand {
            actor,
            doe_id: doe.id,
            product_id: ProductId(Uuid::new_v4()),
        })
        .await
        .unwrap_err();
    assert!(matches!(err, CoreError::Forbidden { .. }), "{err:?}");

    assert_eq!(
        usecase.get_doe(doe.id).await.unwrap().title,
        "Avenue du 8 mai"
    );
    assert!(usecase.doe_document(doe.id).await.is_ok());

    cleanup(
        &pool,
        fixture.organization_id,
        &[fixture.user_id, member_user],
    )
    .await;
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn update_replaces_the_content_and_keeps_identity() {
    let pool = pool().await;
    let usecase = usecase(&pool);
    let fixture = seed_fixture(&pool).await;
    let doe = usecase.create_doe(create_command(&fixture)).await.unwrap();

    let updated = usecase
        .update_doe(UpdateDoeCommand {
            actor: authz::Subject::system(),
            id: doe.id,
            title: "DOE Avenue".to_owned(),
            place: Some("Lapalisse".to_owned()),
            lot: Some("Lot 3".to_owned()),
            issued_on: doe.issued_on,
            author: Some("Erwan".to_owned()),
            revision: crate::Revision::try_from("B".to_owned()).unwrap(),
            parties: doe.parties.clone(),
            sections: doe.sections[..2].to_vec(),
        })
        .await
        .unwrap();

    assert_eq!(updated.id, doe.id);
    assert_eq!(updated.project_id, doe.project_id);
    assert_eq!(updated.organization_id, doe.organization_id);
    assert_eq!(updated.created_at, doe.created_at);
    assert!(updated.updated_at >= doe.updated_at);
    let reloaded = usecase.get_doe(doe.id).await.unwrap();
    assert_eq!(reloaded.title, "DOE Avenue");
    assert_eq!(reloaded.revision.as_str(), "B");
    assert_eq!(reloaded.sections.len(), 2);

    let document = usecase.doe_document(doe.id).await.unwrap();
    assert_eq!(document.sections.len(), 2);
    assert_eq!(document.revision, "B");

    cleanup(&pool, fixture.organization_id, &[fixture.user_id]).await;
}
