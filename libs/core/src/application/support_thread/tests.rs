use authz::Subject;
use common::{CoreError, OrganizationId, UserId, generate_uuid_v7};
use events::testing::RecordingEmitter;
use serde_json::{Value, json};
use sqlx::PgPool;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::{
    AddEntryCommand, AttachFileCommand, Attachment, CreateTicketCommand, CustomFields,
    DraftCommand, EditDraftCommand, EntryKind, GetTicketCommand, ListThreadCommand, MemberId,
    Permissions, PostClientMessageCommand, Ticket, TicketEntry, TicketStatus,
    TransitionTicketCommand,
    application::{
        MestierUseCase, default_authorizer, policy,
        test_support::{purge, scratch_pool},
    },
    domain::support::thread_service::{NewAttachment, ThreadService},
    infrastructure::{
        postgres::with_tx, realtime::EventHub,
        support_thread::postgres::PgSupportTicketEntryRepository,
        support_ticket::postgres::PgSupportTicketRepository,
    },
};

static THREAD_SCRATCH: OnceCell<String> = OnceCell::const_new();

async fn pool() -> PgPool {
    scratch_pool("mestier_support_thread_test_", &THREAD_SCRATCH).await
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
    colleague: Person,
    plain: Person,
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
    label: &str,
    role_id: Option<Uuid>,
) -> Person {
    let user_id = seed_user(pool, label).await;
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

impl World {
    async fn new(pool: &PgPool) -> Self {
        let owner_user = seed_user(pool, "Owner").await;
        let organization_id = OrganizationId(generate_uuid_v7());
        sqlx::query("INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)")
            .bind(organization_id.0)
            .bind("Treyve Paysages")
            .bind(format!("org-{}", organization_id.0))
            .bind(owner_user)
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

        let manager = seed_member(pool, organization_id, "Manager", Some(role_id)).await;
        let colleague = seed_member(pool, organization_id, "Colleague", Some(role_id)).await;
        let plain = seed_member(pool, organization_id, "Plain", None).await;

        Self {
            pool: pool.clone(),
            usecase: MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new()),
            organization_id,
            manager,
            colleague,
            plain,
        }
    }

    async fn ticket(&self, subject: &str) -> Ticket {
        self.usecase
            .create_support_ticket(CreateTicketCommand {
                actor: self.manager.subject(),
                organization_id: self.organization_id,
                subject: subject.to_owned(),
                body: format!("Corps de {subject}"),
                custom_fields: CustomFields::default(),
                external_id: None,
            })
            .await
            .unwrap()
            .ticket
    }

    async fn set_status(&self, ticket: &Ticket, status: TicketStatus) -> Ticket {
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

    async fn stored_ticket(&self, ticket: &Ticket) -> Ticket {
        self.usecase
            .get_support_ticket(GetTicketCommand {
                organization_id: self.organization_id,
                ticket_id: ticket.id,
            })
            .await
            .unwrap()
    }

    fn add(&self, author: &Person, ticket: &Ticket, body: &str) -> AddEntryCommand {
        AddEntryCommand {
            actor: author.subject(),
            organization_id: self.organization_id,
            ticket_id: ticket.id,
            body: body.to_owned(),
        }
    }

    async fn note(&self, ticket: &Ticket, body: &str) -> TicketEntry {
        self.usecase
            .add_support_internal_note(self.add(&self.manager, ticket, body))
            .await
            .unwrap()
    }

    async fn draft(&self, ticket: &Ticket, body: &str) -> TicketEntry {
        self.usecase
            .save_support_draft(self.add(&self.manager, ticket, body))
            .await
            .unwrap()
    }

    async fn reply(&self, ticket: &Ticket, body: &str) -> TicketEntry {
        self.usecase
            .send_support_reply(self.add(&self.manager, ticket, body))
            .await
            .unwrap()
    }

    async fn client_message(&self, ticket: &Ticket, body: &str) -> TicketEntry {
        self.usecase
            .post_support_client_message(PostClientMessageCommand {
                organization_id: self.organization_id,
                ticket_id: ticket.id,
                body: body.to_owned(),
            })
            .await
            .unwrap()
    }

    fn file(&self, entry: &TicketEntry, name: &str, size_bytes: u64) -> AttachFileCommand {
        AttachFileCommand {
            actor: self.manager.subject(),
            organization_id: self.organization_id,
            entry_id: entry.id,
            file_key: format!("support/{}/{name}", generate_uuid_v7()),
            file_name: name.to_owned(),
            mime_type: "application/octet-stream".to_owned(),
            size_bytes,
        }
    }

    async fn attach(&self, entry: &TicketEntry, name: &str, size_bytes: u64) -> Attachment {
        self.usecase
            .attach_to_support_entry(self.file(entry, name, size_bytes))
            .await
            .unwrap()
    }

    async fn thread(&self, ticket: &Ticket, client_visible_only: bool) -> Vec<TicketEntry> {
        self.usecase
            .list_support_thread(ListThreadCommand {
                organization_id: self.organization_id,
                ticket_id: ticket.id,
                client_visible_only,
            })
            .await
            .unwrap()
    }

    async fn count(&self, statement: &str, id: Uuid) -> i64 {
        sqlx::query_scalar(statement)
            .bind(self.organization_id.0)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .unwrap()
    }

    async fn entry_rows(&self, ticket: &Ticket) -> i64 {
        self.count(
            "SELECT COUNT(*) FROM support_ticket_entries WHERE org_id = $1 AND ticket_id = $2",
            ticket.id.0,
        )
        .await
    }

    async fn transitions(&self) -> Vec<Value> {
        sqlx::query_scalar(
            "SELECT payload FROM automation.event \
             WHERE org_id = $1 AND name = 'ticket.transitioned' \
             ORDER BY occurred_at, id",
        )
        .bind(self.organization_id.0)
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }

    async fn cleanup(self) {
        for statement in [
            "DELETE FROM automation.event WHERE org_id = $1",
            "DELETE FROM support_ticket_attachments WHERE org_id = $1",
            "DELETE FROM support_ticket_entries WHERE org_id = $1",
            "DELETE FROM support_tickets WHERE org_id = $1",
            "DELETE FROM support_ticket_counters WHERE org_id = $1",
            "DELETE FROM member_roles WHERE member_id IN (SELECT id FROM organization_members WHERE organization_id = $1)",
            "DELETE FROM roles WHERE organization_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(&self.pool, statement, self.organization_id.0).await;
        }
        for person in [&self.manager, &self.colleague, &self.plain] {
            purge(
                &self.pool,
                "DELETE FROM users WHERE id = $1",
                person.user_id,
            )
            .await;
        }
    }
}

fn bodies(thread: &[TicketEntry]) -> Vec<&str> {
    thread.iter().map(|entry| entry.body.as_str()).collect()
}

fn kinds(thread: &[TicketEntry]) -> Vec<EntryKind> {
    thread.iter().map(|entry| entry.kind).collect()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn list_client_visible_only_never_returns_notes_or_drafts() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;
    world.client_message(&ticket, "Bonjour, ça plante.").await;
    world
        .note(&ticket, "Client déjà vu la semaine passée.")
        .await;
    world.draft(&ticket, "Brouillon de réponse").await;
    world.reply(&ticket, "Nous regardons.").await;
    world.client_message(&ticket, "Merci, j'attends.").await;

    let visible = world.thread(&ticket, true).await;
    let everything = world.thread(&ticket, false).await;

    assert_eq!(
        bodies(&visible),
        [
            "Bonjour, ça plante.",
            "Nous regardons.",
            "Merci, j'attends."
        ]
    );
    assert_eq!(
        kinds(&visible),
        [
            EntryKind::ClientMessage,
            EntryKind::Reply,
            EntryKind::ClientMessage
        ]
    );
    assert_eq!(
        kinds(&everything),
        [
            EntryKind::ClientMessage,
            EntryKind::InternalNote,
            EntryKind::Draft,
            EntryKind::Reply,
            EntryKind::ClientMessage
        ]
    );
    assert!(visible.iter().all(|entry| entry.kind.is_client_visible()));
    assert!(everything.iter().all(|entry| entry.ticket_id == ticket.id));
    assert!(
        everything
            .iter()
            .all(|entry| entry.organization_id == world.organization_id)
    );

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn entries_are_listed_in_creation_order_with_their_attachments() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;
    let note = world.note(&ticket, "Note interne").await;
    let message = world
        .client_message(&ticket, "Capture en pièce jointe")
        .await;
    let reply = world.reply(&ticket, "Bien reçu").await;
    let first = world.attach(&reply, "capture.png", 48_213).await;
    let second = world.attach(&reply, "journal.log", 12).await;
    let sent_by_client = with_tx(&world.pool, async |tx| {
        ThreadService::new(
            PgSupportTicketRepository::new(&tx),
            PgSupportTicketEntryRepository::new(&tx),
            RecordingEmitter::new(),
        )
        .attach_client_file(
            world.organization_id,
            message.id,
            NewAttachment {
                file_key: format!("support/{}/erreur.png", generate_uuid_v7()),
                file_name: " erreur.png ".to_owned(),
                mime_type: "image/png".to_owned(),
                size_bytes: 9_001,
            },
        )
        .await
    })
    .await
    .unwrap();

    let everything = world.thread(&ticket, false).await;
    let visible = world.thread(&ticket, true).await;

    let ids: Vec<_> = everything.iter().map(|entry| entry.id).collect();
    assert_eq!(ids, [note.id, message.id, reply.id]);
    assert!(everything[0].attachments.is_empty());
    assert_eq!(everything[1].attachments, vec![sent_by_client.clone()]);
    assert_eq!(
        everything[2].attachments,
        vec![first.clone(), second.clone()]
    );
    assert_eq!(sent_by_client.file_name, "erreur.png");
    assert_eq!(sent_by_client.size_bytes, 9_001);
    assert_eq!(sent_by_client.mime_type, "image/png");
    assert_eq!(sent_by_client.entry_id, message.id);
    assert_eq!(sent_by_client.ticket_id, ticket.id);
    assert_eq!(first.file_name, "capture.png");
    assert_eq!(first.size_bytes, 48_213);
    assert_eq!(first.entry_id, reply.id);
    assert_eq!(first.ticket_id, ticket.id);
    assert_eq!(first.organization_id, world.organization_id);
    assert_eq!(second.file_name, "journal.log");
    assert_eq!(second.size_bytes, 12);
    assert!(second.created_at > first.created_at);
    assert_eq!(
        visible.iter().map(|entry| entry.id).collect::<Vec<_>>(),
        [message.id, reply.id]
    );
    assert_eq!(visible[0].attachments, vec![sent_by_client]);
    assert_eq!(visible[1].attachments, vec![first, second]);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn sending_a_draft_keeps_one_row_and_moves_it_to_send_time() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;
    let draft = world.draft(&ticket, "Voici la marche à suivre.").await;
    let note = world.note(&ticket, "À relire").await;
    let message = world.client_message(&ticket, "Des nouvelles ?").await;
    let before = world.thread(&ticket, false).await;
    assert_eq!(
        before.iter().map(|entry| entry.id).collect::<Vec<_>>(),
        [draft.id, note.id, message.id]
    );

    let sent = world
        .usecase
        .send_support_draft(DraftCommand {
            actor: world.colleague.subject(),
            organization_id: world.organization_id,
            entry_id: draft.id,
        })
        .await
        .unwrap();

    assert_eq!(sent.id, draft.id);
    assert_eq!(sent.kind, EntryKind::Reply);
    assert_eq!(sent.author, Some(world.colleague.member_id));
    assert_eq!(sent.body, "Voici la marche à suivre.");
    assert_eq!(sent.ticket_id, ticket.id);
    assert!(sent.created_at > message.created_at);
    assert_eq!(sent.updated_at, sent.created_at);
    assert_eq!(
        world
            .count(
                "SELECT COUNT(*) FROM support_ticket_entries WHERE org_id = $1 AND id = $2",
                draft.id.0
            )
            .await,
        1
    );
    assert_eq!(world.entry_rows(&ticket).await, 3);
    let after = world.thread(&ticket, false).await;
    assert_eq!(
        after.iter().map(|entry| entry.id).collect::<Vec<_>>(),
        [note.id, message.id, draft.id]
    );
    assert_eq!(after[2], sent);
    let visible = world.thread(&ticket, true).await;
    assert_eq!(
        visible.iter().map(|entry| entry.id).collect::<Vec<_>>(),
        [message.id, draft.id]
    );

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn client_message_reopens_a_resolved_ticket_and_records_ticket_transitioned() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;
    let resolved = world.set_status(&ticket, TicketStatus::Resolved).await;
    assert_eq!(resolved.status, TicketStatus::Resolved);

    let message = world.client_message(&ticket, "Ça recommence.").await;

    assert_eq!(message.kind, EntryKind::ClientMessage);
    assert_eq!(message.author, None);
    assert_eq!(message.body, "Ça recommence.");
    let reopened = world.stored_ticket(&ticket).await;
    assert_eq!(reopened.status, TicketStatus::InProgress);
    assert!(reopened.updated_at > resolved.updated_at);
    assert_eq!(reopened.created_at, ticket.created_at);
    let transitions = world.transitions().await;
    assert_eq!(transitions.len(), 2);
    assert_eq!(transitions[0]["from"], json!("new"));
    assert_eq!(transitions[0]["to"], json!("resolved"));
    assert_eq!(transitions[1]["from"], json!("resolved"));
    assert_eq!(transitions[1]["to"], json!("in_progress"));
    assert_eq!(transitions[1]["ticket"]["id"], json!(ticket.id.0));
    assert_eq!(transitions[1]["ticket"]["status"], json!("in_progress"));

    world.client_message(&ticket, "Toujours là ?").await;

    assert_eq!(world.transitions().await.len(), 2);
    assert_eq!(world.entry_rows(&ticket).await, 2);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn client_message_on_a_waiting_ticket_moves_it_to_in_progress() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;
    world
        .set_status(&ticket, TicketStatus::WaitingCustomer)
        .await;

    world
        .client_message(&ticket, "Voici les informations.")
        .await;

    assert_eq!(
        world.stored_ticket(&ticket).await.status,
        TicketStatus::InProgress
    );
    let transitions = world.transitions().await;
    assert_eq!(transitions.len(), 2);
    assert_eq!(transitions[1]["from"], json!("waiting_customer"));
    assert_eq!(transitions[1]["to"], json!("in_progress"));

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn client_message_on_a_new_ticket_changes_nothing_else() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;

    world.client_message(&ticket, "Une précision.").await;

    assert_eq!(world.stored_ticket(&ticket).await, ticket);
    assert!(world.transitions().await.is_empty());

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn deleting_a_draft_removes_its_attachments() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;
    let draft = world.draft(&ticket, "À envoyer plus tard").await;
    let kept = world.reply(&ticket, "Réponse envoyée").await;
    world.attach(&draft, "brouillon.pdf", 100).await;
    world.attach(&draft, "brouillon.png", 200).await;
    let kept_file = world.attach(&kept, "reponse.pdf", 300).await;
    let attachments_of = "SELECT COUNT(*) FROM support_ticket_attachments \
                          WHERE org_id = $1 AND entry_id = $2";
    assert_eq!(world.count(attachments_of, draft.id.0).await, 2);

    world
        .usecase
        .delete_support_draft(DraftCommand {
            actor: world.manager.subject(),
            organization_id: world.organization_id,
            entry_id: draft.id,
        })
        .await
        .unwrap();

    assert_eq!(world.count(attachments_of, draft.id.0).await, 0);
    assert_eq!(
        world
            .count(
                "SELECT COUNT(*) FROM support_ticket_attachments \
                 WHERE org_id = $1 AND ticket_id = $2",
                ticket.id.0
            )
            .await,
        1
    );
    let thread = world.thread(&ticket, false).await;
    assert_eq!(
        thread.iter().map(|entry| entry.id).collect::<Vec<_>>(),
        [kept.id]
    );
    assert_eq!(thread[0].attachments, vec![kept_file]);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn editing_a_draft_rewrites_its_body_and_keeps_its_place_and_attachments() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;
    let draft = world.draft(&ticket, "Première version").await;
    let file = world.attach(&draft, "annexe.pdf", 100).await;
    world.note(&ticket, "Après le brouillon").await;

    let edited = world
        .usecase
        .edit_support_draft(EditDraftCommand {
            actor: world.colleague.subject(),
            organization_id: world.organization_id,
            entry_id: draft.id,
            body: "  Seconde version  ".to_owned(),
        })
        .await
        .unwrap();

    assert_eq!(edited.id, draft.id);
    assert_eq!(edited.kind, EntryKind::Draft);
    assert_eq!(edited.author, Some(world.manager.member_id));
    assert_eq!(edited.body, "Seconde version");
    assert_eq!(edited.created_at, draft.created_at);
    assert!(edited.updated_at > draft.updated_at);
    assert_eq!(edited.attachments, vec![file]);
    let thread = world.thread(&ticket, false).await;
    assert_eq!(bodies(&thread), ["Seconde version", "Après le brouillon"]);
    assert_eq!(thread[0], edited);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_sent_reply_can_neither_be_edited_nor_deleted() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;
    let reply = world.reply(&ticket, "Réponse définitive").await;

    let edited = world
        .usecase
        .edit_support_draft(EditDraftCommand {
            actor: world.manager.subject(),
            organization_id: world.organization_id,
            entry_id: reply.id,
            body: "Autre chose".to_owned(),
        })
        .await;
    let deleted = world
        .usecase
        .delete_support_draft(DraftCommand {
            actor: world.manager.subject(),
            organization_id: world.organization_id,
            entry_id: reply.id,
        })
        .await;

    assert!(matches!(edited, Err(CoreError::Conflict(_))));
    assert!(matches!(deleted, Err(CoreError::Conflict(_))));
    assert_eq!(world.thread(&ticket, false).await, vec![reply]);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn an_attachment_of_another_organization_is_not_found() {
    let pool = pool().await;
    let acme = World::new(&pool).await;
    let globex = World::new(&pool).await;
    let ticket = acme.ticket("Chez Acme").await;
    let reply = acme.reply(&ticket, "Réponse Acme").await;
    let attachment = acme.attach(&reply, "contrat.pdf", 4_096).await;

    let found = acme
        .usecase
        .get_support_attachment(acme.organization_id, attachment.id)
        .await
        .unwrap();
    let foreign_lookup = globex
        .usecase
        .get_support_attachment(globex.organization_id, attachment.id)
        .await;
    let foreign_attach = globex
        .usecase
        .attach_to_support_entry(AttachFileCommand {
            actor: globex.manager.subject(),
            organization_id: globex.organization_id,
            entry_id: reply.id,
            file_key: format!("support/{}/intrus.pdf", generate_uuid_v7()),
            file_name: "intrus.pdf".to_owned(),
            mime_type: "application/pdf".to_owned(),
            size_bytes: 1,
        })
        .await;
    let foreign_thread = globex
        .usecase
        .list_support_thread(ListThreadCommand {
            organization_id: globex.organization_id,
            ticket_id: ticket.id,
            client_visible_only: false,
        })
        .await;
    let foreign_message = globex
        .usecase
        .post_support_client_message(PostClientMessageCommand {
            organization_id: globex.organization_id,
            ticket_id: ticket.id,
            body: "Intrus".to_owned(),
        })
        .await;

    assert_eq!(found.attachment, attachment);
    assert_eq!(found.entry_kind, EntryKind::Reply);
    assert!(matches!(foreign_lookup, Err(CoreError::NotFound)));
    assert!(matches!(foreign_attach, Err(CoreError::NotFound)));
    assert!(matches!(foreign_thread, Err(CoreError::NotFound)));
    assert!(matches!(foreign_message, Err(CoreError::NotFound)));
    assert_eq!(acme.thread(&ticket, false).await.len(), 1);

    acme.cleanup().await;
    globex.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_member_without_manage_support_cannot_add_a_note() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;
    let entry = world.note(&ticket, "Note du gestionnaire").await;

    let outcome = world
        .usecase
        .add_support_internal_note(world.add(&world.plain, &ticket, "Note interdite"))
        .await;
    let attached = world
        .usecase
        .attach_to_support_entry(AttachFileCommand {
            actor: world.plain.subject(),
            ..world.file(&entry, "interdit.pdf", 1)
        })
        .await;

    assert!(matches!(outcome, Err(CoreError::Forbidden { .. })));
    assert!(matches!(attached, Err(CoreError::Forbidden { .. })));
    assert_eq!(world.thread(&ticket, false).await, vec![entry]);

    world.cleanup().await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_system_subject_cannot_author_a_note() {
    let world = World::new(&pool().await).await;
    let ticket = world.ticket("Impossible de déployer").await;

    let note = world
        .usecase
        .add_support_internal_note(AddEntryCommand {
            actor: Subject::system(),
            ..world.add(&world.manager, &ticket, "Note système")
        })
        .await;
    let draft = world
        .usecase
        .save_support_draft(AddEntryCommand {
            actor: Subject::system(),
            ..world.add(&world.manager, &ticket, "Brouillon système")
        })
        .await;
    let reply = world
        .usecase
        .send_support_reply(AddEntryCommand {
            actor: Subject::system(),
            ..world.add(&world.manager, &ticket, "Réponse système")
        })
        .await;

    assert!(matches!(note, Err(CoreError::Forbidden { .. })));
    assert!(matches!(draft, Err(CoreError::Forbidden { .. })));
    assert!(matches!(reply, Err(CoreError::Forbidden { .. })));
    assert!(world.thread(&ticket, false).await.is_empty());
    assert_eq!(world.entry_rows(&ticket).await, 0);

    world.cleanup().await;
}
