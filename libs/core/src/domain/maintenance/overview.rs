//! The two read views over `due_for`: the season grid and the weekly recap.
//! Both call the same function with the same history, so they cannot
//! disagree (ADR 0006).

use std::collections::{HashMap, HashSet};

use chrono::{NaiveDate, Weekday};
use common::CoreError;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    Cadence, ContractStatus, CustomerContextId, CustomerId, Intervention, InterventionId,
    InterventionStatus, IsoWeekNumber, MaintenanceContract, MaintenanceContractId, MaintenanceLine,
    MaintenanceLineId, MemberId, OrganizationId, Season,
    domain::{
        customer::ports::CustomerRepository,
        customer_contact::ports::CustomerContactRepository,
        customer_context::ports::CustomerContextRepository,
        maintenance::{
            due::{Due, DueStatus, LineHistory, due_for},
            iso_week_of,
            ports::{InterventionRepository, MaintenanceContractRepository},
        },
    },
};

/// Where a contract's work happens and who to call there, as the paper
/// sheet shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ContractSite {
    pub customer_name: String,
    pub site_label: String,
    pub address_line: Option<String>,
    pub postal_code: Option<String>,
    pub city: Option<String>,
    /// The customer's primary contact. Contacts belong to the customer, not
    /// to the site.
    pub contact: Option<SiteContact>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct SiteContact {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// A visit as a grid cell: the day of the month in its week's column.
/// Rejected visits are not shown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct GridVisit {
    pub id: InterventionId,
    pub done_on: NaiveDate,
    pub week: IsoWeekNumber,
    pub status: InterventionStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct GridLine {
    pub id: MaintenanceLineId,
    pub label: String,
    pub cadence: Cadence,
    pub minutes_per_intervention: Option<u32>,
    pub visits: Vec<GridVisit>,
    pub due: Due,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct GridContract {
    pub id: MaintenanceContractId,
    pub reference: Option<String>,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub site: ContractSite,
    pub season: Season,
    pub manager_member_id: Option<MemberId>,
    pub status: ContractStatus,
    pub lines: Vec<GridLine>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct Grid {
    pub year: i32,
    pub contracts: Vec<GridContract>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct RecapLine {
    pub line_id: MaintenanceLineId,
    pub contract_id: MaintenanceContractId,
    pub reference: Option<String>,
    pub site: ContractSite,
    pub label: String,
    pub minutes_per_intervention: Option<u32>,
    /// The latest validated visit.
    pub last_done_on: Option<NaiveDate>,
    pub due: Due,
}

impl RecapLine {
    pub fn is_overdue(&self) -> bool {
        self.due.status == DueStatus::Overdue
    }
}

/// The lines of one town. `town` is `None` for sites with no city.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct RecapGroup {
    pub town: Option<String>,
    pub lines: Vec<RecapLine>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct Recap {
    pub year: i32,
    pub week: IsoWeekNumber,
    pub groups: Vec<RecapGroup>,
}

/// Monday of an ISO week, or `Conflict` when the year has no such week.
pub fn monday_of_week(year: i32, week: IsoWeekNumber) -> Result<NaiveDate, CoreError> {
    NaiveDate::from_isoywd_opt(year, u32::from(week.get()), Weekday::Mon)
        .ok_or_else(|| CoreError::Conflict(format!("{year} has no week {}", week.get())))
}

struct Histories {
    validated: HashMap<MaintenanceLineId, Vec<NaiveDate>>,
}

impl Histories {
    /// Only validated visits enter a history (invariant 2): a proposed or
    /// rejected one must not move a due date.
    fn of(interventions: &[Intervention]) -> Self {
        let mut validated: HashMap<MaintenanceLineId, Vec<NaiveDate>> = HashMap::new();
        for intervention in interventions {
            if intervention.status == InterventionStatus::Validated {
                validated
                    .entry(intervention.line_id)
                    .or_default()
                    .push(intervention.done_on);
            }
        }
        Self { validated }
    }

    fn due(
        &self,
        contract: &MaintenanceContract,
        line: &MaintenanceLine,
        open_lines: &HashSet<MaintenanceLineId>,
        today: NaiveDate,
    ) -> Due {
        if contract.status == ContractStatus::Draft {
            return Due {
                on: None,
                status: DueStatus::Done,
            };
        }
        let history = LineHistory {
            validated_on: self.validated_on(line.id),
            has_open_task: open_lines.contains(&line.id),
        };
        due_for(contract.season, &line.cadence, history, today)
    }

    fn validated_on(&self, line_id: MaintenanceLineId) -> &[NaiveDate] {
        self.validated.get(&line_id).map_or(&[], Vec::as_slice)
    }

    fn last(&self, line_id: MaintenanceLineId) -> Option<NaiveDate> {
        self.validated_on(line_id).iter().max().copied()
    }
}

fn site_of<'a>(
    sites: &'a HashMap<MaintenanceContractId, ContractSite>,
    contract: &MaintenanceContract,
) -> Result<&'a ContractSite, CoreError> {
    sites
        .get(&contract.id)
        .ok_or_else(|| CoreError::Internal(format!("no site loaded for contract {}", contract.id)))
}

/// Lines by contract, each with its visits and its due date as of `today`.
pub fn build_grid(
    year: i32,
    data: &SeasonData,
    sites: &HashMap<MaintenanceContractId, ContractSite>,
    today: NaiveDate,
) -> Result<Grid, CoreError> {
    let interventions = &data.interventions;
    let open_lines = &data.open_lines;
    let histories = Histories::of(interventions);

    let contracts = data
        .contracts
        .iter()
        .map(|contract| {
            let lines = contract
                .lines
                .iter()
                .map(|line| GridLine {
                    id: line.id,
                    label: line.label.clone(),
                    cadence: line.cadence.clone(),
                    minutes_per_intervention: line.minutes_per_intervention,
                    visits: visits_of(line.id, interventions),
                    due: histories.due(contract, line, open_lines, today),
                })
                .collect();
            Ok(GridContract {
                id: contract.id,
                reference: contract.reference.clone(),
                customer_id: contract.customer_id,
                customer_context_id: contract.customer_context_id,
                site: site_of(sites, contract)?.clone(),
                season: contract.season,
                manager_member_id: contract.manager_member_id,
                status: contract.status,
                lines,
            })
        })
        .collect::<Result<_, CoreError>>()?;

    Ok(Grid { year, contracts })
}

fn visits_of(line_id: MaintenanceLineId, interventions: &[Intervention]) -> Vec<GridVisit> {
    let mut visits: Vec<GridVisit> = interventions
        .iter()
        .filter(|intervention| {
            intervention.line_id == line_id && intervention.status != InterventionStatus::Rejected
        })
        .map(|intervention| GridVisit {
            id: intervention.id,
            done_on: intervention.done_on,
            week: iso_week_of(intervention.done_on).1,
            status: intervention.status,
        })
        .collect();
    visits.sort_by_key(|visit| visit.done_on);
    visits
}

/// What to schedule for ISO week `week` of `year`: the lines due in it, and
/// the ones that were due before it. Lines already carried by an open task,
/// done lines and on-demand lines are left out.
///
/// The due date is computed as of the Monday of that week, so "overdue"
/// means overdue when the week starts, and a window still open that week
/// counts as due in it.
///
/// Towns come overdue-first, then alphabetically, without a town last; in a
/// town, overdue lines come first, then by due date, then in the order of
/// the contracts and of the lines within them.
pub fn build_recap(
    year: i32,
    week: IsoWeekNumber,
    data: &SeasonData,
    sites: &HashMap<MaintenanceContractId, ContractSite>,
) -> Result<Recap, CoreError> {
    let monday = monday_of_week(year, week)?;
    let histories = Histories::of(&data.interventions);
    let open_lines = &data.open_lines;

    let mut lines = Vec::new();
    for contract in &data.contracts {
        let site = site_of(sites, contract)?;
        for line in &contract.lines {
            let due = histories.due(contract, line, open_lines, monday);
            if !matches!(due.status, DueStatus::Overdue | DueStatus::ThisWeek) {
                continue;
            }
            lines.push(RecapLine {
                line_id: line.id,
                contract_id: contract.id,
                reference: contract.reference.clone(),
                site: site.clone(),
                label: line.label.clone(),
                minutes_per_intervention: line.minutes_per_intervention,
                last_done_on: histories.last(line.id),
                due,
            });
        }
    }

    Ok(Recap {
        year,
        week,
        groups: group_by_town(lines),
    })
}

fn group_by_town(lines: Vec<RecapLine>) -> Vec<RecapGroup> {
    let mut groups: Vec<RecapGroup> = Vec::new();
    for line in lines {
        let town = line.site.city.clone();
        match groups.iter_mut().find(|group| group.town == town) {
            Some(group) => group.lines.push(line),
            None => groups.push(RecapGroup {
                town,
                lines: vec![line],
            }),
        }
    }

    for group in &mut groups {
        group
            .lines
            .sort_by_key(|line| (!line.is_overdue(), line.due.on));
    }
    groups.sort_by_key(|group| {
        (
            !group.lines.iter().any(RecapLine::is_overdue),
            group.town.is_none(),
            group.town.as_deref().map(str::to_lowercase),
        )
    });
    groups
}

/// The site and primary contact of each contract, read once per customer
/// and per site however many contracts share them.
pub async fn load_sites(
    contracts: &[MaintenanceContract],
    customers: &mut impl CustomerRepository,
    contexts: &mut impl CustomerContextRepository,
    contacts: &mut impl CustomerContactRepository,
) -> Result<HashMap<MaintenanceContractId, ContractSite>, CoreError> {
    const CONTACTS_PER_CUSTOMER: u64 = 50;

    let mut customer_names: HashMap<CustomerId, (String, Option<SiteContact>)> = HashMap::new();
    let mut sites = HashMap::with_capacity(contracts.len());

    for contract in contracts {
        if !customer_names.contains_key(&contract.customer_id) {
            let customer = customers
                .find_by_id(contract.customer_id)
                .await?
                .ok_or(CoreError::NotFound)?;
            let (people, _) = contacts
                .list_by_customer(contract.customer_id, CONTACTS_PER_CUSTOMER, 0)
                .await?;
            let contact = people
                .into_iter()
                .filter(|person| person.deleted_at.is_none())
                .max_by_key(|person| person.is_primary)
                .map(|person| SiteContact {
                    name: format!("{} {}", person.first_name, person.last_name),
                    phone: person.phone,
                    email: person.email,
                });
            customer_names.insert(contract.customer_id, (customer.name, contact));
        }
        let (customer_name, contact) = &customer_names[&contract.customer_id];

        let context = contexts
            .find_by_id(contract.customer_context_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        sites.insert(
            contract.id,
            ContractSite {
                customer_name: customer_name.clone(),
                site_label: context.label,
                address_line: context.address_line,
                postal_code: context.postal_code,
                city: context.city,
                contact: contact.clone(),
            },
        );
    }

    Ok(sites)
}

/// A season's contracts with every visit on their lines and the lines an
/// open task already covers: everything the two views need besides the sites.
#[derive(Debug, Clone, Default)]
pub struct SeasonData {
    pub contracts: Vec<MaintenanceContract>,
    pub interventions: Vec<Intervention>,
    pub open_lines: HashSet<MaintenanceLineId>,
}

pub async fn load_season(
    contracts: &mut impl MaintenanceContractRepository,
    interventions: &mut impl InterventionRepository,
    organization_id: OrganizationId,
    year: i32,
) -> Result<SeasonData, CoreError> {
    let found = contracts
        .list_by_season(organization_id, year, false)
        .await?;
    let line_ids: Vec<MaintenanceLineId> = found
        .iter()
        .flat_map(|contract| contract.lines.iter().map(|line| line.id))
        .collect();
    Ok(SeasonData {
        interventions: interventions.list_for_lines(&line_ids).await?,
        open_lines: contracts
            .lines_with_open_task(organization_id, &line_ids)
            .await?,
        contracts: found,
    })
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::{Cadence, CustomerContextId, IntervalDays, ProjectId, Window, Windows};

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn week(n: u8) -> IsoWeekNumber {
        IsoWeekNumber::try_from(n).unwrap()
    }

    fn rolling(days: u16) -> Cadence {
        Cadence::Rolling {
            every_days: IntervalDays::try_from(days).unwrap(),
        }
    }

    fn line(label: &str, cadence: Cadence) -> MaintenanceLine {
        MaintenanceLine {
            id: MaintenanceLineId(Uuid::new_v4()),
            label: label.to_owned(),
            cadence,
            minutes_per_intervention: Some(60),
        }
    }

    fn contract(lines: Vec<MaintenanceLine>) -> MaintenanceContract {
        let now = Utc::now();
        MaintenanceContract {
            id: MaintenanceContractId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            customer_id: CustomerId(Uuid::new_v4()),
            customer_context_id: CustomerContextId(Uuid::new_v4()),
            project_id: ProjectId(Uuid::new_v4()),
            reference: None,
            season: Season::new(2026, week(12), week(48)).unwrap(),
            manager_member_id: None,
            amount_cents: None,
            quote_id: None,
            notes: None,
            status: ContractStatus::Active,
            lines,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn site(city: Option<&str>) -> ContractSite {
        ContractSite {
            customer_name: "Client".to_owned(),
            site_label: "Site".to_owned(),
            address_line: None,
            postal_code: None,
            city: city.map(str::to_owned),
            contact: None,
        }
    }

    fn visit(
        line: &MaintenanceLine,
        done_on: NaiveDate,
        status: InterventionStatus,
    ) -> Intervention {
        let resolved = status != InterventionStatus::Proposed;
        Intervention {
            id: InterventionId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            line_id: line.id,
            done_on,
            status,
            task_id: None,
            proposed_by: MemberId(Uuid::new_v4()),
            resolved_by: resolved.then(|| MemberId(Uuid::new_v4())),
            resolved_at: resolved.then(Utc::now),
            note: None,
            created_at: Utc::now(),
        }
    }

    fn data(
        contracts: Vec<MaintenanceContract>,
        interventions: Vec<Intervention>,
        open: &[&MaintenanceLine],
    ) -> SeasonData {
        SeasonData {
            contracts,
            interventions,
            open_lines: open.iter().map(|line| line.id).collect(),
        }
    }

    fn sites(
        entries: &[(&MaintenanceContract, ContractSite)],
    ) -> HashMap<MaintenanceContractId, ContractSite> {
        entries
            .iter()
            .map(|(contract, site)| (contract.id, site.clone()))
            .collect()
    }

    fn labels(recap: &Recap) -> Vec<(Option<&str>, Vec<&str>)> {
        recap
            .groups
            .iter()
            .map(|group| {
                (
                    group.town.as_deref(),
                    group.lines.iter().map(|line| line.label.as_str()).collect(),
                )
            })
            .collect()
    }

    /// Mondays of 2026: S42 is 12 October, S43 is 19 October, S46 is 9 November.
    #[test]
    fn spec_ent_5_the_recap_lists_what_is_due_that_week_and_what_is_overdue_overdue_first() {
        let due_tuesday = line("Due mardi", rolling(14));
        let due_thursday = line("Due jeudi", rolling(14));
        let due_saturday = line("Due samedi", rolling(14));
        let overdue = line("En retard", rolling(14));
        let already_planned = line("Deja planifiee", rolling(14));
        let due_later_a = line("S46 a", rolling(14));
        let due_later_b = line("S46 b", rolling(14));
        let visits = vec![
            visit(
                &due_tuesday,
                date(2026, 10, 6),
                InterventionStatus::Validated,
            ),
            visit(
                &due_thursday,
                date(2026, 10, 8),
                InterventionStatus::Validated,
            ),
            visit(
                &due_saturday,
                date(2026, 10, 10),
                InterventionStatus::Validated,
            ),
            visit(&overdue, date(2026, 9, 21), InterventionStatus::Validated),
            visit(
                &already_planned,
                date(2026, 10, 7),
                InterventionStatus::Validated,
            ),
            visit(
                &due_later_a,
                date(2026, 10, 27),
                InterventionStatus::Validated,
            ),
            visit(
                &due_later_b,
                date(2026, 10, 29),
                InterventionStatus::Validated,
            ),
        ];
        let vichy = contract(vec![due_tuesday, overdue.clone(), already_planned.clone()]);
        let sioule = contract(vec![due_thursday, due_saturday, due_later_a, due_later_b]);
        let known = sites(&[
            (&vichy, site(Some("Vichy"))),
            (&sioule, site(Some("Saint-Pourçain-sur-Sioule"))),
        ]);
        let season = data(
            vec![sioule.clone(), vichy.clone()],
            visits,
            &[&already_planned],
        );

        let recap = build_recap(2026, week(43), &season, &known).unwrap();

        assert_eq!(
            labels(&recap),
            vec![
                (Some("Vichy"), vec!["En retard", "Due mardi"]),
                (
                    Some("Saint-Pourçain-sur-Sioule"),
                    vec!["Due jeudi", "Due samedi"]
                ),
            ]
        );
        let listed: usize = recap.groups.iter().map(|group| group.lines.len()).sum();
        assert_eq!(listed, 4);
        assert_eq!(recap.groups[0].lines[0].due.status, DueStatus::Overdue);
        assert_eq!(recap.groups[0].lines[1].due.on, Some(date(2026, 10, 20)));
        assert_eq!(
            recap.groups[0].lines[0].last_done_on,
            Some(date(2026, 9, 21))
        );
    }

    #[test]
    fn towns_are_alphabetical_after_the_ones_with_overdue_lines_and_no_town_comes_last() {
        let overdue_in_vichy = line("Vichy en retard", rolling(14));
        let in_cusset = line("Cusset", rolling(14));
        let in_abrest = line("Abrest", rolling(14));
        let nowhere = line("Sans commune", rolling(14));
        let visits = vec![
            visit(
                &overdue_in_vichy,
                date(2026, 9, 21),
                InterventionStatus::Validated,
            ),
            visit(&in_cusset, date(2026, 10, 6), InterventionStatus::Validated),
            visit(&in_abrest, date(2026, 10, 6), InterventionStatus::Validated),
            visit(&nowhere, date(2026, 10, 6), InterventionStatus::Validated),
        ];
        let contracts = [
            (contract(vec![in_cusset]), site(Some("Cusset"))),
            (contract(vec![nowhere]), site(None)),
            (contract(vec![overdue_in_vichy]), site(Some("Vichy"))),
            (contract(vec![in_abrest]), site(Some("abrest"))),
        ];
        let known = sites(
            &contracts
                .iter()
                .map(|(c, s)| (c, s.clone()))
                .collect::<Vec<_>>(),
        );
        let season = data(
            contracts.iter().map(|(c, _)| c.clone()).collect(),
            visits,
            &[],
        );

        let recap = build_recap(2026, week(43), &season, &known).unwrap();

        let towns: Vec<_> = recap.groups.iter().map(|g| g.town.as_deref()).collect();
        assert_eq!(towns, [Some("Vichy"), Some("abrest"), Some("Cusset"), None]);
    }

    #[test]
    fn lines_due_the_same_day_keep_the_order_of_their_contract() {
        let zebra = line("Zebre", rolling(14));
        let apple = line("Pomme", rolling(14));
        let only = contract(vec![zebra, apple]);
        let known = sites(&[(&only, site(Some("Vichy")))]);
        let season = data(vec![only], Vec::new(), &[]);

        let recap = build_recap(2026, week(43), &season, &known).unwrap();

        assert_eq!(
            labels(&recap),
            vec![(Some("Vichy"), vec!["Zebre", "Pomme"])]
        );
    }

    #[test]
    fn spec_ent_14_an_on_demand_line_is_never_in_the_recap() {
        let on_demand = line("Entretien T1", Cadence::OnDemand);
        let only = contract(vec![on_demand]);
        let known = sites(&[(&only, site(Some("Vichy")))]);
        let season = data(vec![only], Vec::new(), &[]);

        let recap = build_recap(2026, week(43), &season, &known).unwrap();

        assert!(recap.groups.is_empty());
    }

    #[test]
    fn a_window_still_open_that_week_is_in_the_recap_and_a_closed_one_is_overdue() {
        let open = line(
            "Fenetre ouverte",
            Cadence::Windows {
                windows: Windows::try_from(vec![Window::new(week(40), week(45)).unwrap()]).unwrap(),
            },
        );
        let missed = line(
            "Fenetre manquee",
            Cadence::Windows {
                windows: Windows::try_from(vec![Window::new(week(30), week(32)).unwrap()]).unwrap(),
            },
        );
        let only = contract(vec![open, missed]);
        let known = sites(&[(&only, site(Some("Vichy")))]);
        let season = data(vec![only], Vec::new(), &[]);

        let recap = build_recap(2026, week(43), &season, &known).unwrap();

        let statuses: Vec<_> = recap.groups[0]
            .lines
            .iter()
            .map(|l| (l.label.as_str(), l.due.status))
            .collect();
        assert_eq!(
            statuses,
            [
                ("Fenetre manquee", DueStatus::Overdue),
                ("Fenetre ouverte", DueStatus::ThisWeek)
            ]
        );
    }

    #[test]
    fn a_draft_contract_is_never_due() {
        let never = line("Brouillon", rolling(14));
        let mut draft = contract(vec![never]);
        draft.status = ContractStatus::Draft;
        let known = sites(&[(&draft, site(Some("Vichy")))]);
        let season = data(vec![draft], Vec::new(), &[]);

        let recap = build_recap(2026, week(43), &season, &known).unwrap();
        let grid = build_grid(2026, &season, &known, date(2026, 10, 12)).unwrap();

        assert!(recap.groups.is_empty());
        assert_eq!(grid.contracts[0].lines[0].due.status, DueStatus::Done);
    }

    #[test]
    fn a_week_the_year_lacks_is_a_conflict() {
        let season = SeasonData::default();

        let result = build_recap(2027, week(53), &season, &HashMap::new());

        assert!(matches!(result, Err(CoreError::Conflict(_))));
    }

    #[test]
    fn a_proposed_visit_shows_in_the_grid_but_leaves_the_due_date_unchanged() {
        let mowing = line("Tonte", rolling(14));
        let validated = visit(&mowing, date(2026, 10, 10), InterventionStatus::Validated);
        let proposed = visit(&mowing, date(2026, 10, 20), InterventionStatus::Proposed);
        let rejected = visit(&mowing, date(2026, 10, 21), InterventionStatus::Rejected);
        let only = contract(vec![mowing]);
        let known = sites(&[(&only, site(Some("Vichy")))]);
        let today = date(2026, 10, 22);
        let without = data(vec![only.clone()], vec![validated.clone()], &[]);
        let with = data(vec![only], vec![validated, proposed.clone(), rejected], &[]);

        let before = build_grid(2026, &without, &known, today).unwrap();
        let after = build_grid(2026, &with, &known, today).unwrap();

        let (before_line, after_line) =
            (&before.contracts[0].lines[0], &after.contracts[0].lines[0]);
        assert_eq!(before_line.due, after_line.due);
        assert_eq!(after_line.due.on, Some(date(2026, 10, 24)));
        let shown: Vec<_> = after_line
            .visits
            .iter()
            .map(|v| (v.done_on, v.week.get(), v.status))
            .collect();
        assert_eq!(
            shown,
            [
                (date(2026, 10, 10), 41, InterventionStatus::Validated),
                (date(2026, 10, 20), 43, InterventionStatus::Proposed),
            ]
        );
    }

    #[test]
    fn the_grid_computes_each_due_date_as_of_today_and_marks_planned_lines() {
        let mowing = line("Tonte", rolling(14));
        let hedge = line("Haie", rolling(14));
        let visits = vec![
            visit(&mowing, date(2026, 10, 10), InterventionStatus::Validated),
            visit(&hedge, date(2026, 10, 10), InterventionStatus::Validated),
        ];
        let only = contract(vec![mowing, hedge.clone()]);
        let known = sites(&[(&only, site(Some("Vichy")))]);
        let season = data(vec![only], visits, &[&hedge]);

        let grid = build_grid(2026, &season, &known, date(2026, 10, 12)).unwrap();

        let lines = &grid.contracts[0].lines;
        assert_eq!(lines[0].due.status, DueStatus::NextWeek);
        assert_eq!(lines[1].due.status, DueStatus::Planned);
        assert_eq!(lines[1].due.on, Some(date(2026, 10, 24)));
    }

    #[test]
    fn a_contract_with_no_loaded_site_is_an_internal_error() {
        let only = contract(vec![line("Tonte", rolling(14))]);
        let season = data(vec![only], Vec::new(), &[]);

        let result = build_grid(2026, &season, &HashMap::new(), date(2026, 10, 12));

        assert!(matches!(result, Err(CoreError::Internal(_))));
    }
}
