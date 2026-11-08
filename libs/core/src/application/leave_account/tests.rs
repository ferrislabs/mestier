#[cfg(test)]
#[allow(clippy::module_inception)]
mod tests {
    use common::{OrganizationId, generate_uuid_v7};
    use sqlx::PgPool;

    use crate::application::test_support::{dev_pool, purge};
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::absence_type::commands::{
        AbsenceTypeDraft, CreateAbsenceTypeCommand, ListAbsenceTypesCommand,
        UpdateAbsenceTypeCommand,
    };
    use crate::domain::leave_account::commands::{
        ArchiveLeaveAccountCommand, CreateLeaveAccountCommand, LeaveAccountDraft,
        ListLeaveAccountsCommand, UpdateLeaveAccountCommand,
    };
    use crate::infrastructure::realtime::EventHub;
    use crate::{AbsenceKind, AbsenceTypeError, AcquisitionDraft, CountMode, LeaveAccountError};

    fn make_usecase(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    async fn seed_organization(pool: &PgPool) -> (OrganizationId, uuid::Uuid) {
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
            "Accounts Org",
            format!("accounts-org-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        (OrganizationId(org_id), owner_id)
    }

    async fn cleanup(pool: &PgPool, organization_id: OrganizationId, owner_id: uuid::Uuid) {
        for statement in [
            "DELETE FROM absence_types WHERE org_id = $1",
            "DELETE FROM leave_accounts WHERE org_id = $1",
            "DELETE FROM organizations WHERE id = $1",
        ] {
            purge(pool, statement, organization_id.0).await;
        }
        purge(pool, "DELETE FROM users WHERE id = $1", owner_id).await;
    }

    fn list(organization_id: OrganizationId) -> ListLeaveAccountsCommand {
        ListLeaveAccountsCommand {
            actor: authz::Subject::system(),
            organization_id,
        }
    }

    fn draft(name: &str) -> LeaveAccountDraft {
        LeaveAccountDraft {
            name: name.to_owned(),
            acquisition: AcquisitionDraft::Annual { days: 3.0 },
            period_start_month: 1,
            carry_over_max: 1.0,
        }
    }

    fn type_draft(name: &str, account_id: Option<uuid::Uuid>) -> AbsenceTypeDraft {
        AbsenceTypeDraft {
            name: name.to_owned(),
            hue: 10,
            count_mode: CountMode::Working,
            account_id,
            paid: true,
            needs_approval: true,
            needs_proof: false,
            kind: AbsenceKind::Leave,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_4_1_the_first_read_creates_the_default_accounts_once() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());

        let first = usecase
            .list_leave_accounts(list(organization_id))
            .await
            .unwrap();
        let second = usecase
            .list_leave_accounts(list(organization_id))
            .await
            .unwrap();

        let names: Vec<&str> = first.iter().map(|a| a.name.as_str()).collect();
        assert_eq!(names, ["Congés payés", "RTT"]);
        assert_eq!(first, second);
        assert_eq!(first[0].acquisition.amount().days().hundredths(), 208);
        assert_eq!(first[0].acquisition.kind_str(), "MONTHLY");
        assert_eq!(first[0].period_start_month.get(), 6);
        assert_eq!(first[0].carry_over_max.days().hundredths(), 500);
        assert_eq!(first[1].acquisition.kind_str(), "ANNUAL");
        assert_eq!(first[1].acquisition.amount().days().hundredths(), 1000);

        cleanup(&pool, organization_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_4_1_concurrent_first_reads_create_the_defaults_once() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());

        let (a, b, c, d) = tokio::join!(
            usecase.list_leave_accounts(list(organization_id)),
            usecase.list_leave_accounts(list(organization_id)),
            usecase.list_absence_types(ListAbsenceTypesCommand {
                actor: authz::Subject::system(),
                organization_id
            }),
            usecase.list_leave_accounts(list(organization_id)),
        );

        for read in [a, b, d] {
            assert_eq!(read.unwrap().len(), 2);
        }
        assert_eq!(c.unwrap().len(), 7);
        let accounts = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM leave_accounts WHERE org_id = $1"#,
            organization_id.0
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(accounts, 2);

        cleanup(&pool, organization_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_4_1_the_default_types_are_linked_whichever_read_comes_first() {
        let pool = dev_pool().await;
        let usecase = make_usecase(pool.clone());

        for types_first in [true, false] {
            let (organization_id, owner_id) = seed_organization(&pool).await;
            let types_command = ListAbsenceTypesCommand {
                actor: authz::Subject::system(),
                organization_id,
            };
            let (types, accounts) = if types_first {
                let types = usecase.list_absence_types(types_command).await.unwrap();
                (
                    types,
                    usecase
                        .list_leave_accounts(list(organization_id))
                        .await
                        .unwrap(),
                )
            } else {
                let accounts = usecase
                    .list_leave_accounts(list(organization_id))
                    .await
                    .unwrap();
                (
                    usecase.list_absence_types(types_command).await.unwrap(),
                    accounts,
                )
            };

            let link = |name: &str| {
                types
                    .iter()
                    .find(|t| t.name.as_str() == name)
                    .unwrap()
                    .account_id
            };
            assert_eq!(link("Congés payés"), Some(accounts[0].id.0));
            assert_eq!(link("RTT"), Some(accounts[1].id.0));
            assert_eq!(link("Formation"), None);

            cleanup(&pool, organization_id, owner_id).await;
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_4_4_an_account_is_archived_and_restored_never_deleted() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let (other_id, other_owner) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_leave_account(CreateLeaveAccountCommand {
                actor: authz::Subject::system(),
                organization_id,
                draft: draft("Ancienneté"),
            })
            .await
            .unwrap()
            .unwrap();
        let archived = usecase
            .archive_leave_account(ArchiveLeaveAccountCommand {
                actor: authz::Subject::system(),
                organization_id,
                id: created.id,
                archived: true,
            })
            .await
            .unwrap();
        let foreign = usecase
            .archive_leave_account(ArchiveLeaveAccountCommand {
                actor: authz::Subject::system(),
                organization_id: other_id,
                id: created.id,
                archived: false,
            })
            .await
            .unwrap_err();
        let listed = usecase
            .list_leave_accounts(list(organization_id))
            .await
            .unwrap();
        let restored = usecase
            .archive_leave_account(ArchiveLeaveAccountCommand {
                actor: authz::Subject::system(),
                organization_id,
                id: created.id,
                archived: false,
            })
            .await
            .unwrap();

        assert!(archived.archived_at.is_some());
        assert!(matches!(foreign, common::CoreError::NotFound));
        assert_eq!(listed.len(), 3);
        assert!(listed[2].is_archived());
        assert!(restored.archived_at.is_none());

        cleanup(&pool, organization_id, owner_id).await;
        cleanup(&pool, other_id, other_owner).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn accounts_are_validated_and_updated_in_place() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());

        let created = usecase
            .create_leave_account(CreateLeaveAccountCommand {
                actor: authz::Subject::system(),
                organization_id,
                draft: draft("Ancienneté"),
            })
            .await
            .unwrap()
            .unwrap();
        let updated = usecase
            .update_leave_account(UpdateLeaveAccountCommand {
                actor: authz::Subject::system(),
                organization_id,
                id: created.id,
                draft: LeaveAccountDraft {
                    acquisition: AcquisitionDraft::Monthly { per_month: 0.25 },
                    period_start_month: 9,
                    ..draft("Ancienneté bis")
                },
            })
            .await
            .unwrap()
            .unwrap();
        let refused = usecase
            .update_leave_account(UpdateLeaveAccountCommand {
                actor: authz::Subject::system(),
                organization_id,
                id: created.id,
                draft: LeaveAccountDraft {
                    period_start_month: 13,
                    ..draft("Ancienneté")
                },
            })
            .await
            .unwrap();

        assert_eq!(updated.name.as_str(), "Ancienneté bis");
        assert_eq!(updated.acquisition.kind_str(), "MONTHLY");
        assert_eq!(updated.acquisition.amount().days().hundredths(), 25);
        assert_eq!(updated.period_start_month.get(), 9);
        assert_eq!(refused, Err(LeaveAccountError::PeriodStartMonthOutOfRange));

        cleanup(&pool, organization_id, owner_id).await;
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn spec_rh_4_4_an_archived_account_cannot_be_picked_by_a_type_but_stays_on_it() {
        let pool = dev_pool().await;
        let (organization_id, owner_id) = seed_organization(&pool).await;
        let usecase = make_usecase(pool.clone());

        let account = usecase
            .create_leave_account(CreateLeaveAccountCommand {
                actor: authz::Subject::system(),
                organization_id,
                draft: draft("Ancienneté"),
            })
            .await
            .unwrap()
            .unwrap();
        let linked = usecase
            .create_absence_type(CreateAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id,
                draft: type_draft("Ancienneté", Some(account.id.0)),
            })
            .await
            .unwrap()
            .unwrap();
        usecase
            .archive_leave_account(ArchiveLeaveAccountCommand {
                actor: authz::Subject::system(),
                organization_id,
                id: account.id,
                archived: true,
            })
            .await
            .unwrap();

        let picked = usecase
            .create_absence_type(CreateAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id,
                draft: type_draft("Autre", Some(account.id.0)),
            })
            .await
            .unwrap();
        let kept = usecase
            .update_absence_type(UpdateAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id,
                id: linked.id,
                draft: type_draft("Ancienneté bis", Some(account.id.0)),
            })
            .await
            .unwrap()
            .unwrap();
        let unknown = usecase
            .create_absence_type(CreateAbsenceTypeCommand {
                actor: authz::Subject::system(),
                organization_id,
                draft: type_draft("Fantôme", Some(generate_uuid_v7())),
            })
            .await
            .unwrap();

        assert_eq!(picked, Err(AbsenceTypeError::AccountArchived));
        assert_eq!(kept.account_id, Some(account.id.0));
        assert_eq!(unknown, Err(AbsenceTypeError::AccountUnknown));

        cleanup(&pool, organization_id, owner_id).await;
    }
}
