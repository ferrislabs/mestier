use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    Attachment, AttachmentId, EntryKind, LocatedAttachment, MemberId, OrganizationId, Ticket,
    TicketEntry, TicketEntryId, TicketId, TicketStatus,
    domain::support::{
        events::TicketTransitioned,
        ports::{TicketEntryRepository, TicketRepository},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewAttachment {
    pub file_key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

fn non_blank_body(body: &str) -> Result<&str, CoreError> {
    let body = body.trim();
    if body.is_empty() {
        return Err(CoreError::Conflict(
            "entry body must not be empty".to_owned(),
        ));
    }
    Ok(body)
}

pub struct ThreadService<T, N, E>
where
    T: TicketRepository,
    N: TicketEntryRepository,
    E: EventEmitter,
{
    tickets: T,
    entries: N,
    emitter: E,
}

impl<T, N, E> ThreadService<T, N, E>
where
    T: TicketRepository,
    N: TicketEntryRepository,
    E: EventEmitter,
{
    pub fn new(tickets: T, entries: N, emitter: E) -> Self {
        Self {
            tickets,
            entries,
            emitter,
        }
    }

    pub async fn add_note(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
        author: MemberId,
        body: &str,
    ) -> Result<TicketEntry, CoreError> {
        self.write(
            organization_id,
            ticket_id,
            EntryKind::InternalNote,
            Some(author),
            body,
        )
        .await
    }

    pub async fn add_automation_note(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
        body: &str,
    ) -> Result<TicketEntry, CoreError> {
        self.write(
            organization_id,
            ticket_id,
            EntryKind::InternalNote,
            None,
            body,
        )
        .await
    }

    pub async fn save_draft(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
        author: MemberId,
        body: &str,
    ) -> Result<TicketEntry, CoreError> {
        self.write(
            organization_id,
            ticket_id,
            EntryKind::Draft,
            Some(author),
            body,
        )
        .await
    }

    pub async fn send_reply(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
        author: MemberId,
        body: &str,
    ) -> Result<TicketEntry, CoreError> {
        self.write(
            organization_id,
            ticket_id,
            EntryKind::Reply,
            Some(author),
            body,
        )
        .await
    }

    pub async fn edit_draft(
        &mut self,
        organization_id: OrganizationId,
        entry_id: TicketEntryId,
        body: &str,
    ) -> Result<TicketEntry, CoreError> {
        let body = non_blank_body(body)?;
        let mut entry = self.draft(organization_id, entry_id).await?;

        entry.body = body.to_owned();
        entry.updated_at = Utc::now();
        self.entries.update(&entry).await
    }

    pub async fn delete_draft(
        &mut self,
        organization_id: OrganizationId,
        entry_id: TicketEntryId,
    ) -> Result<(), CoreError> {
        let entry = self.draft(organization_id, entry_id).await?;

        self.entries.delete(organization_id, entry.id).await
    }

    pub async fn send_draft(
        &mut self,
        organization_id: OrganizationId,
        entry_id: TicketEntryId,
        sender: MemberId,
    ) -> Result<TicketEntry, CoreError> {
        let mut entry = self.draft(organization_id, entry_id).await?;
        let now = Utc::now();

        entry.kind = EntryKind::Reply;
        entry.author = Some(sender);
        entry.created_at = now;
        entry.updated_at = now;
        self.entries.update(&entry).await
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
        client_visible_only: bool,
    ) -> Result<Vec<TicketEntry>, CoreError> {
        self.ticket(organization_id, ticket_id).await?;

        self.entries
            .list_for_ticket(organization_id, ticket_id, client_visible_only)
            .await
    }

    pub async fn attach_member_file(
        &mut self,
        organization_id: OrganizationId,
        entry_id: TicketEntryId,
        file: NewAttachment,
    ) -> Result<Attachment, CoreError> {
        let entry = self.entry(organization_id, entry_id).await?;
        if entry.kind == EntryKind::ClientMessage {
            return Err(CoreError::Conflict(
                "a member cannot attach a file to a client message".to_owned(),
            ));
        }

        self.store(&entry, file).await
    }

    pub async fn attach_client_file(
        &mut self,
        organization_id: OrganizationId,
        entry_id: TicketEntryId,
        file: NewAttachment,
    ) -> Result<Attachment, CoreError> {
        let entry = self.entry(organization_id, entry_id).await?;
        if entry.kind != EntryKind::ClientMessage {
            return Err(CoreError::Conflict(
                "a client file only goes on a client message".to_owned(),
            ));
        }

        self.store(&entry, file).await
    }

    pub async fn post_client_message(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
        body: &str,
    ) -> Result<TicketEntry, CoreError> {
        let body = non_blank_body(body)?;
        let mut ticket = self.ticket(organization_id, ticket_id).await?;
        let now = Utc::now();

        let entry = self
            .entries
            .create(&TicketEntry {
                id: TicketEntryId(generate_uuid_v7()),
                organization_id,
                ticket_id,
                kind: EntryKind::ClientMessage,
                author: None,
                body: body.to_owned(),
                attachments: Vec::new(),
                created_at: now,
                updated_at: now,
            })
            .await?;

        let from = ticket.status;
        if matches!(from, TicketStatus::WaitingCustomer | TicketStatus::Resolved) {
            ticket.status = TicketStatus::InProgress;
            ticket.updated_at = now;
            let updated = self.tickets.update(&ticket).await?;

            self.emitter.emit(
                organization_id,
                &TicketTransitioned {
                    ticket: updated,
                    from,
                    to: TicketStatus::InProgress,
                },
            )?;
        }
        Ok(entry)
    }

    pub async fn get_attachment(
        &mut self,
        organization_id: OrganizationId,
        attachment_id: AttachmentId,
    ) -> Result<LocatedAttachment, CoreError> {
        self.entries
            .find_attachment(organization_id, attachment_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn write(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
        kind: EntryKind,
        author: Option<MemberId>,
        body: &str,
    ) -> Result<TicketEntry, CoreError> {
        let body = non_blank_body(body)?;
        self.ticket(organization_id, ticket_id).await?;
        let now = Utc::now();

        self.entries
            .create(&TicketEntry {
                id: TicketEntryId(generate_uuid_v7()),
                organization_id,
                ticket_id,
                kind,
                author,
                body: body.to_owned(),
                attachments: Vec::new(),
                created_at: now,
                updated_at: now,
            })
            .await
    }

    async fn ticket(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
    ) -> Result<Ticket, CoreError> {
        self.tickets
            .find(organization_id, ticket_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn entry(
        &mut self,
        organization_id: OrganizationId,
        entry_id: TicketEntryId,
    ) -> Result<TicketEntry, CoreError> {
        self.entries
            .find(organization_id, entry_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn draft(
        &mut self,
        organization_id: OrganizationId,
        entry_id: TicketEntryId,
    ) -> Result<TicketEntry, CoreError> {
        let entry = self.entry(organization_id, entry_id).await?;
        if entry.kind != EntryKind::Draft {
            return Err(CoreError::Conflict(
                "only a draft can be edited, sent or deleted".to_owned(),
            ));
        }
        Ok(entry)
    }

    async fn store(
        &mut self,
        entry: &TicketEntry,
        file: NewAttachment,
    ) -> Result<Attachment, CoreError> {
        let NewAttachment {
            file_key,
            file_name,
            mime_type,
            size_bytes,
        } = file;
        let file_name = file_name.trim();
        if file_name.is_empty() {
            return Err(CoreError::Conflict(
                "attachment file name must not be empty".to_owned(),
            ));
        }

        self.entries
            .add_attachment(&Attachment {
                id: AttachmentId(generate_uuid_v7()),
                organization_id: entry.organization_id,
                ticket_id: entry.ticket_id,
                entry_id: entry.id,
                file_key,
                file_name: file_name.to_owned(),
                mime_type,
                size_bytes,
                created_at: Utc::now(),
            })
            .await
    }
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use events::testing::RecordingEmitter;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::{
        CustomFields, EntryKind, Ticket, TicketNumber, TicketStatus,
        domain::support::ports::{MockTicketEntryRepository, MockTicketRepository},
    };

    fn organization() -> OrganizationId {
        OrganizationId(Uuid::from_u128(0x02))
    }

    fn other_organization() -> OrganizationId {
        OrganizationId(Uuid::from_u128(0x03))
    }

    fn ticket_id() -> TicketId {
        TicketId(Uuid::from_u128(0x30))
    }

    fn entry_id() -> TicketEntryId {
        TicketEntryId(Uuid::from_u128(0x40))
    }

    fn member() -> MemberId {
        MemberId(Uuid::from_u128(0x12))
    }

    fn past() -> DateTime<Utc> {
        "2026-10-15T07:45:00Z".parse().unwrap()
    }

    fn ticket(status: TicketStatus) -> Ticket {
        Ticket {
            id: ticket_id(),
            organization_id: organization(),
            number: TicketNumber(42),
            subject: "Impossible de déployer".to_owned(),
            body: "Le déploiement échoue.".to_owned(),
            status,
            assignee: None,
            assigned_at: None,
            custom_fields: CustomFields::default(),
            badges: vec![],
            external_id: Some("ZD-48213".to_owned()),
            created_at: past(),
            updated_at: past(),
        }
    }

    fn entry(kind: EntryKind) -> TicketEntry {
        TicketEntry {
            id: entry_id(),
            organization_id: organization(),
            ticket_id: ticket_id(),
            kind,
            author: (kind != EntryKind::ClientMessage).then(member),
            body: "Bonjour, où en est-on ?".to_owned(),
            attachments: vec![],
            created_at: past(),
            updated_at: past(),
        }
    }

    fn new_file(name: &str) -> NewAttachment {
        NewAttachment {
            file_key: "support/2026/10/capture.png".to_owned(),
            file_name: name.to_owned(),
            mime_type: "image/png".to_owned(),
            size_bytes: 48_213,
        }
    }

    fn tickets_holding(ticket: &Ticket) -> MockTicketRepository {
        let held = ticket.clone();
        let mut tickets = MockTicketRepository::new();
        tickets.expect_find().times(1).returning(move |_, _| {
            let held = held.clone();
            Box::pin(async move { Ok(Some(held)) })
        });
        tickets
    }

    fn entries_holding(entry: &TicketEntry) -> MockTicketEntryRepository {
        let held = entry.clone();
        let mut entries = MockTicketEntryRepository::new();
        entries.expect_find().times(1).returning(move |_, _| {
            let held = held.clone();
            Box::pin(async move { Ok(Some(held)) })
        });
        entries
    }

    fn echoing_entry_creates(entries: &mut MockTicketEntryRepository) {
        entries.expect_create().times(1).returning(|entry| {
            let entry = entry.clone();
            Box::pin(async move { Ok(entry) })
        });
    }

    fn echoing_entry_updates(entries: &mut MockTicketEntryRepository) {
        entries.expect_update().times(1).returning(|entry| {
            let entry = entry.clone();
            Box::pin(async move { Ok(entry) })
        });
    }

    fn echoing_attachments(entries: &mut MockTicketEntryRepository) {
        entries
            .expect_add_attachment()
            .times(1)
            .returning(|attachment| {
                let attachment = attachment.clone();
                Box::pin(async move { Ok(attachment) })
            });
    }

    fn conflict_mentioning(outcome: Result<impl std::fmt::Debug, CoreError>, word: &str) {
        match outcome {
            Err(CoreError::Conflict(message)) => assert!(message.contains(word)),
            other => panic!("{other:?}"),
        }
    }

    #[tokio::test]
    async fn sending_a_draft_turns_it_into_a_reply_attributed_to_the_sender() {
        let drafted = entry(EntryKind::Draft);
        let sender = MemberId(Uuid::from_u128(0x13));
        let mut entries = entries_holding(&drafted);
        echoing_entry_updates(&mut entries);
        let emitter = RecordingEmitter::new();
        let mut service = ThreadService::new(MockTicketRepository::new(), entries, &emitter);
        let started = Utc::now();

        let sent = service
            .send_draft(organization(), drafted.id, sender)
            .await
            .unwrap();

        assert_eq!(sent.id, drafted.id);
        assert_eq!(sent.organization_id, organization());
        assert_eq!(sent.ticket_id, ticket_id());
        assert_eq!(sent.kind, EntryKind::Reply);
        assert_eq!(sent.author, Some(sender));
        assert_eq!(sent.body, drafted.body);
        assert!(sent.created_at >= started);
        assert!(sent.created_at > drafted.created_at);
        assert_eq!(sent.updated_at, sent.created_at);
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn a_reply_cannot_be_edited() {
        let reply = entry(EntryKind::Reply);
        let entries = entries_holding(&reply);
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            entries,
            RecordingEmitter::new(),
        );

        let outcome = service
            .edit_draft(organization(), reply.id, "Nouvelle version")
            .await;

        conflict_mentioning(outcome, "draft");
    }

    #[tokio::test]
    async fn a_reply_cannot_be_deleted() {
        let reply = entry(EntryKind::Reply);
        let entries = entries_holding(&reply);
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            entries,
            RecordingEmitter::new(),
        );

        let outcome = service.delete_draft(organization(), reply.id).await;

        conflict_mentioning(outcome, "draft");
    }

    #[tokio::test]
    async fn only_a_draft_can_be_edited_sent_or_deleted() {
        for kind in [
            EntryKind::ClientMessage,
            EntryKind::Reply,
            EntryKind::InternalNote,
        ] {
            let held = entry(kind);
            let mut service = ThreadService::new(
                MockTicketRepository::new(),
                entries_holding(&held),
                RecordingEmitter::new(),
            );
            conflict_mentioning(
                service.send_draft(organization(), held.id, member()).await,
                "draft",
            );

            let mut service = ThreadService::new(
                MockTicketRepository::new(),
                entries_holding(&held),
                RecordingEmitter::new(),
            );
            conflict_mentioning(
                service.edit_draft(organization(), held.id, "Autre").await,
                "draft",
            );

            let mut service = ThreadService::new(
                MockTicketRepository::new(),
                entries_holding(&held),
                RecordingEmitter::new(),
            );
            conflict_mentioning(service.delete_draft(organization(), held.id).await, "draft");
        }
    }

    #[tokio::test]
    async fn a_draft_can_be_edited_and_deleted() {
        let drafted = entry(EntryKind::Draft);
        let mut entries = entries_holding(&drafted);
        echoing_entry_updates(&mut entries);
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            entries,
            RecordingEmitter::new(),
        );

        let edited = service
            .edit_draft(organization(), drafted.id, "  Version corrigée  ")
            .await
            .unwrap();

        assert_eq!(edited.id, drafted.id);
        assert_eq!(edited.kind, EntryKind::Draft);
        assert_eq!(edited.author, drafted.author);
        assert_eq!(edited.body, "Version corrigée");
        assert_eq!(edited.created_at, drafted.created_at);
        assert!(edited.updated_at > drafted.updated_at);

        let mut entries = entries_holding(&drafted);
        entries
            .expect_delete()
            .withf(|organization_id, id| *organization_id == organization() && *id == entry_id())
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            entries,
            RecordingEmitter::new(),
        );

        service
            .delete_draft(organization(), drafted.id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn a_blank_body_is_refused() {
        let emitter = RecordingEmitter::new();
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            MockTicketEntryRepository::new(),
            &emitter,
        );

        for body in ["", "   ", "\n\t "] {
            conflict_mentioning(
                service
                    .add_note(organization(), ticket_id(), member(), body)
                    .await,
                "body",
            );
            conflict_mentioning(
                service
                    .save_draft(organization(), ticket_id(), member(), body)
                    .await,
                "body",
            );
            conflict_mentioning(
                service
                    .send_reply(organization(), ticket_id(), member(), body)
                    .await,
                "body",
            );
            conflict_mentioning(
                service.edit_draft(organization(), entry_id(), body).await,
                "body",
            );
            conflict_mentioning(
                service
                    .post_client_message(organization(), ticket_id(), body)
                    .await,
                "body",
            );
        }
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn a_member_writes_a_note_a_draft_or_a_reply_on_an_existing_ticket() {
        let held = ticket(TicketStatus::InProgress);
        let started = Utc::now();

        let mut written = Vec::new();
        for kind in [EntryKind::InternalNote, EntryKind::Draft, EntryKind::Reply] {
            let mut entries = MockTicketEntryRepository::new();
            echoing_entry_creates(&mut entries);
            let emitter = RecordingEmitter::new();
            let mut service = ThreadService::new(tickets_holding(&held), entries, &emitter);
            let body = "  Je regarde ça tout de suite.  ";

            let created = match kind {
                EntryKind::InternalNote => {
                    service
                        .add_note(organization(), held.id, member(), body)
                        .await
                }
                EntryKind::Draft => {
                    service
                        .save_draft(organization(), held.id, member(), body)
                        .await
                }
                _ => {
                    service
                        .send_reply(organization(), held.id, member(), body)
                        .await
                }
            }
            .unwrap();

            assert!(emitter.names().is_empty());
            written.push(created);
        }

        let kinds: Vec<EntryKind> = written.iter().map(|entry| entry.kind).collect();
        assert_eq!(
            kinds,
            [EntryKind::InternalNote, EntryKind::Draft, EntryKind::Reply]
        );
        for created in &written {
            assert_eq!(created.organization_id, organization());
            assert_eq!(created.ticket_id, held.id);
            assert_eq!(created.author, Some(member()));
            assert_eq!(created.body, "Je regarde ça tout de suite.");
            assert!(created.attachments.is_empty());
            assert!(created.created_at >= started);
            assert_eq!(created.updated_at, created.created_at);
        }
        let ids: Vec<TicketEntryId> = written.iter().map(|entry| entry.id).collect();
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);
    }

    #[tokio::test]
    async fn writing_on_an_unknown_ticket_is_not_found() {
        let mut tickets = MockTicketRepository::new();
        tickets
            .expect_find()
            .times(3)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        let mut service = ThreadService::new(
            tickets,
            MockTicketEntryRepository::new(),
            RecordingEmitter::new(),
        );

        assert!(matches!(
            service
                .add_note(organization(), ticket_id(), member(), "Note")
                .await,
            Err(CoreError::NotFound)
        ));
        assert!(matches!(
            service.list(organization(), ticket_id(), false).await,
            Err(CoreError::NotFound)
        ));
        assert!(matches!(
            service
                .post_client_message(organization(), ticket_id(), "Bonjour")
                .await,
            Err(CoreError::NotFound)
        ));
    }

    async fn post_on(status: TicketStatus) -> (Ticket, TicketEntry, RecordingEmitter) {
        let before = ticket(status);
        let mut tickets = tickets_holding(&before);
        if matches!(
            status,
            TicketStatus::WaitingCustomer | TicketStatus::Resolved
        ) {
            tickets
                .expect_update()
                .withf(|ticket| {
                    ticket.id == ticket_id()
                        && ticket.status == TicketStatus::InProgress
                        && ticket.updated_at > past()
                        && ticket.created_at == past()
                })
                .times(1)
                .returning(|ticket| {
                    let ticket = ticket.clone();
                    Box::pin(async move { Ok(ticket) })
                });
        }
        let mut entries = MockTicketEntryRepository::new();
        echoing_entry_creates(&mut entries);
        let emitter = RecordingEmitter::new();
        let posted = ThreadService::new(tickets, entries, &emitter)
            .post_client_message(organization(), before.id, "  Toujours en panne  ")
            .await
            .unwrap();
        (before, posted, emitter)
    }

    fn assert_client_message(posted: &TicketEntry) {
        assert_eq!(posted.kind, EntryKind::ClientMessage);
        assert_eq!(posted.author, None);
        assert_eq!(posted.body, "Toujours en panne");
        assert_eq!(posted.organization_id, organization());
        assert_eq!(posted.ticket_id, ticket_id());
        assert!(posted.attachments.is_empty());
        assert_eq!(posted.updated_at, posted.created_at);
        assert!(posted.created_at > past());
    }

    #[tokio::test]
    async fn client_message_on_resolved_reopens_to_in_progress_and_emits() {
        let (before, posted, emitter) = post_on(TicketStatus::Resolved).await;

        assert_client_message(&posted);
        assert_eq!(emitter.names(), vec!["ticket.transitioned"]);
        let envelope = emitter.only("ticket.transitioned");
        assert_eq!(envelope.org_id, organization());
        assert_eq!(envelope.subject_kind, "ticket");
        assert_eq!(envelope.subject_id, Some(before.id.0));
        assert_eq!(envelope.payload["from"], json!("resolved"));
        assert_eq!(envelope.payload["to"], json!("in_progress"));
        assert_eq!(envelope.payload["ticket"]["id"], json!(before.id.0));
        assert_eq!(envelope.payload["ticket"]["status"], json!("in_progress"));
    }

    #[tokio::test]
    async fn client_message_on_waiting_customer_moves_to_in_progress() {
        let (before, posted, emitter) = post_on(TicketStatus::WaitingCustomer).await;

        assert_client_message(&posted);
        assert_eq!(emitter.names(), vec!["ticket.transitioned"]);
        let envelope = emitter.only("ticket.transitioned");
        assert_eq!(envelope.subject_id, Some(before.id.0));
        assert_eq!(envelope.payload["from"], json!("waiting_customer"));
        assert_eq!(envelope.payload["to"], json!("in_progress"));
        assert_eq!(envelope.payload["ticket"]["status"], json!("in_progress"));
    }

    #[tokio::test]
    async fn client_message_on_new_changes_nothing() {
        let (_, posted, emitter) = post_on(TicketStatus::New).await;

        assert_client_message(&posted);
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn client_message_on_in_progress_changes_nothing() {
        let (_, posted, emitter) = post_on(TicketStatus::InProgress).await;

        assert_client_message(&posted);
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn a_member_cannot_attach_to_a_client_message() {
        let message = entry(EntryKind::ClientMessage);
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            entries_holding(&message),
            RecordingEmitter::new(),
        );

        let outcome = service
            .attach_member_file(organization(), message.id, new_file("capture.png"))
            .await;

        conflict_mentioning(outcome, "client message");
    }

    #[tokio::test]
    async fn a_member_file_goes_on_a_note_a_draft_or_a_reply() {
        for kind in [EntryKind::InternalNote, EntryKind::Draft, EntryKind::Reply] {
            let held = entry(kind);
            let mut entries = entries_holding(&held);
            echoing_attachments(&mut entries);
            let mut service = ThreadService::new(
                MockTicketRepository::new(),
                entries,
                RecordingEmitter::new(),
            );
            let started = Utc::now();

            let attachment = service
                .attach_member_file(organization(), held.id, new_file("  capture.png  "))
                .await
                .unwrap();

            assert_eq!(attachment.organization_id, organization());
            assert_eq!(attachment.ticket_id, held.ticket_id);
            assert_eq!(attachment.entry_id, held.id);
            assert_eq!(attachment.file_key, "support/2026/10/capture.png");
            assert_eq!(attachment.file_name, "capture.png");
            assert_eq!(attachment.mime_type, "image/png");
            assert_eq!(attachment.size_bytes, 48_213);
            assert!(attachment.created_at >= started);
        }
    }

    #[tokio::test]
    async fn a_client_file_only_goes_on_a_client_message() {
        for kind in [EntryKind::Reply, EntryKind::InternalNote, EntryKind::Draft] {
            let held = entry(kind);
            let mut service = ThreadService::new(
                MockTicketRepository::new(),
                entries_holding(&held),
                RecordingEmitter::new(),
            );

            let outcome = service
                .attach_client_file(organization(), held.id, new_file("capture.png"))
                .await;

            conflict_mentioning(outcome, "client message");
        }

        let message = entry(EntryKind::ClientMessage);
        let mut entries = entries_holding(&message);
        echoing_attachments(&mut entries);
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            entries,
            RecordingEmitter::new(),
        );

        let attachment = service
            .attach_client_file(organization(), message.id, new_file("capture.png"))
            .await
            .unwrap();

        assert_eq!(attachment.entry_id, message.id);
        assert_eq!(attachment.ticket_id, message.ticket_id);
        assert_eq!(attachment.file_name, "capture.png");
    }

    #[tokio::test]
    async fn a_blank_file_name_is_refused() {
        let note = entry(EntryKind::InternalNote);
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            entries_holding(&note),
            RecordingEmitter::new(),
        );

        let outcome = service
            .attach_member_file(organization(), note.id, new_file("   "))
            .await;

        conflict_mentioning(outcome, "file name");
    }

    #[tokio::test]
    async fn an_entry_of_another_organization_is_not_found() {
        let mut entries = MockTicketEntryRepository::new();
        entries
            .expect_find()
            .times(5)
            .returning(|organization_id, _| {
                let held = (organization_id == organization()).then(|| entry(EntryKind::Draft));
                Box::pin(async move { Ok(held) })
            });
        entries
            .expect_find_attachment()
            .times(1)
            .returning(|organization_id, _| {
                let held = (organization_id == organization()).then(|| LocatedAttachment {
                    attachment: Attachment {
                        id: AttachmentId(Uuid::from_u128(0x50)),
                        organization_id: organization(),
                        ticket_id: ticket_id(),
                        entry_id: entry_id(),
                        file_key: "support/capture.png".to_owned(),
                        file_name: "capture.png".to_owned(),
                        mime_type: "image/png".to_owned(),
                        size_bytes: 1,
                        created_at: past(),
                    },
                    entry_kind: EntryKind::Draft,
                });
                Box::pin(async move { Ok(held) })
            });
        let other = other_organization();
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            entries,
            RecordingEmitter::new(),
        );

        assert!(matches!(
            service.edit_draft(other, entry_id(), "Autre").await,
            Err(CoreError::NotFound)
        ));
        assert!(matches!(
            service.delete_draft(other, entry_id()).await,
            Err(CoreError::NotFound)
        ));
        assert!(matches!(
            service.send_draft(other, entry_id(), member()).await,
            Err(CoreError::NotFound)
        ));
        assert!(matches!(
            service
                .attach_member_file(other, entry_id(), new_file("capture.png"))
                .await,
            Err(CoreError::NotFound)
        ));
        assert!(matches!(
            service
                .attach_client_file(other, entry_id(), new_file("capture.png"))
                .await,
            Err(CoreError::NotFound)
        ));
        assert!(matches!(
            service
                .get_attachment(other, AttachmentId(Uuid::from_u128(0x50)))
                .await,
            Err(CoreError::NotFound)
        ));
    }

    #[tokio::test]
    async fn list_of_a_ticket_of_another_organization_is_not_found() {
        let mut tickets = MockTicketRepository::new();
        tickets
            .expect_find()
            .times(1)
            .returning(|organization_id, id| {
                let held = (organization_id == organization() && id == ticket_id())
                    .then(|| ticket(TicketStatus::New));
                Box::pin(async move { Ok(held) })
            });
        let mut service = ThreadService::new(
            tickets,
            MockTicketEntryRepository::new(),
            RecordingEmitter::new(),
        );

        let outcome = service.list(other_organization(), ticket_id(), true).await;

        assert!(matches!(outcome, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn list_hands_the_visibility_flag_to_the_repository() {
        let listed = vec![entry(EntryKind::ClientMessage), entry(EntryKind::Reply)];
        let returned = listed.clone();
        let mut entries = MockTicketEntryRepository::new();
        entries
            .expect_list_for_ticket()
            .withf(|organization_id, id, client_visible_only| {
                *organization_id == organization() && *id == ticket_id() && *client_visible_only
            })
            .times(1)
            .returning(move |_, _, _| {
                let returned = returned.clone();
                Box::pin(async move { Ok(returned) })
            });
        let mut service = ThreadService::new(
            tickets_holding(&ticket(TicketStatus::New)),
            entries,
            RecordingEmitter::new(),
        );

        let thread = service
            .list(organization(), ticket_id(), true)
            .await
            .unwrap();

        assert_eq!(thread, listed);
    }

    #[tokio::test]
    async fn get_attachment_returns_the_located_attachment_and_reports_an_absent_one() {
        let located = LocatedAttachment {
            attachment: Attachment {
                id: AttachmentId(Uuid::from_u128(0x50)),
                organization_id: organization(),
                ticket_id: ticket_id(),
                entry_id: entry_id(),
                file_key: "support/capture.png".to_owned(),
                file_name: "capture.png".to_owned(),
                mime_type: "image/png".to_owned(),
                size_bytes: 48_213,
                created_at: past(),
            },
            entry_kind: EntryKind::Reply,
        };
        let returned = located.clone();
        let mut entries = MockTicketEntryRepository::new();
        entries
            .expect_find_attachment()
            .withf(|organization_id, id| {
                *organization_id == organization() && *id == AttachmentId(Uuid::from_u128(0x50))
            })
            .times(1)
            .returning(move |_, _| {
                let returned = returned.clone();
                Box::pin(async move { Ok(Some(returned)) })
            });
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            entries,
            RecordingEmitter::new(),
        );

        let found = service
            .get_attachment(organization(), located.attachment.id)
            .await
            .unwrap();

        assert_eq!(found, located);

        let mut entries = MockTicketEntryRepository::new();
        entries
            .expect_find_attachment()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        let mut service = ThreadService::new(
            MockTicketRepository::new(),
            entries,
            RecordingEmitter::new(),
        );

        assert!(matches!(
            service
                .get_attachment(organization(), located.attachment.id)
                .await,
            Err(CoreError::NotFound)
        ));
    }
}
