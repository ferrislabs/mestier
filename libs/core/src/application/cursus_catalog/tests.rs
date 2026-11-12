use authz::Subject;
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    ArchivedFilter, CreateClosureReasonCommand, CreateEstablishmentCommand, CreatePromotionCommand,
    CreateTrainingCommand, Establishment, EstablishmentFilter, Permissions, PromotionFilter,
    SearchPattern, TrainingFilter, UpdateEstablishmentCommand,
    application::{
        MestierUseCase, default_authorizer, policy,
        test_support::{purge, scratch_pool},
    },
    infrastructure::realtime::EventHub,
};

static CATALOG_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_cursus_catalog_test_", &CATALOG_SCRATCH).await
}

struct Person {
    user_id: Uuid,
}

impl Person {
    fn subject(&self) -> Subject {
        policy::user_subject(UserId(self.user_id), Vec::new())
    }
}

struct Tenant {
    organization_id: OrganizationId,
    manager: Person,
    reader: Person,
    outsider: Person,
}

struct World {
    pool: PgPool,
    usecase: MestierUseCase,
    first: Tenant,
    second: Tenant,
}

async fn seed_user(pool: &PgPool, label: &str) -> Uuid {
    let user_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!("{label}-{user_id}@example.com"))
    .bind(format!("{label}-{user_id}"))
    .bind(label)
    .bind(format!("sub-{user_id}"))
    .execute(pool)
    .await
    .unwrap();
    user_id
}

async fn seed_member(pool: &PgPool, organization_id: OrganizationId, user_id: Uuid, role: Uuid) {
    let member_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) \
         VALUES ($1, $2, $3, 'Member')",
    )
    .bind(member_id)
    .bind(organization_id.0)
    .bind(user_id)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(generate_uuid_v7())
        .bind(member_id)
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
}

async fn seed_role(
    pool: &PgPool,
    organization_id: OrganizationId,
    name: &str,
    permissions: Permissions,
) -> Uuid {
    let role_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
    )
    .bind(role_id)
    .bind(organization_id.0)
    .bind(name)
    .bind(permissions.bits())
    .execute(pool)
    .await
    .unwrap();
    role_id
}

async fn seed_tenant(pool: &PgPool) -> Tenant {
    let manager = seed_user(pool, "manager").await;
    let reader = seed_user(pool, "reader").await;
    let outsider = seed_user(pool, "outsider").await;
    let organization_id = OrganizationId(generate_uuid_v7());
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id.0)
        .bind("Centre de formation")
        .bind(format!("org-{}", organization_id.0))
        .bind(manager)
        .execute(pool)
        .await
        .unwrap();

    let manager_role = seed_role(
        pool,
        organization_id,
        "Catalog",
        Permissions::VIEW_CURSUS | Permissions::MANAGE_CURSUS_CATALOG,
    )
    .await;
    let reader_role = seed_role(pool, organization_id, "Reader", Permissions::VIEW_CURSUS).await;
    seed_member(pool, organization_id, manager, manager_role).await;
    seed_member(pool, organization_id, reader, reader_role).await;

    Tenant {
        organization_id,
        manager: Person { user_id: manager },
        reader: Person { user_id: reader },
        outsider: Person { user_id: outsider },
    }
}

impl World {
    async fn new() -> Self {
        let pool = pool().await;
        Self {
            usecase: MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new()),
            first: seed_tenant(&pool).await,
            second: seed_tenant(&pool).await,
            pool,
        }
    }

    async fn establishment(&self, tenant: &Tenant, name: &str) -> Establishment {
        self.usecase
            .create_establishment(CreateEstablishmentCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                name: name.to_owned(),
                uai: None,
                siret: None,
                address: None,
                is_main_site: false,
            })
            .await
            .unwrap()
    }

    async fn cleanup(self) {
        for tenant in [&self.first, &self.second] {
            for statement in [
                "DELETE FROM cursus.promotion WHERE org_id = $1",
                "DELETE FROM cursus.training WHERE org_id = $1",
                "DELETE FROM cursus.establishment WHERE org_id = $1",
                "DELETE FROM cursus.closure_reason WHERE org_id = $1",
                "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
                "DELETE FROM roles WHERE organization_id = $1",
                "DELETE FROM organization_members WHERE organization_id = $1",
                "DELETE FROM organizations WHERE id = $1",
            ] {
                purge(&self.pool, statement, tenant.organization_id.0).await;
            }
            for person in [&tenant.manager, &tenant.reader, &tenant.outsider] {
                purge(
                    &self.pool,
                    "DELETE FROM users WHERE id = $1",
                    person.user_id,
                )
                .await;
            }
        }
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_1_a_manager_builds_the_catalog_chain_and_lists_it() {
    let world = World::new().await;
    let tenant = &world.first;

    let establishment = world.establishment(tenant, "Campus Nord").await;
    let training = world
        .usecase
        .create_training(CreateTrainingCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            establishment_id: Some(establishment.id),
            name: "BTS Aménagements paysagers".to_owned(),
            rncp_code: Some("RNCP35214".to_owned()),
            rncp_label: None,
            level: Some("5".to_owned()),
            degree_type: Some("BTS".to_owned()),
        })
        .await
        .unwrap();
    let promotion = world
        .usecase
        .create_promotion(CreatePromotionCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            training_id: training.id,
            label: "2026-2028".to_owned(),
            start_year: 2026,
            end_year: 2028,
        })
        .await
        .unwrap();

    let establishments = world
        .usecase
        .list_establishments(tenant.organization_id, EstablishmentFilter::default())
        .await
        .unwrap();
    let (trainings, training_total) = world
        .usecase
        .list_trainings(
            tenant.organization_id,
            TrainingFilter {
                establishment_id: Some(establishment.id),
                ..TrainingFilter::default()
            },
            10,
            0,
        )
        .await
        .unwrap();
    let (promotions, promotion_total) = world
        .usecase
        .list_promotions(
            tenant.organization_id,
            PromotionFilter {
                training_id: Some(training.id),
                ..PromotionFilter::default()
            },
            10,
            0,
        )
        .await
        .unwrap();

    assert_eq!(establishments, vec![establishment]);
    assert_eq!((trainings, training_total), (vec![training], 1));
    assert_eq!((promotions, promotion_total), (vec![promotion], 1));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_2_without_the_manage_bit_every_catalog_write_is_forbidden() {
    let world = World::new().await;
    let tenant = &world.first;
    let establishment = world.establishment(tenant, "Campus").await;

    for person in [&tenant.reader, &tenant.outsider] {
        let created = world
            .usecase
            .create_establishment(CreateEstablishmentCommand {
                actor: person.subject(),
                organization_id: tenant.organization_id,
                name: "Intrus".to_owned(),
                uai: None,
                siret: None,
                address: None,
                is_main_site: false,
            })
            .await;
        assert!(matches!(created, Err(CoreError::Forbidden { .. })));

        let archived = world
            .usecase
            .archive_establishment(person.subject(), tenant.organization_id, establishment.id)
            .await;
        assert!(matches!(archived, Err(CoreError::Forbidden { .. })));

        let reason = world
            .usecase
            .create_closure_reason(CreateClosureReasonCommand {
                actor: person.subject(),
                organization_id: tenant.organization_id,
                label: "Rupture".to_owned(),
            })
            .await;
        assert!(matches!(reason, Err(CoreError::Forbidden { .. })));
    }

    let untouched = world
        .usecase
        .get_establishment(tenant.organization_id, establishment.id)
        .await
        .unwrap();
    assert_eq!(untouched.archived_at, None);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_12_one_organization_never_sees_or_touches_another_ones_catalog() {
    let world = World::new().await;
    let establishment = world.establishment(&world.first, "Campus Nord").await;

    let read = world
        .usecase
        .get_establishment(world.second.organization_id, establishment.id)
        .await;
    assert!(matches!(read, Err(CoreError::NotFound)));

    let listed = world
        .usecase
        .list_establishments(
            world.second.organization_id,
            EstablishmentFilter {
                archived: ArchivedFilter::All,
                search: None,
            },
        )
        .await
        .unwrap();
    assert!(listed.is_empty());

    let updated = world
        .usecase
        .update_establishment(UpdateEstablishmentCommand {
            actor: world.second.manager.subject(),
            organization_id: world.second.organization_id,
            id: establishment.id,
            name: "Détourné".to_owned(),
            uai: None,
            siret: None,
            address: None,
            is_main_site: true,
        })
        .await;
    assert!(matches!(updated, Err(CoreError::NotFound)));

    let archived = world
        .usecase
        .archive_establishment(
            world.second.manager.subject(),
            world.second.organization_id,
            establishment.id,
        )
        .await;
    assert!(matches!(archived, Err(CoreError::NotFound)));

    let training = world
        .usecase
        .create_training(CreateTrainingCommand {
            actor: world.second.manager.subject(),
            organization_id: world.second.organization_id,
            establishment_id: Some(establishment.id),
            name: "BTS".to_owned(),
            rncp_code: None,
            rncp_label: None,
            level: None,
            degree_type: None,
        })
        .await;
    assert!(matches!(training, Err(CoreError::NotFound)));

    let foreign_manager = world
        .usecase
        .create_establishment(CreateEstablishmentCommand {
            actor: world.second.manager.subject(),
            organization_id: world.first.organization_id,
            name: "Intrus".to_owned(),
            uai: None,
            siret: None,
            address: None,
            is_main_site: false,
        })
        .await;
    assert!(matches!(foreign_manager, Err(CoreError::Forbidden { .. })));

    let intact = world
        .usecase
        .get_establishment(world.first.organization_id, establishment.id)
        .await
        .unwrap();
    assert_eq!(intact.name, "Campus Nord");
    assert_eq!(intact.archived_at, None);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_4_an_archived_closure_reason_is_hidden_by_default_but_stays_readable() {
    let world = World::new().await;
    let tenant = &world.first;
    let reason = world
        .usecase
        .create_closure_reason(CreateClosureReasonCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            label: "Rupture du contrat".to_owned(),
        })
        .await
        .unwrap();

    let archived = world
        .usecase
        .archive_closure_reason(tenant.manager.subject(), tenant.organization_id, reason.id)
        .await
        .unwrap();
    let archived_again = world
        .usecase
        .archive_closure_reason(tenant.manager.subject(), tenant.organization_id, reason.id)
        .await
        .unwrap();

    let by_default = world
        .usecase
        .list_closure_reasons(tenant.organization_id, false)
        .await
        .unwrap();
    let including_archived = world
        .usecase
        .list_closure_reasons(tenant.organization_id, true)
        .await
        .unwrap();
    let read = world
        .usecase
        .get_closure_reason(tenant.organization_id, reason.id)
        .await
        .unwrap();

    assert!(archived.archived_at.is_some());
    assert_eq!(archived_again.archived_at, archived.archived_at);
    assert!(by_default.is_empty());
    assert_eq!(including_archived, vec![read.clone()]);
    assert_eq!(read.archived_at, archived.archived_at);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_refuses_a_promotion_ending_before_it_starts_and_a_cross_organization_training()
 {
    let world = World::new().await;
    let establishment = world.establishment(&world.first, "Campus").await;

    let cross_org = sqlx::query(
        "INSERT INTO cursus.training (id, org_id, establishment_id, name) VALUES ($1, $2, $3, 'BTS')",
    )
    .bind(generate_uuid_v7())
    .bind(world.second.organization_id.0)
    .bind(establishment.id.0)
    .execute(&world.pool)
    .await;
    assert!(cross_org.is_err());

    let training = world
        .usecase
        .create_training(CreateTrainingCommand {
            actor: world.first.manager.subject(),
            organization_id: world.first.organization_id,
            establishment_id: Some(establishment.id),
            name: "BTS".to_owned(),
            rncp_code: None,
            rncp_label: None,
            level: None,
            degree_type: None,
        })
        .await
        .unwrap();
    let backwards = sqlx::query(
        "INSERT INTO cursus.promotion (id, org_id, training_id, label, start_year, end_year) \
         VALUES ($1, $2, $3, 'X', 2028, 2026)",
    )
    .bind(generate_uuid_v7())
    .bind(world.first.organization_id.0)
    .bind(training.id.0)
    .execute(&world.pool)
    .await;
    assert!(backwards.is_err());

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_training_is_created_without_an_establishment() {
    let world = World::new().await;
    let tenant = &world.first;

    let training = world
        .usecase
        .create_training(CreateTrainingCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            establishment_id: None,
            name: "CAP Menuiserie".to_owned(),
            rncp_code: None,
            rncp_label: None,
            level: None,
            degree_type: None,
        })
        .await
        .unwrap();
    assert_eq!(training.establishment_id, None);

    let found = world
        .usecase
        .get_training(tenant.organization_id, training.id)
        .await
        .unwrap();
    assert_eq!(found, training);
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn catalog_lists_search_and_filter_by_archive_state() {
    let world = World::new().await;
    let tenant = &world.first;
    let training = |name: &str, rncp_code: &str| CreateTrainingCommand {
        actor: tenant.manager.subject(),
        organization_id: tenant.organization_id,
        establishment_id: None,
        name: name.to_owned(),
        rncp_code: Some(rncp_code.to_owned()),
        rncp_label: None,
        level: None,
        degree_type: None,
    };
    let bts = world
        .usecase
        .create_training(training("BTS Aménagements paysagers", "RNCP35214"))
        .await
        .unwrap();
    let cap = world
        .usecase
        .create_training(training("CAP Jardinier", "RNCP_99"))
        .await
        .unwrap();
    world
        .usecase
        .archive_training(tenant.manager.subject(), tenant.organization_id, cap.id)
        .await
        .unwrap();
    let promotion = world
        .usecase
        .create_promotion(CreatePromotionCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            training_id: bts.id,
            label: "Promo 2026-2028".to_owned(),
            start_year: 2026,
            end_year: 2028,
        })
        .await
        .unwrap();
    let campus = world.establishment(tenant, "Campus Nord").await;
    let annex = world.establishment(tenant, "Annexe Sud").await;
    world
        .usecase
        .archive_establishment(tenant.manager.subject(), tenant.organization_id, annex.id)
        .await
        .unwrap();

    let trainings = |archived: ArchivedFilter, q: &'static str| {
        world.usecase.list_trainings(
            tenant.organization_id,
            TrainingFilter {
                establishment_id: None,
                archived,
                search: SearchPattern::parse(Some(q)),
            },
            10,
            0,
        )
    };
    let (by_code, by_code_total) = trainings(ArchivedFilter::Active, "rncp352").await.unwrap();
    assert_eq!((by_code, by_code_total), (vec![bts.clone()], 1));
    assert_eq!(
        trainings(ArchivedFilter::Active, "rncp").await.unwrap().1,
        1
    );
    assert_eq!(trainings(ArchivedFilter::All, "rncp").await.unwrap().1, 2);
    let (archived, archived_total) = trainings(ArchivedFilter::Archived, "").await.unwrap();
    assert_eq!(archived_total, 1);
    assert_eq!(archived[0].id, cap.id);
    assert_eq!(trainings(ArchivedFilter::All, "p_9").await.unwrap().1, 1);
    assert_eq!(trainings(ArchivedFilter::All, "_").await.unwrap().1, 1);

    let (promotions, promotion_total) = world
        .usecase
        .list_promotions(
            tenant.organization_id,
            PromotionFilter {
                search: SearchPattern::parse(Some("PROMO 2026")),
                ..PromotionFilter::default()
            },
            10,
            0,
        )
        .await
        .unwrap();
    assert_eq!((promotions, promotion_total), (vec![promotion], 1));

    let establishments = |archived: ArchivedFilter, q: &'static str| {
        world.usecase.list_establishments(
            tenant.organization_id,
            EstablishmentFilter {
                archived,
                search: SearchPattern::parse(Some(q)),
            },
        )
    };
    assert_eq!(
        establishments(ArchivedFilter::Active, "").await.unwrap(),
        vec![campus.clone()]
    );
    assert_eq!(
        establishments(ArchivedFilter::Archived, "")
            .await
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        establishments(ArchivedFilter::All, "nord").await.unwrap(),
        vec![campus]
    );

    world.cleanup().await;
}
