use authz::Subject;
use chrono::{Duration, Utc};
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    AcceptParticipantInvitationCommand, AddCompanyTutorCommand, ApprenticeInput, CompanyTutorInput,
    CreateHostCompanyCommand, CreateParticipantCommand, HostCompany, ParticipantDetail,
    ParticipantKind, ParticipantStatus, Permissions, RemoveCompanyTutorCommand,
    UpdateParticipantCommand, UpsertApprenticeCommand,
    application::{
        MestierUseCase, default_authorizer, policy,
        test_support::{purge, scratch_pool},
    },
    infrastructure::realtime::EventHub,
};

static PARTICIPANT_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_cursus_participant_test_", &PARTICIPANT_SCRATCH).await
}

struct Person {
    user_id: Uuid,
}

impl Person {
    fn subject(&self) -> Subject {
        policy::user_subject(UserId(self.user_id), Vec::new())
    }

    fn id(&self) -> UserId {
        UserId(self.user_id)
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

async fn seed_member(
    pool: &PgPool,
    organization_id: OrganizationId,
    user_id: Uuid,
    permissions: Permissions,
) {
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
        .bind(role_id)
        .execute(pool)
        .await
        .unwrap();
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
    seed_member(
        pool,
        organization_id,
        manager,
        Permissions::VIEW_CURSUS
            | Permissions::MANAGE_CURSUS_TRIADS
            | Permissions::MANAGE_CURSUS_PARTICIPANTS,
    )
    .await;
    seed_member(pool, organization_id, reader, Permissions::VIEW_CURSUS).await;

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

    async fn invitee(&self) -> Person {
        Person {
            user_id: seed_user(&self.pool, "invitee").await,
        }
    }

    async fn company(&self, tenant: &Tenant, name: &str) -> HostCompany {
        self.usecase
            .create_host_company(CreateHostCompanyCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                name: name.to_owned(),
                siret: None,
                address: None,
                legal_form: None,
                naf_code: None,
                employee_count: None,
                director_name: None,
            })
            .await
            .unwrap()
    }

    fn new_participant(
        &self,
        tenant: &Tenant,
        email: &str,
        apprentice: Option<ApprenticeInput>,
        company_tutor: Option<CompanyTutorInput>,
    ) -> CreateParticipantCommand {
        CreateParticipantCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            last_name: "Martin".to_owned(),
            first_name: Some("Léa".to_owned()),
            email: email.to_owned(),
            phone: None,
            apprentice,
            company_tutor,
        }
    }

    async fn apprentice(&self, tenant: &Tenant, email: &str) -> ParticipantDetail {
        self.usecase
            .create_participant(self.new_participant(
                tenant,
                email,
                Some(ApprenticeInput {
                    birth_date: None,
                    ine_code: Some("1234567890A".to_owned()),
                    address: None,
                }),
                None,
            ))
            .await
            .unwrap()
    }

    async fn accept(&self, token: &str, person: &Person) -> Result<crate::CursusSpace, CoreError> {
        self.usecase
            .acting_as(person.id())
            .accept_participant_invitation(AcceptParticipantInvitationCommand {
                token: token.to_owned(),
                user_id: person.id(),
            })
            .await
    }

    async fn cleanup(self, invitees: &[&Person]) {
        for tenant in [&self.first, &self.second] {
            for statement in [
                "DELETE FROM cursus.company_tutor WHERE org_id = $1",
                "DELETE FROM cursus.apprentice WHERE org_id = $1",
                "DELETE FROM cursus.participant WHERE org_id = $1",
                "DELETE FROM cursus.host_company WHERE org_id = $1",
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
        for person in invitees {
            purge(
                &self.pool,
                "DELETE FROM users WHERE id = $1",
                person.user_id,
            )
            .await;
        }
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_6_an_invited_participant_accepts_once_and_the_token_is_then_gone() {
    let world = World::new().await;
    let tenant = &world.first;
    let invitee = world.invitee().await;
    let created = world.apprentice(tenant, "lea.martin@example.com").await;
    assert_eq!(created.participant.status(), ParticipantStatus::Pending);
    assert!(created.apprentice.is_some());

    let invite = world
        .usecase
        .invite_participant(
            tenant.manager.subject(),
            tenant.organization_id,
            created.participant.id,
        )
        .await
        .unwrap();

    let stored: Option<Vec<u8>> =
        sqlx::query_scalar("SELECT invitation_token_hash FROM cursus.participant WHERE id = $1")
            .bind(created.participant.id.0)
            .fetch_one(&world.pool)
            .await
            .unwrap();
    let stored = stored.unwrap();
    assert_ne!(stored, invite.token.as_bytes());
    assert_eq!(
        stored,
        crate::domain::invitation::token::hash(&invite.token)
    );

    let space = world.accept(&invite.token, &invitee).await.unwrap();
    assert_eq!(space.organization_id, tenant.organization_id);
    assert_eq!(space.slug, format!("org-{}", tenant.organization_id.0));

    let linked = world
        .usecase
        .get_participant(tenant.organization_id, created.participant.id)
        .await
        .unwrap();
    assert_eq!(linked.participant.user_id, Some(invitee.id()));
    assert_eq!(linked.participant.status(), ParticipantStatus::Active);
    assert_eq!(linked.participant.invitation_token_hash, None);
    assert_eq!(linked.participant.invitation_expires_at, None);

    let again = world.accept(&invite.token, &invitee).await;
    assert!(matches!(again, Err(CoreError::Gone)));

    let events: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM automation.event WHERE org_id = $1 AND name LIKE 'cursus.participant.%'",
    )
    .bind(tenant.organization_id.0)
    .fetch_one(&world.pool)
    .await
    .unwrap();
    assert_eq!(events, 2);

    world.cleanup(&[&invitee]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn reinviting_replaces_the_token_and_an_expired_or_unlinkable_token_is_refused() {
    let world = World::new().await;
    let tenant = &world.first;
    let invitee = world.invitee().await;
    let other = world.invitee().await;
    let created = world.apprentice(tenant, "lea@example.com").await;
    let id = created.participant.id;

    let first = world
        .usecase
        .invite_participant(tenant.manager.subject(), tenant.organization_id, id)
        .await
        .unwrap();
    let second = world
        .usecase
        .invite_participant(tenant.manager.subject(), tenant.organization_id, id)
        .await
        .unwrap();
    assert_ne!(first.token, second.token);
    assert!(matches!(
        world.accept(&first.token, &invitee).await,
        Err(CoreError::Gone)
    ));

    sqlx::query("UPDATE cursus.participant SET invitation_expires_at = $2 WHERE id = $1")
        .bind(id.0)
        .bind(Utc::now() - Duration::minutes(1))
        .execute(&world.pool)
        .await
        .unwrap();
    assert!(matches!(
        world.accept(&second.token, &invitee).await,
        Err(CoreError::Gone)
    ));

    let third = world
        .usecase
        .invite_participant(tenant.manager.subject(), tenant.organization_id, id)
        .await
        .unwrap();
    let rival = world.apprentice(tenant, "rival@example.com").await;
    let rival_invite = world
        .usecase
        .invite_participant(
            tenant.manager.subject(),
            tenant.organization_id,
            rival.participant.id,
        )
        .await
        .unwrap();
    world.accept(&third.token, &invitee).await.unwrap();
    let second_participant = world.accept(&rival_invite.token, &invitee).await;
    assert!(matches!(second_participant, Err(CoreError::Conflict(_))));

    let linked = world
        .usecase
        .invite_participant(tenant.manager.subject(), tenant.organization_id, id)
        .await;
    assert!(matches!(linked, Err(CoreError::Conflict(_))));

    world
        .usecase
        .deactivate_participant(
            tenant.manager.subject(),
            tenant.organization_id,
            rival.participant.id,
        )
        .await
        .unwrap();
    let deactivated = world.accept(&rival_invite.token, &other).await;
    assert!(matches!(deactivated, Err(CoreError::Gone)));
    let reinvite = world
        .usecase
        .invite_participant(
            tenant.manager.subject(),
            tenant.organization_id,
            rival.participant.id,
        )
        .await;
    assert!(matches!(reinvite, Err(CoreError::Conflict(_))));

    world.cleanup(&[&invitee, &other]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn without_the_participants_bit_every_participant_write_is_forbidden() {
    let world = World::new().await;
    let tenant = &world.first;
    let company = world.company(tenant, "Durand").await;
    let created = world.apprentice(tenant, "lea@example.com").await;
    let id = created.participant.id;

    for person in [&tenant.reader, &tenant.outsider] {
        let command = || CreateParticipantCommand {
            actor: person.subject(),
            ..world.new_participant(tenant, "intrus@example.com", None, None)
        };
        let results = [
            world
                .usecase
                .create_participant(command())
                .await
                .map(|_| ()),
            world
                .usecase
                .update_participant(UpdateParticipantCommand {
                    actor: person.subject(),
                    organization_id: tenant.organization_id,
                    id,
                    last_name: "X".to_owned(),
                    first_name: None,
                    email: "x@example.com".to_owned(),
                    phone: None,
                })
                .await
                .map(|_| ()),
            world
                .usecase
                .invite_participant(person.subject(), tenant.organization_id, id)
                .await
                .map(|_| ()),
            world
                .usecase
                .deactivate_participant(person.subject(), tenant.organization_id, id)
                .await
                .map(|_| ()),
            world
                .usecase
                .reactivate_participant(person.subject(), tenant.organization_id, id)
                .await
                .map(|_| ()),
            world
                .usecase
                .upsert_apprentice(UpsertApprenticeCommand {
                    actor: person.subject(),
                    organization_id: tenant.organization_id,
                    participant_id: id,
                    apprentice: ApprenticeInput {
                        birth_date: None,
                        ine_code: None,
                        address: None,
                    },
                })
                .await
                .map(|_| ()),
            world
                .usecase
                .add_company_tutor(AddCompanyTutorCommand {
                    actor: person.subject(),
                    organization_id: tenant.organization_id,
                    participant_id: id,
                    company_tutor: CompanyTutorInput {
                        host_company_id: company.id,
                        job_title: None,
                    },
                })
                .await
                .map(|_| ()),
            world
                .usecase
                .remove_company_tutor(RemoveCompanyTutorCommand {
                    actor: person.subject(),
                    organization_id: tenant.organization_id,
                    id: crate::CompanyTutorId(Uuid::new_v4()),
                })
                .await,
        ];
        for result in results {
            assert!(matches!(result, Err(CoreError::Forbidden { .. })));
        }
    }

    let (listed, total) = world
        .usecase
        .list_participants(tenant.organization_id, None, None, None, 10, 0)
        .await
        .unwrap();
    assert_eq!((listed.len(), total), (1, 1));
    assert_eq!(listed[0].participant.status(), ParticipantStatus::Pending);

    world.cleanup(&[]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn creation_refuses_both_blocks_a_foreign_host_company_and_a_duplicate_email() {
    let world = World::new().await;
    let tenant = &world.first;
    let company = world.company(tenant, "Durand").await;
    let foreign_company = world.company(&world.second, "Ailleurs").await;
    let tutor = |host_company_id| CompanyTutorInput {
        host_company_id,
        job_title: Some("Chef d'équipe".to_owned()),
    };
    let apprentice = || ApprenticeInput {
        birth_date: None,
        ine_code: None,
        address: None,
    };

    let both = world
        .usecase
        .create_participant(world.new_participant(
            tenant,
            "both@example.com",
            Some(apprentice()),
            Some(tutor(company.id)),
        ))
        .await;
    assert!(matches!(both, Err(CoreError::Conflict(_))));

    let foreign = world
        .usecase
        .create_participant(world.new_participant(
            tenant,
            "foreign@example.com",
            None,
            Some(tutor(foreign_company.id)),
        ))
        .await;
    assert!(matches!(foreign, Err(CoreError::NotFound)));

    let created = world
        .usecase
        .create_participant(world.new_participant(
            tenant,
            "Tuteur@Example.com",
            None,
            Some(tutor(company.id)),
        ))
        .await
        .unwrap();
    assert_eq!(created.company_tutors.len(), 1);
    assert_eq!(created.company_tutors[0].host_company_name, "Durand");
    assert!(created.apprentice.is_none());

    let duplicate = world
        .usecase
        .create_participant(world.new_participant(tenant, "tuteur@example.com", None, None))
        .await;
    assert!(matches!(duplicate, Err(CoreError::Conflict(_))));

    let other_org = world
        .usecase
        .create_participant(world.new_participant(&world.second, "tuteur@example.com", None, None))
        .await;
    assert!(other_org.is_ok());

    let raw_duplicate = sqlx::query(
        "INSERT INTO cursus.participant (id, org_id, last_name, email) VALUES ($1, $2, 'X', 'TUTEUR@example.com')",
    )
    .bind(generate_uuid_v7())
    .bind(tenant.organization_id.0)
    .execute(&world.pool)
    .await;
    assert!(raw_duplicate.is_err());

    world.cleanup(&[]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn records_are_upserted_added_once_per_company_and_removed() {
    let world = World::new().await;
    let tenant = &world.first;
    let durand = world.company(tenant, "Durand").await;
    let petit = world.company(tenant, "Petit").await;
    let created = world.apprentice(tenant, "lea@example.com").await;
    let id = created.participant.id;

    let replaced = world
        .usecase
        .upsert_apprentice(UpsertApprenticeCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            participant_id: id,
            apprentice: ApprenticeInput {
                birth_date: None,
                ine_code: None,
                address: Some("3 rue des Lilas".to_owned()),
            },
        })
        .await
        .unwrap();
    let before = created.apprentice.unwrap();
    let after = replaced.apprentice.unwrap();
    assert_eq!(after.id, before.id);
    assert_eq!(after.ine_code, None);
    assert_eq!(after.address.as_deref(), Some("3 rue des Lilas"));

    let add = |host_company_id| AddCompanyTutorCommand {
        actor: tenant.manager.subject(),
        organization_id: tenant.organization_id,
        participant_id: id,
        company_tutor: CompanyTutorInput {
            host_company_id,
            job_title: None,
        },
    };
    world
        .usecase
        .add_company_tutor(add(durand.id))
        .await
        .unwrap();
    let both = world
        .usecase
        .add_company_tutor(add(petit.id))
        .await
        .unwrap();
    assert_eq!(both.company_tutors.len(), 2);
    let twice = world.usecase.add_company_tutor(add(durand.id)).await;
    assert!(matches!(twice, Err(CoreError::Conflict(_))));

    let by_kind = |kind| {
        world
            .usecase
            .list_participants(tenant.organization_id, Some(kind), None, None, 10, 0)
    };
    assert_eq!(by_kind(ParticipantKind::Apprentice).await.unwrap().1, 1);
    assert_eq!(by_kind(ParticipantKind::CompanyTutor).await.unwrap().1, 1);

    let removed = both.company_tutors[0].tutor.id;
    world
        .usecase
        .remove_company_tutor(RemoveCompanyTutorCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            id: removed,
        })
        .await
        .unwrap();
    let missing = world
        .usecase
        .remove_company_tutor(RemoveCompanyTutorCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            id: removed,
        })
        .await;
    assert!(matches!(missing, Err(CoreError::NotFound)));

    let by_status = |status| {
        world
            .usecase
            .list_participants(tenant.organization_id, None, Some(status), None, 10, 0)
    };
    assert_eq!(by_status(ParticipantStatus::Pending).await.unwrap().1, 1);
    assert_eq!(by_status(ParticipantStatus::Active).await.unwrap().1, 0);
    world
        .usecase
        .deactivate_participant(tenant.manager.subject(), tenant.organization_id, id)
        .await
        .unwrap();
    assert_eq!(by_status(ParticipantStatus::Inactive).await.unwrap().1, 1);
    assert_eq!(by_status(ParticipantStatus::Pending).await.unwrap().1, 0);
    let reactivated = world
        .usecase
        .reactivate_participant(tenant.manager.subject(), tenant.organization_id, id)
        .await
        .unwrap();
    assert_eq!(reactivated.status(), ParticipantStatus::Pending);

    world.cleanup(&[]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_list_embeds_the_records_and_filters_by_host_company_within_the_organization() {
    let world = World::new().await;
    let tenant = &world.first;
    let durand = world.company(tenant, "Durand").await;
    let petit = world.company(tenant, "Petit").await;
    let foreign = world.company(&world.second, "Ailleurs").await;
    let apprentice = world.apprentice(tenant, "lea@example.com").await;
    let tutor_input = |host_company_id| CompanyTutorInput {
        host_company_id,
        job_title: Some("Chef d'équipe".to_owned()),
    };
    let tutor = world
        .usecase
        .create_participant(world.new_participant(
            tenant,
            "paul@example.com",
            None,
            Some(tutor_input(durand.id)),
        ))
        .await
        .unwrap();
    world
        .usecase
        .add_company_tutor(AddCompanyTutorCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            participant_id: tutor.participant.id,
            company_tutor: tutor_input(petit.id),
        })
        .await
        .unwrap();
    world
        .usecase
        .create_participant(world.new_participant(
            &world.second,
            "autre@example.com",
            None,
            Some(tutor_input(foreign.id)),
        ))
        .await
        .unwrap();

    let (listed, total) = world
        .usecase
        .list_participants(tenant.organization_id, None, None, None, 10, 0)
        .await
        .unwrap();
    assert_eq!((listed.len(), total), (2, 2));
    let lea = listed
        .iter()
        .find(|item| item.participant.id == apprentice.participant.id)
        .unwrap();
    assert_eq!(
        lea.apprentice_id,
        apprentice.apprentice.as_ref().map(|a| a.id)
    );
    assert!(lea.company_tutors.is_empty());
    let paul = listed
        .iter()
        .find(|item| item.participant.id == tutor.participant.id)
        .unwrap();
    assert_eq!(paul.apprentice_id, None);
    let names: Vec<&str> = paul
        .company_tutors
        .iter()
        .map(|record| record.host_company_name.as_str())
        .collect();
    assert_eq!(names, ["Durand", "Petit"]);

    let at = |host_company_id| {
        world.usecase.list_participants(
            tenant.organization_id,
            None,
            None,
            Some(host_company_id),
            10,
            0,
        )
    };
    let (at_petit, total) = at(petit.id).await.unwrap();
    assert_eq!(total, 1);
    assert_eq!(at_petit[0].participant.id, tutor.participant.id);
    assert_eq!(at_petit[0].company_tutors.len(), 2);
    assert_eq!(at(foreign.id).await.unwrap().1, 0);

    world.cleanup(&[]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_12_one_organization_never_sees_or_touches_another_ones_participants() {
    let world = World::new().await;
    let created = world.apprentice(&world.first, "lea@example.com").await;
    let id = created.participant.id;
    let second = &world.second;

    let read = world
        .usecase
        .get_participant(second.organization_id, id)
        .await;
    assert!(matches!(read, Err(CoreError::NotFound)));

    let (listed, total) = world
        .usecase
        .list_participants(second.organization_id, None, None, None, 10, 0)
        .await
        .unwrap();
    assert_eq!((listed, total), (vec![], 0));

    let invited = world
        .usecase
        .invite_participant(second.manager.subject(), second.organization_id, id)
        .await;
    assert!(matches!(invited, Err(CoreError::NotFound)));

    let updated = world
        .usecase
        .upsert_apprentice(UpsertApprenticeCommand {
            actor: second.manager.subject(),
            organization_id: second.organization_id,
            participant_id: id,
            apprentice: ApprenticeInput {
                birth_date: None,
                ine_code: None,
                address: None,
            },
        })
        .await;
    assert!(matches!(updated, Err(CoreError::NotFound)));

    let foreign = world
        .usecase
        .invite_participant(second.manager.subject(), world.first.organization_id, id)
        .await;
    assert!(matches!(foreign, Err(CoreError::Forbidden { .. })));

    let intact = world
        .usecase
        .get_participant(world.first.organization_id, id)
        .await
        .unwrap();
    assert_eq!(intact.participant.invitation_token_hash, None);

    world.cleanup(&[]).await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_refuses_records_that_cross_organizations() {
    let world = World::new().await;
    let created = world.apprentice(&world.first, "lea@example.com").await;
    let company = world.company(&world.first, "Durand").await;

    let apprentice = sqlx::query(
        "INSERT INTO cursus.apprentice (id, org_id, participant_id) VALUES ($1, $2, $3)",
    )
    .bind(generate_uuid_v7())
    .bind(world.second.organization_id.0)
    .bind(created.participant.id.0)
    .execute(&world.pool)
    .await;
    assert!(apprentice.is_err());

    let other = world
        .usecase
        .create_participant(world.new_participant(&world.second, "other@example.com", None, None))
        .await
        .unwrap();
    let cross_company = sqlx::query(
        "INSERT INTO cursus.company_tutor (id, org_id, participant_id, host_company_id) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(generate_uuid_v7())
    .bind(world.second.organization_id.0)
    .bind(other.participant.id.0)
    .bind(company.id.0)
    .execute(&world.pool)
    .await;
    assert!(cross_company.is_err());

    let cross_participant = sqlx::query(
        "INSERT INTO cursus.company_tutor (id, org_id, participant_id, host_company_id) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(generate_uuid_v7())
    .bind(world.first.organization_id.0)
    .bind(other.participant.id.0)
    .bind(company.id.0)
    .execute(&world.pool)
    .await;
    assert!(cross_participant.is_err());

    world.cleanup(&[]).await;
}
