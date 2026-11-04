use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    CreatedTicket, OrganizationId, Ticket, TicketId, TicketStatus,
    domain::support::{
        assignment::pick_least_recently_assigned,
        commands::{
            AddBadgeCommand, AssignTicketCommand, AssignToLeastRecentCommand, CreateTicketCommand,
            ListTicketsCommand, RemoveBadgeCommand, TransitionTicketCommand,
        },
        events::{TicketCreated, TicketTransitioned},
        ports::{TicketPage, TicketRepository},
    },
};

pub struct TicketService<R, E>
where
    R: TicketRepository,
    E: EventEmitter,
{
    tickets: R,
    emitter: E,
}

impl<R, E> TicketService<R, E>
where
    R: TicketRepository,
    E: EventEmitter,
{
    pub fn new(tickets: R, emitter: E) -> Self {
        Self { tickets, emitter }
    }

    pub async fn create(
        &mut self,
        command: CreateTicketCommand,
    ) -> Result<CreatedTicket, CoreError> {
        let subject = command.subject.trim();
        if subject.is_empty() {
            return Err(CoreError::Conflict(
                "ticket subject must not be empty".to_owned(),
            ));
        }
        let external_id = command
            .external_id
            .as_deref()
            .map(str::trim)
            .filter(|external_id| !external_id.is_empty());

        if let Some(external_id) = external_id
            && let Some(existing) = self
                .tickets
                .find_by_external_id(command.organization_id, external_id)
                .await?
        {
            return Ok(CreatedTicket {
                ticket: existing,
                created: false,
            });
        }

        let number = self.tickets.next_number(command.organization_id).await?;
        let now = Utc::now();
        let built = Ticket {
            id: TicketId(generate_uuid_v7()),
            organization_id: command.organization_id,
            number,
            subject: subject.to_owned(),
            body: command.body,
            status: TicketStatus::New,
            assignee: None,
            assigned_at: None,
            custom_fields: command.custom_fields,
            badges: Vec::new(),
            external_id: external_id.map(str::to_owned),
            created_at: now,
            updated_at: now,
        };

        let stored = self.tickets.create(&built).await?;
        if stored.id != built.id {
            return Ok(CreatedTicket {
                ticket: stored,
                created: false,
            });
        }

        self.emitter.emit(
            stored.organization_id,
            &TicketCreated {
                ticket: stored.clone(),
            },
        )?;
        Ok(CreatedTicket {
            ticket: stored,
            created: true,
        })
    }

    pub async fn get(
        &mut self,
        organization_id: OrganizationId,
        ticket_id: TicketId,
    ) -> Result<Ticket, CoreError> {
        self.tickets
            .find(organization_id, ticket_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn list(&mut self, command: ListTicketsCommand) -> Result<TicketPage, CoreError> {
        self.tickets
            .list(
                command.organization_id,
                &command.filter,
                command.offset,
                command.limit,
            )
            .await
    }

    pub async fn transition(
        &mut self,
        command: TransitionTicketCommand,
    ) -> Result<Ticket, CoreError> {
        let mut ticket = self.get(command.organization_id, command.ticket_id).await?;
        let from = ticket.status;
        if from == command.status {
            return Ok(ticket);
        }

        ticket.status = command.status;
        ticket.updated_at = Utc::now();
        let updated = self.tickets.update(&ticket).await?;

        self.emitter.emit(
            updated.organization_id,
            &TicketTransitioned {
                ticket: updated.clone(),
                from,
                to: command.status,
            },
        )?;
        Ok(updated)
    }

    pub async fn assign(&mut self, command: AssignTicketCommand) -> Result<Ticket, CoreError> {
        let mut ticket = self.get(command.organization_id, command.ticket_id).await?;
        let now = Utc::now();

        ticket.assignee = command.assignee;
        ticket.assigned_at = command.assignee.map(|_| now);
        ticket.updated_at = now;
        self.tickets.update(&ticket).await
    }

    pub async fn assign_to_least_recent(
        &mut self,
        command: AssignToLeastRecentCommand,
    ) -> Result<Ticket, CoreError> {
        if command.members.is_empty() {
            return Err(CoreError::Conflict(
                "at least one member is required to assign a ticket".to_owned(),
            ));
        }

        let last_assigned = self
            .tickets
            .last_assigned_at(command.organization_id, &command.members)
            .await?;
        let assignee = pick_least_recently_assigned(&command.members, &last_assigned)
            .ok_or_else(|| CoreError::Internal("no member could be picked".to_owned()))?;

        self.assign(AssignTicketCommand {
            actor: command.actor,
            organization_id: command.organization_id,
            ticket_id: command.ticket_id,
            assignee: Some(assignee),
        })
        .await
    }

    pub async fn add_badge(&mut self, command: AddBadgeCommand) -> Result<Ticket, CoreError> {
        let mut ticket = self.get(command.organization_id, command.ticket_id).await?;

        match ticket
            .badges
            .iter()
            .position(|badge| badge.label() == command.badge.label())
        {
            Some(index) => ticket.badges[index] = command.badge,
            None => ticket.badges.push(command.badge),
        }
        ticket.updated_at = Utc::now();
        self.tickets.update(&ticket).await
    }

    pub async fn remove_badge(&mut self, command: RemoveBadgeCommand) -> Result<Ticket, CoreError> {
        let label = command.label.trim();
        let mut ticket = self.get(command.organization_id, command.ticket_id).await?;
        let Some(index) = ticket
            .badges
            .iter()
            .position(|badge| badge.label() == label)
        else {
            return Ok(ticket);
        };

        ticket.badges.remove(index);
        ticket.updated_at = Utc::now();
        self.tickets.update(&ticket).await
    }
}

#[cfg(test)]
mod tests {
    use authz::Subject;
    use chrono::{DateTime, Utc};
    use common::CoreError;
    use events::testing::RecordingEmitter;
    use serde_json::json;
    use uuid::Uuid;

    use super::*;
    use crate::{
        Badge, BadgeTone, CustomFields, MemberId, OrganizationId, Ticket, TicketId, TicketNumber,
        TicketStatus,
        domain::support::{
            commands::{
                AddBadgeCommand, AssignTicketCommand, CreateTicketCommand, ListTicketsCommand,
                RemoveBadgeCommand, TransitionTicketCommand,
            },
            ports::{MockTicketRepository, TicketFilter, TicketPage},
        },
    };

    fn organization() -> OrganizationId {
        OrganizationId(Uuid::from_u128(0x02))
    }

    fn past() -> DateTime<Utc> {
        "2026-10-15T07:45:00Z".parse().unwrap()
    }

    fn ticket(status: TicketStatus) -> Ticket {
        Ticket {
            id: TicketId(Uuid::from_u128(0x30)),
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

    fn badge(label: &str, tone: BadgeTone) -> Badge {
        Badge::new(label, tone).unwrap()
    }

    fn holding(ticket: &Ticket) -> MockTicketRepository {
        let held = ticket.clone();
        let mut tickets = MockTicketRepository::new();
        tickets.expect_find().times(1).returning(move |_, _| {
            let held = held.clone();
            Box::pin(async move { Ok(Some(held)) })
        });
        tickets
    }

    fn echoing_updates(tickets: &mut MockTicketRepository) {
        tickets.expect_update().times(1).returning(|ticket| {
            let ticket = ticket.clone();
            Box::pin(async move { Ok(ticket) })
        });
    }

    fn create_command(subject: &str, external_id: Option<&str>) -> CreateTicketCommand {
        CreateTicketCommand {
            actor: Subject::system(),
            organization_id: organization(),
            subject: subject.to_owned(),
            body: "Le déploiement dpl_8f2a91 échoue.".to_owned(),
            custom_fields: CustomFields::try_from(json!({
                "deployment_id": "dpl_8f2a91",
                "team": "tech",
            }))
            .unwrap(),
            external_id: external_id.map(str::to_owned),
        }
    }

    #[tokio::test]
    async fn create_emits_ticket_created_with_the_full_ticket() {
        let mut tickets = MockTicketRepository::new();
        tickets
            .expect_find_by_external_id()
            .withf(|organization_id, external_id| {
                *organization_id == organization() && external_id == "ZD-48213"
            })
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        tickets
            .expect_next_number()
            .times(1)
            .returning(|_| Box::pin(async { Ok(TicketNumber(7)) }));
        echoing_creates(&mut tickets);
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);

        let created = service
            .create(create_command(
                "  Impossible de déployer  ",
                Some(" ZD-48213 "),
            ))
            .await
            .unwrap();

        assert!(created.created);
        let ticket = &created.ticket;
        assert_eq!(ticket.organization_id, organization());
        assert_eq!(ticket.number, TicketNumber(7));
        assert_eq!(ticket.subject, "Impossible de déployer");
        assert_eq!(ticket.body, "Le déploiement dpl_8f2a91 échoue.");
        assert_eq!(ticket.status, TicketStatus::New);
        assert_eq!(ticket.assignee, None);
        assert_eq!(ticket.assigned_at, None);
        assert!(ticket.badges.is_empty());
        assert_eq!(ticket.external_id.as_deref(), Some("ZD-48213"));
        assert_eq!(ticket.created_at, ticket.updated_at);
        assert_eq!(
            ticket.custom_fields.as_map().get("deployment_id"),
            Some(&json!("dpl_8f2a91"))
        );
        assert_eq!(emitter.names(), vec!["ticket.created"]);
        let envelope = emitter.only("ticket.created");
        assert_eq!(envelope.org_id, organization());
        assert_eq!(envelope.version, 1);
        assert_eq!(envelope.subject_kind, "ticket");
        assert_eq!(envelope.subject_id, Some(ticket.id.0));
        assert_eq!(envelope.payload["ticket"]["id"], json!(ticket.id.0));
        assert_eq!(envelope.payload["ticket"]["number"], json!(7));
        assert_eq!(
            envelope.payload["ticket"]["subject"],
            json!("Impossible de déployer")
        );
        assert_eq!(envelope.payload["ticket"]["status"], json!("new"));
        assert_eq!(envelope.payload["ticket"]["external_id"], json!("ZD-48213"));
        assert_eq!(
            envelope.payload["ticket"]["custom_fields"],
            json!({ "deployment_id": "dpl_8f2a91", "team": "tech" })
        );
    }

    #[tokio::test]
    async fn create_with_a_known_external_id_returns_the_existing_ticket_and_emits_nothing() {
        let existing = ticket(TicketStatus::InProgress);
        let known = existing.clone();
        let mut tickets = MockTicketRepository::new();
        tickets
            .expect_find_by_external_id()
            .times(1)
            .returning(move |_, _| {
                let known = known.clone();
                Box::pin(async move { Ok(Some(known)) })
            });
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);

        let created = service
            .create(create_command("Doublon", Some("ZD-48213")))
            .await
            .unwrap();

        assert!(!created.created);
        assert_eq!(created.ticket, existing);
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn create_when_a_concurrent_insert_won_returns_it_and_emits_nothing() {
        let winner = ticket(TicketStatus::New);
        let stored = winner.clone();
        let mut tickets = MockTicketRepository::new();
        tickets
            .expect_find_by_external_id()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        tickets
            .expect_next_number()
            .times(1)
            .returning(|_| Box::pin(async { Ok(TicketNumber(43)) }));
        tickets.expect_create().times(1).returning(move |built| {
            assert_ne!(built.id, stored.id);
            let stored = stored.clone();
            Box::pin(async move { Ok(stored) })
        });
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);

        let created = service
            .create(create_command("Doublon concurrent", Some("ZD-48213")))
            .await
            .unwrap();

        assert!(!created.created);
        assert_eq!(created.ticket, winner);
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn create_rejects_a_blank_subject() {
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(MockTicketRepository::new(), &emitter);

        let outcome = service.create(create_command("   ", None)).await;

        match outcome {
            Err(CoreError::Conflict(message)) => assert!(message.contains("subject")),
            other => panic!("{other:?}"),
        }
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn create_without_an_external_id_skips_the_lookup_and_stores_none() {
        let mut tickets = MockTicketRepository::new();
        tickets
            .expect_next_number()
            .times(1)
            .returning(|_| Box::pin(async { Ok(TicketNumber(1)) }));
        echoing_creates(&mut tickets);
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);

        let created = service
            .create(create_command("Sans identifiant", Some("   ")))
            .await
            .unwrap();

        assert!(created.created);
        assert_eq!(created.ticket.external_id, None);
        assert_eq!(emitter.names(), vec!["ticket.created"]);
    }

    #[tokio::test]
    async fn transition_emits_ticket_transitioned_with_from_and_to() {
        let before = ticket(TicketStatus::WaitingCustomer);
        let mut tickets = holding(&before);
        echoing_updates(&mut tickets);
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);

        let updated = service
            .transition(TransitionTicketCommand {
                actor: Subject::system(),
                organization_id: organization(),
                ticket_id: before.id,
                status: TicketStatus::InProgress,
            })
            .await
            .unwrap();

        assert_eq!(updated.status, TicketStatus::InProgress);
        assert!(updated.updated_at > before.updated_at);
        assert_eq!(updated.created_at, before.created_at);
        assert_eq!(emitter.names(), vec!["ticket.transitioned"]);
        let envelope = emitter.only("ticket.transitioned");
        assert_eq!(envelope.org_id, organization());
        assert_eq!(envelope.subject_kind, "ticket");
        assert_eq!(envelope.subject_id, Some(before.id.0));
        assert_eq!(envelope.payload["from"], json!("waiting_customer"));
        assert_eq!(envelope.payload["to"], json!("in_progress"));
        assert_eq!(envelope.payload["ticket"]["id"], json!(before.id.0));
        assert_eq!(envelope.payload["ticket"]["status"], json!("in_progress"));
    }

    #[tokio::test]
    async fn transition_to_the_same_status_is_a_no_op_without_event() {
        let before = ticket(TicketStatus::New);
        let tickets = holding(&before);
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);

        let unchanged = service
            .transition(TransitionTicketCommand {
                actor: Subject::system(),
                organization_id: organization(),
                ticket_id: before.id,
                status: TicketStatus::New,
            })
            .await
            .unwrap();

        assert_eq!(unchanged, before);
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn transition_of_an_unknown_ticket_is_not_found() {
        let mut tickets = MockTicketRepository::new();
        tickets
            .expect_find()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);

        let outcome = service
            .transition(TransitionTicketCommand {
                actor: Subject::system(),
                organization_id: organization(),
                ticket_id: TicketId(Uuid::from_u128(0x99)),
                status: TicketStatus::Resolved,
            })
            .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)));
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn assign_sets_assigned_at() {
        let before = ticket(TicketStatus::New);
        let member = MemberId(Uuid::from_u128(0x12));
        let mut tickets = holding(&before);
        echoing_updates(&mut tickets);
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);
        let started = Utc::now();

        let updated = service
            .assign(AssignTicketCommand {
                actor: Subject::system(),
                organization_id: organization(),
                ticket_id: before.id,
                assignee: Some(member),
            })
            .await
            .unwrap();

        assert_eq!(updated.assignee, Some(member));
        let assigned_at = updated.assigned_at.unwrap();
        assert!(assigned_at >= started);
        assert_eq!(updated.updated_at, assigned_at);
        assert_eq!(updated.status, TicketStatus::New);
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn unassign_clears_assigned_at() {
        let mut before = ticket(TicketStatus::InProgress);
        before.assignee = Some(MemberId(Uuid::from_u128(0x12)));
        before.assigned_at = Some(past());
        let mut tickets = holding(&before);
        echoing_updates(&mut tickets);
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);

        let updated = service
            .assign(AssignTicketCommand {
                actor: Subject::system(),
                organization_id: organization(),
                ticket_id: before.id,
                assignee: None,
            })
            .await
            .unwrap();

        assert_eq!(updated.assignee, None);
        assert_eq!(updated.assigned_at, None);
        assert!(updated.updated_at > before.updated_at);
        assert!(emitter.names().is_empty());
    }

    fn least_recent_command(
        ticket_id: TicketId,
        members: &[MemberId],
    ) -> AssignToLeastRecentCommand {
        AssignToLeastRecentCommand {
            actor: Subject::system(),
            organization_id: organization(),
            ticket_id,
            members: members.to_vec(),
        }
    }

    fn knowing(
        tickets: &mut MockTicketRepository,
        members: Vec<MemberId>,
        known: Vec<(MemberId, Option<DateTime<Utc>>)>,
    ) {
        tickets
            .expect_last_assigned_at()
            .withf(move |organization_id, requested| {
                *organization_id == organization() && requested == members.as_slice()
            })
            .times(1)
            .returning(move |_, _| {
                let known = known.clone();
                Box::pin(async move { Ok(known) })
            });
    }

    #[tokio::test]
    async fn assign_to_least_recent_picks_a_never_assigned_member_and_stamps_assigned_at() {
        let before = ticket(TicketStatus::New);
        let members = [
            MemberId(Uuid::from_u128(0x11)),
            MemberId(Uuid::from_u128(0x12)),
            MemberId(Uuid::from_u128(0x13)),
        ];
        let mut tickets = holding(&before);
        knowing(
            &mut tickets,
            members.to_vec(),
            vec![
                (members[0], Some(past())),
                (members[1], None),
                (members[2], Some(past())),
            ],
        );
        echoing_updates(&mut tickets);
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);
        let started = Utc::now();

        let updated = service
            .assign_to_least_recent(least_recent_command(before.id, &members))
            .await
            .unwrap();

        assert_eq!(updated.assignee, Some(members[1]));
        let assigned_at = updated.assigned_at.unwrap();
        assert!(assigned_at >= started);
        assert_eq!(updated.updated_at, assigned_at);
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn assign_to_least_recent_picks_the_oldest_assignment_when_everyone_has_one() {
        let before = ticket(TicketStatus::New);
        let members = [
            MemberId(Uuid::from_u128(0x11)),
            MemberId(Uuid::from_u128(0x12)),
        ];
        let older: DateTime<Utc> = "2026-10-10T07:45:00Z".parse().unwrap();
        let mut tickets = holding(&before);
        knowing(
            &mut tickets,
            members.to_vec(),
            vec![(members[0], Some(past())), (members[1], Some(older))],
        );
        echoing_updates(&mut tickets);
        let mut service = TicketService::new(tickets, RecordingEmitter::new());

        let updated = service
            .assign_to_least_recent(least_recent_command(before.id, &members))
            .await
            .unwrap();

        assert_eq!(updated.assignee, Some(members[1]));
    }

    #[tokio::test]
    async fn assign_to_least_recent_rejects_an_empty_list_without_reading_or_writing() {
        let mut service = TicketService::new(MockTicketRepository::new(), RecordingEmitter::new());

        let outcome = service
            .assign_to_least_recent(least_recent_command(TicketId(Uuid::from_u128(0x30)), &[]))
            .await;

        match outcome {
            Err(CoreError::Conflict(message)) => assert!(message.contains("member")),
            other => panic!("{other:?}"),
        }
    }

    #[tokio::test]
    async fn assign_to_least_recent_of_an_unknown_ticket_is_not_found_and_writes_nothing() {
        let members = [MemberId(Uuid::from_u128(0x11))];
        let mut tickets = MockTicketRepository::new();
        knowing(&mut tickets, members.to_vec(), vec![(members[0], None)]);
        tickets
            .expect_find()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        tickets.expect_update().never();
        let mut service = TicketService::new(tickets, RecordingEmitter::new());

        let outcome = service
            .assign_to_least_recent(least_recent_command(
                TicketId(Uuid::from_u128(0x99)),
                &members,
            ))
            .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn add_badge_deduplicates_by_label() {
        let mut before = ticket(TicketStatus::New);
        before.badges = vec![
            badge("Bloquant", BadgeTone::Neutral),
            badge("VIP", BadgeTone::Brand),
        ];
        let emitter = RecordingEmitter::new();

        let mut replacing = holding(&before);
        echoing_updates(&mut replacing);
        let replaced = TicketService::new(replacing, &emitter)
            .add_badge(AddBadgeCommand {
                actor: Subject::system(),
                organization_id: organization(),
                ticket_id: before.id,
                badge: badge("Bloquant", BadgeTone::Error),
            })
            .await
            .unwrap();

        assert_eq!(
            replaced.badges,
            vec![
                badge("Bloquant", BadgeTone::Error),
                badge("VIP", BadgeTone::Brand)
            ]
        );

        let mut appending = holding(&before);
        echoing_updates(&mut appending);
        let appended = TicketService::new(appending, &emitter)
            .add_badge(AddBadgeCommand {
                actor: Subject::system(),
                organization_id: organization(),
                ticket_id: before.id,
                badge: badge("Nouveau client", BadgeTone::Success),
            })
            .await
            .unwrap();

        assert_eq!(
            appended.badges,
            vec![
                badge("Bloquant", BadgeTone::Neutral),
                badge("VIP", BadgeTone::Brand),
                badge("Nouveau client", BadgeTone::Success)
            ]
        );
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn remove_badge_drops_the_badge_with_that_label() {
        let mut before = ticket(TicketStatus::New);
        before.badges = vec![
            badge("Bloquant", BadgeTone::Error),
            badge("VIP", BadgeTone::Brand),
        ];
        let mut tickets = holding(&before);
        echoing_updates(&mut tickets);
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);

        let updated = service
            .remove_badge(RemoveBadgeCommand {
                actor: Subject::system(),
                organization_id: organization(),
                ticket_id: before.id,
                label: "  Bloquant ".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(updated.badges, vec![badge("VIP", BadgeTone::Brand)]);
        assert!(updated.updated_at > before.updated_at);
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn remove_badge_of_an_unknown_label_is_a_no_op() {
        let mut before = ticket(TicketStatus::New);
        before.badges = vec![badge("Bloquant", BadgeTone::Error)];
        let tickets = holding(&before);
        let emitter = RecordingEmitter::new();
        let mut service = TicketService::new(tickets, &emitter);

        let unchanged = service
            .remove_badge(RemoveBadgeCommand {
                actor: Subject::system(),
                organization_id: organization(),
                ticket_id: before.id,
                label: " VIP ".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(unchanged, before);
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn get_returns_the_ticket_and_reports_an_absent_one_as_not_found() {
        let held = ticket(TicketStatus::Resolved);
        let mut found = TicketService::new(holding(&held), RecordingEmitter::new());

        assert_eq!(found.get(organization(), held.id).await.unwrap(), held);

        let mut tickets = MockTicketRepository::new();
        tickets
            .expect_find()
            .times(1)
            .returning(|_, _| Box::pin(async { Ok(None) }));
        let mut absent = TicketService::new(tickets, RecordingEmitter::new());

        assert!(matches!(
            absent.get(organization(), held.id).await,
            Err(CoreError::NotFound)
        ));
    }

    #[tokio::test]
    async fn list_passes_the_filter_and_the_window_through() {
        let listed = ticket(TicketStatus::New);
        let page = TicketPage {
            items: vec![listed.clone()],
            total: 31,
        };
        let returned = page.clone();
        let mut tickets = MockTicketRepository::new();
        tickets
            .expect_list()
            .withf(|organization_id, filter, offset, limit| {
                *organization_id == organization()
                    && filter.statuses == [TicketStatus::New, TicketStatus::InProgress]
                    && *offset == 20
                    && *limit == 10
            })
            .times(1)
            .returning(move |_, _, _, _| {
                let returned = returned.clone();
                Box::pin(async move { Ok(returned) })
            });
        let mut service = TicketService::new(tickets, RecordingEmitter::new());

        let result = service
            .list(ListTicketsCommand {
                organization_id: organization(),
                filter: TicketFilter {
                    statuses: vec![TicketStatus::New, TicketStatus::InProgress],
                    ..TicketFilter::default()
                },
                offset: 20,
                limit: 10,
            })
            .await
            .unwrap();

        assert_eq!(result, page);
    }

    fn echoing_creates(tickets: &mut MockTicketRepository) {
        tickets.expect_create().times(1).returning(|ticket| {
            let ticket = ticket.clone();
            Box::pin(async move { Ok(ticket) })
        });
    }
}
