use std::collections::HashMap;

use chrono::{Duration, Utc};
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    Apprentice, ApprenticeId, CompanyTutor, CompanyTutorId, CompanyTutorRecord, OrganizationId,
    Participant, ParticipantDetail, ParticipantId, ParticipantInvite, ParticipantListItem,
    domain::{
        cursus_participant::{
            INVITATION_VALIDITY_DAYS,
            commands::{
                AcceptParticipantInvitationCommand, AddCompanyTutorCommand, ApprenticeInput,
                CompanyTutorInput, CreateParticipantCommand, ParticipantFilter,
                RemoveCompanyTutorCommand, UpdateParticipantCommand, UpsertApprenticeCommand,
            },
            events::{ParticipantActivated, ParticipantInvited},
            ports::CursusParticipantRepository,
        },
        invitation::token,
    },
};

pub struct CursusParticipantService<R, E>
where
    R: CursusParticipantRepository,
    E: EventEmitter,
{
    repo: R,
    emitter: E,
}

impl<R, E> CursusParticipantService<R, E>
where
    R: CursusParticipantRepository,
    E: EventEmitter,
{
    pub fn new(repo: R, emitter: E) -> Self {
        Self { repo, emitter }
    }

    pub async fn create(
        &mut self,
        command: CreateParticipantCommand,
    ) -> Result<ParticipantDetail, CoreError> {
        let last_name = required("participant last name", &command.last_name)?;
        let email = valid_email(&command.email)?;
        if command.apprentice.is_some() && command.company_tutor.is_some() {
            return Err(CoreError::Conflict(
                "a participant is either an apprentice or a company tutor at creation".to_owned(),
            ));
        }
        self.ensure_email_free(command.organization_id, &email, None)
            .await?;

        let now = Utc::now();
        let participant = self
            .repo
            .insert_participant(&Participant {
                id: ParticipantId(generate_uuid_v7()),
                organization_id: command.organization_id,
                user_id: None,
                last_name,
                first_name: optional(command.first_name),
                email,
                phone: optional(command.phone),
                invitation_token_hash: None,
                invitation_expires_at: None,
                deactivated_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        if let Some(input) = command.apprentice {
            self.repo
                .upsert_apprentice(&new_apprentice(&participant, input))
                .await?;
        }
        if let Some(input) = command.company_tutor {
            self.repo
                .insert_company_tutor(&new_company_tutor(&participant, input))
                .await?;
        }

        self.detail(participant).await
    }

    pub async fn get(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> Result<Participant, CoreError> {
        self.repo
            .find_participant(organization_id, id)
            .await?
            .filter(|found| found.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn get_apprentice(
        &mut self,
        organization_id: OrganizationId,
        id: ApprenticeId,
    ) -> Result<Apprentice, CoreError> {
        self.repo
            .find_apprentice_by_id(organization_id, id)
            .await?
            .filter(|found| found.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn get_company_tutor(
        &mut self,
        organization_id: OrganizationId,
        id: CompanyTutorId,
    ) -> Result<CompanyTutor, CoreError> {
        self.repo
            .find_company_tutor(organization_id, id)
            .await?
            .filter(|found| found.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn active_participant(
        &mut self,
        organization_id: OrganizationId,
        user_id: crate::UserId,
    ) -> Result<Participant, CoreError> {
        self.repo
            .find_participant_by_user(organization_id, user_id)
            .await?
            .filter(|found| {
                found.organization_id == organization_id && found.deactivated_at.is_none()
            })
            .ok_or(CoreError::Forbidden { reason: None })
    }

    pub async fn list_active_participations(
        &mut self,
        user_id: crate::UserId,
    ) -> Result<Vec<Participant>, CoreError> {
        self.repo.list_active_participants_by_user(user_id).await
    }

    pub async fn get_detail(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> Result<ParticipantDetail, CoreError> {
        let participant = self.get(organization_id, id).await?;
        self.detail(participant).await
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
        filter: &ParticipantFilter,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ParticipantListItem>, u64), CoreError> {
        let (participants, total) = self
            .repo
            .list_participants(organization_id, filter, limit, offset)
            .await?;
        let ids: Vec<ParticipantId> = participants.iter().map(|p| p.id).collect();
        let mut apprentice_ids: HashMap<ParticipantId, ApprenticeId> = self
            .repo
            .list_apprentice_ids(organization_id, &ids)
            .await?
            .into_iter()
            .collect();
        let mut tutors: HashMap<ParticipantId, Vec<CompanyTutorRecord>> = HashMap::new();
        for record in self
            .repo
            .list_company_tutors_of(organization_id, &ids)
            .await?
        {
            tutors
                .entry(record.tutor.participant_id)
                .or_default()
                .push(record);
        }

        let items = participants
            .into_iter()
            .map(|participant| ParticipantListItem {
                apprentice_id: apprentice_ids.remove(&participant.id),
                company_tutors: tutors.remove(&participant.id).unwrap_or_default(),
                participant,
            })
            .collect();

        Ok((items, total))
    }

    pub async fn update(
        &mut self,
        command: UpdateParticipantCommand,
    ) -> Result<ParticipantDetail, CoreError> {
        let last_name = required("participant last name", &command.last_name)?;
        let email = valid_email(&command.email)?;

        let mut participant = self.get(command.organization_id, command.id).await?;
        self.ensure_email_free(command.organization_id, &email, Some(participant.id))
            .await?;
        participant.last_name = last_name;
        participant.first_name = optional(command.first_name);
        participant.email = email;
        participant.phone = optional(command.phone);
        participant.updated_at = Utc::now();

        let updated = self.repo.update_participant(&participant).await?;
        self.detail(updated).await
    }

    pub async fn invite(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> Result<ParticipantInvite, CoreError> {
        let participant = self.get(organization_id, id).await?;
        if participant.user_id.is_some() {
            return Err(CoreError::Conflict(
                "participant is already linked to a user".to_owned(),
            ));
        }
        if participant.deactivated_at.is_some() {
            return Err(CoreError::Conflict("participant is deactivated".to_owned()));
        }

        let now = Utc::now();
        let expires_at = now + Duration::days(INVITATION_VALIDITY_DAYS);
        let (clear_token, token_hash) = token::generate()?;
        let invited = self
            .repo
            .set_invitation(organization_id, id, &token_hash, expires_at, now)
            .await?;

        self.emitter.emit(
            organization_id,
            &ParticipantInvited {
                participant: invited.clone(),
                expires_at,
            },
        )?;

        Ok(ParticipantInvite {
            participant: invited,
            token: clear_token,
            expires_at,
        })
    }

    pub async fn deactivate(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> Result<Participant, CoreError> {
        let participant = self.get(organization_id, id).await?;
        if participant.deactivated_at.is_some() {
            return Ok(participant);
        }

        let now = Utc::now();
        self.repo
            .set_deactivated_at(organization_id, id, Some(now), now)
            .await
    }

    pub async fn reactivate(
        &mut self,
        organization_id: OrganizationId,
        id: ParticipantId,
    ) -> Result<Participant, CoreError> {
        let participant = self.get(organization_id, id).await?;
        if participant.deactivated_at.is_none() {
            return Ok(participant);
        }

        self.repo
            .set_deactivated_at(organization_id, id, None, Utc::now())
            .await
    }

    pub async fn upsert_apprentice(
        &mut self,
        command: UpsertApprenticeCommand,
    ) -> Result<ParticipantDetail, CoreError> {
        let participant = self
            .get(command.organization_id, command.participant_id)
            .await?;
        self.repo
            .upsert_apprentice(&new_apprentice(&participant, command.apprentice))
            .await?;

        self.detail(participant).await
    }

    pub async fn add_company_tutor(
        &mut self,
        command: AddCompanyTutorCommand,
    ) -> Result<ParticipantDetail, CoreError> {
        let participant = self
            .get(command.organization_id, command.participant_id)
            .await?;
        self.repo
            .insert_company_tutor(&new_company_tutor(&participant, command.company_tutor))
            .await?;

        self.detail(participant).await
    }

    pub async fn remove_company_tutor(
        &mut self,
        command: RemoveCompanyTutorCommand,
    ) -> Result<(), CoreError> {
        self.repo
            .delete_company_tutor(command.organization_id, command.id)
            .await
    }

    pub async fn accept(
        &mut self,
        command: AcceptParticipantInvitationCommand,
    ) -> Result<Participant, CoreError> {
        let token_hash = token::hash(&command.token);
        let participant = self
            .repo
            .find_participant_by_token_hash(&token_hash)
            .await?
            .ok_or(CoreError::Gone)?;

        let now = Utc::now();
        if participant
            .invitation_expires_at
            .is_none_or(|expires_at| expires_at <= now)
        {
            return Err(CoreError::Gone);
        }
        if participant.deactivated_at.is_some() {
            return Err(CoreError::Gone);
        }
        match participant.user_id {
            Some(linked) if linked == command.user_id => return Err(CoreError::Gone),
            Some(_) => {
                return Err(CoreError::Conflict(
                    "participant is already linked to another user".to_owned(),
                ));
            }
            None => {}
        }
        if self
            .repo
            .find_participant_by_user(participant.organization_id, command.user_id)
            .await?
            .is_some()
        {
            return Err(CoreError::Conflict(
                "user is already a participant of this organization".to_owned(),
            ));
        }

        let activated = self
            .repo
            .link_user(
                participant.organization_id,
                participant.id,
                command.user_id,
                now,
            )
            .await?;

        self.emitter.emit(
            activated.organization_id,
            &ParticipantActivated {
                participant: activated.clone(),
                user_id: command.user_id,
                activated_at: now,
            },
        )?;

        Ok(activated)
    }

    async fn detail(&mut self, participant: Participant) -> Result<ParticipantDetail, CoreError> {
        let apprentice = self
            .repo
            .find_apprentice(participant.organization_id, participant.id)
            .await?;
        let company_tutors = self
            .repo
            .list_company_tutors(participant.organization_id, participant.id)
            .await?;

        Ok(ParticipantDetail {
            participant,
            apprentice,
            company_tutors,
        })
    }

    async fn ensure_email_free(
        &mut self,
        organization_id: OrganizationId,
        email: &str,
        own_id: Option<ParticipantId>,
    ) -> Result<(), CoreError> {
        let taken = self
            .repo
            .find_participant_by_email(organization_id, email)
            .await?
            .is_some_and(|found| Some(found.id) != own_id);
        if taken {
            return Err(CoreError::Conflict(
                "a participant with this email already exists".to_owned(),
            ));
        }

        Ok(())
    }
}

fn new_apprentice(participant: &Participant, input: ApprenticeInput) -> Apprentice {
    let now = Utc::now();
    Apprentice {
        id: ApprenticeId(generate_uuid_v7()),
        organization_id: participant.organization_id,
        participant_id: participant.id,
        birth_date: input.birth_date,
        ine_code: optional(input.ine_code),
        address: optional(input.address),
        created_at: now,
        updated_at: now,
    }
}

fn new_company_tutor(participant: &Participant, input: CompanyTutorInput) -> CompanyTutor {
    let now = Utc::now();
    CompanyTutor {
        id: CompanyTutorId(generate_uuid_v7()),
        organization_id: participant.organization_id,
        participant_id: participant.id,
        host_company_id: input.host_company_id,
        job_title: optional(input.job_title),
        created_at: now,
        updated_at: now,
    }
}

fn required(field: &str, value: &str) -> Result<String, CoreError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(CoreError::Conflict(format!("{field} cannot be empty")));
    }

    Ok(trimmed.to_owned())
}

fn optional(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn valid_email(value: &str) -> Result<String, CoreError> {
    let email = required("participant email", value)?;
    let well_formed = email
        .split_once('@')
        .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'))
        && !email.contains(char::is_whitespace);
    if !well_formed {
        return Err(CoreError::Conflict(
            "participant email is not a valid address".to_owned(),
        ));
    }

    Ok(email)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HostCompanyId, ParticipantStatus, UserId,
        domain::cursus_participant::ports::MockCursusParticipantRepository,
    };
    use events::testing::RecordingEmitter;
    use uuid::Uuid;

    fn org() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn actor() -> authz::Subject {
        crate::application::policy::user_subject(UserId(Uuid::new_v4()), Vec::new())
    }

    fn participant_in(organization_id: OrganizationId) -> Participant {
        let now = Utc::now();
        Participant {
            id: ParticipantId(Uuid::new_v4()),
            organization_id,
            user_id: None,
            last_name: "Martin".to_owned(),
            first_name: None,
            email: "martin@example.com".to_owned(),
            phone: None,
            invitation_token_hash: None,
            invitation_expires_at: None,
            deactivated_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn invited_with(
        organization_id: OrganizationId,
        clear: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Participant {
        let mut participant = participant_in(organization_id);
        participant.invitation_token_hash = Some(token::hash(clear));
        participant.invitation_expires_at = Some(expires_at);
        participant
    }

    fn create_command(organization_id: OrganizationId) -> CreateParticipantCommand {
        CreateParticipantCommand {
            actor: actor(),
            organization_id,
            last_name: " Martin ".to_owned(),
            first_name: Some("  ".to_owned()),
            email: "martin@example.com".to_owned(),
            phone: None,
            apprentice: None,
            company_tutor: None,
        }
    }

    fn apprentice_input() -> ApprenticeInput {
        ApprenticeInput {
            birth_date: None,
            ine_code: Some(" 1234567890A ".to_owned()),
            address: None,
        }
    }

    fn tutor_input() -> CompanyTutorInput {
        CompanyTutorInput {
            host_company_id: HostCompanyId(Uuid::new_v4()),
            job_title: Some("Chef de chantier".to_owned()),
        }
    }

    fn echo_insert(repo: &mut MockCursusParticipantRepository) {
        repo.expect_find_participant_by_email()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repo.expect_insert_participant().times(1).returning(|p| {
            let cloned = p.clone();
            Box::pin(async move { Ok(cloned) })
        });
        repo.expect_find_apprentice()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repo.expect_list_company_tutors()
            .returning(|_, _| Box::pin(async { Ok(Vec::new()) }));
    }

    fn service(
        repo: MockCursusParticipantRepository,
    ) -> CursusParticipantService<MockCursusParticipantRepository, RecordingEmitter> {
        CursusParticipantService::new(repo, RecordingEmitter::new())
    }

    fn returning_participant(repo: &mut MockCursusParticipantRepository, found: Participant) {
        repo.expect_find_participant().returning(move |_, _| {
            let found = found.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
    }

    #[tokio::test]
    async fn create_with_an_apprentice_block_writes_the_apprentice_record() {
        let mut repo = MockCursusParticipantRepository::new();
        echo_insert(&mut repo);
        repo.expect_upsert_apprentice().times(1).returning(|a| {
            let cloned = a.clone();
            Box::pin(async move { Ok(cloned) })
        });
        let mut command = create_command(org());
        command.apprentice = Some(apprentice_input());

        let detail = service(repo).create(command).await.unwrap();

        assert_eq!(detail.participant.last_name, "Martin");
        assert_eq!(detail.participant.first_name, None);
        assert_eq!(detail.participant.status(), ParticipantStatus::Pending);
    }

    #[tokio::test]
    async fn create_with_a_company_tutor_block_writes_the_tutor_record() {
        let mut repo = MockCursusParticipantRepository::new();
        echo_insert(&mut repo);
        let input = tutor_input();
        let expected_company = input.host_company_id;
        repo.expect_insert_company_tutor()
            .times(1)
            .withf(move |tutor| {
                tutor.host_company_id == expected_company
                    && tutor.job_title.as_deref() == Some("Chef de chantier")
            })
            .returning(|t| {
                let cloned = t.clone();
                Box::pin(async move { Ok(cloned) })
            });
        let mut command = create_command(org());
        command.company_tutor = Some(input);

        service(repo).create(command).await.unwrap();
    }

    #[tokio::test]
    async fn create_with_both_blocks_is_refused_before_touching_the_repository() {
        let mut command = create_command(org());
        command.apprentice = Some(apprentice_input());
        command.company_tutor = Some(tutor_input());

        let result = service(MockCursusParticipantRepository::new())
            .create(command)
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn create_refuses_a_duplicate_email() {
        let organization_id = org();
        let mut repo = MockCursusParticipantRepository::new();
        repo.expect_find_participant_by_email()
            .returning(move |_, _| {
                let existing = participant_in(organization_id);
                Box::pin(async move { Ok(Some(existing)) })
            });

        let result = service(repo).create(create_command(organization_id)).await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn create_refuses_a_blank_name_and_a_malformed_email() {
        let mut blank = create_command(org());
        blank.last_name = "  ".to_owned();
        let mut malformed = create_command(org());
        malformed.email = "not-an-email".to_owned();

        for command in [blank, malformed] {
            let result = service(MockCursusParticipantRepository::new())
                .create(command)
                .await;
            assert!(matches!(result, Err(CoreError::Conflict(_))));
        }
    }

    #[tokio::test]
    async fn update_keeps_the_own_email_but_refuses_another_participants() {
        let organization_id = org();
        let current = participant_in(organization_id);
        let own_id = current.id;
        let mut repo = MockCursusParticipantRepository::new();
        returning_participant(&mut repo, current);
        repo.expect_find_participant_by_email()
            .returning(move |organization_id, _| {
                let mut same = participant_in(organization_id);
                same.id = own_id;
                Box::pin(async move { Ok(Some(same)) })
            });
        repo.expect_update_participant().times(1).returning(|p| {
            let cloned = p.clone();
            Box::pin(async move { Ok(cloned) })
        });
        repo.expect_find_apprentice()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repo.expect_list_company_tutors()
            .returning(|_, _| Box::pin(async { Ok(Vec::new()) }));

        let updated = service(repo)
            .update(UpdateParticipantCommand {
                actor: actor(),
                organization_id,
                id: own_id,
                last_name: "Martin".to_owned(),
                first_name: Some("Léa".to_owned()),
                email: "martin@example.com".to_owned(),
                phone: None,
            })
            .await
            .unwrap();
        assert_eq!(updated.participant.first_name.as_deref(), Some("Léa"));

        let mut taken = MockCursusParticipantRepository::new();
        returning_participant(&mut taken, participant_in(organization_id));
        taken
            .expect_find_participant_by_email()
            .returning(move |organization_id, _| {
                let other = participant_in(organization_id);
                Box::pin(async move { Ok(Some(other)) })
            });
        let refused = service(taken)
            .update(UpdateParticipantCommand {
                actor: actor(),
                organization_id,
                id: own_id,
                last_name: "Martin".to_owned(),
                first_name: None,
                email: "other@example.com".to_owned(),
                phone: None,
            })
            .await;
        assert!(matches!(refused, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn get_hides_a_participant_of_another_organization() {
        let mut repo = MockCursusParticipantRepository::new();
        returning_participant(&mut repo, participant_in(org()));

        let result = service(repo)
            .get(org(), ParticipantId(Uuid::new_v4()))
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn list_attaches_the_batched_records_to_their_participants() {
        let organization_id = org();
        let apprentice = participant_in(organization_id);
        let tutor = participant_in(organization_id);
        let bare = participant_in(organization_id);
        let apprentice_record = ApprenticeId(Uuid::new_v4());
        let tutor_record = CompanyTutorRecord {
            tutor: CompanyTutor {
                id: CompanyTutorId(Uuid::new_v4()),
                organization_id,
                participant_id: tutor.id,
                host_company_id: HostCompanyId(Uuid::new_v4()),
                job_title: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            host_company_name: "Durand".to_owned(),
        };
        let page = vec![apprentice.clone(), tutor.clone(), bare.clone()];
        let apprentice_id = apprentice.id;
        let expected_tutor = tutor_record.clone();
        let mut repo = MockCursusParticipantRepository::new();
        repo.expect_list_participants()
            .times(1)
            .returning(move |_, _, _, _| {
                let page = page.clone();
                Box::pin(async move { Ok((page, 3)) })
            });
        repo.expect_list_apprentice_ids()
            .times(1)
            .returning(move |_, ids| {
                assert_eq!(ids.len(), 3);
                Box::pin(async move { Ok(vec![(apprentice_id, apprentice_record)]) })
            });
        repo.expect_list_company_tutors_of()
            .times(1)
            .returning(move |_, _| {
                let record = tutor_record.clone();
                Box::pin(async move { Ok(vec![record]) })
            });

        let (items, total) = service(repo)
            .list(organization_id, &ParticipantFilter::default(), 10, 0)
            .await
            .unwrap();

        assert_eq!(total, 3);
        assert_eq!(items[0].apprentice_id, Some(apprentice_record));
        assert!(items[0].company_tutors.is_empty());
        assert_eq!(items[1].apprentice_id, None);
        assert_eq!(items[1].company_tutors, vec![expected_tutor]);
        assert_eq!(items[2].apprentice_id, None);
        assert!(items[2].company_tutors.is_empty());
    }

    #[tokio::test]
    async fn invite_stores_only_the_hash_and_emits_without_the_token() {
        let organization_id = org();
        let pending = participant_in(organization_id);
        let id = pending.id;
        let mut repo = MockCursusParticipantRepository::new();
        returning_participant(&mut repo, pending);
        repo.expect_set_invitation().times(1).returning(
            |organization_id, id, hash, expires_at, _| {
                let mut stored = participant_in(organization_id);
                stored.id = id;
                stored.invitation_token_hash = Some(hash.to_vec());
                stored.invitation_expires_at = Some(expires_at);
                Box::pin(async move { Ok(stored) })
            },
        );
        let emitter = RecordingEmitter::new();
        let mut service = CursusParticipantService::new(repo, &emitter);

        let invite = service.invite(organization_id, id).await.unwrap();

        assert_eq!(invite.token.len(), 64);
        assert_eq!(
            invite.participant.invitation_token_hash,
            Some(token::hash(&invite.token))
        );
        assert!(invite.expires_at > Utc::now() + Duration::days(13));
        let event = emitter.only("cursus.participant.invited");
        let payload = event.payload.to_string();
        assert!(!payload.contains(&invite.token));
        assert_eq!(event.version, 1);
    }

    #[tokio::test]
    async fn invite_is_refused_for_a_linked_or_a_deactivated_participant() {
        let organization_id = org();
        let mut linked = participant_in(organization_id);
        linked.user_id = Some(UserId(Uuid::new_v4()));
        let mut deactivated = participant_in(organization_id);
        deactivated.deactivated_at = Some(Utc::now());

        for found in [linked, deactivated] {
            let id = found.id;
            let mut repo = MockCursusParticipantRepository::new();
            returning_participant(&mut repo, found);
            let emitter = RecordingEmitter::new();

            let result = CursusParticipantService::new(repo, &emitter)
                .invite(organization_id, id)
                .await;

            assert!(matches!(result, Err(CoreError::Conflict(_))));
            assert!(emitter.names().is_empty());
        }
    }

    #[tokio::test]
    async fn deactivate_and_reactivate_are_idempotent() {
        let organization_id = org();
        let mut deactivated = participant_in(organization_id);
        deactivated.deactivated_at = Some(Utc::now());
        let mut repo = MockCursusParticipantRepository::new();
        returning_participant(&mut repo, deactivated.clone());

        let again = service(repo)
            .deactivate(organization_id, deactivated.id)
            .await
            .unwrap();
        assert_eq!(again, deactivated);

        let active = participant_in(organization_id);
        let mut repo = MockCursusParticipantRepository::new();
        returning_participant(&mut repo, active.clone());
        let unchanged = service(repo)
            .reactivate(organization_id, active.id)
            .await
            .unwrap();
        assert_eq!(unchanged, active);
    }

    #[tokio::test]
    async fn deactivate_stamps_the_participant() {
        let organization_id = org();
        let active = participant_in(organization_id);
        let id = active.id;
        let mut repo = MockCursusParticipantRepository::new();
        returning_participant(&mut repo, active);
        repo.expect_set_deactivated_at()
            .times(1)
            .withf(|_, _, at, _| at.is_some())
            .returning(|organization_id, id, at, _| {
                let mut stored = participant_in(organization_id);
                stored.id = id;
                stored.deactivated_at = at;
                Box::pin(async move { Ok(stored) })
            });

        let result = service(repo).deactivate(organization_id, id).await.unwrap();

        assert_eq!(result.status(), ParticipantStatus::Inactive);
    }

    fn accept_command(clear: &str, user_id: UserId) -> AcceptParticipantInvitationCommand {
        AcceptParticipantInvitationCommand {
            token: clear.to_owned(),
            user_id,
        }
    }

    fn finding_by_hash(repo: &mut MockCursusParticipantRepository, found: Option<Participant>) {
        repo.expect_find_participant_by_token_hash()
            .returning(move |_| {
                let found = found.clone();
                Box::pin(async move { Ok(found) })
            });
    }

    #[tokio::test]
    async fn accept_links_the_user_and_emits_activated() {
        let organization_id = org();
        let user_id = UserId(Uuid::new_v4());
        let invited = invited_with(organization_id, "clear", Utc::now() + Duration::days(1));
        let mut repo = MockCursusParticipantRepository::new();
        finding_by_hash(&mut repo, Some(invited));
        repo.expect_find_participant_by_user()
            .returning(|_, _| Box::pin(async { Ok(None) }));
        repo.expect_link_user()
            .times(1)
            .returning(|organization_id, id, user_id, _| {
                let mut stored = participant_in(organization_id);
                stored.id = id;
                stored.user_id = Some(user_id);
                Box::pin(async move { Ok(stored) })
            });
        let emitter = RecordingEmitter::new();

        let activated = CursusParticipantService::new(repo, &emitter)
            .accept(accept_command("clear", user_id))
            .await
            .unwrap();

        assert_eq!(activated.user_id, Some(user_id));
        assert_eq!(activated.status(), ParticipantStatus::Active);
        let event = emitter.only("cursus.participant.activated");
        assert_eq!(event.payload["user_id"], user_id.0.to_string());
    }

    #[tokio::test]
    async fn accept_of_an_unknown_token_is_gone() {
        let mut repo = MockCursusParticipantRepository::new();
        finding_by_hash(&mut repo, None);

        let result = service(repo)
            .accept(accept_command("nope", UserId(Uuid::new_v4())))
            .await;

        assert!(matches!(result, Err(CoreError::Gone)));
    }

    #[tokio::test]
    async fn accept_of_an_expired_or_cleared_invitation_is_gone() {
        let organization_id = org();
        let expired = invited_with(organization_id, "clear", Utc::now() - Duration::seconds(1));
        let mut cleared = participant_in(organization_id);
        cleared.invitation_expires_at = None;

        for found in [expired, cleared] {
            let mut repo = MockCursusParticipantRepository::new();
            finding_by_hash(&mut repo, Some(found));

            let result = service(repo)
                .accept(accept_command("clear", UserId(Uuid::new_v4())))
                .await;

            assert!(matches!(result, Err(CoreError::Gone)));
        }
    }

    #[tokio::test]
    async fn accept_of_a_deactivated_participant_is_gone() {
        let mut invited = invited_with(org(), "clear", Utc::now() + Duration::days(1));
        invited.deactivated_at = Some(Utc::now());
        let mut repo = MockCursusParticipantRepository::new();
        finding_by_hash(&mut repo, Some(invited));

        let result = service(repo)
            .accept(accept_command("clear", UserId(Uuid::new_v4())))
            .await;

        assert!(matches!(result, Err(CoreError::Gone)));
    }

    #[tokio::test]
    async fn accept_of_a_participant_linked_elsewhere_is_a_conflict() {
        let mut invited = invited_with(org(), "clear", Utc::now() + Duration::days(1));
        invited.user_id = Some(UserId(Uuid::new_v4()));
        let mut repo = MockCursusParticipantRepository::new();
        finding_by_hash(&mut repo, Some(invited));
        let emitter = RecordingEmitter::new();

        let result = CursusParticipantService::new(repo, &emitter)
            .accept(accept_command("clear", UserId(Uuid::new_v4())))
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn accept_by_a_user_who_already_has_a_participant_is_a_conflict() {
        let organization_id = org();
        let invited = invited_with(organization_id, "clear", Utc::now() + Duration::days(1));
        let mut repo = MockCursusParticipantRepository::new();
        finding_by_hash(&mut repo, Some(invited));
        repo.expect_find_participant_by_user()
            .returning(move |_, _| {
                let existing = participant_in(organization_id);
                Box::pin(async move { Ok(Some(existing)) })
            });
        let emitter = RecordingEmitter::new();

        let result = CursusParticipantService::new(repo, &emitter)
            .accept(accept_command("clear", UserId(Uuid::new_v4())))
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn accept_again_by_the_same_user_is_gone() {
        let user_id = UserId(Uuid::new_v4());
        let mut invited = invited_with(org(), "clear", Utc::now() + Duration::days(1));
        invited.user_id = Some(user_id);
        let mut repo = MockCursusParticipantRepository::new();
        finding_by_hash(&mut repo, Some(invited));

        let result = service(repo).accept(accept_command("clear", user_id)).await;

        assert!(matches!(result, Err(CoreError::Gone)));
    }
}
