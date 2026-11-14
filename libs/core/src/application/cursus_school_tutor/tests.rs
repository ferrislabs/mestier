use authz::Subject;
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    ApprenticeId, ApprenticeInput, ArchivedFilter, CreateEstablishmentCommand,
    CreateParticipantCommand, CreatePromotionCommand, CreateSchoolTutorCommand,
    CreateTrainingCommand, CreateTriadCommand, MemberId, Permissions, Promotion, SchoolTutorFilter,
    SchoolTutorView, SearchPattern, UpdateSchoolTutorCommand,
    application::{
        MestierUseCase, default_authorizer, policy,
        test_support::{purge, scratch_pool},
    },
    infrastructure::realtime::EventHub,
};

static SCHOOL_TUTOR_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_cursus_school_tutor_test_", &SCHOOL_TUTOR_SCRATCH).await
}

struct Person {
    user_id: Uuid,
    member_id: MemberId,
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
    staff: Vec<Person>,
}

struct World {
    pool: PgPool,
    usecase: MestierUseCase,
    first: Tenant,
    second: Tenant,
}

async fn seed_user(pool: &PgPool, last_name: &str) -> Uuid {
    let user_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO users (id, email, username, display_name, sub) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(user_id)
    .bind(format!(
        "{}-{user_id}@example.com",
        last_name.to_lowercase()
    ))
    .bind(format!("{last_name}-{user_id}"))
    .bind(last_name)
    .bind(format!("sub-{user_id}"))
    .execute(pool)
    .await
    .unwrap();
    user_id
}

async fn seed_member(
    pool: &PgPool,
    organization_id: OrganizationId,
    user_id: Uuid,
    (last_name, first_name): (&str, &str),
    permissions: Permissions,
) -> Person {
    let role_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
    )
    .bind(role_id)
    .bind(organization_id.0)
    .bind(format!("role-{role_id}"))
    .bind(permissions.bits())
    .execute(pool)
    .await
    .unwrap();
    let member_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name, first_name) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(member_id)
    .bind(organization_id.0)
    .bind(user_id)
    .bind(last_name)
    .bind(first_name)
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
        .bind(generate_uuid_v7())
        .bind(member_id)
        .bind(role_id)
        .execute(pool)
        .await
        .unwrap();

    Person {
        user_id,
        member_id: MemberId(member_id),
    }
}

async fn seed_tenant(pool: &PgPool) -> Tenant {
    let manager_user = seed_user(pool, "Manager").await;
    let organization_id = OrganizationId(generate_uuid_v7());
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id.0)
        .bind("Centre de formation")
        .bind(format!("org-{}", organization_id.0))
        .bind(manager_user)
        .execute(pool)
        .await
        .unwrap();
    let manager = seed_member(
        pool,
        organization_id,
        manager_user,
        ("Manager", "Marie"),
        Permissions::VIEW_CURSUS
            | Permissions::MANAGE_CURSUS_CATALOG
            | Permissions::MANAGE_CURSUS_TRIADS
            | Permissions::MANAGE_CURSUS_PARTICIPANTS,
    )
    .await;
    let reader_user = seed_user(pool, "Lecteur").await;
    let reader = seed_member(
        pool,
        organization_id,
        reader_user,
        ("Lecteur", "Luc"),
        Permissions::VIEW_CURSUS | Permissions::MANAGE_CURSUS_TRIADS,
    )
    .await;
    let mut staff = Vec::new();
    for names in [("Bonnal", "Baptiste"), ("Durand", "Claire")] {
        let user_id = seed_user(pool, names.0).await;
        staff.push(seed_member(pool, organization_id, user_id, names, Permissions::NONE).await);
    }

    Tenant {
        organization_id,
        manager,
        reader,
        staff,
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

    async fn declare(
        &self,
        tenant: &Tenant,
        member_id: MemberId,
        job_title: &str,
    ) -> SchoolTutorView {
        self.usecase
            .create_school_tutor(CreateSchoolTutorCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                member_id,
                job_title: Some(job_title.to_owned()),
            })
            .await
            .unwrap()
    }

    async fn promotion(&self, tenant: &Tenant) -> Promotion {
        let actor = tenant.manager.subject();
        let establishment = self
            .usecase
            .create_establishment(CreateEstablishmentCommand {
                actor: actor.clone(),
                organization_id: tenant.organization_id,
                name: "Campus".to_owned(),
                uai: None,
                siret: None,
                address: None,
                is_main_site: true,
            })
            .await
            .unwrap();
        let training = self
            .usecase
            .create_training(CreateTrainingCommand {
                actor: actor.clone(),
                organization_id: tenant.organization_id,
                establishment_id: Some(establishment.id),
                name: "BTS Aménagements paysagers".to_owned(),
                rncp_code: None,
                rncp_label: None,
                level: None,
                degree_type: None,
            })
            .await
            .unwrap();
        self.usecase
            .create_promotion(CreatePromotionCommand {
                actor,
                organization_id: tenant.organization_id,
                training_id: training.id,
                label: "BTS 2026".to_owned(),
                start_year: 2026,
                end_year: 2028,
            })
            .await
            .unwrap()
    }

    async fn apprentice(&self, tenant: &Tenant, email: &str) -> ApprenticeId {
        self.usecase
            .create_participant(CreateParticipantCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                last_name: "Martin".to_owned(),
                first_name: Some("Léa".to_owned()),
                email: email.to_owned(),
                phone: None,
                apprentice: Some(ApprenticeInput {
                    birth_date: None,
                    ine_code: None,
                    address: None,
                }),
                company_tutor: None,
            })
            .await
            .unwrap()
            .apprentice
            .unwrap()
            .id
    }

    async fn cleanup(self) {
        for tenant in [&self.first, &self.second] {
            for statement in [
                "DELETE FROM cursus.triad WHERE org_id = $1",
                "DELETE FROM cursus.school_tutor WHERE org_id = $1",
                "DELETE FROM cursus.apprentice WHERE org_id = $1",
                "DELETE FROM cursus.participant WHERE org_id = $1",
                "DELETE FROM cursus.promotion WHERE org_id = $1",
                "DELETE FROM cursus.training WHERE org_id = $1",
                "DELETE FROM cursus.establishment WHERE org_id = $1",
                "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
                "DELETE FROM roles WHERE organization_id = $1",
                "DELETE FROM organization_members WHERE organization_id = $1",
                "DELETE FROM organizations WHERE id = $1",
            ] {
                purge(&self.pool, statement, tenant.organization_id.0).await;
            }
            for person in [&tenant.manager, &tenant.reader]
                .into_iter()
                .chain(tenant.staff.iter())
            {
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

fn filter(archived: ArchivedFilter, q: Option<&str>) -> SchoolTutorFilter {
    SchoolTutorFilter {
        archived,
        search: SearchPattern::parse(q),
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_is_declared_school_tutor_once_and_a_second_declaration_is_a_conflict() {
    let world = World::new().await;
    let tenant = &world.first;
    let member = tenant.staff[0].member_id;

    let created = world.declare(tenant, member, " Formateur référent ").await;
    let again = world
        .usecase
        .create_school_tutor(CreateSchoolTutorCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            member_id: member,
            job_title: None,
        })
        .await;
    let fetched = world
        .usecase
        .get_school_tutor(tenant.organization_id, created.tutor.id)
        .await
        .unwrap();

    assert_eq!(created.tutor.member_id, member);
    assert_eq!(
        created.tutor.job_title.as_deref(),
        Some("Formateur référent")
    );
    assert_eq!(created.member_name, "Bonnal Baptiste");
    assert!(
        created
            .member_email
            .as_deref()
            .is_some_and(|email| email.starts_with("bonnal-"))
    );
    assert_eq!(created.open_triad_count, 0);
    assert!(matches!(again, Err(CoreError::Conflict(_))));
    assert_eq!(fetched, created);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn only_a_member_of_the_organization_can_be_declared() {
    let world = World::new().await;

    let foreign = world
        .usecase
        .create_school_tutor(CreateSchoolTutorCommand {
            actor: world.first.manager.subject(),
            organization_id: world.first.organization_id,
            member_id: world.second.staff[0].member_id,
            job_title: None,
        })
        .await;

    assert!(matches!(foreign, Err(CoreError::NotFound)));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn writes_need_the_participants_bit() {
    let world = World::new().await;
    let tenant = &world.first;
    let declared = world
        .declare(tenant, tenant.staff[0].member_id, "Formateur")
        .await;

    let created = world
        .usecase
        .create_school_tutor(CreateSchoolTutorCommand {
            actor: tenant.reader.subject(),
            organization_id: tenant.organization_id,
            member_id: tenant.staff[1].member_id,
            job_title: None,
        })
        .await;
    let updated = world
        .usecase
        .update_school_tutor(UpdateSchoolTutorCommand {
            actor: tenant.reader.subject(),
            organization_id: tenant.organization_id,
            id: declared.tutor.id,
            job_title: None,
        })
        .await;
    let archived = world
        .usecase
        .archive_school_tutor(
            tenant.reader.subject(),
            tenant.organization_id,
            declared.tutor.id,
        )
        .await;

    assert!(matches!(created, Err(CoreError::Forbidden { .. })));
    assert!(matches!(updated, Err(CoreError::Forbidden { .. })));
    assert!(matches!(archived, Err(CoreError::Forbidden { .. })));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_list_searches_name_or_email_filters_archives_and_counts_open_triads() {
    let world = World::new().await;
    let tenant = &world.first;
    let bonnal = world
        .declare(tenant, tenant.staff[0].member_id, "Formateur")
        .await;
    let durand = world
        .declare(tenant, tenant.staff[1].member_id, "Directrice")
        .await;
    let promotion = world.promotion(tenant).await;
    for email in ["a@example.com", "b@example.com", "c@example.com"] {
        let apprentice_id = world.apprentice(tenant, email).await;
        world
            .usecase
            .create_triad(CreateTriadCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                promotion_id: promotion.id,
                apprentice_id,
                pedagogic_tutor_member_id: Some(bonnal.tutor.member_id),
            })
            .await
            .unwrap();
    }
    sqlx::query(
        "UPDATE cursus.triad SET closed_at = DATE '2027-06-30' \
         WHERE id = (SELECT id FROM cursus.triad WHERE org_id = $1 ORDER BY id LIMIT 1)",
    )
    .bind(tenant.organization_id.0)
    .execute(&world.pool)
    .await
    .unwrap();
    world
        .usecase
        .archive_school_tutor(
            tenant.manager.subject(),
            tenant.organization_id,
            durand.tutor.id,
        )
        .await
        .unwrap();
    let list = |archived, q| {
        world
            .usecase
            .list_school_tutors(tenant.organization_id, filter(archived, q), 10, 0)
    };

    let (active, active_total) = list(ArchivedFilter::Active, None).await.unwrap();
    let (archived, archived_total) = list(ArchivedFilter::Archived, None).await.unwrap();
    let (all, all_total) = list(ArchivedFilter::All, None).await.unwrap();
    let (by_name, by_name_total) = list(ArchivedFilter::All, Some("baptiste bon"))
        .await
        .unwrap();
    let (by_email, _) = list(ArchivedFilter::All, Some("DURAND-")).await.unwrap();
    let (none, none_total) = list(ArchivedFilter::Active, Some("durand")).await.unwrap();
    let (foreign, foreign_total) = world
        .usecase
        .list_school_tutors(
            world.second.organization_id,
            filter(ArchivedFilter::All, None),
            10,
            0,
        )
        .await
        .unwrap();

    assert_eq!(active_total, 1);
    assert_eq!(active[0].tutor.id, bonnal.tutor.id);
    assert_eq!(active[0].open_triad_count, 2);
    assert_eq!(archived_total, 1);
    assert_eq!(archived[0].tutor.id, durand.tutor.id);
    assert!(archived[0].tutor.archived_at.is_some());
    assert_eq!(archived[0].open_triad_count, 0);
    assert_eq!(all_total, 2);
    assert_eq!(all[0].member_name, "Bonnal Baptiste");
    assert_eq!(by_name_total, 1);
    assert_eq!(by_name[0].tutor.id, bonnal.tutor.id);
    assert_eq!(by_email.len(), 1);
    assert_eq!(by_email[0].tutor.id, durand.tutor.id);
    assert_eq!((none, none_total), (vec![], 0));
    assert_eq!((foreign, foreign_total), (vec![], 0));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_school_tutor_is_updated_archived_and_unarchived_within_its_organization() {
    let world = World::new().await;
    let tenant = &world.first;
    let declared = world
        .declare(tenant, tenant.staff[0].member_id, "Formateur")
        .await;

    let updated = world
        .usecase
        .update_school_tutor(UpdateSchoolTutorCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            id: declared.tutor.id,
            job_title: Some("Responsable pédagogique".to_owned()),
        })
        .await
        .unwrap();
    let archived = world
        .usecase
        .archive_school_tutor(
            tenant.manager.subject(),
            tenant.organization_id,
            declared.tutor.id,
        )
        .await
        .unwrap();
    let restored = world
        .usecase
        .unarchive_school_tutor(
            tenant.manager.subject(),
            tenant.organization_id,
            declared.tutor.id,
        )
        .await
        .unwrap();
    let foreign_read = world
        .usecase
        .get_school_tutor(world.second.organization_id, declared.tutor.id)
        .await;
    let foreign_archive = world
        .usecase
        .archive_school_tutor(
            world.second.manager.subject(),
            world.second.organization_id,
            declared.tutor.id,
        )
        .await;

    assert_eq!(
        updated.tutor.job_title.as_deref(),
        Some("Responsable pédagogique")
    );
    assert!(archived.tutor.archived_at.is_some());
    assert!(restored.tutor.archived_at.is_none());
    assert!(matches!(foreign_read, Err(CoreError::NotFound)));
    assert!(matches!(foreign_archive, Err(CoreError::NotFound)));

    world.cleanup().await;
}
