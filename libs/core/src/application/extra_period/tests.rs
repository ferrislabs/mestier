#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use chrono::NaiveDate;
    use common::{OrganizationId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::extra_period::commands::{
        CreateExtraPeriodCommand, DecideExtraPeriodCommand, ExtraPeriodDraft, ExtraPeriodFilter,
        ListExtraPeriodsCommand, RejectExtraPeriodCommand,
    };
    use crate::domain::surcharge::commands::{
        ArchiveSurchargeCommand, ListSurchargesCommand, SurchargeDraft, UpdateSurchargeCommand,
    };
    use crate::infrastructure::realtime::EventHub;
    use crate::{
        DayRefusal, ExtraPeriod, ExtraPeriodError, ExtraPeriodId, ExtraStatus, MemberId, Surcharge,
        SurchargeKind,
    };

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        owner_id: uuid::Uuid,
        member_id: MemberId,
        surcharges: Vec<Surcharge>,
    }

    async fn seed(pool: &PgPool, usecase: &MestierUseCase) -> Fixture {
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
            "Extras Org",
            format!("extras-org-{org_id}"),
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
        let surcharges = usecase
            .list_surcharges(ListSurchargesCommand {
                actor: authz::Subject::system(),
                organization_id,
            })
            .await
            .unwrap();

        Fixture {
            organization_id,
            owner_id,
            member_id: MemberId(member_id),
            surcharges,
        }
    }

    async fn cleanup(pool: &PgPool, fixture: &Fixture) {
        let org = fixture.organization_id.0;
        for statement in [
            "DELETE FROM extra_periods WHERE org_id = $1",
            "DELETE FROM surcharges WHERE org_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(pool, statement, org).await;
        }
        purge(pool, "DELETE FROM users WHERE id = $1", fixture.owner_id).await;
    }

    fn draft(fixture: &Fixture) -> ExtraPeriodDraft {
        ExtraPeriodDraft {
            member_id: fixture.member_id,
            date: NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
            start: "08:00".to_owned(),
            end: "12:00".to_owned(),
            lunch: None,
            reason: "Rattrapage".to_owned(),
            surcharge_id: Some(fixture.surcharges[2].id),
            rate_bp: None,
            approve: false,
        }
    }

    async fn create(
        usecase: &MestierUseCase,
        fixture: &Fixture,
        draft: ExtraPeriodDraft,
    ) -> crate::ExtraPeriodOutcome<ExtraPeriod> {
        usecase
            .create_extra_period(CreateExtraPeriodCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                draft,
            })
            .await
            .unwrap()
    }

    fn decide(fixture: &Fixture, id: ExtraPeriodId) -> DecideExtraPeriodCommand {
        DecideExtraPeriodCommand {
            actor: authz::Subject::system(),
            organization_id: fixture.organization_id,
            id,
        }
    }

    async fn list(
        usecase: &MestierUseCase,
        fixture: &Fixture,
        filter: ExtraPeriodFilter,
    ) -> (Vec<ExtraPeriod>, u64) {
        usecase
            .list_extra_periods(ListExtraPeriodsCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                filter,
                limit: 100,
                offset: 0,
            })
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_6_2_a_period_copies_the_rate_and_a_later_change_leaves_it_unchanged() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool, &usecase).await;

        let period = create(&usecase, &fixture, draft(&fixture)).await.unwrap();
        usecase
            .update_surcharge(UpdateSurchargeCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: fixture.surcharges[2].id,
                draft: SurchargeDraft {
                    name: "Dimanche".to_owned(),
                    rate_bp: 5000,
                    kind: SurchargeKind::Sunday,
                },
            })
            .await
            .unwrap()
            .unwrap();
        let (stored, total) = list(&usecase, &fixture, ExtraPeriodFilter::default()).await;

        assert_eq!(period.rate.get(), 10_000);
        assert_eq!(period.status, ExtraStatus::Pending);
        assert_eq!(total, 1);
        assert_eq!(stored[0].rate.get(), 10_000);
        assert_eq!(stored[0].surcharge_id, Some(fixture.surcharges[2].id));

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_6_3_approve_and_reject_follow_the_absence_rules() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool, &usecase).await;
        let first = create(&usecase, &fixture, draft(&fixture)).await.unwrap();
        let second = create(&usecase, &fixture, draft(&fixture)).await.unwrap();

        let approved = usecase
            .approve_extra_period(decide(&fixture, first.id))
            .await
            .unwrap()
            .unwrap();
        let approved_again = usecase
            .approve_extra_period(decide(&fixture, first.id))
            .await
            .unwrap();
        let rejected_after = usecase
            .reject_extra_period(RejectExtraPeriodCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: first.id,
                comment: "Non".to_owned(),
            })
            .await
            .unwrap();
        let blank = usecase
            .reject_extra_period(RejectExtraPeriodCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: second.id,
                comment: "  ".to_owned(),
            })
            .await
            .unwrap();
        let rejected = usecase
            .reject_extra_period(RejectExtraPeriodCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: second.id,
                comment: "Pas de commande".to_owned(),
            })
            .await
            .unwrap()
            .unwrap();
        let cancelled_after = usecase
            .cancel_extra_period(decide(&fixture, second.id))
            .await
            .unwrap();
        let cancelled = usecase
            .cancel_extra_period(decide(&fixture, first.id))
            .await
            .unwrap()
            .unwrap();

        assert_eq!(approved.status, ExtraStatus::Approved);
        assert!(approved.decided_at.is_some());
        assert_eq!(approved_again, Err(ExtraPeriodError::AlreadyDecided));
        assert_eq!(rejected_after, Err(ExtraPeriodError::AlreadyDecided));
        assert_eq!(blank, Err(ExtraPeriodError::CommentRequired));
        assert_eq!(rejected.status, ExtraStatus::Rejected);
        assert_eq!(
            rejected.decision_comment.as_deref(),
            Some("Pas de commande")
        );
        assert_eq!(cancelled_after, Err(ExtraPeriodError::AlreadyDecided));
        assert_eq!(cancelled.status, ExtraStatus::Cancelled);

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_6_4_an_end_not_after_the_start_is_refused_and_nothing_is_stored() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool, &usecase).await;

        let equal = create(
            &usecase,
            &fixture,
            ExtraPeriodDraft {
                end: "08:00".to_owned(),
                ..draft(&fixture)
            },
        )
        .await;
        let before = create(
            &usecase,
            &fixture,
            ExtraPeriodDraft {
                end: "07:30".to_owned(),
                ..draft(&fixture)
            },
        )
        .await;

        assert_eq!(
            equal,
            Err(ExtraPeriodError::Day(DayRefusal::EndBeforeStart))
        );
        assert_eq!(
            before,
            Err(ExtraPeriodError::Day(DayRefusal::EndBeforeStart))
        );
        assert_eq!(
            list(&usecase, &fixture, ExtraPeriodFilter::default())
                .await
                .1,
            0
        );

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_archived_surcharge_is_refused_for_new_periods_and_kept_on_past_ones() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool, &usecase).await;
        let past = create(&usecase, &fixture, draft(&fixture)).await.unwrap();

        usecase
            .archive_surcharge(ArchiveSurchargeCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: fixture.surcharges[2].id,
                archived: true,
            })
            .await
            .unwrap();
        let refused = create(&usecase, &fixture, draft(&fixture)).await;
        let (stored, _) = list(&usecase, &fixture, ExtraPeriodFilter::default()).await;

        assert_eq!(refused, Err(ExtraPeriodError::SurchargeArchived));
        assert_eq!(stored, vec![past]);

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn periods_filter_stay_in_their_organization_and_refuse_foreign_ids() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool, &usecase).await;
        let other = seed(&pool, &usecase).await;
        let november = create(&usecase, &fixture, draft(&fixture)).await.unwrap();
        let october = create(
            &usecase,
            &fixture,
            ExtraPeriodDraft {
                date: NaiveDate::from_ymd_opt(2026, 10, 3).unwrap(),
                approve: true,
                surcharge_id: None,
                rate_bp: Some(0),
                ..draft(&fixture)
            },
        )
        .await
        .unwrap();
        let foreign_member = create(
            &usecase,
            &fixture,
            ExtraPeriodDraft {
                member_id: other.member_id,
                ..draft(&fixture)
            },
        )
        .await;
        let foreign_surcharge = create(
            &usecase,
            &fixture,
            ExtraPeriodDraft {
                surcharge_id: Some(other.surcharges[0].id),
                ..draft(&fixture)
            },
        )
        .await;
        let foreign_decision = usecase
            .approve_extra_period(decide(&other, november.id))
            .await
            .unwrap_err();

        let pending = list(
            &usecase,
            &fixture,
            ExtraPeriodFilter {
                status: Some(ExtraStatus::Pending),
                ..Default::default()
            },
        )
        .await;
        let in_october = list(
            &usecase,
            &fixture,
            ExtraPeriodFilter {
                from: NaiveDate::from_ymd_opt(2026, 10, 1),
                to: NaiveDate::from_ymd_opt(2026, 10, 31),
                ..Default::default()
            },
        )
        .await;
        let everything = list(&usecase, &fixture, ExtraPeriodFilter::default()).await;

        assert_eq!(foreign_member, Err(ExtraPeriodError::MemberUnknown));
        assert_eq!(foreign_surcharge, Err(ExtraPeriodError::SurchargeUnknown));
        assert!(matches!(foreign_decision, common::CoreError::NotFound));
        assert_eq!(pending, (vec![november.clone()], 1));
        assert_eq!(in_october, (vec![october.clone()], 1));
        assert_eq!(everything, (vec![november, october], 2));
        assert_eq!(
            list(&usecase, &other, ExtraPeriodFilter::default()).await.1,
            0
        );

        cleanup(&pool, &fixture).await;
        cleanup(&pool, &other).await;
    }
}
