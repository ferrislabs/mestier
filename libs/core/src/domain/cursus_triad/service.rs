use chrono::Utc;
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    Closure, CompanyTutor, Contract, ContractFileKind, ContractId, ContractTutor, ContractView,
    FileKey, MyTriad, MyTriadDetail, OrganizationId, ParticipantScope, PromotionId, Triad,
    TriadDetail, TriadId, TriadRole, TriadSummary,
    domain::cursus_triad::{
        ContractPeriod,
        commands::{
            CreateContractCommand, CreateTriadCommand, SetContractFileCommand,
            UpdateContractCommand, UpdateTriadCommand,
        },
        events::{ContractClosed, ContractCreated, TriadClosed, TriadCreated},
        ports::CursusTriadRepository,
    },
};

const DUPLICATE_TRIAD_CONSTRAINT: &str = "uq_cursus_triad_promotion_apprentice";

pub struct CursusTriadService<R, E>
where
    R: CursusTriadRepository,
    E: EventEmitter,
{
    repo: R,
    emitter: E,
}

impl<R, E> CursusTriadService<R, E>
where
    R: CursusTriadRepository,
    E: EventEmitter,
{
    pub fn new(repo: R, emitter: E) -> Self {
        Self { repo, emitter }
    }

    pub async fn create_triad(&mut self, command: CreateTriadCommand) -> Result<Triad, CoreError> {
        let now = Utc::now();
        let inserted = self
            .repo
            .insert_triad(&Triad {
                id: TriadId(generate_uuid_v7()),
                organization_id: command.organization_id,
                promotion_id: command.promotion_id,
                apprentice_id: command.apprentice_id,
                pedagogic_tutor_member_id: command.pedagogic_tutor_member_id,
                closure: None,
                created_at: now,
                updated_at: now,
            })
            .await
            .map_err(|error| match error {
                CoreError::Conflict(constraint) if constraint == DUPLICATE_TRIAD_CONSTRAINT => {
                    CoreError::Conflict(
                        "this apprentice already has a triad in this promotion".to_owned(),
                    )
                }
                other => other,
            })?;

        self.emitter
            .emit(inserted.organization_id, &TriadCreated { triad: &inserted })?;

        Ok(inserted)
    }

    pub async fn get_triad(
        &mut self,
        organization_id: OrganizationId,
        id: TriadId,
    ) -> Result<Triad, CoreError> {
        self.repo
            .find_triad(organization_id, id)
            .await?
            .filter(|found| found.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn get_triad_detail(
        &mut self,
        organization_id: OrganizationId,
        id: TriadId,
    ) -> Result<TriadDetail, CoreError> {
        self.repo
            .find_triad_detail(organization_id, id)
            .await?
            .filter(|found| found.triad.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_triads(
        &mut self,
        organization_id: OrganizationId,
        promotion_id: Option<PromotionId>,
        closed: Option<bool>,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<TriadSummary>, u64), CoreError> {
        self.repo
            .list_triads(organization_id, promotion_id, closed, limit, offset)
            .await
    }

    pub async fn update_triad(&mut self, command: UpdateTriadCommand) -> Result<Triad, CoreError> {
        let mut triad = self.get_triad(command.organization_id, command.id).await?;
        triad.ensure_open()?;
        triad.pedagogic_tutor_member_id = command.pedagogic_tutor_member_id;
        triad.updated_at = Utc::now();

        self.repo.save_triad(&triad).await
    }

    pub async fn close_triad(
        &mut self,
        organization_id: OrganizationId,
        id: TriadId,
        closure: Closure,
    ) -> Result<Triad, CoreError> {
        let mut triad = self.get_triad(organization_id, id).await?;
        let now = Utc::now();
        triad.close(closure.clone(), now)?;

        for mut contract in self.repo.list_contracts(organization_id, id).await? {
            if contract.is_closed() {
                continue;
            }
            contract.close(closure.clone(), now)?;
            let closed = self.repo.save_contract(&contract).await?;
            self.repo
                .archive_contract_tutors(organization_id, closed.id, now)
                .await?;
            self.emitter.emit(
                organization_id,
                &ContractClosed {
                    contract: &closed,
                    closure: &closure,
                },
            )?;
        }

        let saved = self.repo.save_triad(&triad).await?;
        self.emitter.emit(
            organization_id,
            &TriadClosed {
                triad: &saved,
                closure: &closure,
            },
        )?;

        Ok(saved)
    }

    pub async fn create_contract(
        &mut self,
        command: CreateContractCommand,
    ) -> Result<Contract, CoreError> {
        let period = ContractPeriod::new(command.start_date, command.end_date)?;
        self.get_triad(command.organization_id, command.triad_id)
            .await?
            .ensure_open()?;

        let now = Utc::now();
        let inserted = self
            .repo
            .insert_contract(&Contract {
                id: ContractId(generate_uuid_v7()),
                organization_id: command.organization_id,
                triad_id: command.triad_id,
                host_company_id: command.host_company_id,
                kind: command.kind,
                number: optional(command.number),
                period,
                closure: None,
                cerfa_file_key: None,
                mission_sheet_file_key: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        self.emitter.emit(
            inserted.organization_id,
            &ContractCreated {
                contract: &inserted,
            },
        )?;

        Ok(inserted)
    }

    pub async fn get_contract(
        &mut self,
        organization_id: OrganizationId,
        id: ContractId,
    ) -> Result<Contract, CoreError> {
        self.repo
            .find_contract(organization_id, id)
            .await?
            .filter(|found| found.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn get_contract_view(
        &mut self,
        organization_id: OrganizationId,
        id: ContractId,
    ) -> Result<ContractView, CoreError> {
        self.repo
            .find_contract_view(organization_id, id)
            .await?
            .filter(|found| found.contract.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_contract_views(
        &mut self,
        organization_id: OrganizationId,
        triad_id: TriadId,
    ) -> Result<Vec<ContractView>, CoreError> {
        self.get_triad(organization_id, triad_id).await?;
        self.repo
            .list_contract_views(organization_id, triad_id)
            .await
    }

    pub async fn update_contract(
        &mut self,
        command: UpdateContractCommand,
    ) -> Result<Contract, CoreError> {
        let period = ContractPeriod::new(command.start_date, command.end_date)?;
        let mut contract = self
            .get_contract(command.organization_id, command.id)
            .await?;
        contract.ensure_open()?;

        if contract.host_company_id != command.host_company_id
            && !self
                .repo
                .list_contract_tutors(command.organization_id, contract.id)
                .await?
                .is_empty()
        {
            return Err(CoreError::Conflict(
                "the host company cannot change while tutors are linked to the contract".to_owned(),
            ));
        }

        contract.host_company_id = command.host_company_id;
        contract.kind = command.kind;
        contract.number = optional(command.number);
        contract.period = period;
        contract.updated_at = Utc::now();

        self.repo.save_contract(&contract).await
    }

    pub async fn close_contract(
        &mut self,
        organization_id: OrganizationId,
        id: ContractId,
        closure: Closure,
    ) -> Result<Contract, CoreError> {
        let mut contract = self.get_contract(organization_id, id).await?;
        let now = Utc::now();
        contract.close(closure.clone(), now)?;

        let saved = self.repo.save_contract(&contract).await?;
        self.repo
            .archive_contract_tutors(organization_id, saved.id, now)
            .await?;
        self.emitter.emit(
            organization_id,
            &ContractClosed {
                contract: &saved,
                closure: &closure,
            },
        )?;

        Ok(saved)
    }

    pub async fn set_tutors(
        &mut self,
        organization_id: OrganizationId,
        contract_id: ContractId,
        main: Option<&CompanyTutor>,
        secondary: Option<&CompanyTutor>,
    ) -> Result<Vec<ContractTutor>, CoreError> {
        let contract = self.get_contract(organization_id, contract_id).await?;
        let tutors = contract.assign_tutors(main, secondary)?;
        self.repo
            .replace_contract_tutors(organization_id, contract_id, &tutors)
            .await?;

        Ok(tutors)
    }

    pub async fn set_file(
        &mut self,
        command: SetContractFileCommand,
    ) -> Result<Contract, CoreError> {
        let key = command
            .file_key
            .as_deref()
            .map(FileKey::try_from)
            .transpose()
            .map_err(|error| CoreError::Conflict(error.to_string()))?;
        let mut contract = self
            .get_contract(command.organization_id, command.contract_id)
            .await?;
        contract.set_file_key(command.kind, key);
        contract.updated_at = Utc::now();

        self.repo.save_contract(&contract).await
    }

    pub async fn get_file_key(
        &mut self,
        organization_id: OrganizationId,
        contract_id: ContractId,
        kind: ContractFileKind,
    ) -> Result<FileKey, CoreError> {
        self.get_contract(organization_id, contract_id)
            .await?
            .file_key(kind)
            .cloned()
            .ok_or(CoreError::NotFound)
    }

    pub async fn company_tutor_is_linked(
        &mut self,
        organization_id: OrganizationId,
        id: crate::CompanyTutorId,
    ) -> Result<bool, CoreError> {
        self.repo.company_tutor_is_linked(organization_id, id).await
    }

    pub async fn participant_scope(
        &mut self,
        organization_id: OrganizationId,
        participant_id: crate::ParticipantId,
    ) -> Result<ParticipantScope, CoreError> {
        let apprentice_triads = self
            .repo
            .list_apprentice_triad_ids(organization_id, participant_id)
            .await?;
        let tutor_links = self
            .repo
            .list_tutor_links(organization_id, participant_id)
            .await?;

        Ok(ParticipantScope::new(
            participant_id,
            organization_id,
            &apprentice_triads,
            &tutor_links,
        ))
    }

    pub async fn my_triads(&mut self, scope: &ParticipantScope) -> Result<Vec<MyTriad>, CoreError> {
        let ids: Vec<TriadId> = scope.links.iter().map(|link| link.triad_id).collect();
        let summaries = self
            .repo
            .list_triad_summaries(scope.organization_id, &ids)
            .await?;

        let mut mine: Vec<MyTriad> = summaries
            .into_iter()
            .filter(|summary| summary.triad.organization_id == scope.organization_id)
            .filter_map(|summary| {
                scope
                    .link(summary.triad.id)
                    .copied()
                    .map(|link| MyTriad { summary, link })
            })
            .collect();
        mine.sort_by(|left, right| {
            left.summary
                .promotion_label
                .cmp(&right.summary.promotion_label)
                .then_with(|| {
                    left.summary
                        .apprentice_name
                        .cmp(&right.summary.apprentice_name)
                })
        });

        Ok(mine)
    }

    pub async fn my_triad(
        &mut self,
        scope: &ParticipantScope,
        triad_id: TriadId,
    ) -> Result<MyTriadDetail, CoreError> {
        let link = *scope.link(triad_id).ok_or(CoreError::NotFound)?;
        let detail = self
            .get_triad_detail(scope.organization_id, triad_id)
            .await?;
        let detail = match link.role {
            TriadRole::Apprentice => detail,
            TriadRole::MainTutor | TriadRole::SecondaryTutor => {
                detail.restricted_to_tutor(scope.participant_id)
            }
        };

        Ok(MyTriadDetail { detail, link })
    }
}

fn optional(value: Option<String>) -> Option<String> {
    value
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ApprenticeId, ClosureReasonId, CompanyTutorId, HostCompanyId, ParticipantId, TriadLink,
        TutorRank, TutorTriadLink,
        domain::cursus_triad::{ContractKind, ports::MockCursusTriadRepository},
    };
    use chrono::NaiveDate;
    use events::testing::RecordingEmitter;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn org() -> OrganizationId {
        OrganizationId(Uuid::new_v4())
    }

    fn actor() -> authz::Subject {
        crate::application::policy::user_subject(crate::UserId(Uuid::new_v4()), Vec::new())
    }

    fn day(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }

    fn closure() -> Closure {
        Closure {
            closed_at: day(2027, 6, 30),
            reason_id: ClosureReasonId(Uuid::new_v4()),
            note: Some("Fin de formation".to_owned()),
        }
    }

    fn triad_in(organization_id: OrganizationId, id: TriadId) -> Triad {
        let now = Utc::now();
        Triad {
            id,
            organization_id,
            promotion_id: PromotionId(Uuid::new_v4()),
            apprentice_id: ApprenticeId(Uuid::new_v4()),
            pedagogic_tutor_member_id: None,
            closure: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn contract_in(organization_id: OrganizationId, triad_id: TriadId, closed: bool) -> Contract {
        let now = Utc::now();
        Contract {
            id: ContractId(Uuid::new_v4()),
            organization_id,
            triad_id,
            host_company_id: HostCompanyId(Uuid::new_v4()),
            kind: ContractKind::Apprenticeship,
            number: None,
            period: ContractPeriod::new(day(2026, 9, 1), day(2028, 8, 31)).unwrap(),
            closure: closed.then(closure),
            cerfa_file_key: None,
            mission_sheet_file_key: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn echo_triad(repo: &mut MockCursusTriadRepository) {
        repo.expect_save_triad().returning(|triad| {
            let saved = triad.clone();
            Box::pin(async move { Ok(saved) })
        });
    }

    fn echo_contract(repo: &mut MockCursusTriadRepository) {
        repo.expect_save_contract().returning(|contract| {
            let saved = contract.clone();
            Box::pin(async move { Ok(saved) })
        });
    }

    fn create_triad_command(organization_id: OrganizationId) -> CreateTriadCommand {
        CreateTriadCommand {
            actor: actor(),
            organization_id,
            promotion_id: PromotionId(Uuid::new_v4()),
            apprentice_id: ApprenticeId(Uuid::new_v4()),
            pedagogic_tutor_member_id: None,
        }
    }

    #[tokio::test]
    async fn creating_a_triad_inserts_it_open_and_emits_cursus_triad_created() {
        let organization_id = org();
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_insert_triad().times(1).returning(|triad| {
            let inserted = triad.clone();
            Box::pin(async move { Ok(inserted) })
        });
        let emitter = RecordingEmitter::new();

        let created = CursusTriadService::new(repo, &emitter)
            .create_triad(create_triad_command(organization_id))
            .await
            .unwrap();

        assert!(!created.is_closed());
        assert_eq!(created.organization_id, organization_id);
        let event = emitter.only("cursus.triad.created");
        assert_eq!(event.subject_kind, "triad");
        assert_eq!(event.subject_id, Some(created.id.0));
    }

    #[tokio::test]
    async fn spec_cur_5_a_second_triad_for_the_same_apprentice_and_promotion_is_a_conflict() {
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_insert_triad().returning(|_| {
            Box::pin(async {
                Err(CoreError::Conflict(
                    "uq_cursus_triad_promotion_apprentice".to_owned(),
                ))
            })
        });
        let emitter = RecordingEmitter::new();

        let result = CursusTriadService::new(repo, &emitter)
            .create_triad(create_triad_command(org()))
            .await;

        match result {
            Err(CoreError::Conflict(message)) => {
                assert!(!message.contains("uq_cursus"));
            }
            other => panic!("expected a conflict, got {other:?}"),
        }
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn a_triad_of_another_organization_is_hidden() {
        let asked = org();
        let id = TriadId(Uuid::new_v4());
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_triad()
            .with(eq(asked), eq(id))
            .returning(move |_, id| {
                let foreign = triad_in(org(), id);
                Box::pin(async move { Ok(Some(foreign)) })
            });

        let result = CursusTriadService::new(repo, RecordingEmitter::new())
            .get_triad(asked, id)
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn a_closed_triad_rejects_pedagogic_tutor_changes() {
        let organization_id = org();
        let id = TriadId(Uuid::new_v4());
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_triad()
            .returning(move |organization_id, id| {
                let mut closed = triad_in(organization_id, id);
                closed.closure = Some(closure());
                Box::pin(async move { Ok(Some(closed)) })
            });

        let result = CursusTriadService::new(repo, RecordingEmitter::new())
            .update_triad(UpdateTriadCommand {
                actor: actor(),
                organization_id,
                id,
                pedagogic_tutor_member_id: None,
            })
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn spec_cur_10_closing_a_triad_closes_its_open_contracts_with_the_same_closure() {
        let organization_id = org();
        let id = TriadId(Uuid::new_v4());
        let open_one = contract_in(organization_id, id, false);
        let open_two = contract_in(organization_id, id, false);
        let already_closed = contract_in(organization_id, id, true);
        let open_ids = [open_one.id, open_two.id];
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_triad()
            .returning(move |organization_id, id| {
                let open = triad_in(organization_id, id);
                Box::pin(async move { Ok(Some(open)) })
            });
        repo.expect_list_contracts().returning(move |_, _| {
            let contracts = vec![open_one.clone(), already_closed.clone(), open_two.clone()];
            Box::pin(async move { Ok(contracts) })
        });
        repo.expect_save_contract().times(2).returning(|contract| {
            let saved = contract.clone();
            Box::pin(async move { Ok(saved) })
        });
        repo.expect_archive_contract_tutors()
            .times(2)
            .returning(|_, _, _| Box::pin(async { Ok(()) }));
        echo_triad(&mut repo);
        let emitter = RecordingEmitter::new();
        let shared = closure();

        let closed = CursusTriadService::new(repo, &emitter)
            .close_triad(organization_id, id, shared.clone())
            .await
            .unwrap();

        assert_eq!(closed.closure, Some(shared.clone()));
        assert_eq!(
            emitter.names(),
            vec![
                "cursus.contract.closed",
                "cursus.contract.closed",
                "cursus.triad.closed",
            ]
        );
        let emitted: Vec<Option<Uuid>> = emitter
            .recorded()
            .into_iter()
            .filter(|envelope| envelope.name == "cursus.contract.closed")
            .map(|envelope| envelope.subject_id)
            .collect();
        assert_eq!(emitted, open_ids.map(|id| Some(id.0)).to_vec());
        let payload = emitter.only("cursus.triad.closed").payload;
        assert_eq!(payload["closure_note"], "Fin de formation");
    }

    #[tokio::test]
    async fn spec_cur_10_a_second_triad_closure_is_a_conflict_and_changes_nothing() {
        let organization_id = org();
        let id = TriadId(Uuid::new_v4());
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_triad()
            .returning(move |organization_id, id| {
                let mut closed = triad_in(organization_id, id);
                closed.closure = Some(closure());
                Box::pin(async move { Ok(Some(closed)) })
            });
        let emitter = RecordingEmitter::new();

        let result = CursusTriadService::new(repo, &emitter)
            .close_triad(organization_id, id, closure())
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn spec_cur_9_closing_a_contract_archives_its_tutor_links_and_emits_once() {
        let organization_id = org();
        let contract = contract_in(organization_id, TriadId(Uuid::new_v4()), false);
        let id = contract.id;
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_contract().returning(move |_, _| {
            let found = contract.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
        echo_contract(&mut repo);
        repo.expect_archive_contract_tutors()
            .times(1)
            .withf(move |_, contract_id, _| *contract_id == id)
            .returning(|_, _, _| Box::pin(async { Ok(()) }));
        let emitter = RecordingEmitter::new();

        let closed = CursusTriadService::new(repo, &emitter)
            .close_contract(organization_id, id, closure())
            .await
            .unwrap();

        assert!(closed.is_closed());
        assert_eq!(emitter.names(), vec!["cursus.contract.closed"]);
        assert_eq!(
            emitter.only("cursus.contract.closed").payload["triad_id"],
            closed.triad_id.0.to_string()
        );
    }

    #[tokio::test]
    async fn a_second_contract_closure_is_a_conflict_and_archives_nothing() {
        let organization_id = org();
        let closed = contract_in(organization_id, TriadId(Uuid::new_v4()), true);
        let id = closed.id;
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_contract().returning(move |_, _| {
            let found = closed.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
        let emitter = RecordingEmitter::new();

        let result = CursusTriadService::new(repo, &emitter)
            .close_contract(organization_id, id, closure())
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
        assert!(emitter.names().is_empty());
    }

    fn create_contract_command(
        organization_id: OrganizationId,
        triad_id: TriadId,
        start: NaiveDate,
        end: NaiveDate,
    ) -> CreateContractCommand {
        CreateContractCommand {
            actor: actor(),
            organization_id,
            triad_id,
            host_company_id: HostCompanyId(Uuid::new_v4()),
            kind: ContractKind::Professionalization,
            number: Some("  C-12 ".to_owned()),
            start_date: start,
            end_date: end,
        }
    }

    #[tokio::test]
    async fn creating_a_contract_emits_cursus_contract_created() {
        let organization_id = org();
        let triad_id = TriadId(Uuid::new_v4());
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_triad()
            .returning(move |organization_id, id| {
                let open = triad_in(organization_id, id);
                Box::pin(async move { Ok(Some(open)) })
            });
        repo.expect_insert_contract()
            .times(1)
            .returning(|contract| {
                let inserted = contract.clone();
                Box::pin(async move { Ok(inserted) })
            });
        let emitter = RecordingEmitter::new();

        let created = CursusTriadService::new(repo, &emitter)
            .create_contract(create_contract_command(
                organization_id,
                triad_id,
                day(2026, 9, 1),
                day(2028, 8, 31),
            ))
            .await
            .unwrap();

        assert_eq!(created.number.as_deref(), Some("C-12"));
        assert_eq!(created.triad_id, triad_id);
        let event = emitter.only("cursus.contract.created");
        assert_eq!(event.subject_kind, "contract");
        assert_eq!(event.payload["kind"], "PROFESSIONALIZATION");
    }

    #[tokio::test]
    async fn a_contract_ending_before_it_starts_is_refused_before_any_lookup() {
        let result =
            CursusTriadService::new(MockCursusTriadRepository::new(), RecordingEmitter::new())
                .create_contract(create_contract_command(
                    org(),
                    TriadId(Uuid::new_v4()),
                    day(2026, 9, 2),
                    day(2026, 9, 1),
                ))
                .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn a_closed_triad_rejects_new_contracts() {
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_triad()
            .returning(move |organization_id, id| {
                let mut closed = triad_in(organization_id, id);
                closed.closure = Some(closure());
                Box::pin(async move { Ok(Some(closed)) })
            });
        let emitter = RecordingEmitter::new();

        let result = CursusTriadService::new(repo, &emitter)
            .create_contract(create_contract_command(
                org(),
                TriadId(Uuid::new_v4()),
                day(2026, 9, 1),
                day(2028, 8, 31),
            ))
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
        assert!(emitter.names().is_empty());
    }

    #[tokio::test]
    async fn a_closed_contract_rejects_updates() {
        let organization_id = org();
        let closed = contract_in(organization_id, TriadId(Uuid::new_v4()), true);
        let command = UpdateContractCommand {
            actor: actor(),
            organization_id,
            id: closed.id,
            host_company_id: closed.host_company_id,
            kind: ContractKind::Apprenticeship,
            number: None,
            start_date: day(2026, 9, 1),
            end_date: day(2028, 8, 31),
        };
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_contract().returning(move |_, _| {
            let found = closed.clone();
            Box::pin(async move { Ok(Some(found)) })
        });

        let result = CursusTriadService::new(repo, RecordingEmitter::new())
            .update_contract(command)
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn changing_the_host_company_while_tutors_are_linked_is_a_conflict() {
        let organization_id = org();
        let current = contract_in(organization_id, TriadId(Uuid::new_v4()), false);
        let command = UpdateContractCommand {
            actor: actor(),
            organization_id,
            id: current.id,
            host_company_id: HostCompanyId(Uuid::new_v4()),
            kind: ContractKind::Apprenticeship,
            number: None,
            start_date: day(2026, 9, 1),
            end_date: day(2028, 8, 31),
        };
        let contract_id = current.id;
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_contract().returning(move |_, _| {
            let found = current.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
        repo.expect_list_contract_tutors()
            .returning(move |organization_id, _| {
                let linked = vec![ContractTutor {
                    contract_id,
                    company_tutor_id: CompanyTutorId(Uuid::new_v4()),
                    organization_id,
                    rank: TutorRank::Main,
                    archived_at: None,
                }];
                Box::pin(async move { Ok(linked) })
            });

        let result = CursusTriadService::new(repo, RecordingEmitter::new())
            .update_contract(command)
            .await;

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[tokio::test]
    async fn setting_tutors_replaces_the_links_with_the_validated_assignment() {
        let organization_id = org();
        let current = contract_in(organization_id, TriadId(Uuid::new_v4()), false);
        let now = Utc::now();
        let main = CompanyTutor {
            id: CompanyTutorId(Uuid::new_v4()),
            organization_id,
            participant_id: ParticipantId(Uuid::new_v4()),
            host_company_id: current.host_company_id,
            job_title: None,
            created_at: now,
            updated_at: now,
        };
        let id = current.id;
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_contract().returning(move |_, _| {
            let found = current.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
        repo.expect_replace_contract_tutors()
            .times(1)
            .withf(|_, _, tutors| tutors.len() == 1 && tutors[0].rank == TutorRank::Main)
            .returning(|_, _, _| Box::pin(async { Ok(()) }));

        let tutors = CursusTriadService::new(repo, RecordingEmitter::new())
            .set_tutors(organization_id, id, Some(&main), None)
            .await
            .unwrap();

        assert_eq!(tutors.len(), 1);
        assert_eq!(tutors[0].company_tutor_id, main.id);
    }

    #[tokio::test]
    async fn an_invalid_file_key_is_a_conflict_and_a_valid_one_is_stored() {
        let organization_id = org();
        let current = contract_in(organization_id, TriadId(Uuid::new_v4()), false);
        let id = current.id;
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_contract().returning(move |_, _| {
            let found = current.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
        echo_contract(&mut repo);
        let mut service = CursusTriadService::new(repo, RecordingEmitter::new());
        let command = |file_key: Option<&str>| SetContractFileCommand {
            actor: actor(),
            organization_id,
            contract_id: id,
            kind: ContractFileKind::MissionSheet,
            file_key: file_key.map(str::to_owned),
        };

        let refused = service.set_file(command(Some("../etc/passwd"))).await;
        let stored = service
            .set_file(command(Some("uploads/mission-1")))
            .await
            .unwrap();
        let cleared = service.set_file(command(None)).await.unwrap();

        assert!(matches!(refused, Err(CoreError::Conflict(_))));
        assert_eq!(
            stored.mission_sheet_file_key.as_ref().map(FileKey::as_str),
            Some("uploads/mission-1")
        );
        assert_eq!(cleared.mission_sheet_file_key, None);
    }

    #[tokio::test]
    async fn asking_for_a_file_that_was_never_attached_is_not_found() {
        let organization_id = org();
        let current = contract_in(organization_id, TriadId(Uuid::new_v4()), false);
        let id = current.id;
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_find_contract().returning(move |_, _| {
            let found = current.clone();
            Box::pin(async move { Ok(Some(found)) })
        });

        let result = CursusTriadService::new(repo, RecordingEmitter::new())
            .get_file_key(organization_id, id, ContractFileKind::Cerfa)
            .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    async fn the_scope_merges_the_apprentice_triads_and_the_tutor_links() {
        let organization_id = org();
        let participant_id = ParticipantId(Uuid::new_v4());
        let own = TriadId(Uuid::new_v4());
        let tutored = TriadId(Uuid::new_v4());
        let archived_at = Utc::now();
        let mut repo = MockCursusTriadRepository::new();
        repo.expect_list_apprentice_triad_ids()
            .returning(move |_, _| Box::pin(async move { Ok(vec![own]) }));
        repo.expect_list_tutor_links().returning(move |_, _| {
            let links = vec![TutorTriadLink {
                triad_id: tutored,
                rank: TutorRank::Main,
                archived_at: Some(archived_at),
            }];
            Box::pin(async move { Ok(links) })
        });

        let scope = CursusTriadService::new(repo, RecordingEmitter::new())
            .participant_scope(organization_id, participant_id)
            .await
            .unwrap();

        assert_eq!(scope.participant_id, participant_id);
        assert_eq!(scope.links.len(), 2);
        assert_eq!(
            scope.link(tutored).copied(),
            Some(TriadLink {
                triad_id: tutored,
                role: crate::TriadRole::MainTutor,
                access: crate::TriadAccess::ReadOnly { since: archived_at },
            })
        );
    }

    #[tokio::test]
    async fn spec_cur_7_a_triad_outside_the_scope_is_not_found_without_touching_the_repository() {
        let scope = ParticipantScope::new(
            ParticipantId(Uuid::new_v4()),
            org(),
            &[TriadId(Uuid::new_v4())],
            &[],
        );

        let result =
            CursusTriadService::new(MockCursusTriadRepository::new(), RecordingEmitter::new())
                .my_triad(&scope, TriadId(Uuid::new_v4()))
                .await;

        assert!(matches!(result, Err(CoreError::NotFound)));
    }
}
