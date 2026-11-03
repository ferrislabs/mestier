use authz::Subject;
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    AddBadgeCommand, AssignTicketCommand, Badge, BadgeTone, CreateTicketCommand, CreatedTicket,
    CustomFields, GetTicketCommand, ListTicketsCommand, MemberId, Permissions, RemoveBadgeCommand,
    Ticket, TicketFilter, TicketId, TicketNumber, TicketPage, TicketRepository, TicketStatus,
    TransitionTicketCommand,
    application::{
        MestierUseCase, default_authorizer, policy,
        test_support::{purge, scratch_pool},
    },
    infrastructure::{
        postgres::with_tx, realtime::EventHub, support_ticket::postgres::PgSupportTicketRepository,
    },
};

static SUPPORT_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_support_test_", &SUPPORT_SCRATCH).await
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

struct World {
    pool: PgPool,
    usecase: MestierUseCase,
    organization_id: OrganizationId,
    manager: Person,
    plain: Person,
    spare: Person,
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
    label: &str,
    role_id: Option<Uuid>,
) -> Person {
    let member_id = generate_uuid_v7();
    sqlx::query(
        "INSERT INTO organization_members (id, organization_id, user_id, last_name) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(member_id)
    .bind(organization_id.0)
    .bind(user_id)
    .bind(label)
    .execute(pool)
    .await
    .unwrap();
    if let Some(role_id) = role_id {
        sqlx::query("INSERT INTO member_roles (id, member_id, role_id) VALUES ($1, $2, $3)")
            .bind(generate_uuid_v7())
            .bind(member_id)
            .bind(role_id)
            .execute(pool)
            .await
            .unwrap();
    }
    Person {
        user_id,
        member_id: MemberId(member_id),
    }
}

async fn seed_person(
    pool: &PgPool,
    organization_id: OrganizationId,
    label: &str,
    role_id: Option<Uuid>,
) -> Person {
    let user_id = seed_user(pool, label).await;
    seed_member(pool, organization_id, user_id, label, role_id).await
}

fn fields(value: Value) -> CustomFields {
    CustomFields::try_from(value).unwrap()
}

fn numbers(page: &TicketPage) -> Vec<i32> {
    page.items.iter().map(|ticket| ticket.number.0).collect()
}

impl World {
    async fn new(pool: &PgPool) -> Self {
        let manager_user = seed_user(pool, "Manager").await;
        let organization_id = OrganizationId(generate_uuid_v7());
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id.0)
            .bind("Treyve Paysages")
            .bind(format!("org-{}", organization_id.0))
            .bind(manager_user)
            .execute(pool)
            .await
            .unwrap();

        let role_id = generate_uuid_v7();
        sqlx::query(
            "INSERT INTO roles (id, organization_id, name, permissions) VALUES ($1, $2, $3, $4)",
        )
        .bind(role_id)
        .bind(organization_id.0)
        .bind("Support")
        .bind(Permissions::MANAGE_SUPPORT.bits())
        .execute(pool)
        .await
        .unwrap();

        Self {
            pool: pool.clone(),
            usecase: MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new()),
            organization_id,
            manager: seed_member(
                pool,
                organization_id,
                manager_user,
                "Manager",
                Some(role_id),
            )
            .await,
            plain: seed_person(pool, organization_id, "Plain", None).await,
            spare: seed_person(pool, organization_id, "Spare", None).await,
        }
    }

    async fn create(
        &self,
        subject: &str,
        external_id: Option<&str>,
        custom_fields: Value,
    ) -> CreatedTicket {
        self.usecase
            .create_support_ticket(CreateTicketCommand {
                actor: self.manager.subject(),
                organization_id: self.organization_id,
                subject: subject.to_owned(),
                body: format!("Corps de {subject}"),
                custom_fields: fields(custom_fields),
                external_id: external_id.map(str::to_owned),
            })
            .await
            .unwrap()
    }

    async fn ticket(&self, subject: &str) -> Ticket {
        self.create(subject, None, json!({})).await.ticket
    }

    async fn transition(&self, ticket: &Ticket, status: TicketStatus) -> Ticket {
        self.usecase
            .transition_support_ticket(TransitionTicketCommand {
                actor: self.manager.subject(),
                organization_id: self.organization_id,
                ticket_id: ticket.id,
                status,
            })
            .await
            .unwrap()
    }

    async fn assign(&self, ticket: &Ticket, assignee: Option<&Person>) -> Ticket {
        self.usecase
            .assign_support_ticket(AssignTicketCommand {
                actor: self.manager.subject(),
                organization_id: self.organization_id,
                ticket_id: ticket.id,
                assignee: assignee.map(|person| person.member_id),
            })
            .await
            .unwrap()
    }

    async fn badge(&self, ticket: &Ticket, label: &str, tone: BadgeTone) -> Ticket {
        self.usecase
            .add_support_ticket_badge(AddBadgeCommand {
                actor: self.manager.subject(),
                organization_id: self.organization_id,
                ticket_id: ticket.id,
                badge: Badge::new(label, tone).unwrap(),
            })
            .await
            .unwrap()
    }

    async fn get(&self, id: TicketId) -> Result<Ticket, CoreError> {
        self.usecase
            .get_support_ticket(GetTicketCommand {
                organization_id: self.organization_id,
                ticket_id: id,
            })
            .await
    }

    async fn list(&self, filter: TicketFilter) -> TicketPage {
        self.list_window(filter, 0, 50).await
    }

    async fn list_window(&self, filter: TicketFilter, offset: u64, limit: u64) -> TicketPage {
        self.usecase
            .list_support_tickets(ListTicketsCommand {
                organization_id: self.organization_id,
                filter,
                offset,
                limit,
            })
            .await
            .unwrap()
    }

    async fn event_names(&self) -> Vec<String> {
        sqlx::query_scalar(
            "SELECT name FROM automation.event \
             WHERE org_id = $1 AND name LIKE 'ticket.%' \
             ORDER BY occurred_at, id",
        )
        .bind(self.organization_id.0)
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    async fn ticket_rows(&self) -> i64 {
        sqlx::query_scalar("SELECT COUNT(*) FROM support_tickets WHERE org_id = $1")
            .bind(self.organization_id.0)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn cleanup(self) {
        for statement in [
            "DELETE FROM automation.event WHERE org_id = $1",
            "DELETE FROM support_tickets WHERE org_id = $1",
            "DELETE FROM support_ticket_counters WHERE org_id = $1",
            "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
            "DELETE FROM roles WHERE organization_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(&self.pool, statement, self.organization_id.0).await;
        }
        for person in [&self.manager, &self.plain, &self.spare] {
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
async fn create_persists_the_ticket_and_records_ticket_created_in_the_event_log() {
    let world = World::new(&pool().await).await;

    let created = world
        .usecase
        .create_support_ticket(CreateTicketCommand {
            actor: world.manager.subject(),
            organization_id: world.organization_id,
            subject: "  Impossible de déployer  ".to_owned(),
            body: "Le déploiement dpl_8f2a91 échoue.".to_owned(),
            custom_fields: fields(json!({ "deployment_id": "dpl_8f2a91", "seats": 12 })),
            external_id: Some(" ZD-48213 ".to_owned()),
        })
        .await
        .unwrap();

    assert!(created.created);
    let ticket = &created.ticket;
    assert_eq!(ticket.organization_id, world.organization_id);
    assert_eq!(ticket.number, TicketNumber(1));
    assert_eq!(ticket.subject, "Impossible de déployer");
    assert_eq!(ticket.body, "Le déploiement dpl_8f2a91 échoue.");
    assert_eq!(ticket.status, TicketStatus::New);
    assert_eq!(ticket.assignee, None);
    assert_eq!(ticket.assigned_at, None);
    assert!(ticket.badges.is_empty());
    assert_eq!(ticket.external_id.as_deref(), Some("ZD-48213"));
    assert_eq!(
        ticket.custom_fields.as_map().get("deployment_id"),
        Some(&json!("dpl_8f2a91"))
    );
    assert_eq!(ticket.custom_fields.as_map().get("seats"), Some(&json!(12)));
    assert_eq!(world.get(ticket.id).await.unwrap(), *ticket);
    assert_eq!(world.ticket_rows().await, 1);

    let events: Vec<(String, Option<Uuid>, Value)> = sqlx::query_as(
        "SELECT name, subject_id, payload FROM automation.event \
         WHERE org_id = $1 AND name LIKE 'ticket.%'",
    )
    .bind(world.organization_id.0)
    .fetch_all(&world.pool)
    .await
    .unwrap();
    assert_eq!(events.len(), 1);
    let (name, subject_id, payload) = &events[0];
    assert_eq!(name, "ticket.created");
    assert_eq!(*subject_id, Some(ticket.id.0));
    assert_eq!(payload["ticket"]["id"], json!(ticket.id.0));
    assert_eq!(payload["ticket"]["number"], json!(1));
    assert_eq!(
        payload["ticket"]["subject"],
        json!("Impossible de déployer")
    );
    assert_eq!(payload["ticket"]["status"], json!("new"));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn two_concurrent_creations_get_distinct_numbers() {
    let world = World::new(&pool().await).await;

    let (first, second) = tokio::join!(
        world.create("Premier", None, json!({})),
        world.create("Second", None, json!({}))
    );

    assert!(first.created);
    assert!(second.created);
    assert_ne!(first.ticket.id, second.ticket.id);
    let mut assigned = [first.ticket.number, second.ticket.number];
    assigned.sort();
    assert_eq!(assigned, [TicketNumber(1), TicketNumber(2)]);
    assert_eq!(world.ticket_rows().await, 2);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn two_concurrent_creations_with_the_same_external_id_yield_one_ticket() {
    let world = World::new(&pool().await).await;

    let (first, second) = tokio::join!(
        world.create("Doublon", Some("ZD-1"), json!({})),
        world.create("Doublon", Some("ZD-1"), json!({}))
    );

    assert_eq!(first.ticket.id, second.ticket.id);
    assert_eq!(first.ticket, second.ticket);
    assert_eq!(
        [first.created, second.created]
            .into_iter()
            .filter(|created| *created)
            .count(),
        1
    );
    assert_eq!(world.ticket_rows().await, 1);
    assert_eq!(world.event_names().await, vec!["ticket.created"]);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_repeated_external_id_returns_the_existing_ticket_without_a_second_event() {
    let world = World::new(&pool().await).await;

    let first = world.create("Première", Some("ZD-7"), json!({})).await;
    let again = world.create("Autre sujet", Some("ZD-7"), json!({})).await;

    assert!(first.created);
    assert!(!again.created);
    assert_eq!(again.ticket, first.ticket);
    assert_eq!(again.ticket.subject, "Première");
    assert_eq!(world.ticket_rows().await, 1);
    assert_eq!(world.event_names().await, vec!["ticket.created"]);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_same_external_id_in_two_organizations_makes_two_tickets() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;

    let in_acme = acme.create("Panne", Some("ZD-9"), json!({})).await;
    let in_globex = globex.create("Panne", Some("ZD-9"), json!({})).await;

    assert!(in_acme.created);
    assert!(in_globex.created);
    assert_ne!(in_acme.ticket.id, in_globex.ticket.id);
    assert_eq!(in_acme.ticket.organization_id, acme.organization_id);
    assert_eq!(in_globex.ticket.organization_id, globex.organization_id);

    acme.cleanup().await;
    globex.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn numbers_restart_per_organization() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;

    let acme_first = acme.ticket("A1").await;
    let acme_second = acme.ticket("A2").await;
    let globex_first = globex.ticket("G1").await;
    let acme_third = acme.ticket("A3").await;
    let globex_second = globex.ticket("G2").await;

    assert_eq!(
        [acme_first.number, acme_second.number, acme_third.number],
        [TicketNumber(1), TicketNumber(2), TicketNumber(3)]
    );
    assert_eq!(
        [globex_first.number, globex_second.number],
        [TicketNumber(1), TicketNumber(2)]
    );

    acme.cleanup().await;
    globex.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn list_filters_on_a_custom_field_key_and_text_value() {
    let world = World::new(&pool().await).await;
    world
        .create("T1", None, json!({ "team": "tech", "seats": 12 }))
        .await;
    world
        .create("T2", None, json!({ "team": "sales", "seats": 8 }))
        .await;
    world
        .create("T3", None, json!({ "team": "tech", "seats": "12" }))
        .await;
    world.create("T4", None, json!({})).await;

    let by = |key: &str, value: &str| TicketFilter {
        custom_field: Some((key.to_owned(), value.to_owned())),
        ..TicketFilter::default()
    };

    let tech = world.list(by("team", "tech")).await;
    assert_eq!(numbers(&tech), vec![3, 1]);
    assert_eq!(tech.total, 2);

    let twelve = world.list(by("seats", "12")).await;
    assert_eq!(numbers(&twelve), vec![3, 1]);
    assert_eq!(twelve.total, 2);

    let eight = world.list(by("seats", "8")).await;
    assert_eq!(numbers(&eight), vec![2]);

    assert!(world.list(by("team", "Tech")).await.items.is_empty());
    assert!(world.list(by("team", "12")).await.items.is_empty());
    assert!(world.list(by("plan", "pro")).await.items.is_empty());
    assert_eq!(world.list(TicketFilter::default()).await.total, 4);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn list_filters_on_status_badge_and_assignee() {
    let world = World::new(&pool().await).await;
    let first = world.ticket("T1").await;
    let second = world.ticket("T2").await;
    let third = world.ticket("T3").await;
    let fourth = world.ticket("T4").await;

    world.badge(&first, "VIP", BadgeTone::Brand).await;
    world.assign(&first, Some(&world.manager)).await;
    world.transition(&second, TicketStatus::InProgress).await;
    world.badge(&second, "VIP", BadgeTone::Error).await;
    world.assign(&second, Some(&world.plain)).await;
    world.transition(&third, TicketStatus::Resolved).await;
    world.badge(&third, "Bloquant", BadgeTone::Error).await;
    world.transition(&fourth, TicketStatus::InProgress).await;
    world.assign(&fourth, Some(&world.manager)).await;

    let statuses = |statuses: &[TicketStatus]| TicketFilter {
        statuses: statuses.to_vec(),
        ..TicketFilter::default()
    };
    let assignee = |person: &Person| TicketFilter {
        assignee: Some(person.member_id),
        ..TicketFilter::default()
    };
    let badge = |label: &str| TicketFilter {
        badge: Some(label.to_owned()),
        ..TicketFilter::default()
    };

    let open = world
        .list(statuses(&[TicketStatus::New, TicketStatus::InProgress]))
        .await;
    assert_eq!(numbers(&open), vec![4, 2, 1]);
    assert_eq!(open.total, 3);
    assert_eq!(
        numbers(&world.list(statuses(&[TicketStatus::Resolved])).await),
        vec![3]
    );
    assert!(
        world
            .list(statuses(&[TicketStatus::WaitingCustomer]))
            .await
            .items
            .is_empty()
    );
    assert_eq!(world.list(statuses(&[])).await.total, 4);

    assert_eq!(
        numbers(&world.list(assignee(&world.manager)).await),
        vec![4, 1]
    );
    assert_eq!(numbers(&world.list(assignee(&world.plain)).await), vec![2]);
    assert!(world.list(assignee(&world.spare)).await.items.is_empty());

    let vip = world.list(badge("VIP")).await;
    assert_eq!(numbers(&vip), vec![2, 1]);
    assert_eq!(vip.total, 2);
    assert_eq!(numbers(&world.list(badge("Bloquant")).await), vec![3]);
    assert!(world.list(badge("vip")).await.items.is_empty());
    assert!(world.list(badge("Inconnu")).await.items.is_empty());

    let in_progress_of_manager = TicketFilter {
        statuses: vec![TicketStatus::InProgress],
        assignee: Some(world.manager.member_id),
        ..TicketFilter::default()
    };
    assert_eq!(numbers(&world.list(in_progress_of_manager).await), vec![4]);
    let in_progress_vip = TicketFilter {
        statuses: vec![TicketStatus::InProgress],
        badge: Some("VIP".to_owned()),
        ..TicketFilter::default()
    };
    assert_eq!(numbers(&world.list(in_progress_vip).await), vec![2]);
    let manager_vip = TicketFilter {
        assignee: Some(world.manager.member_id),
        badge: Some("VIP".to_owned()),
        ..TicketFilter::default()
    };
    let narrowed = world.list(manager_vip).await;
    assert_eq!(numbers(&narrowed), vec![1]);
    assert_eq!(narrowed.total, 1);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn list_pages_newest_first_and_totals_the_whole_filtered_set() {
    let world = World::new(&pool().await).await;
    for index in 1..=5 {
        world.ticket(&format!("T{index}")).await;
    }

    let page = world.list_window(TicketFilter::default(), 1, 2).await;
    assert_eq!(numbers(&page), vec![4, 3]);
    assert_eq!(page.total, 5);

    let last = world.list_window(TicketFilter::default(), 4, 10).await;
    assert_eq!(numbers(&last), vec![1]);
    assert_eq!(last.total, 5);

    let beyond = world.list_window(TicketFilter::default(), 9, 10).await;
    assert!(beyond.items.is_empty());
    assert_eq!(beyond.total, 5);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn list_is_scoped_by_org_id() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let acme_ticket = acme.ticket("Chez Acme").await;
    let globex_ticket = globex.ticket("Chez Globex").await;

    let acme_page = acme.list(TicketFilter::default()).await;
    let globex_page = globex.list(TicketFilter::default()).await;

    assert_eq!(acme_page.items, vec![acme_ticket.clone()]);
    assert_eq!(acme_page.total, 1);
    assert_eq!(globex_page.items, vec![globex_ticket.clone()]);
    assert_eq!(globex_page.total, 1);
    assert!(matches!(
        acme.get(globex_ticket.id).await,
        Err(CoreError::NotFound)
    ));
    assert_eq!(acme.get(acme_ticket.id).await.unwrap(), acme_ticket);

    acme.cleanup().await;
    globex.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn transition_records_ticket_transitioned_with_from_and_to() {
    let world = World::new(&pool().await).await;
    let created = world.ticket("Impossible de déployer").await;

    let updated = world
        .transition(&created, TicketStatus::WaitingCustomer)
        .await;

    assert_eq!(updated.status, TicketStatus::WaitingCustomer);
    assert!(updated.updated_at > created.updated_at);
    assert_eq!(updated.created_at, created.created_at);
    assert_eq!(updated.number, created.number);
    assert_eq!(
        world.get(created.id).await.unwrap().status,
        TicketStatus::WaitingCustomer
    );
    assert_eq!(
        world.event_names().await,
        vec!["ticket.created", "ticket.transitioned"]
    );
    let payload: Value = sqlx::query_scalar(
        "SELECT payload FROM automation.event WHERE org_id = $1 AND name = 'ticket.transitioned'",
    )
    .bind(world.organization_id.0)
    .fetch_one(&world.pool)
    .await
    .unwrap();
    assert_eq!(payload["from"], json!("new"));
    assert_eq!(payload["to"], json!("waiting_customer"));
    assert_eq!(payload["ticket"]["id"], json!(created.id.0));

    let unchanged = world
        .transition(&updated, TicketStatus::WaitingCustomer)
        .await;
    assert_eq!(unchanged, updated);
    assert_eq!(world.event_names().await.len(), 2);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_without_manage_support_cannot_transition() {
    let world = World::new(&pool().await).await;
    let created = world.ticket("Impossible de déployer").await;

    let outcome = world
        .usecase
        .transition_support_ticket(TransitionTicketCommand {
            actor: world.plain.subject(),
            organization_id: world.organization_id,
            ticket_id: created.id,
            status: TicketStatus::Resolved,
        })
        .await;

    assert!(matches!(outcome, Err(CoreError::Forbidden { .. })));
    assert_eq!(world.get(created.id).await.unwrap(), created);
    assert_eq!(world.event_names().await, vec!["ticket.created"]);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_system_subject_can_assign_and_add_badges() {
    let world = World::new(&pool().await).await;
    let created = world.ticket("Impossible de déployer").await;

    let assigned = world
        .usecase
        .assign_support_ticket(AssignTicketCommand {
            actor: Subject::system(),
            organization_id: world.organization_id,
            ticket_id: created.id,
            assignee: Some(world.plain.member_id),
        })
        .await
        .unwrap();
    assert_eq!(assigned.assignee, Some(world.plain.member_id));
    assert!(assigned.assigned_at.is_some());
    assert_eq!(assigned.status, TicketStatus::New);

    let badged = world
        .usecase
        .add_support_ticket_badge(AddBadgeCommand {
            actor: Subject::system(),
            organization_id: world.organization_id,
            ticket_id: created.id,
            badge: Badge::new("VIP", BadgeTone::Brand).unwrap(),
        })
        .await
        .unwrap();
    assert_eq!(
        badged.badges,
        vec![Badge::new("VIP", BadgeTone::Brand).unwrap()]
    );
    assert_eq!(badged.assignee, Some(world.plain.member_id));

    let stored = world.get(created.id).await.unwrap();
    assert_eq!(stored, badged);
    assert_eq!(stored.badges[0].label(), "VIP");
    assert_eq!(stored.badges[0].tone(), BadgeTone::Brand);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn badges_are_replaced_by_label_and_removed_and_assignment_can_be_cleared() {
    let world = World::new(&pool().await).await;
    let created = world.ticket("Impossible de déployer").await;

    world.badge(&created, "Bloquant", BadgeTone::Neutral).await;
    world.badge(&created, "VIP", BadgeTone::Brand).await;
    let replaced = world.badge(&created, "Bloquant", BadgeTone::Error).await;
    assert_eq!(
        replaced.badges,
        vec![
            Badge::new("Bloquant", BadgeTone::Error).unwrap(),
            Badge::new("VIP", BadgeTone::Brand).unwrap()
        ]
    );

    let removed = world
        .usecase
        .remove_support_ticket_badge(RemoveBadgeCommand {
            actor: world.manager.subject(),
            organization_id: world.organization_id,
            ticket_id: created.id,
            label: " Bloquant ".to_owned(),
        })
        .await
        .unwrap();
    assert_eq!(
        removed.badges,
        vec![Badge::new("VIP", BadgeTone::Brand).unwrap()]
    );
    assert_eq!(world.get(created.id).await.unwrap(), removed);

    let assigned = world.assign(&created, Some(&world.manager)).await;
    assert_eq!(assigned.assignee, Some(world.manager.member_id));
    let cleared = world.assign(&created, None).await;
    assert_eq!(cleared.assignee, None);
    assert_eq!(cleared.assigned_at, None);
    assert_eq!(world.get(created.id).await.unwrap(), cleared);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn assigning_a_member_of_another_organization_is_not_found() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let created = acme.ticket("Chez Acme").await;

    let outcome = acme
        .usecase
        .assign_support_ticket(AssignTicketCommand {
            actor: Subject::system(),
            organization_id: acme.organization_id,
            ticket_id: created.id,
            assignee: Some(globex.plain.member_id),
        })
        .await;

    assert!(matches!(outcome, Err(CoreError::NotFound)));
    let stored = acme.get(created.id).await.unwrap();
    assert_eq!(stored.assignee, None);
    assert_eq!(stored.assigned_at, None);

    acme.cleanup().await;
    globex.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn last_assigned_at_lists_never_assigned_members_as_none() {
    let world = World::new(&pool().await).await;
    let first = world.ticket("T1").await;
    let second = world.ticket("T2").await;
    let third = world.ticket("T3").await;
    let to_manager = world.assign(&first, Some(&world.manager)).await;
    let to_plain = world.assign(&second, Some(&world.plain)).await;
    let to_manager_again = world.assign(&third, Some(&world.manager)).await;
    let manager = world.manager.member_id;
    let plain = world.plain.member_id;
    let spare = world.spare.member_id;

    let listed = with_tx(&world.pool, async |tx| {
        let mut repository = PgSupportTicketRepository::new(&tx);
        repository
            .last_assigned_at(world.organization_id, &[spare, manager, plain])
            .await
    })
    .await
    .unwrap();

    assert!(to_manager_again.assigned_at > to_manager.assigned_at);
    assert_eq!(
        listed,
        vec![
            (spare, None),
            (manager, to_manager_again.assigned_at),
            (plain, to_plain.assigned_at),
        ]
    );

    let nobody = with_tx(&world.pool, async |tx| {
        let mut repository = PgSupportTicketRepository::new(&tx);
        repository
            .last_assigned_at(world.organization_id, &[])
            .await
    })
    .await
    .unwrap();
    assert!(nobody.is_empty());

    world.cleanup().await;
}
