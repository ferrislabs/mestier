#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use chrono::NaiveDate;
    use common::{OrganizationId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::leave_account::commands::ListLeaveAccountsCommand;
    use crate::domain::leave_adjustment::commands::{
        CreateLeaveAdjustmentCommand, DeleteLeaveAdjustmentCommand, LeaveAdjustmentDraft,
        ListLeaveAdjustmentsCommand,
    };
    use crate::infrastructure::realtime::EventHub;
    use crate::{AdjustmentKind, LeaveAccount, LeaveAdjustmentError, LeaveAdjustmentId, MemberId};

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    struct Fixture {
        organization_id: OrganizationId,
        owner_id: uuid::Uuid,
        member_id: MemberId,
        accounts: Vec<LeaveAccount>,
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
            "Adjustments Org",
            format!("adjustments-org-{org_id}"),
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
        let accounts = usecase
            .list_leave_accounts(ListLeaveAccountsCommand {
                actor: authz::Subject::system(),
                organization_id,
            })
            .await
            .unwrap();

        Fixture {
            organization_id,
            owner_id,
            member_id: MemberId(member_id),
            accounts,
        }
    }

    async fn cleanup(pool: &PgPool, fixture: &Fixture) {
        let org = fixture.organization_id.0;
        for statement in [
            "DELETE FROM leave_adjustments WHERE org_id = $1",
            "DELETE FROM absence_types WHERE org_id = $1",
            "DELETE FROM leave_accounts WHERE org_id = $1",
            "DELETE FROM organization_members WHERE organization_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(pool, statement, org).await;
        }
        purge(pool, "DELETE FROM users WHERE id = $1", fixture.owner_id).await;
    }

    fn draft(fixture: &Fixture, days: f64, reason: &str) -> LeaveAdjustmentDraft {
        LeaveAdjustmentDraft {
            member_id: fixture.member_id,
            account_id: fixture.accounts[0].id,
            date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            days,
            kind: AdjustmentKind::Adjustment,
            reason: reason.to_owned(),
        }
    }

    async fn create(
        usecase: &MestierUseCase,
        fixture: &Fixture,
        draft: LeaveAdjustmentDraft,
    ) -> crate::LeaveAdjustmentOutcome<crate::LeaveAdjustment> {
        usecase
            .create_leave_adjustment(CreateLeaveAdjustmentCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                draft,
            })
            .await
            .unwrap()
    }

    async fn list(
        usecase: &MestierUseCase,
        fixture: &Fixture,
        member_id: Option<MemberId>,
        account_id: Option<crate::LeaveAccountId>,
    ) -> Vec<crate::LeaveAdjustment> {
        usecase
            .list_leave_adjustments(ListLeaveAdjustmentsCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                member_id,
                account_id,
            })
            .await
            .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_4_2_an_adjustment_of_zero_days_or_without_a_reason_is_refused() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool, &usecase).await;

        let zero = create(&usecase, &fixture, draft(&fixture, 0.0, "Erreur")).await;
        let blank = create(&usecase, &fixture, draft(&fixture, 2.0, "  ")).await;
        let kept = create(
            &usecase,
            &fixture,
            draft(&fixture, -1.5, "Erreur de saisie"),
        )
        .await;

        assert_eq!(zero, Err(LeaveAdjustmentError::DaysInvalid));
        assert_eq!(blank, Err(LeaveAdjustmentError::ReasonEmpty));
        let kept = kept.unwrap();
        assert_eq!(kept.days.days().hundredths(), -150);
        assert_eq!(list(&usecase, &fixture, None, None).await, vec![kept]);

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_4_3_deleting_an_adjustment_removes_it_from_the_list() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool, &usecase).await;

        let first = create(&usecase, &fixture, draft(&fixture, 2.0, "Geste"))
            .await
            .unwrap();
        let second = create(&usecase, &fixture, draft(&fixture, 1.0, "Geste bis"))
            .await
            .unwrap();
        usecase
            .delete_leave_adjustment(DeleteLeaveAdjustmentCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: first.id,
            })
            .await
            .unwrap();
        let missing = usecase
            .delete_leave_adjustment(DeleteLeaveAdjustmentCommand {
                actor: authz::Subject::system(),
                organization_id: fixture.organization_id,
                id: LeaveAdjustmentId(generate_uuid_v7()),
            })
            .await
            .unwrap_err();

        assert_eq!(list(&usecase, &fixture, None, None).await, vec![second]);
        assert!(matches!(missing, common::CoreError::NotFound));

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_4_4_an_archived_account_refuses_new_adjustments_and_keeps_the_old_ones() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool, &usecase).await;

        let before = create(&usecase, &fixture, draft(&fixture, 2.0, "Geste"))
            .await
            .unwrap();
        usecase
            .archive_leave_account(
                crate::domain::leave_account::commands::ArchiveLeaveAccountCommand {
                    actor: authz::Subject::system(),
                    organization_id: fixture.organization_id,
                    id: fixture.accounts[0].id,
                    archived: true,
                },
            )
            .await
            .unwrap();
        let after = create(&usecase, &fixture, draft(&fixture, 1.0, "Trop tard")).await;

        assert_eq!(after, Err(LeaveAdjustmentError::AccountArchived));
        assert_eq!(list(&usecase, &fixture, None, None).await, vec![before]);

        cleanup(&pool, &fixture).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn adjustments_filter_by_member_and_account_and_stay_in_their_organization() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());
        let fixture = seed(&pool, &usecase).await;
        let other = seed(&pool, &usecase).await;

        let on_cp = create(&usecase, &fixture, draft(&fixture, 2.0, "CP"))
            .await
            .unwrap();
        let on_rtt = create(
            &usecase,
            &fixture,
            LeaveAdjustmentDraft {
                account_id: fixture.accounts[1].id,
                ..draft(&fixture, 1.0, "RTT")
            },
        )
        .await
        .unwrap();
        let stranger_member = create(
            &usecase,
            &fixture,
            LeaveAdjustmentDraft {
                member_id: other.member_id,
                ..draft(&fixture, 1.0, "Etranger")
            },
        )
        .await;
        let stranger_account = create(
            &usecase,
            &fixture,
            LeaveAdjustmentDraft {
                account_id: other.accounts[0].id,
                ..draft(&fixture, 1.0, "Etranger")
            },
        )
        .await;

        assert_eq!(stranger_member, Err(LeaveAdjustmentError::MemberUnknown));
        assert_eq!(stranger_account, Err(LeaveAdjustmentError::AccountUnknown));
        assert_eq!(list(&usecase, &fixture, None, None).await.len(), 2);
        assert_eq!(
            list(
                &usecase,
                &fixture,
                Some(fixture.member_id),
                Some(fixture.accounts[1].id)
            )
            .await,
            vec![on_rtt]
        );
        assert_eq!(
            list(&usecase, &fixture, None, Some(fixture.accounts[0].id)).await,
            vec![on_cp]
        );
        assert!(list(&usecase, &other, None, None).await.is_empty());

        cleanup(&pool, &fixture).await;
        cleanup(&pool, &other).await;
    }
}
