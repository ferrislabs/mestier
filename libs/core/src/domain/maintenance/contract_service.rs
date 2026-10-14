//! Creating, editing, archiving and duplicating a maintenance contract.
//!
//! Every reference a contract carries (customer, site, manager, quote) is
//! checked here, in the contract's organization, because a foreign-key
//! violation reaches the caller as a 500 and a reference to another
//! organization's row would otherwise only be caught by luck.

use std::collections::HashSet;

use chrono::{NaiveDate, Utc, Weekday};
use common::{CoreError, generate_uuid_v7};

use crate::{
    Cadence, IsoWeekNumber, MaintenanceContract, MaintenanceContractId, MaintenanceLine,
    MaintenanceLineId, MemberId, OrganizationId, Project, ProjectId, QuoteId, Season, Window,
    Windows,
    domain::{
        customer::ports::CustomerRepository,
        customer_context::ports::CustomerContextRepository,
        maintenance::{
            commands::{
                CreateMaintenanceContractCommand, MaintenanceLineInput,
                UpdateMaintenanceContractCommand,
            },
            ports::MaintenanceContractRepository,
        },
        member::ports::MemberRepository,
        project::ports::ProjectRepository,
        quote::ports::QuoteRepository,
    },
};

pub struct MaintenanceContractService<R>
where
    R: MaintenanceContractRepository,
{
    repo: R,
}

impl<R> MaintenanceContractService<R>
where
    R: MaintenanceContractRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    /// `NotFound` when the contract belongs to another organization, so a
    /// foreign id is indistinguishable from an unknown one.
    pub async fn get(
        &mut self,
        organization_id: OrganizationId,
        id: MaintenanceContractId,
    ) -> Result<MaintenanceContract, CoreError> {
        self.repo
            .find_by_id(id)
            .await?
            .filter(|contract| contract.organization_id == organization_id)
            .ok_or(CoreError::NotFound)
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
        year: i32,
        include_archived: bool,
    ) -> Result<Vec<MaintenanceContract>, CoreError> {
        self.repo
            .list_by_season(organization_id, year, include_archived)
            .await
    }

    pub async fn create<C, X, M, Q, P>(
        &mut self,
        command: CreateMaintenanceContractCommand,
        customers: &mut C,
        contexts: &mut X,
        members: &mut M,
        quotes: &mut Q,
        projects: &mut P,
    ) -> Result<MaintenanceContract, CoreError>
    where
        C: CustomerRepository,
        X: CustomerContextRepository,
        M: MemberRepository,
        Q: QuoteRepository,
        P: ProjectRepository,
    {
        let organization_id = command.organization_id;
        let reference = clean_text(command.reference);
        let notes = clean_text(command.notes);
        check_amount(command.amount_cents)?;
        let lines = build_lines(command.lines, command.season, &[])?;

        let customer = customers
            .find_by_id(command.customer_id)
            .await?
            .filter(|customer| customer.organization_id == organization_id)
            .ok_or(CoreError::NotFound)?;
        let site = contexts
            .find_by_id(command.customer_context_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        if site.customer_id != customer.id {
            return Err(CoreError::Conflict(
                "the site does not belong to the customer".to_owned(),
            ));
        }
        check_manager(members, organization_id, command.manager_member_id).await?;
        check_quote(quotes, organization_id, command.quote_id).await?;

        let now = Utc::now();
        let project = projects
            .insert(&Project {
                id: ProjectId(generate_uuid_v7()),
                organization_id,
                name: project_name(&customer.name, &site.label, command.season),
                customer_id: Some(customer.id),
                customer_context_id: Some(site.id),
                quote_id: None,
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        self.repo
            .insert(&MaintenanceContract {
                id: MaintenanceContractId(generate_uuid_v7()),
                organization_id,
                customer_id: customer.id,
                customer_context_id: site.id,
                project_id: project.id,
                reference,
                season: command.season,
                manager_member_id: command.manager_member_id,
                amount_cents: command.amount_cents,
                quote_id: command.quote_id,
                notes,
                status: command.status,
                lines,
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    /// Replaces the contract's fields and lines. The customer, the site and
    /// the year stay as created: the project, and the cost already booked on
    /// it, are named after them.
    pub async fn update<M, Q>(
        &mut self,
        command: UpdateMaintenanceContractCommand,
        members: &mut M,
        quotes: &mut Q,
    ) -> Result<MaintenanceContract, CoreError>
    where
        M: MemberRepository,
        Q: QuoteRepository,
    {
        let mut contract = self
            .repo
            .find_by_id(command.id)
            .await?
            .ok_or(CoreError::NotFound)?;
        if command.season.year() != contract.season.year() {
            return Err(CoreError::Conflict(
                "the year of a contract cannot change, duplicate it for another season".to_owned(),
            ));
        }
        check_amount(command.amount_cents)?;
        let lines = build_lines(command.lines, command.season, &contract.lines)?;
        check_manager(members, contract.organization_id, command.manager_member_id).await?;
        check_quote(quotes, contract.organization_id, command.quote_id).await?;

        contract.reference = clean_text(command.reference);
        contract.notes = clean_text(command.notes);
        contract.season = command.season;
        contract.manager_member_id = command.manager_member_id;
        contract.amount_cents = command.amount_cents;
        contract.quote_id = command.quote_id;
        contract.status = command.status;
        contract.lines = lines;
        contract.updated_at = Utc::now();

        self.repo.update(&contract).await
    }

    /// Archiving twice keeps the first date.
    pub async fn archive(
        &mut self,
        id: MaintenanceContractId,
    ) -> Result<MaintenanceContract, CoreError> {
        let mut contract = self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        if contract.archived_at.is_some() {
            return Ok(contract);
        }

        let now = Utc::now();
        contract.archived_at = Some(now);
        contract.updated_at = now;
        self.repo.update(&contract).await
    }

    /// The next season's contract: same lines under new ids, a project of its
    /// own, no intervention. The quote is not carried over, it belonged to
    /// the season just ended.
    pub async fn duplicate<C, X, P>(
        &mut self,
        id: MaintenanceContractId,
        customers: &mut C,
        contexts: &mut X,
        projects: &mut P,
    ) -> Result<MaintenanceContract, CoreError>
    where
        C: CustomerRepository,
        X: CustomerContextRepository,
        P: ProjectRepository,
    {
        let source = self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)?;
        let season = source.season.next()?;
        let lines = source
            .lines
            .iter()
            .map(|line| carry_line(line, season))
            .collect::<Result<Vec<_>, _>>()?;

        let customer = customers
            .find_by_id(source.customer_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let site = contexts
            .find_by_id(source.customer_context_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let now = Utc::now();
        let project = projects
            .insert(&Project {
                id: ProjectId(generate_uuid_v7()),
                organization_id: source.organization_id,
                name: project_name(&customer.name, &site.label, season),
                customer_id: Some(customer.id),
                customer_context_id: Some(site.id),
                quote_id: None,
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await?;

        self.repo
            .insert(&MaintenanceContract {
                id: MaintenanceContractId(generate_uuid_v7()),
                organization_id: source.organization_id,
                customer_id: source.customer_id,
                customer_context_id: source.customer_context_id,
                project_id: project.id,
                reference: source.reference,
                season,
                manager_member_id: source.manager_member_id,
                amount_cents: source.amount_cents,
                quote_id: None,
                notes: source.notes,
                status: source.status,
                lines,
                archived_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }
}

/// "Entretien 2026 - Commune de Lapalisse - Mairie".
pub fn project_name(customer: &str, site: &str, season: Season) -> String {
    format!("Entretien {} - {customer} - {site}", season.year())
}

fn clean_text(text: Option<String>) -> Option<String> {
    text.map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty())
}

fn check_amount(amount_cents: Option<i64>) -> Result<(), CoreError> {
    if amount_cents.is_some_and(|amount| amount < 0) {
        return Err(CoreError::Conflict(
            "the contract amount cannot be negative".to_owned(),
        ));
    }
    Ok(())
}

async fn check_manager<M: MemberRepository>(
    members: &mut M,
    organization_id: OrganizationId,
    manager: Option<MemberId>,
) -> Result<(), CoreError> {
    let Some(manager) = manager else {
        return Ok(());
    };
    members
        .find_by_id(manager)
        .await?
        .filter(|member| member.organization_id == organization_id)
        .map(|_| ())
        .ok_or(CoreError::NotFound)
}

async fn check_quote<Q: QuoteRepository>(
    quotes: &mut Q,
    organization_id: OrganizationId,
    quote: Option<QuoteId>,
) -> Result<(), CoreError> {
    let Some(quote) = quote else {
        return Ok(());
    };
    quotes
        .find_by_id(quote)
        .await?
        .filter(|quote| quote.organization_id == organization_id)
        .map(|_| ())
        .ok_or(CoreError::NotFound)
}

/// A line sent with an id must already belong to the contract: taking an id
/// from the request would let one contract write into another's lines.
fn build_lines(
    inputs: Vec<MaintenanceLineInput>,
    season: Season,
    existing: &[MaintenanceLine],
) -> Result<Vec<MaintenanceLine>, CoreError> {
    let known: HashSet<MaintenanceLineId> = existing.iter().map(|line| line.id).collect();
    let mut seen = HashSet::with_capacity(inputs.len());

    inputs
        .into_iter()
        .map(|input| {
            let id = match input.id {
                Some(id) if !known.contains(&id) => {
                    return Err(CoreError::Conflict(
                        "a line does not belong to this contract".to_owned(),
                    ));
                }
                Some(id) => id,
                None => MaintenanceLineId(generate_uuid_v7()),
            };
            if !seen.insert(id) {
                return Err(CoreError::Conflict(
                    "a line appears twice in the contract".to_owned(),
                ));
            }

            let label = input.label.trim();
            if label.is_empty() {
                return Err(CoreError::Conflict("a line needs a label".to_owned()));
            }
            if input
                .minutes_per_intervention
                .is_some_and(|minutes| minutes == 0 || i32::try_from(minutes).is_err())
            {
                return Err(CoreError::Conflict(
                    "minutes per intervention must be a positive number".to_owned(),
                ));
            }
            input.cadence.check_fits(season)?;

            Ok(MaintenanceLine {
                id,
                label: label.to_owned(),
                cadence: input.cadence,
                minutes_per_intervention: input.minutes_per_intervention,
            })
        })
        .collect()
}

/// A copy of `line` for `season`, under a new id. Windows on a week the new
/// year lacks (53) move to week 52, as the season's own weeks do.
fn carry_line(line: &MaintenanceLine, season: Season) -> Result<MaintenanceLine, CoreError> {
    let cadence = match &line.cadence {
        Cadence::Windows { windows } => {
            let carried = windows
                .as_slice()
                .iter()
                .map(|window| {
                    Window::new(
                        clamp_week(window.from_week(), season.year())?,
                        clamp_week(window.to_week(), season.year())?,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            Cadence::Windows {
                windows: Windows::try_from(carried)?,
            }
        }
        other => other.clone(),
    };
    cadence.check_fits(season)?;

    Ok(MaintenanceLine {
        id: MaintenanceLineId(generate_uuid_v7()),
        label: line.label.clone(),
        cadence,
        minutes_per_intervention: line.minutes_per_intervention,
    })
}

fn clamp_week(week: IsoWeekNumber, year: i32) -> Result<IsoWeekNumber, CoreError> {
    if NaiveDate::from_isoywd_opt(year, u32::from(week.get()), Weekday::Mon).is_some() {
        return Ok(week);
    }
    IsoWeekNumber::try_from(52)
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use chrono::DateTime;
    use uuid::Uuid;

    use super::*;
    use crate::{
        ContractStatus, Customer, CustomerContext, CustomerContextId, CustomerId,
        CustomerPipelineStage, CustomerStatus, IntervalDays, Member,
        domain::{
            customer::ports::MockCustomerRepository,
            customer_context::ports::MockCustomerContextRepository,
            maintenance::ports::MockMaintenanceContractRepository,
            member::ports::MockMemberRepository, project::ports::MockProjectRepository,
            quote::ports::MockQuoteRepository,
        },
    };

    fn week(n: u8) -> IsoWeekNumber {
        IsoWeekNumber::try_from(n).unwrap()
    }

    fn season() -> Season {
        Season::new(2026, week(12), week(48)).unwrap()
    }

    fn rolling(days: u16) -> Cadence {
        Cadence::Rolling {
            every_days: IntervalDays::try_from(days).unwrap(),
        }
    }

    fn windows(spans: &[(u8, u8)]) -> Cadence {
        Cadence::Windows {
            windows: Windows::try_from(
                spans
                    .iter()
                    .map(|(from, to)| Window::new(week(*from), week(*to)).unwrap())
                    .collect::<Vec<_>>(),
            )
            .unwrap(),
        }
    }

    fn line_input(label: &str, cadence: Cadence) -> MaintenanceLineInput {
        MaintenanceLineInput {
            id: None,
            label: label.to_owned(),
            cadence,
            minutes_per_intervention: Some(90),
        }
    }

    struct Scene {
        organization_id: OrganizationId,
        customer_id: CustomerId,
        site_id: CustomerContextId,
    }

    impl Scene {
        fn new() -> Self {
            Self {
                organization_id: OrganizationId(Uuid::new_v4()),
                customer_id: CustomerId(Uuid::new_v4()),
                site_id: CustomerContextId(Uuid::new_v4()),
            }
        }

        fn customer(&self) -> Customer {
            let now = Utc::now();
            Customer {
                id: self.customer_id,
                organization_id: self.organization_id,
                status: CustomerStatus::Client,
                pipeline_stage: CustomerPipelineStage::Won,
                name: "GAUME".to_owned(),
                registration_number: None,
                phone: None,
                email: None,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            }
        }

        fn site(&self, customer_id: CustomerId) -> CustomerContext {
            let now = Utc::now();
            CustomerContext {
                id: self.site_id,
                customer_id,
                label: "Parc".to_owned(),
                address_line: None,
                postal_code: None,
                city: Some("Vichy".to_owned()),
                photo_key: None,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            }
        }

        fn customers(&self) -> MockCustomerRepository {
            let customer = self.customer();
            let mut customers = MockCustomerRepository::new();
            customers.expect_find_by_id().returning(move |_| {
                let customer = customer.clone();
                Box::pin(async move { Ok(Some(customer)) })
            });
            customers
        }

        fn contexts(&self, owner: CustomerId) -> MockCustomerContextRepository {
            let site = self.site(owner);
            let mut contexts = MockCustomerContextRepository::new();
            contexts.expect_find_by_id().returning(move |_| {
                let site = site.clone();
                Box::pin(async move { Ok(Some(site)) })
            });
            contexts
        }

        fn create_command(
            &self,
            lines: Vec<MaintenanceLineInput>,
        ) -> CreateMaintenanceContractCommand {
            CreateMaintenanceContractCommand {
                actor: authz::Subject::system(),
                organization_id: self.organization_id,
                customer_id: self.customer_id,
                customer_context_id: self.site_id,
                reference: Some("  C25029 ".to_owned()),
                season: season(),
                manager_member_id: None,
                amount_cents: Some(120_000),
                quote_id: None,
                notes: Some("   ".to_owned()),
                status: ContractStatus::Active,
                lines,
            }
        }

        fn contract(&self, lines: Vec<MaintenanceLine>) -> MaintenanceContract {
            let now = Utc::now();
            MaintenanceContract {
                id: MaintenanceContractId(Uuid::new_v4()),
                organization_id: self.organization_id,
                customer_id: self.customer_id,
                customer_context_id: self.site_id,
                project_id: ProjectId(Uuid::new_v4()),
                reference: Some("C25029".to_owned()),
                season: season(),
                manager_member_id: None,
                amount_cents: Some(120_000),
                quote_id: Some(QuoteId(Uuid::new_v4())),
                notes: None,
                status: ContractStatus::Active,
                lines,
                archived_at: None,
                created_at: now,
                updated_at: now,
            }
        }
    }

    fn existing_line(label: &str, cadence: Cadence) -> MaintenanceLine {
        MaintenanceLine {
            id: MaintenanceLineId(Uuid::new_v4()),
            label: label.to_owned(),
            cadence,
            minutes_per_intervention: None,
        }
    }

    fn storing_repository() -> (
        MockMaintenanceContractRepository,
        Arc<Mutex<Vec<MaintenanceContract>>>,
    ) {
        let stored = Arc::new(Mutex::new(Vec::new()));
        let mut repo = MockMaintenanceContractRepository::new();
        let sink = Arc::clone(&stored);
        repo.expect_insert().returning(move |contract| {
            sink.lock().unwrap().push(contract.clone());
            let contract = contract.clone();
            Box::pin(async move { Ok(contract) })
        });
        let sink = Arc::clone(&stored);
        repo.expect_update().returning(move |contract| {
            sink.lock().unwrap().push(contract.clone());
            let contract = contract.clone();
            Box::pin(async move { Ok(contract) })
        });
        (repo, stored)
    }

    fn finding(
        mut repo: MockMaintenanceContractRepository,
        contract: &MaintenanceContract,
    ) -> MockMaintenanceContractRepository {
        let contract = contract.clone();
        repo.expect_find_by_id().returning(move |_| {
            let contract = contract.clone();
            Box::pin(async move { Ok(Some(contract)) })
        });
        repo
    }

    fn storing_projects() -> (MockProjectRepository, Arc<Mutex<Vec<Project>>>) {
        let stored = Arc::new(Mutex::new(Vec::new()));
        let mut projects = MockProjectRepository::new();
        let sink = Arc::clone(&stored);
        projects.expect_insert().returning(move |project| {
            sink.lock().unwrap().push(project.clone());
            let project = project.clone();
            Box::pin(async move { Ok(project) })
        });
        (projects, stored)
    }

    async fn create(
        scene: &Scene,
        command: CreateMaintenanceContractCommand,
        contexts_owner: CustomerId,
    ) -> Result<MaintenanceContract, CoreError> {
        let (repo, _) = storing_repository();
        let (mut projects, _) = storing_projects();
        MaintenanceContractService::new(repo)
            .create(
                command,
                &mut scene.customers(),
                &mut scene.contexts(contexts_owner),
                &mut MockMemberRepository::new(),
                &mut MockQuoteRepository::new(),
                &mut projects,
            )
            .await
    }

    #[tokio::test]
    async fn creating_a_contract_creates_its_project_named_after_customer_site_and_season() {
        let scene = Scene::new();
        let (repo, stored) = storing_repository();
        let (mut projects, created_projects) = storing_projects();

        let contract = MaintenanceContractService::new(repo)
            .create(
                scene.create_command(vec![line_input("Tonte", rolling(14))]),
                &mut scene.customers(),
                &mut scene.contexts(scene.customer_id),
                &mut MockMemberRepository::new(),
                &mut MockQuoteRepository::new(),
                &mut projects,
            )
            .await
            .unwrap();

        let projects = created_projects.lock().unwrap();
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Entretien 2026 - GAUME - Parc");
        assert_eq!(projects[0].customer_id, Some(scene.customer_id));
        assert_eq!(projects[0].customer_context_id, Some(scene.site_id));
        assert_eq!(contract.project_id, projects[0].id);
        assert_eq!(stored.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn blank_text_fields_are_stored_as_absent_and_others_trimmed() {
        let scene = Scene::new();

        let contract = create(&scene, scene.create_command(Vec::new()), scene.customer_id)
            .await
            .unwrap();

        assert_eq!(contract.reference.as_deref(), Some("C25029"));
        assert_eq!(contract.notes, None);
    }

    #[tokio::test]
    async fn a_window_outside_the_season_is_refused() {
        let scene = Scene::new();
        let command = scene.create_command(vec![line_input("Taille", windows(&[(46, 50)]))]);

        let err = create(&scene, command, scene.customer_id)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)), "{err:?}");
    }

    #[tokio::test]
    async fn a_site_of_another_customer_is_refused() {
        let scene = Scene::new();
        let command = scene.create_command(Vec::new());

        let err = create(&scene, command, CustomerId(Uuid::new_v4()))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)), "{err:?}");
    }

    #[tokio::test]
    async fn a_customer_of_another_organization_is_not_found() {
        let scene = Scene::new();
        let mut command = scene.create_command(Vec::new());
        command.organization_id = OrganizationId(Uuid::new_v4());

        let err = create(&scene, command, scene.customer_id)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound), "{err:?}");
    }

    #[tokio::test]
    async fn a_manager_of_another_organization_is_not_found() {
        let scene = Scene::new();
        let mut command = scene.create_command(Vec::new());
        command.manager_member_id = Some(MemberId(Uuid::new_v4()));
        let mut members = MockMemberRepository::new();
        members.expect_find_by_id().returning(|id| {
            let member = Member {
                id,
                organization_id: OrganizationId(Uuid::new_v4()),
                user_id: None,
                last_name: "Autre".to_owned(),
                first_name: None,
                joined_at: None,
                created_at: Utc::now(),
                deleted_at: None,
            };
            Box::pin(async move { Ok(Some(member)) })
        });
        let (repo, _) = storing_repository();
        let (mut projects, _) = storing_projects();

        let err = MaintenanceContractService::new(repo)
            .create(
                command,
                &mut scene.customers(),
                &mut scene.contexts(scene.customer_id),
                &mut members,
                &mut MockQuoteRepository::new(),
                &mut projects,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound), "{err:?}");
    }

    #[tokio::test]
    async fn a_quote_that_is_not_in_the_organization_is_not_found() {
        let scene = Scene::new();
        let mut command = scene.create_command(Vec::new());
        command.quote_id = Some(QuoteId(Uuid::new_v4()));
        let mut quotes = MockQuoteRepository::new();
        quotes
            .expect_find_by_id()
            .returning(|_| Box::pin(async { Ok(None) }));
        let (repo, _) = storing_repository();
        let (mut projects, _) = storing_projects();

        let err = MaintenanceContractService::new(repo)
            .create(
                command,
                &mut scene.customers(),
                &mut scene.contexts(scene.customer_id),
                &mut MockMemberRepository::new(),
                &mut quotes,
                &mut projects,
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound), "{err:?}");
    }

    #[tokio::test]
    async fn a_line_needs_a_label_and_a_positive_duration() {
        let scene = Scene::new();
        let blank = scene.create_command(vec![line_input("   ", rolling(14))]);
        let mut zero = line_input("Tonte", rolling(14));
        zero.minutes_per_intervention = Some(0);
        let zero = scene.create_command(vec![zero]);

        for command in [blank, zero] {
            let err = create(&scene, command, scene.customer_id)
                .await
                .unwrap_err();
            assert!(matches!(err, CoreError::Conflict(_)), "{err:?}");
        }
    }

    #[tokio::test]
    async fn a_negative_amount_is_refused() {
        let scene = Scene::new();
        let mut command = scene.create_command(Vec::new());
        command.amount_cents = Some(-1);

        let err = create(&scene, command, scene.customer_id)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)), "{err:?}");
    }

    fn update_command(
        contract: &MaintenanceContract,
        lines: Vec<MaintenanceLineInput>,
    ) -> UpdateMaintenanceContractCommand {
        UpdateMaintenanceContractCommand {
            actor: authz::Subject::system(),
            id: contract.id,
            reference: Some("C25030".to_owned()),
            season: contract.season,
            manager_member_id: None,
            amount_cents: None,
            quote_id: None,
            notes: None,
            status: ContractStatus::Active,
            lines,
        }
    }

    async fn update(
        contract: &MaintenanceContract,
        command: UpdateMaintenanceContractCommand,
    ) -> Result<MaintenanceContract, CoreError> {
        let (repo, _) = storing_repository();
        MaintenanceContractService::new(finding(repo, contract))
            .update(
                command,
                &mut MockMemberRepository::new(),
                &mut MockQuoteRepository::new(),
            )
            .await
    }

    #[tokio::test]
    async fn an_update_keeps_the_ids_it_is_given_and_assigns_new_ones() {
        let scene = Scene::new();
        let kept = existing_line("Tonte", rolling(14));
        let dropped = existing_line("Taille", windows(&[(14, 17)]));
        let contract = scene.contract(vec![kept.clone(), dropped]);
        let mut keep = line_input("Tonte avec ramassage", rolling(7));
        keep.id = Some(kept.id);
        let command = update_command(&contract, vec![keep, line_input("Arrosage", rolling(3))]);

        let updated = update(&contract, command).await.unwrap();

        assert_eq!(updated.lines.len(), 2);
        assert_eq!(updated.lines[0].id, kept.id);
        assert_eq!(updated.lines[0].label, "Tonte avec ramassage");
        assert_ne!(updated.lines[1].id, kept.id);
        assert_eq!(updated.reference.as_deref(), Some("C25030"));
        assert_eq!(updated.project_id, contract.project_id);
    }

    #[tokio::test]
    async fn an_update_cannot_change_the_year() {
        let scene = Scene::new();
        let contract = scene.contract(Vec::new());
        let mut command = update_command(&contract, Vec::new());
        command.season = Season::new(2027, week(12), week(48)).unwrap();

        let err = update(&contract, command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)), "{err:?}");
    }

    #[tokio::test]
    async fn an_update_refuses_a_line_id_from_another_contract() {
        let scene = Scene::new();
        let contract = scene.contract(vec![existing_line("Tonte", rolling(14))]);
        let mut foreign = line_input("Tonte", rolling(14));
        foreign.id = Some(MaintenanceLineId(Uuid::new_v4()));

        let err = update(&contract, update_command(&contract, vec![foreign]))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)), "{err:?}");
    }

    #[tokio::test]
    async fn an_update_refuses_the_same_line_twice() {
        let scene = Scene::new();
        let line = existing_line("Tonte", rolling(14));
        let contract = scene.contract(vec![line.clone()]);
        let mut twice = line_input("Tonte", rolling(14));
        twice.id = Some(line.id);

        let err = update(
            &contract,
            update_command(&contract, vec![twice.clone(), twice]),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)), "{err:?}");
    }

    #[tokio::test]
    async fn an_update_refuses_a_window_outside_the_new_season() {
        let scene = Scene::new();
        let contract = scene.contract(Vec::new());
        let mut command =
            update_command(&contract, vec![line_input("Taille", windows(&[(40, 46)]))]);
        command.season = Season::new(2026, week(12), week(30)).unwrap();

        let err = update(&contract, command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)), "{err:?}");
    }

    #[tokio::test]
    async fn archiving_stamps_the_contract_once() {
        let scene = Scene::new();
        let contract = scene.contract(Vec::new());
        let (repo, stored) = storing_repository();

        let archived = MaintenanceContractService::new(finding(repo, &contract))
            .archive(contract.id)
            .await
            .unwrap();

        assert!(archived.archived_at.is_some());
        assert_eq!(stored.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn archiving_an_archived_contract_changes_nothing() {
        let scene = Scene::new();
        let mut contract = scene.contract(Vec::new());
        let first: DateTime<Utc> = Utc::now() - chrono::Duration::days(3);
        contract.archived_at = Some(first);
        let (repo, stored) = storing_repository();

        let archived = MaintenanceContractService::new(finding(repo, &contract))
            .archive(contract.id)
            .await
            .unwrap();

        assert_eq!(archived.archived_at, Some(first));
        assert!(stored.lock().unwrap().is_empty());
    }

    async fn duplicate(
        scene: &Scene,
        contract: &MaintenanceContract,
    ) -> (Result<MaintenanceContract, CoreError>, Vec<Project>) {
        let (repo, _) = storing_repository();
        let (mut projects, created_projects) = storing_projects();
        let result = MaintenanceContractService::new(finding(repo, contract))
            .duplicate(
                contract.id,
                &mut scene.customers(),
                &mut scene.contexts(scene.customer_id),
                &mut projects,
            )
            .await;
        let created = created_projects.lock().unwrap().clone();
        (result, created)
    }

    #[tokio::test]
    async fn duplicating_copies_the_lines_into_the_next_season_with_a_new_project() {
        let scene = Scene::new();
        let tonte = existing_line("Tonte", rolling(14));
        let taille = existing_line("Taille", windows(&[(14, 17), (44, 47)]));
        let contract = scene.contract(vec![tonte.clone(), taille.clone()]);

        let (copy, projects) = duplicate(&scene, &contract).await;
        let copy = copy.unwrap();

        assert_eq!(copy.season.year(), 2027);
        assert_eq!(copy.reference, contract.reference);
        assert_eq!(copy.amount_cents, contract.amount_cents);
        assert_eq!(copy.quote_id, None);
        assert_eq!(copy.archived_at, None);
        assert_ne!(copy.id, contract.id);
        assert_ne!(copy.project_id, contract.project_id);
        assert_eq!(projects.len(), 1);
        assert_eq!(projects[0].name, "Entretien 2027 - GAUME - Parc");
        assert_eq!(copy.lines.len(), 2);
        assert_eq!(copy.lines[0].label, "Tonte");
        assert_eq!(copy.lines[1].cadence, taille.cadence);
        assert!(
            copy.lines
                .iter()
                .all(|line| line.id != tonte.id && line.id != taille.id)
        );
    }

    #[tokio::test]
    async fn duplicating_moves_week_fifty_three_to_fifty_two() {
        let scene = Scene::new();
        let mut contract = scene.contract(vec![existing_line("Taille", windows(&[(50, 53)]))]);
        contract.season = Season::new(2026, week(12), week(53)).unwrap();

        let (copy, _) = duplicate(&scene, &contract).await;
        let copy = copy.unwrap();

        assert_eq!(copy.season.last_week(), week(52));
        assert_eq!(copy.lines[0].cadence, windows(&[(50, 52)]));
    }

    #[tokio::test]
    async fn an_unknown_contract_is_not_found() {
        let mut repo = MockMaintenanceContractRepository::new();
        repo.expect_find_by_id()
            .returning(|_| Box::pin(async { Ok(None) }));

        let err = MaintenanceContractService::new(repo)
            .get(
                OrganizationId(Uuid::new_v4()),
                MaintenanceContractId(Uuid::new_v4()),
            )
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn a_contract_of_another_organization_is_not_found() {
        let scene = Scene::new();
        let contract = scene.contract(Vec::new());

        let err = MaintenanceContractService::new(finding(
            MockMaintenanceContractRepository::new(),
            &contract,
        ))
        .get(OrganizationId(Uuid::new_v4()), contract.id)
        .await
        .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[test]
    fn week_fifty_three_only_moves_in_a_year_without_one() {
        assert_eq!(clamp_week(week(53), 2026).unwrap(), week(53));
        assert_eq!(clamp_week(week(53), 2027).unwrap(), week(52));
        assert_eq!(clamp_week(week(30), 2027).unwrap(), week(30));
    }
}
