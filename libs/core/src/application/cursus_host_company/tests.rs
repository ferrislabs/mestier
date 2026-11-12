use authz::Subject;
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    ArchivedFilter, CreateHostCompanyCommand, HostCompany, HostCompanyAddress, HostCompanyFilter,
    Permissions, SearchPattern, UpdateHostCompanyCommand,
    application::{
        MestierUseCase, default_authorizer, policy,
        test_support::{purge, scratch_pool},
    },
    infrastructure::realtime::EventHub,
};

static HOST_COMPANY_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_cursus_host_company_test_", &HOST_COMPANY_SCRATCH).await
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
        Permissions::VIEW_CURSUS | Permissions::MANAGE_CURSUS_TRIADS,
    )
    .await;
    seed_member(pool, organization_id, reader, Permissions::VIEW_CURSUS).await;

    Tenant {
        organization_id,
        manager: Person { user_id: manager },
        reader: Person { user_id: reader },
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

    async fn cleanup(self) {
        for tenant in [&self.first, &self.second] {
            for statement in [
                "DELETE FROM cursus.host_company WHERE org_id = $1",
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
    }
}

fn all() -> HostCompanyFilter {
    HostCompanyFilter {
        archived: ArchivedFilter::All,
        search: None,
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn host_companies_filter_by_archive_state_and_search_name_or_siret() {
    let world = World::new().await;
    let tenant = &world.first;
    let kept = world.company(tenant, "Paysages Durand").await;
    let gone = world.company(tenant, "Jardins Martin").await;
    let sited = world
        .usecase
        .update_host_company(UpdateHostCompanyCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            id: kept.id,
            name: kept.name.clone(),
            group_name: None,
            siret: Some("55210055400013".to_owned()),
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
        .unwrap();
    world
        .usecase
        .archive_host_company(tenant.manager.subject(), tenant.organization_id, gone.id)
        .await
        .unwrap();

    let (archived, archived_total) = world
        .usecase
        .list_host_companies(
            tenant.organization_id,
            HostCompanyFilter {
                archived: ArchivedFilter::Archived,
                search: None,
            },
            10,
            0,
        )
        .await
        .unwrap();
    let (active, active_total) = world
        .usecase
        .list_host_companies(tenant.organization_id, HostCompanyFilter::default(), 10, 0)
        .await
        .unwrap();
    let (_, all_total) = world
        .usecase
        .list_host_companies(tenant.organization_id, all(), 10, 0)
        .await
        .unwrap();
    let (by_siret, by_siret_total) = world
        .usecase
        .list_host_companies(
            tenant.organization_id,
            HostCompanyFilter {
                archived: ArchivedFilter::All,
                search: SearchPattern::parse(Some(" 552100 ")),
            },
            10,
            0,
        )
        .await
        .unwrap();
    let (by_name, by_name_total) = world
        .usecase
        .list_host_companies(
            tenant.organization_id,
            HostCompanyFilter {
                archived: ArchivedFilter::Archived,
                search: SearchPattern::parse(Some("MARTIN")),
            },
            10,
            0,
        )
        .await
        .unwrap();

    assert_eq!(archived_total, 1);
    assert_eq!(archived[0].id, gone.id);
    assert!(archived[0].archived_at.is_some());
    assert_eq!((active, active_total), (vec![sited.clone()], 1));
    assert_eq!(all_total, 2);
    assert_eq!((by_siret, by_siret_total), (vec![sited], 1));
    assert_eq!(by_name_total, 1);
    assert_eq!(by_name[0].id, gone.id);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_triads_manager_manages_host_companies_and_archived_ones_are_hidden_by_default() {
    let world = World::new().await;
    let tenant = &world.first;
    let company = world.company(tenant, "Paysages Durand").await;

    let updated = world
        .usecase
        .update_host_company(UpdateHostCompanyCommand {
            actor: tenant.manager.subject(),
            organization_id: tenant.organization_id,
            id: company.id,
            name: "Durand SAS".to_owned(),
            group_name: None,
            siret: Some("12345678901234".to_owned()),
            legal_form: Some("SAS".to_owned()),
            naf_code: None,
            capital: None,
            employee_count: Some(8),
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
        .unwrap();
    assert_eq!(updated.name, "Durand SAS");
    assert_eq!(updated.employee_count, Some(8));

    world
        .usecase
        .archive_host_company(tenant.manager.subject(), tenant.organization_id, company.id)
        .await
        .unwrap();

    let (visible, visible_total) = world
        .usecase
        .list_host_companies(tenant.organization_id, HostCompanyFilter::default(), 10, 0)
        .await
        .unwrap();
    let (all, all_total) = world
        .usecase
        .list_host_companies(tenant.organization_id, all(), 10, 0)
        .await
        .unwrap();
    assert_eq!((visible, visible_total), (vec![], 0));
    assert_eq!(all_total, 1);
    assert!(all[0].archived_at.is_some());

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn without_the_triads_bit_every_host_company_write_is_forbidden() {
    let world = World::new().await;
    let tenant = &world.first;
    let company = world.company(tenant, "Paysages Durand").await;

    let created = world
        .usecase
        .create_host_company(CreateHostCompanyCommand {
            actor: tenant.reader.subject(),
            organization_id: tenant.organization_id,
            name: "Intrus".to_owned(),
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
        .await;
    assert!(matches!(created, Err(CoreError::Forbidden { .. })));

    let archived = world
        .usecase
        .archive_host_company(tenant.reader.subject(), tenant.organization_id, company.id)
        .await;
    assert!(matches!(archived, Err(CoreError::Forbidden { .. })));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn spec_cur_12_one_organization_never_sees_or_touches_another_ones_host_companies() {
    let world = World::new().await;
    let company = world.company(&world.first, "Paysages Durand").await;

    let read = world
        .usecase
        .get_host_company(world.second.organization_id, company.id)
        .await;
    assert!(matches!(read, Err(CoreError::NotFound)));

    let (listed, total) = world
        .usecase
        .list_host_companies(world.second.organization_id, all(), 10, 0)
        .await
        .unwrap();
    assert_eq!((listed, total), (vec![], 0));

    let archived = world
        .usecase
        .archive_host_company(
            world.second.manager.subject(),
            world.second.organization_id,
            company.id,
        )
        .await;
    assert!(matches!(archived, Err(CoreError::NotFound)));

    let foreign_write = world
        .usecase
        .create_host_company(CreateHostCompanyCommand {
            actor: world.second.manager.subject(),
            organization_id: world.first.organization_id,
            name: "Intrus".to_owned(),
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
        .await;
    assert!(matches!(foreign_write, Err(CoreError::Forbidden { .. })));

    let intact = world
        .usecase
        .get_host_company(world.first.organization_id, company.id)
        .await
        .unwrap();
    assert_eq!(intact.archived_at, None);

    world.cleanup().await;
}

fn address(line1: &str, is_main: bool) -> HostCompanyAddress {
    HostCompanyAddress {
        label: None,
        line1: line1.to_owned(),
        line2: None,
        postal_code: Some("69000".to_owned()),
        city: Some("Lyon".to_owned()),
        is_main,
    }
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn addresses_are_replaced_as_a_whole_ordered_and_returned_by_every_read() {
    let world = World::new().await;
    let tenant = &world.first;
    let company = world.company(tenant, "Paysages Durand").await;
    assert!(company.addresses.is_empty());

    let update = |addresses: Vec<HostCompanyAddress>| UpdateHostCompanyCommand {
        actor: tenant.manager.subject(),
        organization_id: tenant.organization_id,
        id: company.id,
        name: company.name.clone(),
        group_name: Some("Groupe Durand".to_owned()),
        siret: None,
        legal_form: None,
        naf_code: None,
        capital: Some("10 000 EUR".to_owned()),
        employee_count: None,
        group_employee_count: Some(40),
        director_name: None,
        phone: Some("04 00 00 00 00".to_owned()),
        email: Some("contact@durand.fr".to_owned()),
        website: Some("durand.fr".to_owned()),
        activity_domains: Some("Paysage".to_owned()),
        missions: Some("- tonte\n- taille".to_owned()),
        addresses,
    };

    let two = world
        .usecase
        .update_host_company(update(vec![
            address("2 rue B", false),
            address("1 rue A", true),
        ]))
        .await
        .unwrap();
    let lines: Vec<_> = two.addresses.iter().map(|a| a.line1.as_str()).collect();
    assert_eq!(lines, ["2 rue B", "1 rue A"]);
    assert_eq!(two.group_employee_count, Some(40));
    assert_eq!(two.missions.as_deref(), Some("- tonte\n- taille"));

    let (listed, _) = world
        .usecase
        .list_host_companies(tenant.organization_id, HostCompanyFilter::default(), 10, 0)
        .await
        .unwrap();
    assert_eq!(listed[0].addresses, two.addresses);

    let one = world
        .usecase
        .update_host_company(update(vec![address("9 rue C", true)]))
        .await
        .unwrap();
    let fetched = world
        .usecase
        .get_host_company(tenant.organization_id, company.id)
        .await
        .unwrap();
    assert_eq!(one.addresses.len(), 1);
    assert_eq!(fetched.addresses, one.addresses);

    let refused = world
        .usecase
        .update_host_company(update(vec![address("a", true), address("b", true)]))
        .await;
    assert!(matches!(refused, Err(CoreError::Conflict(_))));
    let unchanged = world
        .usecase
        .get_host_company(tenant.organization_id, company.id)
        .await
        .unwrap();
    assert_eq!(unchanged.addresses, one.addresses);

    let cleared = world
        .usecase
        .update_host_company(update(Vec::new()))
        .await
        .unwrap();
    assert!(cleared.addresses.is_empty());

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_database_refuses_a_second_main_address_for_one_company() {
    let world = World::new().await;
    let tenant = &world.first;
    let company = world.company(tenant, "Paysages Durand").await;

    let insert = |is_main: bool| {
        sqlx::query(
            "INSERT INTO cursus.host_company_address \
             (id, org_id, host_company_id, line1, is_main, position) \
             VALUES ($1, $2, $3, 'rue', $4, 0)",
        )
        .bind(generate_uuid_v7())
        .bind(tenant.organization_id.0)
        .bind(company.id.0)
        .bind(is_main)
        .execute(&world.pool)
    };
    insert(true).await.unwrap();
    insert(false).await.unwrap();
    let second_main = insert(true).await;

    assert!(second_main.is_err());

    world.cleanup().await;
}
