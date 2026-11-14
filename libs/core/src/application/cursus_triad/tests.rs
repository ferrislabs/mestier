use std::sync::Mutex;

use authz::Subject;
use chrono::NaiveDate;
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    AcceptParticipantInvitationCommand, ApprenticeId, ApprenticeInput, CloseContractCommand,
    CloseTriadCommand, ClosureReason, CompanyTutorId, CompanyTutorInput, ContractFileKind,
    ContractTutorInput, ContractView, CreateClosureReasonCommand, CreateContractCommand,
    CreateEstablishmentCommand, CreateHostCompanyCommand, CreateParticipantCommand,
    CreatePromotionCommand, CreateTrainingCommand, CreateTriadCommand, CursusContractKind,
    HostCompany, MemberId, Permissions, Promotion, RemoveCompanyTutorCommand, SearchPattern,
    SetContractFileCommand, SetContractTutorsCommand, Triad, TriadAccess, TriadFilter, TriadRole,
    UpdateContractCommand, UpdateTriadCommand,
    application::{
        MestierUseCase, default_authorizer, policy,
        test_support::{purge, scratch_pool},
    },
    infrastructure::realtime::EventHub,
};

static TRIAD_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_cursus_triad_test_", &TRIAD_SCRATCH).await
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
    member_id: MemberId,
}

struct World {
    pool: PgPool,
    usecase: MestierUseCase,
    first: Tenant,
    second: Tenant,
    extra_users: Mutex<Vec<Uuid>>,
}

struct Catalog {
    promotion: Promotion,
    reason: ClosureReason,
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
) -> MemberId {
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
    MemberId(member_id)
}

async fn seed_school_tutor(
    pool: &PgPool,
    organization_id: OrganizationId,
    member_id: MemberId,
    archived: bool,
) {
    sqlx::query(
        "INSERT INTO cursus.school_tutor (id, org_id, member_id, archived_at) \
         VALUES ($1, $2, $3, CASE WHEN $4 THEN now() END)",
    )
    .bind(generate_uuid_v7())
    .bind(organization_id.0)
    .bind(member_id.0)
    .bind(archived)
    .execute(pool)
    .await
    .unwrap();
}

async fn seed_tenant(pool: &PgPool) -> Tenant {
    let manager = seed_user(pool, "manager").await;
    let reader = seed_user(pool, "reader").await;
    let organization_id = OrganizationId(generate_uuid_v7());
    sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
        .bind(organization_id.0)
        .bind("Centre de formation")
        .bind(format!("org-{}", organization_id.0))
        .bind(manager)
        .execute(pool)
        .await
        .unwrap();
    let member_id = seed_member(
        pool,
        organization_id,
        manager,
        Permissions::VIEW_CURSUS
            | Permissions::MANAGE_CURSUS_CATALOG
            | Permissions::MANAGE_CURSUS_TRIADS
            | Permissions::MANAGE_CURSUS_PARTICIPANTS,
    )
    .await;
    seed_member(pool, organization_id, reader, Permissions::VIEW_CURSUS).await;
    seed_school_tutor(pool, organization_id, member_id, false).await;

    Tenant {
        organization_id,
        manager: Person { user_id: manager },
        reader: Person { user_id: reader },
        member_id,
    }
}

fn day(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

impl World {
    async fn new() -> Self {
        let pool = pool().await;
        Self {
            usecase: MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new()),
            first: seed_tenant(&pool).await,
            second: seed_tenant(&pool).await,
            extra_users: Mutex::new(Vec::new()),
            pool,
        }
    }

    async fn person(&self, label: &str) -> Person {
        let user_id = seed_user(&self.pool, label).await;
        self.extra_users.lock().unwrap().push(user_id);
        Person { user_id }
    }

    async fn catalog(&self, tenant: &Tenant) -> Catalog {
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
        let promotion = self
            .usecase
            .create_promotion(CreatePromotionCommand {
                actor: actor.clone(),
                organization_id: tenant.organization_id,
                training_id: training.id,
                label: "BTS 2026".to_owned(),
                start_year: 2026,
                end_year: 2028,
            })
            .await
            .unwrap();
        let reason = self.reason(tenant, "Fin de formation").await;

        Catalog { promotion, reason }
    }

    async fn reason(&self, tenant: &Tenant, label: &str) -> ClosureReason {
        self.usecase
            .create_closure_reason(CreateClosureReasonCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                label: label.to_owned(),
            })
            .await
            .unwrap()
    }

    async fn company(&self, tenant: &Tenant, name: &str) -> HostCompany {
        self.usecase
            .create_host_company(CreateHostCompanyCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                name: name.to_owned(),
                group_name: None,
                siret: None,
                legal_form: None,
                naf_code: None,
                capital: None,
                employee_count: None,
                group_employee_count: None,
                director_name: None,
                phone: None,
                email: None,
                website: None,
                activity_domains: None,
                missions: None,
                addresses: Vec::new(),
            })
            .await
            .unwrap()
    }

    async fn link_user(&self, tenant: &Tenant, participant_id: crate::ParticipantId) -> Person {
        let person = self.person("participant").await;
        let invite = self
            .usecase
            .invite_participant(
                tenant.manager.subject(),
                tenant.organization_id,
                participant_id,
            )
            .await
            .unwrap();
        self.usecase
            .acting_as(person.id())
            .accept_participant_invitation(AcceptParticipantInvitationCommand {
                token: invite.token,
                user_id: person.id(),
            })
            .await
            .unwrap();
        person
    }

    async fn apprentice(&self, tenant: &Tenant, email: &str) -> (ApprenticeId, Person) {
        let detail = self
            .usecase
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
            .unwrap();
        let person = self.link_user(tenant, detail.participant.id).await;

        (detail.apprentice.unwrap().id, person)
    }

    async fn tutor(
        &self,
        tenant: &Tenant,
        company: &HostCompany,
        email: &str,
    ) -> (CompanyTutorId, Person) {
        let detail = self
            .usecase
            .create_participant(CreateParticipantCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                last_name: "Durand".to_owned(),
                first_name: Some("Paul".to_owned()),
                email: email.to_owned(),
                phone: None,
                apprentice: None,
                company_tutor: Some(CompanyTutorInput {
                    host_company_id: company.id,
                    job_title: None,
                }),
            })
            .await
            .unwrap();
        let person = self.link_user(tenant, detail.participant.id).await;

        (detail.company_tutors[0].tutor.id, person)
    }

    async fn triad(
        &self,
        tenant: &Tenant,
        catalog: &Catalog,
        apprentice_id: ApprenticeId,
    ) -> Triad {
        self.usecase
            .create_triad(CreateTriadCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                promotion_id: catalog.promotion.id,
                apprentice_id,
                pedagogic_tutor_member_id: Some(tenant.member_id),
            })
            .await
            .unwrap()
    }

    async fn contract(
        &self,
        tenant: &Tenant,
        triad: &Triad,
        company: &HostCompany,
    ) -> ContractView {
        self.usecase
            .create_contract(CreateContractCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                triad_id: triad.id,
                host_company_id: company.id,
                kind: CursusContractKind::Apprenticeship,
                number: Some("C-1".to_owned()),
                comment: Some("**Note**".to_owned()),
                start_date: day(2026, 9, 1),
                end_date: day(2028, 8, 31),
            })
            .await
            .unwrap()
    }

    async fn tutors(&self, tenant: &Tenant, contract: &ContractView, main: Option<CompanyTutorId>) {
        self.usecase
            .set_contract_tutors(SetContractTutorsCommand {
                actor: tenant.manager.subject(),
                organization_id: tenant.organization_id,
                contract_id: contract.contract.id,
                tutors: main
                    .into_iter()
                    .map(|company_tutor_id| ContractTutorInput {
                        company_tutor_id,
                        is_main: true,
                    })
                    .collect(),
            })
            .await
            .unwrap();
    }

    fn close_triad_command(
        &self,
        tenant: &Tenant,
        triad: &Triad,
        reason: &ClosureReason,
    ) -> CloseTriadCommand {
        CloseTriadCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            id: triad.id,
            closed_at: day(2027, 6, 30),
            closure_reason_id: reason.id,
            closure_note: Some("Fin de parcours".to_owned()),
        }
    }

    fn close_contract_command(
        &self,
        tenant: &Tenant,
        contract: &ContractView,
        reason: &ClosureReason,
    ) -> CloseContractCommand {
        CloseContractCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            id: contract.contract.id,
            closed_at: day(2027, 3, 31),
            closure_reason_id: reason.id,
            closure_note: Some("Rupture".to_owned()),
        }
    }

    async fn event_count(&self, tenant: &Tenant, name: &str) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM automation.event WHERE org_id = $1 AND name = $2")
            .bind(tenant.organization_id.0)
            .bind(name)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn cleanup(self) {
        for tenant in [&self.first, &self.second] {
            for statement in [
                "DELETE FROM cursus.contract_tutor WHERE org_id = $1",
                "DELETE FROM cursus.contract WHERE org_id = $1",
                "DELETE FROM cursus.triad WHERE org_id = $1",
                "DELETE FROM cursus.school_tutor WHERE org_id = $1",
                "DELETE FROM cursus.company_tutor WHERE org_id = $1",
                "DELETE FROM cursus.apprentice WHERE org_id = $1",
                "DELETE FROM cursus.participant WHERE org_id = $1",
                "DELETE FROM cursus.host_company WHERE org_id = $1",
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
            for person in [&tenant.manager, &tenant.reader] {
                purge(
                    &self.pool,
                    "DELETE FROM users WHERE id = $1",
                    person.user_id,
                )
                .await;
            }
        }
        let extras = self.extra_users.lock().unwrap().clone();
        for user_id in extras {
            purge(&self.pool, "DELETE FROM users WHERE id = $1", user_id).await;
        }
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_5_a_second_triad_for_the_same_apprentice_and_promotion_is_a_conflict() {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let (apprentice_id, _) = world.apprentice(tenant, "lea@example.com").await;
    let first = world.triad(tenant, &catalog, apprentice_id).await;

    let second = world
        .usecase
        .create_triad(CreateTriadCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            promotion_id: catalog.promotion.id,
            apprentice_id,
            pedagogic_tutor_member_id: None,
        })
        .await;

    assert!(matches!(second, Err(CoreError::Conflict(_))));
    let (listed, total) = world
        .usecase
        .list_triads(tenant.organization_id, TriadFilter::default(), 10, 0)
        .await
        .unwrap();
    assert_eq!(total, 1);
    assert_eq!(listed[0].triad.id, first.id);
    assert_eq!(world.event_count(tenant, "cursus.triad.created").await, 1);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_triad_needs_a_promotion_an_apprentice_and_a_pedagogic_tutor_of_the_organization() {
    let world = World::new().await;
    let catalog = world.catalog(&world.first).await;
    let foreign_catalog = world.catalog(&world.second).await;
    let (apprentice_id, _) = world.apprentice(&world.first, "lea@example.com").await;
    let (foreign_apprentice, _) = world.apprentice(&world.second, "lea@example.com").await;
    let command = |promotion: &Promotion, apprentice: ApprenticeId, member: Option<MemberId>| {
        CreateTriadCommand {
            actor: world.first.manager.subject(),
            organization_id: world.first.organization_id,
            promotion_id: promotion.id,
            apprentice_id: apprentice,
            pedagogic_tutor_member_id: member,
        }
    };

    let foreign_promotion = world
        .usecase
        .create_triad(command(&foreign_catalog.promotion, apprentice_id, None))
        .await;
    let foreign_apprentice_result = world
        .usecase
        .create_triad(command(&catalog.promotion, foreign_apprentice, None))
        .await;
    let foreign_member = world
        .usecase
        .create_triad(command(
            &catalog.promotion,
            apprentice_id,
            Some(world.second.member_id),
        ))
        .await;
    let valid = world
        .usecase
        .create_triad(command(
            &catalog.promotion,
            apprentice_id,
            Some(world.first.member_id),
        ))
        .await
        .unwrap();

    assert!(matches!(foreign_promotion, Err(CoreError::NotFound)));
    assert!(matches!(
        foreign_apprentice_result,
        Err(CoreError::NotFound)
    ));
    assert!(matches!(foreign_member, Err(CoreError::NotFound)));
    assert_eq!(valid.pedagogic_tutor_member_id, Some(world.first.member_id));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_pedagogic_tutor_of_a_triad_must_be_an_active_school_tutor() {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let (apprentice_id, _) = world.apprentice(tenant, "lea@example.com").await;
    let (other_apprentice, _) = world.apprentice(tenant, "tom@example.com").await;
    let plain = world.person("plain").await;
    let plain_member = seed_member(
        &world.pool,
        tenant.organization_id,
        plain.user_id,
        Permissions::VIEW_CURSUS,
    )
    .await;
    let retired = world.person("retired").await;
    let retired_member = seed_member(
        &world.pool,
        tenant.organization_id,
        retired.user_id,
        Permissions::VIEW_CURSUS,
    )
    .await;
    seed_school_tutor(&world.pool, tenant.organization_id, retired_member, true).await;
    let command = |apprentice: ApprenticeId, member: MemberId| CreateTriadCommand {
        actor: tenant.manager.subject(),
        organization_id: tenant.organization_id,
        promotion_id: catalog.promotion.id,
        apprentice_id: apprentice,
        pedagogic_tutor_member_id: Some(member),
    };

    let not_declared = world
        .usecase
        .create_triad(command(apprentice_id, plain_member))
        .await;
    let archived = world
        .usecase
        .create_triad(command(apprentice_id, retired_member))
        .await;
    let triad = world
        .usecase
        .create_triad(command(other_apprentice, tenant.member_id))
        .await
        .unwrap();
    let update = |member: MemberId| UpdateTriadCommand {
        actor: tenant.manager.subject(),
        organization_id: tenant.organization_id,
        id: triad.id,
        pedagogic_tutor_member_id: Some(member),
    };
    let updated_to_plain = world.usecase.update_triad(update(plain_member)).await;
    sqlx::query("UPDATE cursus.school_tutor SET archived_at = now() WHERE member_id = $1")
        .bind(tenant.member_id.0)
        .execute(&world.pool)
        .await
        .unwrap();
    let kept_archived = world.usecase.update_triad(update(tenant.member_id)).await;

    for error in [
        not_declared.map(|_| ()),
        archived.map(|_| ()),
        updated_to_plain.map(|_| ()),
    ] {
        assert!(matches!(
            error,
            Err(CoreError::Unprocessable { code, .. }) if code == crate::NOT_A_SCHOOL_TUTOR
        ));
    }
    assert_eq!(
        kept_archived.unwrap().triad.pedagogic_tutor_member_id,
        Some(tenant.member_id)
    );

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn without_the_triads_bit_every_triad_and_contract_write_is_forbidden() {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let company = world.company(tenant, "Paysages Durand").await;
    let (apprentice_id, _) = world.apprentice(tenant, "lea@example.com").await;
    let triad = world.triad(tenant, &catalog, apprentice_id).await;
    let contract = world.contract(tenant, &triad, &company).await;
    let reader = tenant.reader.subject();

    let created_triad = world
        .usecase
        .create_triad(CreateTriadCommand {
            actor: reader.clone(),
            organization_id: tenant.organization_id,
            promotion_id: catalog.promotion.id,
            apprentice_id,
            pedagogic_tutor_member_id: None,
        })
        .await;
    let updated_triad = world
        .usecase
        .update_triad(UpdateTriadCommand {
            actor: reader.clone(),
            organization_id: tenant.organization_id,
            id: triad.id,
            pedagogic_tutor_member_id: None,
        })
        .await;
    let created_contract = world
        .usecase
        .create_contract(CreateContractCommand {
            actor: reader.clone(),
            organization_id: tenant.organization_id,
            triad_id: triad.id,
            host_company_id: company.id,
            kind: CursusContractKind::Apprenticeship,
            number: None,
            comment: None,
            start_date: day(2026, 9, 1),
            end_date: day(2028, 8, 31),
        })
        .await;
    let updated_contract = world
        .usecase
        .update_contract(UpdateContractCommand {
            actor: reader.clone(),
            organization_id: tenant.organization_id,
            id: contract.contract.id,
            host_company_id: company.id,
            kind: CursusContractKind::Apprenticeship,
            number: None,
            comment: None,
            start_date: day(2026, 9, 1),
            end_date: day(2028, 8, 31),
        })
        .await;
    let closed_triad = world
        .usecase
        .close_triad(CloseTriadCommand {
            actor: reader.clone(),
            ..world.close_triad_command(tenant, &triad, &catalog.reason)
        })
        .await;
    let closed_contract = world
        .usecase
        .close_contract(CloseContractCommand {
            actor: reader.clone(),
            ..world.close_contract_command(tenant, &contract, &catalog.reason)
        })
        .await;
    let tutors = world
        .usecase
        .set_contract_tutors(SetContractTutorsCommand {
            actor: reader.clone(),
            organization_id: tenant.organization_id,
            contract_id: contract.contract.id,
            tutors: Vec::new(),
        })
        .await;
    let file = world
        .usecase
        .set_contract_file(SetContractFileCommand {
            actor: reader,
            organization_id: tenant.organization_id,
            contract_id: contract.contract.id,
            kind: ContractFileKind::MissionSheet,
            file_key: Some("uploads/mission".to_owned()),
        })
        .await;

    assert!(matches!(created_triad, Err(CoreError::Forbidden { .. })));
    assert!(matches!(updated_triad, Err(CoreError::Forbidden { .. })));
    assert!(matches!(created_contract, Err(CoreError::Forbidden { .. })));
    assert!(matches!(updated_contract, Err(CoreError::Forbidden { .. })));
    assert!(matches!(closed_triad, Err(CoreError::Forbidden { .. })));
    assert!(matches!(closed_contract, Err(CoreError::Forbidden { .. })));
    assert!(matches!(tutors, Err(CoreError::Forbidden { .. })));
    assert!(matches!(file, Err(CoreError::Forbidden { .. })));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_7_an_apprentice_sees_their_own_triad_and_another_one_of_the_organization_is_not_found()
 {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let company = world.company(tenant, "Paysages Durand").await;
    let (apprentice_id, apprentice) = world.apprentice(tenant, "lea@example.com").await;
    let (other_apprentice, _) = world.apprentice(tenant, "tom@example.com").await;
    let own = world.triad(tenant, &catalog, apprentice_id).await;
    let other = world.triad(tenant, &catalog, other_apprentice).await;
    world.contract(tenant, &own, &company).await;

    let listed = world
        .usecase
        .list_my_triads(apprentice.id(), tenant.organization_id)
        .await
        .unwrap();
    let detail = world
        .usecase
        .get_my_triad(apprentice.id(), tenant.organization_id, own.id)
        .await
        .unwrap();
    let hidden = world
        .usecase
        .get_my_triad(apprentice.id(), tenant.organization_id, other.id)
        .await;

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].summary.triad.id, own.id);
    assert_eq!(listed[0].summary.promotion_label, "BTS 2026");
    assert_eq!(
        listed[0].summary.training_name,
        "BTS Aménagements paysagers"
    );
    assert_eq!(listed[0].link.role, TriadRole::Apprentice);
    assert_eq!(listed[0].link.access, TriadAccess::Active);
    assert_eq!(detail.detail.apprentice_email, "lea@example.com");
    assert_eq!(detail.detail.contracts.len(), 1);
    assert_eq!(
        detail.detail.contracts[0].host_company_name,
        "Paysages Durand"
    );
    assert!(matches!(hidden, Err(CoreError::NotFound)));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_8_a_tutor_of_company_a_never_sees_a_triad_whose_contract_is_at_company_b() {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let company_a = world.company(tenant, "Entreprise A").await;
    let company_b = world.company(tenant, "Entreprise B").await;
    let (tutor_a, tutor_person) = world.tutor(tenant, &company_a, "tutor-a@example.com").await;
    let (first_apprentice, _) = world.apprentice(tenant, "lea@example.com").await;
    let (second_apprentice, _) = world.apprentice(tenant, "tom@example.com").await;
    let at_a = world.triad(tenant, &catalog, first_apprentice).await;
    let at_b = world.triad(tenant, &catalog, second_apprentice).await;
    let contract_a = world.contract(tenant, &at_a, &company_a).await;
    world.contract(tenant, &at_b, &company_b).await;
    world.tutors(tenant, &contract_a, Some(tutor_a)).await;

    let listed = world
        .usecase
        .list_my_triads(tutor_person.id(), tenant.organization_id)
        .await
        .unwrap();
    let visible = world
        .usecase
        .get_my_triad(tutor_person.id(), tenant.organization_id, at_a.id)
        .await
        .unwrap();
    let hidden = world
        .usecase
        .get_my_triad(tutor_person.id(), tenant.organization_id, at_b.id)
        .await;

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].summary.triad.id, at_a.id);
    assert_eq!(listed[0].link.role, TriadRole::MainTutor);
    assert_eq!(visible.link.access, TriadAccess::Active);
    assert!(matches!(hidden, Err(CoreError::NotFound)));

    let refused = world
        .usecase
        .set_contract_tutors(SetContractTutorsCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            contract_id: world.contract(tenant, &at_b, &company_b).await.contract.id,
            tutors: vec![ContractTutorInput {
                company_tutor_id: tutor_a,
                is_main: true,
            }],
        })
        .await;
    assert!(matches!(refused, Err(CoreError::Conflict(_))));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_9_closing_a_contract_archives_its_tutor_links_and_the_tutor_keeps_a_read_only_view()
 {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let company = world.company(tenant, "Paysages Durand").await;
    let (tutor_id, tutor_person) = world.tutor(tenant, &company, "tutor@example.com").await;
    let (apprentice_id, _) = world.apprentice(tenant, "lea@example.com").await;
    let triad = world.triad(tenant, &catalog, apprentice_id).await;
    let contract = world.contract(tenant, &triad, &company).await;
    world.tutors(tenant, &contract, Some(tutor_id)).await;

    let closed = world
        .usecase
        .close_contract(world.close_contract_command(tenant, &contract, &catalog.reason))
        .await
        .unwrap();

    assert!(closed.contract.is_closed());
    assert_eq!(
        closed.closure_reason_label.as_deref(),
        Some("Fin de formation")
    );
    assert_eq!(closed.tutors.len(), 1);
    let archived_at = closed.tutors[0].archived_at.expect("tutor link archived");

    let detail = world
        .usecase
        .get_my_triad(tutor_person.id(), tenant.organization_id, triad.id)
        .await
        .unwrap();
    assert_eq!(detail.link.role, TriadRole::MainTutor);
    assert_eq!(
        detail.link.access,
        TriadAccess::ReadOnly { since: archived_at }
    );
    assert_eq!(detail.detail.contracts.len(), 1);
    assert_eq!(world.event_count(tenant, "cursus.contract.closed").await, 1);

    let again = world
        .usecase
        .close_contract(world.close_contract_command(tenant, &contract, &catalog.reason))
        .await;
    assert!(matches!(again, Err(CoreError::Conflict(_))));
    let retutor = world
        .usecase
        .set_contract_tutors(SetContractTutorsCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            contract_id: contract.contract.id,
            tutors: Vec::new(),
        })
        .await;
    assert!(matches!(retutor, Err(CoreError::Conflict(_))));
    assert_eq!(world.event_count(tenant, "cursus.contract.closed").await, 1);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_10_closing_a_triad_closes_its_open_contracts_and_a_second_closure_is_a_conflict()
{
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let company = world.company(tenant, "Paysages Durand").await;
    let (tutor_id, tutor_person) = world.tutor(tenant, &company, "tutor@example.com").await;
    let (apprentice_id, apprentice) = world.apprentice(tenant, "lea@example.com").await;
    let triad = world.triad(tenant, &catalog, apprentice_id).await;
    let already_closed = world.contract(tenant, &triad, &company).await;
    let open = world.contract(tenant, &triad, &company).await;
    world.tutors(tenant, &open, Some(tutor_id)).await;
    let earlier_reason = world.reason(tenant, "Rupture anticipée").await;
    world
        .usecase
        .close_contract(world.close_contract_command(tenant, &already_closed, &earlier_reason))
        .await
        .unwrap();

    let closed = world
        .usecase
        .close_triad(world.close_triad_command(tenant, &triad, &catalog.reason))
        .await
        .unwrap();

    assert!(closed.triad.is_closed());
    assert_eq!(
        closed.closure_reason_label.as_deref(),
        Some("Fin de formation")
    );
    let by_id = |id| {
        closed
            .contracts
            .iter()
            .find(|view| view.contract.id == id)
            .unwrap()
    };
    let cascaded = by_id(open.contract.id);
    let cascaded_closure = cascaded.contract.closure.as_ref().unwrap();
    let triad_closure = closed.triad.closure.as_ref().unwrap();
    assert_eq!(cascaded_closure, triad_closure);
    assert_eq!(cascaded_closure.note.as_deref(), Some("Fin de parcours"));
    assert!(cascaded.tutors[0].archived_at.is_some());
    let untouched = by_id(already_closed.contract.id);
    assert_eq!(
        untouched.closure_reason_label.as_deref(),
        Some("Rupture anticipée")
    );
    assert_eq!(world.event_count(tenant, "cursus.triad.closed").await, 1);
    assert_eq!(world.event_count(tenant, "cursus.contract.closed").await, 2);

    let again = world
        .usecase
        .close_triad(world.close_triad_command(tenant, &triad, &catalog.reason))
        .await;
    assert!(matches!(again, Err(CoreError::Conflict(_))));
    assert_eq!(world.event_count(tenant, "cursus.triad.closed").await, 1);

    let new_contract = world
        .usecase
        .create_contract(CreateContractCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            triad_id: triad.id,
            host_company_id: company.id,
            kind: CursusContractKind::Professionalization,
            number: None,
            comment: None,
            start_date: day(2027, 9, 1),
            end_date: day(2028, 8, 31),
        })
        .await;
    let patch = world
        .usecase
        .update_triad(UpdateTriadCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            id: triad.id,
            pedagogic_tutor_member_id: None,
        })
        .await;
    assert!(matches!(new_contract, Err(CoreError::Conflict(_))));
    assert!(matches!(patch, Err(CoreError::Conflict(_))));

    let apprentice_view = world
        .usecase
        .get_my_triad(apprentice.id(), tenant.organization_id, triad.id)
        .await
        .unwrap();
    assert_eq!(apprentice_view.link.access, TriadAccess::Active);
    assert!(apprentice_view.detail.triad.is_closed());
    let tutor_view = world
        .usecase
        .get_my_triad(tutor_person.id(), tenant.organization_id, triad.id)
        .await
        .unwrap();
    assert!(matches!(
        tutor_view.link.access,
        TriadAccess::ReadOnly { .. }
    ));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_archived_closure_reason_is_refused_for_new_closures_but_stays_on_past_ones() {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let company = world.company(tenant, "Paysages Durand").await;
    let (first_apprentice, _) = world.apprentice(tenant, "lea@example.com").await;
    let (second_apprentice, _) = world.apprentice(tenant, "tom@example.com").await;
    let closed_triad = world.triad(tenant, &catalog, first_apprentice).await;
    let open_triad = world.triad(tenant, &catalog, second_apprentice).await;
    let contract = world.contract(tenant, &open_triad, &company).await;
    world
        .usecase
        .close_triad(world.close_triad_command(tenant, &closed_triad, &catalog.reason))
        .await
        .unwrap();
    world
        .usecase
        .archive_closure_reason(
            tenant.manager.subject(),
            tenant.organization_id,
            catalog.reason.id,
        )
        .await
        .unwrap();

    let refused_contract = world
        .usecase
        .close_contract(world.close_contract_command(tenant, &contract, &catalog.reason))
        .await;
    let refused_triad = world
        .usecase
        .close_triad(world.close_triad_command(tenant, &open_triad, &catalog.reason))
        .await;
    let past = world
        .usecase
        .get_triad(tenant.organization_id, closed_triad.id)
        .await
        .unwrap();

    assert!(matches!(refused_contract, Err(CoreError::Conflict(_))));
    assert!(matches!(refused_triad, Err(CoreError::Conflict(_))));
    assert_eq!(
        past.closure_reason_label.as_deref(),
        Some("Fin de formation")
    );

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn contract_rules_period_host_company_tutors_and_files_hold_in_the_database() {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let company = world.company(tenant, "Paysages Durand").await;
    let other_company = world.company(tenant, "Jardins Petit").await;
    let (main_id, _) = world.tutor(tenant, &company, "main@example.com").await;
    let (secondary_id, _) = world.tutor(tenant, &company, "secondary@example.com").await;
    let (extra_id, _) = world.tutor(tenant, &company, "extra@example.com").await;
    let (elsewhere_id, _) = world
        .tutor(tenant, &other_company, "else@example.com")
        .await;
    let (apprentice_id, _) = world.apprentice(tenant, "lea@example.com").await;
    let triad = world.triad(tenant, &catalog, apprentice_id).await;
    let new_contract = |company: &HostCompany, start, end| CreateContractCommand {
        actor: tenant.manager.subject(),
        organization_id: tenant.organization_id,
        triad_id: triad.id,
        host_company_id: company.id,
        kind: CursusContractKind::Apprenticeship,
        number: None,
        comment: None,
        start_date: start,
        end_date: end,
    };

    let backwards = world
        .usecase
        .create_contract(new_contract(&company, day(2026, 9, 2), day(2026, 9, 1)))
        .await;
    world
        .usecase
        .archive_host_company(
            tenant.manager.subject(),
            tenant.organization_id,
            other_company.id,
        )
        .await
        .unwrap();
    let archived_company = world
        .usecase
        .create_contract(new_contract(
            &other_company,
            day(2026, 9, 1),
            day(2028, 8, 31),
        ))
        .await;
    let contract = world.contract(tenant, &triad, &company).await;

    assert!(matches!(backwards, Err(CoreError::Conflict(_))));
    assert!(matches!(archived_company, Err(CoreError::Conflict(_))));

    assert_eq!(contract.contract.comment.as_deref(), Some("**Note**"));
    assert_eq!(contract.contract.duration_months(), 24);

    let set = |tutors: &[(CompanyTutorId, bool)]| SetContractTutorsCommand {
        actor: tenant.manager.subject(),
        organization_id: tenant.organization_id,
        contract_id: contract.contract.id,
        tutors: tutors
            .iter()
            .map(|&(company_tutor_id, is_main)| ContractTutorInput {
                company_tutor_id,
                is_main,
            })
            .collect(),
    };
    let three = world
        .usecase
        .set_contract_tutors(set(&[
            (secondary_id, false),
            (main_id, true),
            (extra_id, false),
        ]))
        .await
        .unwrap();
    let order: Vec<_> = three
        .tutors
        .iter()
        .map(|tutor| (tutor.company_tutor_id, tutor.is_main))
        .collect();
    assert_eq!(
        order,
        [(secondary_id, false), (main_id, true), (extra_id, false)]
    );
    let same = world
        .usecase
        .set_contract_tutors(set(&[(main_id, true), (main_id, false)]))
        .await;
    let no_main = world
        .usecase
        .set_contract_tutors(set(&[(main_id, false), (secondary_id, false)]))
        .await;
    let two_mains = world
        .usecase
        .set_contract_tutors(set(&[(main_id, true), (secondary_id, true)]))
        .await;
    let foreign = world
        .usecase
        .set_contract_tutors(set(&[(elsewhere_id, true)]))
        .await;
    assert!(matches!(same, Err(CoreError::Conflict(_))));
    assert!(matches!(no_main, Err(CoreError::Conflict(_))));
    assert!(matches!(two_mains, Err(CoreError::Conflict(_))));
    assert!(matches!(foreign, Err(CoreError::Conflict(_))));
    let unchanged = world
        .usecase
        .get_contract(tenant.organization_id, contract.contract.id)
        .await
        .unwrap();
    assert_eq!(unchanged.tutors.len(), 3);

    let cleared = world.usecase.set_contract_tutors(set(&[])).await.unwrap();
    assert!(cleared.tutors.is_empty());
    let swapped = world
        .usecase
        .set_contract_tutors(set(&[(secondary_id, true)]))
        .await
        .unwrap();
    assert_eq!(swapped.tutors.len(), 1);
    assert_eq!(swapped.tutors[0].company_tutor_id, secondary_id);
    assert!(swapped.tutors[0].is_main);

    let removal = |id| RemoveCompanyTutorCommand {
        actor: tenant.manager.subject(),
        organization_id: tenant.organization_id,
        id,
    };
    let linked = world
        .usecase
        .remove_company_tutor(removal(secondary_id))
        .await;
    assert!(matches!(linked, Err(CoreError::Conflict(_))));
    world
        .usecase
        .remove_company_tutor(removal(main_id))
        .await
        .unwrap();

    let moved = world
        .usecase
        .update_contract(UpdateContractCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            id: contract.contract.id,
            host_company_id: world.company(tenant, "Autre").await.id,
            kind: CursusContractKind::Professionalization,
            number: None,
            comment: None,
            start_date: day(2026, 9, 1),
            end_date: day(2028, 8, 31),
        })
        .await;
    assert!(matches!(moved, Err(CoreError::Conflict(_))));

    let missing = world
        .usecase
        .get_contract_file_key(
            tenant.organization_id,
            contract.contract.id,
            ContractFileKind::MissionSheet,
        )
        .await;
    assert!(matches!(missing, Err(CoreError::NotFound)));
    let attach = |file_key: Option<&str>| SetContractFileCommand {
        actor: tenant.manager.subject(),
        organization_id: tenant.organization_id,
        contract_id: contract.contract.id,
        kind: ContractFileKind::MissionSheet,
        file_key: file_key.map(str::to_owned),
    };
    let attached = world
        .usecase
        .set_contract_file(attach(Some("uploads/mission-1")))
        .await
        .unwrap();
    assert!(attached.contract.mission_sheet_file_key.is_some());
    let key = world
        .usecase
        .get_contract_file_key(
            tenant.organization_id,
            contract.contract.id,
            ContractFileKind::MissionSheet,
        )
        .await
        .unwrap();
    assert_eq!(key.as_str(), "uploads/mission-1");
    let invalid = world
        .usecase
        .set_contract_file(attach(Some("/absolute")))
        .await;
    assert!(matches!(invalid, Err(CoreError::Conflict(_))));
    world.usecase.set_contract_file(attach(None)).await.unwrap();
    let cleared = world
        .usecase
        .get_contract_file_key(
            tenant.organization_id,
            contract.contract.id,
            ContractFileKind::MissionSheet,
        )
        .await;
    assert!(matches!(cleared, Err(CoreError::NotFound)));

    world.cleanup().await;
}

fn by_closure(closed: Option<bool>) -> TriadFilter {
    TriadFilter {
        closed,
        ..TriadFilter::default()
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_operator_list_carries_names_and_the_current_host_company_and_filters_by_promotion_and_closure()
 {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let company = world.company(tenant, "Paysages Durand").await;
    let (first_apprentice, _) = world.apprentice(tenant, "lea@example.com").await;
    let (second_apprentice, _) = world.apprentice(tenant, "tom@example.com").await;
    let open = world.triad(tenant, &catalog, first_apprentice).await;
    let closed = world.triad(tenant, &catalog, second_apprentice).await;
    world.contract(tenant, &open, &company).await;
    world
        .usecase
        .close_triad(world.close_triad_command(tenant, &closed, &catalog.reason))
        .await
        .unwrap();

    let (all, total) = world
        .usecase
        .list_triads(
            tenant.organization_id,
            TriadFilter {
                promotion_id: Some(catalog.promotion.id),
                ..TriadFilter::default()
            },
            10,
            0,
        )
        .await
        .unwrap();
    let (only_open, open_total) = world
        .usecase
        .list_triads(tenant.organization_id, by_closure(Some(false)), 10, 0)
        .await
        .unwrap();
    let (only_closed, closed_total) = world
        .usecase
        .list_triads(tenant.organization_id, by_closure(Some(true)), 10, 0)
        .await
        .unwrap();
    let (elsewhere, elsewhere_total) = world
        .usecase
        .list_triads(
            tenant.organization_id,
            TriadFilter {
                promotion_id: Some(crate::PromotionId(Uuid::new_v4())),
                ..TriadFilter::default()
            },
            10,
            0,
        )
        .await
        .unwrap();
    let (paged, paged_total) = world
        .usecase
        .list_triads(tenant.organization_id, TriadFilter::default(), 1, 1)
        .await
        .unwrap();

    assert_eq!(total, 2);
    assert_eq!((open_total, closed_total, elsewhere_total), (1, 1, 0));
    assert!(elsewhere.is_empty());
    assert_eq!((paged.len(), paged_total), (1, 2));
    assert_eq!(only_open[0].triad.id, open.id);
    assert_eq!(only_closed[0].triad.id, closed.id);
    let summary = all.iter().find(|row| row.triad.id == open.id).unwrap();
    assert_eq!(summary.apprentice_name, "Martin Léa");
    assert_eq!(summary.promotion_label, "BTS 2026");
    assert_eq!(summary.pedagogic_tutor_name.as_deref(), Some("Member"));
    assert_eq!(
        summary.current_host_company_name.as_deref(),
        Some("Paysages Durand")
    );
    let finished = all.iter().find(|row| row.triad.id == closed.id).unwrap();
    assert_eq!(finished.current_host_company_name, None);

    let search = |q: &'static str| {
        world.usecase.list_triads(
            tenant.organization_id,
            TriadFilter {
                search: SearchPattern::parse(Some(q)),
                ..TriadFilter::default()
            },
            10,
            0,
        )
    };
    let (by_company, by_company_total) = search("paysages dur").await.unwrap();
    assert_eq!(by_company_total, 1);
    assert_eq!(by_company[0].triad.id, open.id);
    assert_eq!(search("bts 20").await.unwrap().1, 2);
    assert_eq!(search("léa mart").await.unwrap().1, 2);
    assert_eq!(search("nobody").await.unwrap().1, 0);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_11_a_participant_who_is_not_a_member_is_refused_on_operator_use_cases_and_scope_needs_an_active_participant()
 {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let (apprentice_id, apprentice) = world.apprentice(tenant, "lea@example.com").await;
    let triad = world.triad(tenant, &catalog, apprentice_id).await;
    let stranger = world.person("stranger").await;

    let write = world
        .usecase
        .create_triad(CreateTriadCommand {
            actor: apprentice.subject(),
            organization_id: tenant.organization_id,
            promotion_id: catalog.promotion.id,
            apprentice_id,
            pedagogic_tutor_member_id: None,
        })
        .await;
    let close = world
        .usecase
        .close_triad(CloseTriadCommand {
            actor: apprentice.subject(),
            ..world.close_triad_command(tenant, &triad, &catalog.reason)
        })
        .await;
    let no_participant = world
        .usecase
        .participant_scope(stranger.id(), tenant.organization_id)
        .await;
    let participant_id = world
        .usecase
        .participant_scope(apprentice.id(), tenant.organization_id)
        .await
        .unwrap()
        .participant_id;
    world
        .usecase
        .deactivate_participant(
            tenant.manager.subject(),
            tenant.organization_id,
            participant_id,
        )
        .await
        .unwrap();
    let deactivated = world
        .usecase
        .list_my_triads(apprentice.id(), tenant.organization_id)
        .await;

    assert!(matches!(write, Err(CoreError::Forbidden { .. })));
    assert!(matches!(close, Err(CoreError::Forbidden { .. })));
    assert!(matches!(no_participant, Err(CoreError::Forbidden { .. })));
    assert!(matches!(deactivated, Err(CoreError::Forbidden { .. })));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_12_organizations_never_see_each_others_triads_and_me_routes_are_scoped_to_one_organization()
 {
    let world = World::new().await;
    let catalog = world.catalog(&world.first).await;
    let (apprentice_id, apprentice) = world.apprentice(&world.first, "lea@example.com").await;
    let triad = world.triad(&world.first, &catalog, apprentice_id).await;

    let read = world
        .usecase
        .get_triad(world.second.organization_id, triad.id)
        .await;
    let (listed, total) = world
        .usecase
        .list_triads(world.second.organization_id, TriadFilter::default(), 10, 0)
        .await
        .unwrap();
    let foreign_write = world
        .usecase
        .create_triad(CreateTriadCommand {
            actor: world.second.manager.subject(),
            organization_id: world.first.organization_id,
            promotion_id: catalog.promotion.id,
            apprentice_id,
            pedagogic_tutor_member_id: None,
        })
        .await;
    let foreign_close = world
        .usecase
        .close_triad(CloseTriadCommand {
            actor: world.second.manager.subject(),
            organization_id: world.second.organization_id,
            id: triad.id,
            closed_at: day(2027, 6, 30),
            closure_reason_id: world.reason(&world.second, "Fin").await.id,
            closure_note: None,
        })
        .await;
    let me_elsewhere = world
        .usecase
        .list_my_triads(apprentice.id(), world.second.organization_id)
        .await;
    let me_detail_elsewhere = world
        .usecase
        .get_my_triad(apprentice.id(), world.second.organization_id, triad.id)
        .await;

    assert!(matches!(read, Err(CoreError::NotFound)));
    assert_eq!((listed.len(), total), (0, 0));
    assert!(matches!(foreign_write, Err(CoreError::Forbidden { .. })));
    assert!(matches!(foreign_close, Err(CoreError::NotFound)));
    assert!(matches!(me_elsewhere, Err(CoreError::Forbidden { .. })));
    assert!(matches!(
        me_detail_elsewhere,
        Err(CoreError::Forbidden { .. })
    ));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn my_spaces_lists_every_organization_with_an_active_participant_and_flags_membership() {
    let world = World::new().await;
    let (_, outsider) = world.apprentice(&world.first, "lea@example.com").await;
    let detail = world
        .usecase
        .create_participant(CreateParticipantCommand {
            actor: world.second.manager.subject(),
            organization_id: world.second.organization_id,
            last_name: "Manager".to_owned(),
            first_name: None,
            email: "manager-as-participant@example.com".to_owned(),
            phone: None,
            apprentice: Some(ApprenticeInput {
                birth_date: None,
                ine_code: None,
                address: None,
            }),
            company_tutor: None,
        })
        .await
        .unwrap();
    let invite = world
        .usecase
        .invite_participant(
            world.second.manager.subject(),
            world.second.organization_id,
            detail.participant.id,
        )
        .await
        .unwrap();
    let member_space = world
        .usecase
        .acting_as(world.second.manager.id())
        .accept_participant_invitation(AcceptParticipantInvitationCommand {
            token: invite.token,
            user_id: world.second.manager.id(),
        })
        .await
        .unwrap();

    let outsider_spaces = world
        .usecase
        .list_my_cursus_spaces(outsider.id())
        .await
        .unwrap();
    let member_spaces = world
        .usecase
        .list_my_cursus_spaces(world.second.manager.id())
        .await
        .unwrap();
    let nobody = world
        .usecase
        .list_my_cursus_spaces(world.first.reader.id())
        .await
        .unwrap();

    assert!(member_space.is_member);
    assert_eq!(outsider_spaces.len(), 1);
    assert_eq!(
        outsider_spaces[0].organization_id,
        world.first.organization_id
    );
    assert!(!outsider_spaces[0].is_member);
    assert_eq!(member_spaces.len(), 1);
    assert_eq!(
        member_spaces[0].organization_id,
        world.second.organization_id
    );
    assert!(member_spaces[0].is_member);
    assert!(nobody.is_empty());

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_company_tutor_with_a_contract_link_cannot_be_removed_and_one_without_can() {
    let world = World::new().await;
    let tenant = &world.first;
    let catalog = world.catalog(tenant).await;
    let company = world.company(tenant, "Paysages Durand").await;
    let (linked_id, _) = world.tutor(tenant, &company, "linked@example.com").await;
    let (apprentice_id, _) = world.apprentice(tenant, "lea@example.com").await;
    let triad = world.triad(tenant, &catalog, apprentice_id).await;
    let contract = world.contract(tenant, &triad, &company).await;
    world.tutors(tenant, &contract, Some(linked_id)).await;
    world
        .usecase
        .close_contract(world.close_contract_command(tenant, &contract, &catalog.reason))
        .await
        .unwrap();
    let spare = world
        .usecase
        .create_participant(CreateParticipantCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            last_name: "Spare".to_owned(),
            first_name: None,
            email: "spare@example.com".to_owned(),
            phone: None,
            apprentice: None,
            company_tutor: Some(CompanyTutorInput {
                host_company_id: company.id,
                job_title: None,
            }),
        })
        .await
        .unwrap();

    let removal = |id| RemoveCompanyTutorCommand {
        actor: tenant.manager.subject(),
        organization_id: tenant.organization_id,
        id,
    };
    let refused = world.usecase.remove_company_tutor(removal(linked_id)).await;
    world
        .usecase
        .remove_company_tutor(removal(spare.company_tutors[0].tutor.id))
        .await
        .unwrap();

    assert!(matches!(refused, Err(CoreError::Conflict(_))));

    world.cleanup().await;
}
