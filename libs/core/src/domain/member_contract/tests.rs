use std::sync::{Arc, Mutex};

use authz::{Decision, MockAuthorizer, Subject};
use chrono::{NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};
use serde_json::json;

use super::{
    ContractSchedule, ContractType, MemberContract, MemberContractError, MemberContractId,
    MemberContractInput, MemberContractService, ScheduleInput,
    commands::{
        CreateMemberContractCommand, DeleteMemberContractCommand, ListMemberContractsCommand,
        UpdateMemberContractCommand,
    },
    ports::MockMemberContractRepository,
};
use crate::{Member, MemberId, MockMemberRepository, MockRoleRepository, OrganizationId};

fn day(from: &str, to: &str) -> serde_json::Value {
    json!({ "start": from, "end": to })
}

fn week_of(days: [serde_json::Value; 7]) -> serde_json::Value {
    json!(days)
}

fn nominal_week() -> serde_json::Value {
    let open =
        json!({ "start": "08:00", "end": "17:00", "lunch": { "start": "12:00", "end": "13:00" } });
    week_of([
        open.clone(),
        open.clone(),
        open.clone(),
        open.clone(),
        open,
        json!(null),
        json!(null),
    ])
}

fn schedule(value: serde_json::Value) -> ScheduleInput {
    serde_json::from_value(value).expect("a well-formed wire schedule")
}

fn weekly() -> ScheduleInput {
    schedule(json!({ "kind": "weekly", "week": nominal_week() }))
}

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).expect("a calendar date")
}

fn input(start: NaiveDate, end: Option<NaiveDate>) -> MemberContractInput {
    MemberContractInput {
        contract_type: ContractType::Cdi,
        start_date: start,
        end_date: end,
        trial_end_date: None,
        schedule: weekly(),
    }
}

fn refused(value: serde_json::Value) -> super::ScheduleError {
    ContractSchedule::try_from(schedule(value)).expect_err("the schedule is refused")
}

#[test]
fn spec_rh_2_1_a_weekly_a_rotation_and_a_variable_schedule_round_trip_unchanged() {
    let samples = [
        json!({ "kind": "weekly", "week": nominal_week() }),
        json!({
            "kind": "rotation",
            "anchor": "2026-01-05",
            "blocks": [
                { "length": 2, "unit": "weeks", "label": "Chantier", "week": nominal_week() },
                { "length": 1, "unit": "months", "label": "Repos", "week": week_of([
                    json!(null), json!(null), json!(null), json!(null), json!(null), json!(null), json!(null),
                ]) },
            ],
        }),
        json!({
            "kind": "variable",
            "default_day": { "start": "09:00", "end": "17:30", "lunch": { "start": "12:30", "end": "13:30" } },
        }),
    ];
    for sample in samples {
        let parsed = ContractSchedule::try_from(schedule(sample.clone())).expect("valid");
        let back = serde_json::to_value(ScheduleInput::from(&parsed)).expect("serializes");
        assert_eq!(back, sample);
    }
}

#[test]
fn spec_rh_2_2_a_day_ending_before_it_starts_is_refused_naming_the_day() {
    let mut week = nominal_week();
    week[2] = day("17:00", "08:00");
    let error = refused(json!({ "kind": "weekly", "week": week }));
    assert_eq!(error.field(), "schedule.week[2]");
}

#[test]
fn spec_rh_2_2_a_lunch_outside_the_day_is_refused_naming_the_day() {
    let mut week = nominal_week();
    week[4] =
        json!({ "start": "08:00", "end": "17:00", "lunch": { "start": "07:00", "end": "08:30" } });
    let error = refused(json!({ "kind": "weekly", "week": week }));
    assert_eq!(error.field(), "schedule.week[4]");

    let mut week = nominal_week();
    week[0] =
        json!({ "start": "08:00", "end": "17:00", "lunch": { "start": "16:00", "end": "17:00" } });
    assert_eq!(
        refused(json!({ "kind": "weekly", "week": week })).field(),
        "schedule.week[0]"
    );

    let mut week = nominal_week();
    week[1] =
        json!({ "start": "08:00", "end": "17:00", "lunch": { "start": "13:00", "end": "12:00" } });
    assert_eq!(
        refused(json!({ "kind": "weekly", "week": week })).field(),
        "schedule.week[1]"
    );
}

#[test]
fn spec_rh_2_2_times_are_strict_hh_mm_and_a_week_has_seven_days() {
    let mut week = nominal_week();
    week[3] = day("8:00", "17:00");
    assert_eq!(
        refused(json!({ "kind": "weekly", "week": week })).field(),
        "schedule.week[3]"
    );
    let mut week = nominal_week();
    week[3] = day("08:00", "24:00");
    assert_eq!(
        refused(json!({ "kind": "weekly", "week": week })).field(),
        "schedule.week[3]"
    );
    let short = json!([null, null, null]);
    assert_eq!(
        refused(json!({ "kind": "weekly", "week": short })).field(),
        "schedule.week"
    );
}

#[test]
fn spec_rh_2_2_the_variable_default_day_and_rotation_blocks_are_checked_too() {
    assert_eq!(
        refused(json!({ "kind": "variable", "default_day": day("18:00", "09:00") })).field(),
        "schedule.default_day"
    );
    let mut week = nominal_week();
    week[6] = day("10:00", "09:00");
    let rotation = |blocks: serde_json::Value, anchor: &str| json!({ "kind": "rotation", "anchor": anchor, "blocks": blocks });
    let block = |week: serde_json::Value| json!({ "length": 1, "unit": "weeks", "label": "", "week": week });
    assert_eq!(
        refused(rotation(
            json!([block(nominal_week()), block(week)]),
            "2026-01-05"
        ))
        .field(),
        "schedule.blocks[1].week[6]"
    );
    assert_eq!(
        refused(rotation(json!([]), "2026-01-05")).field(),
        "schedule.blocks"
    );
    let too_long = json!([{ "length": 53, "unit": "weeks", "label": "", "week": nominal_week() }]);
    assert_eq!(
        refused(rotation(too_long, "2026-01-05")).field(),
        "schedule.blocks[0].length"
    );
    let zero = json!([{ "length": 0, "unit": "months", "label": "", "week": nominal_week() }]);
    assert_eq!(
        refused(rotation(zero, "2026-01-05")).field(),
        "schedule.blocks[0].length"
    );
    let months = json!([{ "length": 2, "unit": "months", "label": "", "week": nominal_week() }]);
    assert_eq!(
        refused(rotation(months, "2026-01-30")).field(),
        "schedule.anchor"
    );
}

#[test]
fn spec_rh_2_4_a_trial_end_outside_the_contract_is_refused() {
    let period = |start, end, trial| super::ContractPeriod::new(start, end, trial);
    let start = date(2026, 3, 1);
    let end = Some(date(2026, 9, 30));
    assert!(period(start, end, Some(date(2026, 3, 31))).is_ok());
    assert!(period(start, end, Some(date(2026, 9, 30))).is_ok());
    assert_eq!(
        period(start, end, Some(date(2026, 2, 28))),
        Err(MemberContractError::TrialOutsideContract)
    );
    assert_eq!(
        period(start, end, Some(date(2026, 10, 1))),
        Err(MemberContractError::TrialOutsideContract)
    );
    assert!(period(start, None, Some(date(2030, 1, 1))).is_ok());
    assert_eq!(
        period(start, Some(date(2026, 2, 1)), None),
        Err(MemberContractError::EndBeforeStart)
    );
    assert_eq!(
        MemberContractError::TrialOutsideContract.field(),
        "trial_end_date"
    );
}

#[test]
fn fixed_term_types_need_an_end_date() {
    let mut cdd = input(date(2026, 3, 1), None);
    cdd.contract_type = ContractType::Cdd;
    let refusal = MemberContract::draft(
        MemberContractId(generate_uuid_v7()),
        OrganizationId(generate_uuid_v7()),
        MemberId(generate_uuid_v7()),
        cdd,
        Utc::now(),
    );
    assert_eq!(
        refusal.unwrap_err(),
        MemberContractError::FixedTermNeedsEndDate
    );
}

#[test]
fn periods_overlap_by_a_shared_day_and_an_open_end_reaches_forever() {
    let period = |start, end| super::ContractPeriod::new(start, end, None).expect("valid");
    let first = period(date(2026, 1, 1), Some(date(2026, 3, 31)));
    assert!(first.overlaps(&period(date(2026, 3, 31), None)));
    assert!(!first.overlaps(&period(date(2026, 4, 1), None)));
    assert!(period(date(2026, 1, 1), None).overlaps(&period(date(2040, 1, 1), None)));
    assert!(!period(date(2026, 6, 1), None).overlaps(&first));
}

fn member(id: MemberId, organization_id: OrganizationId) -> Member {
    Member {
        id,
        organization_id,
        user_id: None,
        last_name: "Claire".to_owned(),
        first_name: None,
        joined_at: None,
        created_at: Utc::now(),
        deleted_at: None,
    }
}

fn allow_all() -> MockAuthorizer {
    let mut authz = MockAuthorizer::new();
    authz
        .expect_evaluate()
        .returning(|_| Box::pin(async { Ok(Decision::allow()) }));
    authz
}

type Service = MemberContractService<
    MockMemberContractRepository,
    MockMemberRepository,
    MockRoleRepository,
    MockAuthorizer,
>;

struct World {
    organization: OrganizationId,
    member: MemberId,
    stored: Vec<MemberContract>,
}

impl World {
    fn new() -> Self {
        Self {
            organization: OrganizationId(generate_uuid_v7()),
            member: MemberId(generate_uuid_v7()),
            stored: Vec::new(),
        }
    }

    fn with_contract(
        mut self,
        start: NaiveDate,
        end: Option<NaiveDate>,
    ) -> (Self, MemberContractId) {
        let id = MemberContractId(generate_uuid_v7());
        let contract = MemberContract::draft(
            id,
            self.organization,
            self.member,
            input(start, end),
            Utc::now(),
        )
        .expect("valid contract");
        self.stored.push(contract);
        (self, id)
    }

    fn service(self) -> (Service, Arc<Mutex<Vec<MemberContract>>>) {
        let mut members = MockMemberRepository::new();
        let known = member(self.member, self.organization);
        members.expect_find_by_id().returning(move |id| {
            let found = (id == known.id).then(|| known.clone());
            Box::pin(async move { Ok(found) })
        });
        let mut contracts = MockMemberContractRepository::new();
        let stored = self.stored.clone();
        contracts
            .expect_list_by_member()
            .returning(move |_, member_id| {
                let own: Vec<MemberContract> = stored
                    .iter()
                    .filter(|contract| contract.member_id == member_id)
                    .cloned()
                    .collect();
                Box::pin(async move { Ok(own) })
            });
        let listed = self.stored.clone();
        contracts.expect_list_by_organization().returning(move |_| {
            let all = listed.clone();
            Box::pin(async move { Ok(all) })
        });
        let found = self.stored.clone();
        contracts.expect_find().returning(move |_, id| {
            let hit = found.iter().find(|contract| contract.id == id).cloned();
            Box::pin(async move { Ok(hit) })
        });
        let log = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&log);
        contracts.expect_insert().returning(move |contract| {
            let saved = contract.clone();
            sink.lock().expect("log").push(saved.clone());
            Box::pin(async move { Ok(saved) })
        });
        let sink = Arc::clone(&log);
        contracts.expect_update().returning(move |contract| {
            let saved = contract.clone();
            sink.lock().expect("log").push(saved.clone());
            Box::pin(async move { Ok(saved) })
        });
        contracts
            .expect_delete()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        (
            MemberContractService::new(contracts, members, MockRoleRepository::new(), allow_all()),
            log,
        )
    }
}

fn create(world: &World, input: MemberContractInput) -> CreateMemberContractCommand {
    CreateMemberContractCommand {
        actor: Subject::system(),
        organization_id: world.organization,
        member_id: world.member,
        input,
    }
}

#[tokio::test]
async fn spec_rh_2_3_two_contracts_overlapping_for_one_member_are_refused() {
    let (world, _) = World::new().with_contract(date(2026, 1, 1), Some(date(2026, 6, 30)));
    let command = create(&world, input(date(2026, 6, 30), None));
    let (mut service, log) = world.service();
    let outcome = service.create(command).await.expect("no storage failure");
    assert!(matches!(outcome, Err(MemberContractError::Overlap)));
    assert!(log.lock().expect("log").is_empty());
}

#[tokio::test]
async fn spec_rh_2_3_a_successor_starting_the_day_after_is_accepted() {
    let (world, _) = World::new().with_contract(date(2026, 1, 1), Some(date(2026, 6, 30)));
    let command = create(&world, input(date(2026, 7, 1), None));
    let (mut service, log) = world.service();
    assert!(service.create(command).await.expect("stored").is_ok());
    assert_eq!(log.lock().expect("log").len(), 1);
}

#[tokio::test]
async fn spec_rh_2_3_an_update_ignores_the_contract_itself_but_not_its_neighbours() {
    let (world, _) = World::new().with_contract(date(2026, 1, 1), Some(date(2026, 6, 30)));
    let (world, second) = world.with_contract(date(2026, 7, 1), None);
    let command = |start| UpdateMemberContractCommand {
        actor: Subject::system(),
        organization_id: world.organization,
        member_id: world.member,
        contract_id: second,
        input: input(start, None),
    };
    let own = command(date(2026, 7, 15));
    let clash = command(date(2026, 6, 1));
    let (mut service, _) = world.service();
    assert!(service.update(own).await.expect("stored").is_ok());
    assert!(matches!(
        service.update(clash).await.expect("no storage failure"),
        Err(MemberContractError::Overlap)
    ));
}

#[tokio::test]
async fn a_contract_of_another_member_or_organization_is_not_found() {
    let (world, id) = World::new().with_contract(date(2026, 1, 1), None);
    let stranger = MemberId(generate_uuid_v7());
    let wrong_member = DeleteMemberContractCommand {
        actor: Subject::system(),
        organization_id: world.organization,
        member_id: stranger,
        contract_id: id,
    };
    let right = DeleteMemberContractCommand {
        member_id: world.member,
        ..wrong_member.clone()
    };
    let listed = ListMemberContractsCommand {
        actor: Subject::system(),
        organization_id: OrganizationId(generate_uuid_v7()),
        member_id: Some(world.member),
    };
    let (mut service, _) = world.service();
    assert!(matches!(
        service.delete(wrong_member).await,
        Err(CoreError::NotFound)
    ));
    assert!(service.delete(right).await.is_ok());
    assert!(matches!(
        service.list(listed).await,
        Err(CoreError::NotFound)
    ));
}
