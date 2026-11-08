#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use chrono::NaiveDate;
    use common::{CoreError, OrganizationId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::absence::commands::{AbsenceSchedule, CreateAbsenceCommand};
    use crate::domain::day_override::commands::{
        DeleteDayOverrideCommand, HrPlanningFeedCommand, ListDayOverridesCommand,
        PutDayOverrideCommand,
    };
    use crate::domain::member_contract::commands::CreateMemberContractCommand;
    use crate::infrastructure::realtime::EventHub;
    use crate::{
        AbsenceKind, ContractType, DayOverride, DayOverrideError, DayOverrideInput, DayPart,
        DayRefusal, MemberContractInput, MemberId, OverrideKind, PlanningPeriod, ScheduleInput,
        WorkDayInput,
    };

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        owner_id: uuid::Uuid,
        member_id: MemberId,
    }

    async fn seed(pool: &PgPool) -> Fixture {
        let owner_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            owner_id,
            format!("owner-{owner_id}@example.com"),
            format!("owner-{owner_id}"),
            "Owner User",
            format!("sub-owner-{owner_id}"),
        )
        .execute(pool)
        .await
        .unwrap();
        let org_id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO organizations (id, name, slug, owner_id)
               VALUES ($1, $2, $3, $4)"#,
            org_id,
            "Overrides Org",
            format!("overrides-org-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();
        let member_id = generate_uuid_v7();
        sqlx::query!(
            "INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, 'Claire')",
            member_id,
            org_id,
        )
        .execute(pool)
        .await
        .unwrap();
        let organization_id = OrganizationId(org_id);

        Fixture {
            organization_id,
            owner_id,
            member_id: MemberId(member_id),
        }
    }

    async fn cleanup(pool: &PgPool, fixture: &Fixture) {
        let org = fixture.organization_id.0;
        for statement in [
            "DELETE FROM day_overrides WHERE org_id = $1",
            "DELETE FROM absences WHERE org_id = $1",
            "DELETE FROM member_contracts WHERE org_id = $1",
            "DELETE FROM member_hr_profiles WHERE org_id = $1",
            "DELETE FROM absence_types WHERE org_id = $1",
            "DELETE FROM leave_accounts WHERE org_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(pool, statement, org).await;
        }
        purge(pool, "DELETE FROM users WHERE id = $1", fixture.owner_id).await;
    }

    fn date(month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, month, day).unwrap()
    }

    fn period(from: NaiveDate, to: NaiveDate) -> PlanningPeriod {
        PlanningPeriod::new(from, to, 62).unwrap()
    }

    fn work(start: &str, end: &str) -> DayOverrideInput {
        DayOverrideInput {
            kind: OverrideKind::Work,
            start: Some(start.to_owned()),
            end: Some(end.to_owned()),
            lunch: None,
            note: String::new(),
        }
    }

    async fn put(
        usecase: &MestierUseCase,
        fixture: &Fixture,
        day: NaiveDate,
        input: DayOverrideInput,
    ) -> Result<crate::DayOverrideOutcome<DayOverride>, CoreError> {
        usecase
            .put_day_override(PutDayOverrideCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                member_id: fixture.member_id,
                date: day,
                input,
            })
            .await
    }

    async fn list(
        usecase: &MestierUseCase,
        fixture: &Fixture,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Vec<DayOverride> {
        usecase
            .list_day_overrides(ListDayOverridesCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                period: period(from, to),
                member_id: None,
            })
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_5_1_a_second_override_for_the_same_day_replaces_the_first() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool).await;

        put(&usecase, &fixture, date(11, 10), work("08:00", "17:00"))
            .await
            .unwrap()
            .unwrap();
        let replaced = put(&usecase, &fixture, date(11, 10), work("09:00", "12:00"))
            .await
            .unwrap()
            .unwrap();
        let listed = list(&usecase, &fixture, date(11, 1), date(11, 30)).await;

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, replaced.id);
        assert_eq!(listed[0].shape.work().unwrap().worked_minutes(), 180);

        usecase
            .delete_day_override(DeleteDayOverrideCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                member_id: fixture.member_id,
                date: date(11, 10),
            })
            .await
            .unwrap();
        let again = usecase
            .delete_day_override(DeleteDayOverrideCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                member_id: fixture.member_id,
                date: date(11, 10),
            })
            .await
            .unwrap_err();
        assert!(matches!(again, CoreError::NotFound));
        assert!(
            list(&usecase, &fixture, date(11, 1), date(11, 30))
                .await
                .is_empty()
        );

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_5_2_a_work_override_with_end_before_start_is_refused_and_nothing_is_stored() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool).await;

        let refused = put(&usecase, &fixture, date(11, 10), work("17:00", "08:00"))
            .await
            .unwrap();

        assert_eq!(
            refused.unwrap_err(),
            DayOverrideError::Day(DayRefusal::EndBeforeStart)
        );
        assert!(
            list(&usecase, &fixture, date(11, 1), date(11, 30))
                .await
                .is_empty()
        );

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_override_for_a_member_of_another_organization_is_not_found() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool).await;
        let other = seed(&pool).await;

        let outcome = usecase
            .put_day_override(PutDayOverrideCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                member_id: other.member_id,
                date: date(11, 10),
                input: work("08:00", "17:00"),
            })
            .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)));

        cleanup(&pool, &fixture).await;
        cleanup(&pool, &other).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_5_3_the_planning_feed_returns_only_the_requested_period() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool).await;
        let other = seed(&pool).await;

        for day in [date(10, 28), date(11, 10), date(12, 20)] {
            put(&usecase, &fixture, day, work("08:00", "17:00"))
                .await
                .unwrap()
                .unwrap();
        }
        put(&usecase, &other, date(11, 10), work("08:00", "17:00"))
            .await
            .unwrap()
            .unwrap();
        for (start, end) in [
            (date(10, 20), date(10, 25)),
            (date(10, 30), date(11, 2)),
            (date(11, 12), date(11, 13)),
            (date(12, 1), date(12, 3)),
        ] {
            usecase
                .create_absence(CreateAbsenceCommand {
                    actor: authz::Subject::system(),
                    organization_id: fixture.organization_id,
                    member_id: fixture.member_id,
                    kind: Some(AbsenceKind::Leave),
                    type_id: None,
                    schedule: AbsenceSchedule::Days {
                        start_date: start,
                        start_part: DayPart::Morning,
                        end_date: end,
                        end_part: DayPart::Afternoon,
                    },
                    note: None,
                    proof_file_key: None,
                })
                .await
                .unwrap()
                .unwrap();
        }
        let variable = |start: NaiveDate, end: Option<NaiveDate>| CreateMemberContractCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            member_id: fixture.member_id,
            input: MemberContractInput {
                contract_type: ContractType::Cdi,
                start_date: start,
                end_date: end,
                trial_end_date: None,
                schedule: ScheduleInput::Variable {
                    default_day: WorkDayInput {
                        start: "08:00".to_owned(),
                        end: "16:00".to_owned(),
                        lunch: None,
                    },
                },
            },
        };
        usecase
            .create_member_contract(variable(date(1, 1), Some(date(10, 31))))
            .await
            .unwrap()
            .unwrap();
        usecase
            .create_member_contract(variable(date(11, 1), None))
            .await
            .unwrap()
            .unwrap();
        let colleague = generate_uuid_v7();
        sqlx::query("INSERT INTO organization_members (id, organization_id, last_name) VALUES ($1, $2, 'Archive')")
            .bind(colleague)
            .bind(fixture.organization_id.0)
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO member_hr_profiles (member_id, org_id, archived_at) VALUES ($1, $2, now())")
            .bind(colleague)
            .bind(fixture.organization_id.0)
            .execute(&pool)
            .await
            .unwrap();

        let feed = usecase
            .hr_planning_feed(HrPlanningFeedCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                period: period(date(11, 1), date(11, 30)),
            })
            .await
            .unwrap();

        assert_eq!(
            feed.overrides
                .iter()
                .map(|row| row.date)
                .collect::<Vec<_>>(),
            vec![date(11, 10)]
        );
        assert_eq!(
            feed.absences
                .iter()
                .map(|row| (row.start_date, row.end_date))
                .collect::<Vec<_>>(),
            vec![(date(10, 30), date(11, 2)), (date(11, 12), date(11, 13))]
        );
        assert_eq!(feed.contracts.len(), 1);
        assert_eq!(feed.contracts[0].period.start(), date(11, 1));
        assert_eq!(feed.members.len(), 2);
        let flags: Vec<_> = feed
            .members
            .iter()
            .map(|entry| (entry.member.id == fixture.member_id, entry.archived))
            .collect();
        assert!(flags.contains(&(true, false)));
        assert!(flags.contains(&(false, true)));
        assert!(
            feed.absence_types
                .iter()
                .any(|absence_type| absence_type.name.as_str() == "Congés payés")
        );

        cleanup(&pool, &fixture).await;
        cleanup(&pool, &other).await;
    }
}
