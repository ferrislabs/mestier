use chrono::{DateTime, Duration, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use super::*;
use crate::{
    ApprenticeId, ClosureReason, ClosureReasonId, CompanyTutor, CompanyTutorId, HostCompanyId,
    OrganizationId, ParticipantId, Promotion, PromotionId, TrainingId,
};

fn day(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn org() -> OrganizationId {
    OrganizationId(Uuid::new_v4())
}

fn reason(organization_id: OrganizationId, archived: bool) -> ClosureReason {
    ClosureReason {
        id: ClosureReasonId(Uuid::new_v4()),
        organization_id,
        label: "Fin de formation".to_owned(),
        archived_at: archived.then(Utc::now),
        created_at: Utc::now(),
    }
}

fn closure() -> Closure {
    Closure {
        closed_at: day(2027, 6, 30),
        reason_id: ClosureReasonId(Uuid::new_v4()),
        note: None,
    }
}

fn triad(organization_id: OrganizationId) -> Triad {
    let now = Utc::now();
    Triad {
        id: TriadId(Uuid::new_v4()),
        organization_id,
        promotion_id: PromotionId(Uuid::new_v4()),
        apprentice_id: ApprenticeId(Uuid::new_v4()),
        pedagogic_tutor_member_id: None,
        closure: None,
        created_at: now,
        updated_at: now,
    }
}

fn contract(organization_id: OrganizationId, host_company_id: HostCompanyId) -> Contract {
    let now = Utc::now();
    Contract {
        id: ContractId(Uuid::new_v4()),
        organization_id,
        triad_id: TriadId(Uuid::new_v4()),
        host_company_id,
        kind: ContractKind::Apprenticeship,
        number: None,
        period: ContractPeriod::new(day(2026, 9, 1), day(2028, 8, 31)).unwrap(),
        closure: None,
        cerfa_file_key: None,
        mission_sheet_file_key: None,
        created_at: now,
        updated_at: now,
    }
}

fn tutor(organization_id: OrganizationId, host_company_id: HostCompanyId) -> CompanyTutor {
    let now = Utc::now();
    CompanyTutor {
        id: CompanyTutorId(Uuid::new_v4()),
        organization_id,
        participant_id: ParticipantId(Uuid::new_v4()),
        host_company_id,
        job_title: None,
        created_at: now,
        updated_at: now,
    }
}

#[test]
fn a_contract_period_accepts_equal_and_ordered_dates() {
    assert!(ContractPeriod::new(day(2026, 9, 1), day(2026, 9, 1)).is_ok());

    let period = ContractPeriod::new(day(2026, 9, 1), day(2028, 8, 31)).unwrap();
    assert_eq!(period.start(), day(2026, 9, 1));
    assert_eq!(period.end(), day(2028, 8, 31));
}

#[test]
fn a_contract_period_rejects_an_end_before_the_start() {
    let result = ContractPeriod::new(day(2026, 9, 2), day(2026, 9, 1));

    assert!(matches!(result, Err(CoreError::Conflict(_))));
}

#[test]
fn a_closure_refuses_an_archived_reason_and_a_foreign_one() {
    let organization_id = org();

    let archived = Closure::new(
        organization_id,
        day(2027, 6, 30),
        &reason(organization_id, true),
        None,
    );
    let foreign = Closure::new(
        organization_id,
        day(2027, 6, 30),
        &reason(org(), false),
        None,
    );

    assert!(matches!(archived, Err(CoreError::Conflict(_))));
    assert!(matches!(foreign, Err(CoreError::NotFound)));
}

#[test]
fn a_closure_trims_the_note_and_drops_a_blank_one() {
    let organization_id = org();
    let open_reason = reason(organization_id, false);

    let noted = Closure::new(
        organization_id,
        day(2027, 6, 30),
        &open_reason,
        Some("  Rupture  ".to_owned()),
    )
    .unwrap();
    let blank = Closure::new(
        organization_id,
        day(2027, 6, 30),
        &open_reason,
        Some("   ".to_owned()),
    )
    .unwrap();

    assert_eq!(noted.note.as_deref(), Some("Rupture"));
    assert_eq!(noted.reason_id, open_reason.id);
    assert_eq!(blank.note, None);
}

#[test]
fn closing_a_triad_twice_is_a_conflict() {
    let mut subject = triad(org());

    assert!(subject.close(closure(), Utc::now()).is_ok());
    assert!(subject.is_closed());
    assert!(matches!(
        subject.close(closure(), Utc::now()),
        Err(CoreError::Conflict(_))
    ));
}

#[test]
fn closing_a_contract_twice_is_a_conflict() {
    let mut subject = contract(org(), HostCompanyId(Uuid::new_v4()));

    assert!(subject.close(closure(), Utc::now()).is_ok());
    assert!(matches!(
        subject.close(closure(), Utc::now()),
        Err(CoreError::Conflict(_))
    ));
}

#[test]
fn a_contract_file_key_is_set_and_read_by_kind() {
    let mut subject = contract(org(), HostCompanyId(Uuid::new_v4()));
    let key = FileKey::try_from("uploads/cerfa-1").unwrap();

    subject.set_file_key(ContractFileKind::Cerfa, Some(key.clone()));

    assert_eq!(subject.file_key(ContractFileKind::Cerfa), Some(&key));
    assert_eq!(subject.file_key(ContractFileKind::MissionSheet), None);

    subject.set_file_key(ContractFileKind::Cerfa, None);
    assert_eq!(subject.file_key(ContractFileKind::Cerfa), None);
}

#[test]
fn tutors_of_the_contract_company_become_ranked_links() {
    let organization_id = org();
    let company = HostCompanyId(Uuid::new_v4());
    let subject = contract(organization_id, company);
    let main = tutor(organization_id, company);
    let secondary = tutor(organization_id, company);

    let links = subject
        .assign_tutors(Some(&main), Some(&secondary))
        .unwrap();

    assert_eq!(links.len(), 2);
    assert_eq!(links[0].rank, TutorRank::Main);
    assert_eq!(links[0].company_tutor_id, main.id);
    assert_eq!(links[1].rank, TutorRank::Secondary);
    assert_eq!(links[1].company_tutor_id, secondary.id);
    assert!(links.iter().all(|link| link.archived_at.is_none()));
}

#[test]
fn clearing_both_ranks_yields_no_links() {
    let subject = contract(org(), HostCompanyId(Uuid::new_v4()));

    assert_eq!(subject.assign_tutors(None, None).unwrap(), vec![]);
}

#[test]
fn a_tutor_of_another_company_is_a_conflict() {
    let organization_id = org();
    let subject = contract(organization_id, HostCompanyId(Uuid::new_v4()));
    let elsewhere = tutor(organization_id, HostCompanyId(Uuid::new_v4()));

    assert!(matches!(
        subject.assign_tutors(Some(&elsewhere), None),
        Err(CoreError::Conflict(_))
    ));
    assert!(matches!(
        subject.assign_tutors(None, Some(&elsewhere)),
        Err(CoreError::Conflict(_))
    ));
}

#[test]
fn the_same_tutor_cannot_hold_both_ranks() {
    let organization_id = org();
    let company = HostCompanyId(Uuid::new_v4());
    let subject = contract(organization_id, company);
    let only = tutor(organization_id, company);

    assert!(matches!(
        subject.assign_tutors(Some(&only), Some(&only)),
        Err(CoreError::Conflict(_))
    ));
}

#[test]
fn a_closed_contract_refuses_tutor_changes() {
    let organization_id = org();
    let company = HostCompanyId(Uuid::new_v4());
    let mut subject = contract(organization_id, company);
    subject.close(closure(), Utc::now()).unwrap();

    assert!(matches!(
        subject.assign_tutors(None, None),
        Err(CoreError::Conflict(_))
    ));
}

#[test]
fn contract_kind_and_rank_round_trip_their_storage_literals() {
    for kind in [
        ContractKind::Apprenticeship,
        ContractKind::Professionalization,
    ] {
        assert_eq!(kind.as_str().parse::<ContractKind>(), Ok(kind));
    }
    for rank in [TutorRank::Main, TutorRank::Secondary] {
        assert_eq!(rank.as_str().parse::<TutorRank>(), Ok(rank));
    }
    assert!("OTHER".parse::<ContractKind>().is_err());
    assert!("TERTIARY".parse::<TutorRank>().is_err());
}

#[test]
fn triad_ids_parse_uuids() {
    let uuid = Uuid::new_v4();

    assert_eq!(TriadId::from_str(&uuid.to_string()).unwrap().0, uuid);
    assert!(ContractId::from_str("not-a-uuid").is_err());
}

fn since(offset_days: i64) -> DateTime<Utc> {
    Utc::now() - Duration::days(offset_days)
}

#[test]
fn an_apprentice_triad_is_an_active_apprentice_link() {
    let triad_id = TriadId(Uuid::new_v4());
    let participant_id = ParticipantId(Uuid::new_v4());
    let organization_id = org();

    let scope = ParticipantScope::new(participant_id, organization_id, &[triad_id], &[]);

    assert_eq!(scope.participant_id, participant_id);
    assert_eq!(scope.organization_id, organization_id);
    assert_eq!(
        scope.link(triad_id),
        Some(&TriadLink {
            triad_id,
            role: TriadRole::Apprentice,
            access: TriadAccess::Active,
        })
    );
    assert_eq!(scope.link(TriadId(Uuid::new_v4())), None);
}

#[test]
fn an_archived_tutor_link_is_read_only_since_its_archival() {
    let triad_id = TriadId(Uuid::new_v4());
    let archived_at = since(3);
    let links = [TutorTriadLink {
        triad_id,
        rank: TutorRank::Secondary,
        archived_at: Some(archived_at),
    }];

    let scope = ParticipantScope::new(ParticipantId(Uuid::new_v4()), org(), &[], &links);

    assert_eq!(
        scope.link(triad_id).copied(),
        Some(TriadLink {
            triad_id,
            role: TriadRole::SecondaryTutor,
            access: TriadAccess::ReadOnly { since: archived_at },
        })
    );
}

#[test]
fn an_active_link_beats_a_read_only_one_whatever_the_role() {
    let triad_id = TriadId(Uuid::new_v4());
    let links = [
        TutorTriadLink {
            triad_id,
            rank: TutorRank::Main,
            archived_at: Some(since(10)),
        },
        TutorTriadLink {
            triad_id,
            rank: TutorRank::Secondary,
            archived_at: None,
        },
    ];

    let merged = merge_links(links.iter().copied().map(TriadLink::from));

    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].role, TriadRole::SecondaryTutor);
    assert_eq!(merged[0].access, TriadAccess::Active);
}

#[test]
fn on_equal_access_the_apprentice_beats_the_main_tutor_who_beats_the_secondary() {
    let triad_id = TriadId(Uuid::new_v4());
    let as_tutor = |rank| TutorTriadLink {
        triad_id,
        rank,
        archived_at: None,
    };

    let tutors_only = merge_links(
        [as_tutor(TutorRank::Secondary), as_tutor(TutorRank::Main)]
            .into_iter()
            .map(TriadLink::from),
    );
    let with_apprentice = ParticipantScope::new(
        ParticipantId(Uuid::new_v4()),
        org(),
        &[triad_id],
        &[as_tutor(TutorRank::Main)],
    );

    assert_eq!(tutors_only[0].role, TriadRole::MainTutor);
    assert_eq!(
        with_apprentice.link(triad_id).map(|link| link.role),
        Some(TriadRole::Apprentice)
    );
}

#[test]
fn between_two_read_only_links_the_stronger_role_wins() {
    let triad_id = TriadId(Uuid::new_v4());
    let links = [
        TutorTriadLink {
            triad_id,
            rank: TutorRank::Secondary,
            archived_at: Some(since(1)),
        },
        TutorTriadLink {
            triad_id,
            rank: TutorRank::Main,
            archived_at: Some(since(5)),
        },
    ];

    let merged = merge_links(links.iter().copied().map(TriadLink::from));

    assert_eq!(merged.len(), 1);
    assert_eq!(merged[0].role, TriadRole::MainTutor);
}

#[test]
fn links_to_different_triads_stay_separate() {
    let first = TriadId(Uuid::new_v4());
    let second = TriadId(Uuid::new_v4());
    let links = [TutorTriadLink {
        triad_id: second,
        rank: TutorRank::Main,
        archived_at: None,
    }];

    let scope = ParticipantScope::new(ParticipantId(Uuid::new_v4()), org(), &[first], &links);

    assert_eq!(scope.links.len(), 2);
    assert!(scope.link(first).is_some());
    assert!(scope.link(second).is_some());
}

fn promotion(organization_id: OrganizationId) -> Promotion {
    let now = Utc::now();
    Promotion {
        id: PromotionId(Uuid::new_v4()),
        organization_id,
        training_id: TrainingId(Uuid::new_v4()),
        label: "BTS 2026".to_owned(),
        start_year: 2026,
        end_year: 2028,
        archived_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn view_with_tutor(participant_id: ParticipantId) -> ContractView {
    let organization_id = org();
    let company = HostCompanyId(Uuid::new_v4());
    ContractView {
        contract: contract(organization_id, company),
        host_company_name: "Durand".to_owned(),
        closure_reason_label: None,
        tutors: vec![ContractTutorView {
            company_tutor_id: CompanyTutorId(Uuid::new_v4()),
            participant_id,
            name: "Martin Paul".to_owned(),
            rank: TutorRank::Main,
            archived_at: None,
        }],
    }
}

#[test]
fn a_tutor_only_sees_the_contracts_they_are_linked_to() {
    let organization_id = org();
    let mine = ParticipantId(Uuid::new_v4());
    let theirs = ParticipantId(Uuid::new_v4());
    let own_view = view_with_tutor(mine);
    let other_view = view_with_tutor(theirs);
    let detail = TriadDetail {
        triad: triad(organization_id),
        promotion: promotion(organization_id),
        training_name: "BTS Aménagements paysagers".to_owned(),
        apprentice_name: "Martin Léa".to_owned(),
        apprentice_email: "lea@example.com".to_owned(),
        pedagogic_tutor_name: None,
        closure_reason_label: None,
        contracts: vec![own_view.clone(), other_view],
    };

    let restricted = detail.restricted_to_tutor(mine);

    assert_eq!(restricted.contracts, vec![own_view]);
}
