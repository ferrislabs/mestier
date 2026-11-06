use chrono::Utc;
use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::transactional;
use uuid::Uuid;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        folder::{
            CreateFolderCommand, FolderRefusal, FolderTree, UpdateFolderCommand, WorkflowFolder,
        },
        ports::WorkflowFolderRepository,
    },
};

pub(super) async fn ensure_folder_exists(
    folders: &mut impl WorkflowFolderRepository,
    org_id: OrganizationId,
    folder_id: Uuid,
) -> Result<(), CoreError> {
    folders
        .find_by_id(org_id, folder_id)
        .await?
        .ok_or(FolderRefusal::Missing)?;

    Ok(())
}

impl MestierUseCase {
    #[transactional(workflow_folder)]
    pub async fn create_workflow_folder(
        &self,
        command: CreateFolderCommand,
    ) -> Result<WorkflowFolder, CoreError> {
        let mut folders = workflow_folder_repository;
        folders.lock_tree(command.org_id).await?;
        let existing = folders.list_by_organization(command.org_id).await?;
        FolderTree::new(&existing).check_creation(command.parent_id)?;

        folders
            .insert(&WorkflowFolder {
                id: generate_uuid_v7(),
                org_id: command.org_id,
                parent_id: command.parent_id,
                name: command.name,
                created_at: Utc::now(),
            })
            .await
    }

    #[transactional(workflow_folder)]
    pub async fn list_workflow_folders(
        &self,
        org_id: OrganizationId,
    ) -> Result<Vec<WorkflowFolder>, CoreError> {
        let mut folders = workflow_folder_repository;
        folders.list_by_organization(org_id).await
    }

    #[transactional(workflow_folder)]
    pub async fn update_workflow_folder(
        &self,
        command: UpdateFolderCommand,
    ) -> Result<WorkflowFolder, CoreError> {
        let mut folders = workflow_folder_repository;
        let current = folders
            .find_by_id(command.org_id, command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let moves_to = command
            .parent_id
            .filter(|target| *target != current.parent_id);
        if let Some(target) = moves_to {
            folders.lock_tree(command.org_id).await?;
            let tree = folders.list_by_organization(command.org_id).await?;
            FolderTree::new(&tree).check_move(command.id, target)?;
        }

        folders
            .update(command.org_id, command.id, command.name.as_ref(), moves_to)
            .await
    }

    #[transactional(workflow_folder)]
    pub async fn delete_workflow_folder(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<(), CoreError> {
        let mut folders = workflow_folder_repository;
        folders.delete(org_id, id).await
    }
}

#[cfg(test)]
mod tests {
    use sqlx::PgPool;

    use super::*;
    use crate::application::default_authorizer;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::folder::FolderName;
    use crate::domain::automation::workflow::{
        CreateWorkflowCommand, UpdateWorkflowCommand, WorkflowInitialState,
    };
    use crate::infrastructure::realtime::EventHub;

    async fn seed_organization(pool: &PgPool, label: &str) -> OrganizationId {
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
            format!("{label} Org"),
            format!("{label}-{org_id}"),
            owner_id,
        )
        .execute(pool)
        .await
        .unwrap();

        OrganizationId(org_id)
    }

    fn use_case(pool: PgPool) -> MestierUseCase {
        MestierUseCase::new(pool, default_authorizer(), EventHub::new())
    }

    fn name(raw: &str) -> FolderName {
        FolderName::try_from(raw.to_owned()).unwrap()
    }

    async fn create(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        parent_id: Option<Uuid>,
        label: &str,
    ) -> WorkflowFolder {
        usecase
            .create_workflow_folder(CreateFolderCommand {
                org_id,
                name: name(label),
                parent_id,
            })
            .await
            .unwrap()
    }

    fn move_to(org_id: OrganizationId, id: Uuid, parent_id: Option<Uuid>) -> UpdateFolderCommand {
        UpdateFolderCommand {
            org_id,
            id,
            name: None,
            parent_id: Some(parent_id),
        }
    }

    fn refusal_of(error: CoreError) -> Option<FolderRefusal> {
        match error {
            CoreError::Conflict(message) => FolderRefusal::from_conflict(&message),
            _ => None,
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn there_is_no_default_folder() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-none").await;
        let usecase = use_case(pool);

        assert!(
            usecase
                .list_workflow_folders(org_id)
                .await
                .unwrap()
                .is_empty()
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_folder_is_created_at_the_root_or_under_an_existing_parent() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-create").await;
        let usecase = use_case(pool);

        let root = create(&usecase, org_id, None, "Facturation").await;
        let child = create(&usecase, org_id, Some(root.id), "Relances").await;

        assert_eq!(root.parent_id, None);
        assert_eq!(child.parent_id, Some(root.id));
        assert_eq!(
            usecase.list_workflow_folders(org_id).await.unwrap().len(),
            2
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_under_a_missing_or_foreign_parent_is_refused() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-parent").await;
        let stranger = seed_organization(&pool, "folder-parent-stranger").await;
        let usecase = use_case(pool);
        let foreign = create(&usecase, stranger, None, "Theirs").await;

        for parent_id in [generate_uuid_v7(), foreign.id] {
            let error = usecase
                .create_workflow_folder(CreateFolderCommand {
                    org_id,
                    name: name("Orphan"),
                    parent_id: Some(parent_id),
                })
                .await
                .expect_err("the parent is not in this organization");

            assert_eq!(refusal_of(error), Some(FolderRefusal::Missing));
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_duplicate_name_at_the_same_level_is_refused() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-duplicate").await;
        let usecase = use_case(pool);
        create(&usecase, org_id, None, "Facturation").await;

        let error = usecase
            .create_workflow_folder(CreateFolderCommand {
                org_id,
                name: name("facturation"),
                parent_id: None,
            })
            .await
            .expect_err("same name, same level");

        assert_eq!(refusal_of(error), Some(FolderRefusal::NameTaken));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_folder_can_be_renamed_and_moved_back_to_the_root() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-move").await;
        let usecase = use_case(pool);
        let root = create(&usecase, org_id, None, "Root").await;
        let child = create(&usecase, org_id, Some(root.id), "Child").await;

        let moved = usecase
            .update_workflow_folder(UpdateFolderCommand {
                org_id,
                id: child.id,
                name: Some(name("Renamed")),
                parent_id: Some(None),
            })
            .await
            .unwrap();

        assert_eq!(moved.name.as_str(), "Renamed");
        assert_eq!(moved.parent_id, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn leaving_the_parent_out_of_an_update_keeps_it() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-keep-parent").await;
        let usecase = use_case(pool);
        let root = create(&usecase, org_id, None, "Root").await;
        let child = create(&usecase, org_id, Some(root.id), "Child").await;

        let renamed = usecase
            .update_workflow_folder(UpdateFolderCommand {
                org_id,
                id: child.id,
                name: Some(name("Renamed")),
                parent_id: None,
            })
            .await
            .unwrap();

        assert_eq!(renamed.parent_id, Some(root.id));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn moving_a_folder_into_itself_or_a_descendant_is_refused() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-cycle").await;
        let usecase = use_case(pool);
        let root = create(&usecase, org_id, None, "Root").await;
        let child = create(&usecase, org_id, Some(root.id), "Child").await;
        let grandchild = create(&usecase, org_id, Some(child.id), "Grandchild").await;

        for target in [root.id, child.id, grandchild.id] {
            let error = usecase
                .update_workflow_folder(move_to(org_id, root.id, Some(target)))
                .await
                .expect_err("that would close a loop");

            assert_eq!(refusal_of(error), Some(FolderRefusal::WouldCycle));
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn moving_under_a_missing_parent_is_refused_and_a_missing_folder_is_not_found() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-move-missing").await;
        let usecase = use_case(pool);
        let folder = create(&usecase, org_id, None, "Folder").await;

        let no_parent = usecase
            .update_workflow_folder(move_to(org_id, folder.id, Some(generate_uuid_v7())))
            .await
            .expect_err("no such parent");
        let no_folder = usecase
            .update_workflow_folder(move_to(org_id, generate_uuid_v7(), None))
            .await
            .expect_err("no such folder");

        assert_eq!(refusal_of(no_parent), Some(FolderRefusal::Missing));
        assert!(matches!(no_folder, CoreError::NotFound));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn another_organizations_folder_cannot_be_updated_or_used_as_a_parent() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-tenant").await;
        let stranger = seed_organization(&pool, "folder-tenant-stranger").await;
        let usecase = use_case(pool);
        let mine = create(&usecase, org_id, None, "Mine").await;
        let theirs = create(&usecase, stranger, None, "Theirs").await;

        let updated = usecase
            .update_workflow_folder(move_to(org_id, theirs.id, None))
            .await
            .expect_err("not mine");
        let adopted = usecase
            .update_workflow_folder(move_to(org_id, mine.id, Some(theirs.id)))
            .await
            .expect_err("not mine either");

        assert!(matches!(updated, CoreError::NotFound));
        assert_eq!(refusal_of(adopted), Some(FolderRefusal::Missing));
    }

    async fn chain_of_folders(
        usecase: &MestierUseCase,
        org_id: OrganizationId,
        length: usize,
        prefix: &str,
    ) -> Vec<WorkflowFolder> {
        let mut chain: Vec<WorkflowFolder> = Vec::with_capacity(length);
        for level in 0..length {
            let parent_id = level.checked_sub(1).map(|previous| chain[previous].id);
            chain.push(create(usecase, org_id, parent_id, &format!("{prefix}{level}")).await);
        }
        chain
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_organization_cannot_hold_more_than_the_maximum_number_of_folders() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-cap").await;
        let other_org = seed_organization(&pool, "folder-cap-other").await;
        sqlx::query(
            "INSERT INTO automation.workflow_folder (id, org_id, name)
             SELECT gen_random_uuid(), $1, 'bulk-' || n FROM generate_series(1, $2) AS n",
        )
        .bind(org_id.0)
        .bind((crate::domain::automation::folder::MAX_FOLDERS_PER_ORGANIZATION - 1) as i32)
        .execute(&pool)
        .await
        .unwrap();
        let usecase = use_case(pool);

        let last_allowed = create(&usecase, org_id, None, "The 500th").await;
        let error = usecase
            .create_workflow_folder(CreateFolderCommand {
                org_id,
                name: name("The 501st"),
                parent_id: Some(last_allowed.id),
            })
            .await
            .expect_err("the cap is reached");
        let elsewhere = create(&usecase, other_org, None, "Unaffected").await;

        assert_eq!(refusal_of(error), Some(FolderRefusal::TooMany));
        assert_eq!(elsewhere.parent_id, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn folders_nest_at_most_eight_levels_deep() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-depth").await;
        let usecase = use_case(pool);
        let chain = chain_of_folders(
            &usecase,
            org_id,
            crate::domain::automation::folder::MAX_FOLDER_DEPTH,
            "level-",
        )
        .await;

        let error = usecase
            .create_workflow_folder(CreateFolderCommand {
                org_id,
                name: name("Too deep"),
                parent_id: Some(chain[chain.len() - 1].id),
            })
            .await
            .expect_err("a ninth level is refused");
        let sibling = create(&usecase, org_id, Some(chain[chain.len() - 2].id), "Sibling").await;

        assert_eq!(refusal_of(error), Some(FolderRefusal::TooDeep));
        assert_eq!(sibling.parent_id, Some(chain[chain.len() - 2].id));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_move_is_refused_when_the_moved_subtree_would_pass_the_eighth_level() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-move-depth").await;
        let usecase = use_case(pool);
        let target = chain_of_folders(&usecase, org_id, 5, "target-").await;
        let subtree = chain_of_folders(&usecase, org_id, 4, "subtree-").await;

        let refused = usecase
            .update_workflow_folder(move_to(org_id, subtree[0].id, Some(target[4].id)))
            .await
            .expect_err("5 + 4 levels do not fit in 8");
        let accepted = usecase
            .update_workflow_folder(move_to(org_id, subtree[0].id, Some(target[3].id)))
            .await
            .unwrap();

        assert_eq!(refusal_of(refused), Some(FolderRefusal::TooDeep));
        assert_eq!(accepted.parent_id, Some(target[3].id));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_rename_never_waits_for_the_tree_lock_but_a_move_does() {
        use crate::domain::automation::ports::WorkflowFolderRepository;
        use crate::infrastructure::automation::postgres::PgWorkflowFolderRepository;
        use crate::infrastructure::postgres::with_tx;

        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-lock-scope").await;
        let usecase = use_case(pool.clone());
        let root = create(&usecase, org_id, None, "Root").await;
        let child = create(&usecase, org_id, Some(root.id), "Child").await;
        let (locked_tx, locked_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();
        let holder = tokio::spawn({
            let pool = pool.clone();
            async move {
                with_tx(&pool, async |tx| {
                    let mut repo = PgWorkflowFolderRepository::new(&tx);
                    repo.lock_tree(org_id).await?;
                    locked_tx.send(()).ok();
                    release_rx.await.ok();
                    Ok::<_, CoreError>(())
                })
                .await
            }
        });
        locked_rx.await.unwrap();

        let renamed = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            usecase.update_workflow_folder(UpdateFolderCommand {
                org_id,
                id: child.id,
                name: Some(name("Renamed")),
                parent_id: None,
            }),
        )
        .await
        .expect("a rename must not wait for the tree lock")
        .unwrap();
        let same_parent = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            usecase.update_workflow_folder(UpdateFolderCommand {
                org_id,
                id: child.id,
                name: Some(name("Renamed again")),
                parent_id: Some(Some(root.id)),
            }),
        )
        .await
        .expect("re-stating the current parent is not a move")
        .unwrap();
        let mut moving = Box::pin(usecase.update_workflow_folder(move_to(org_id, child.id, None)));
        let waiting =
            tokio::time::timeout(std::time::Duration::from_millis(300), &mut moving).await;

        assert_eq!(renamed.name.as_str(), "Renamed");
        assert_eq!(same_parent.name.as_str(), "Renamed again");
        assert!(waiting.is_err(), "a move must wait for the tree lock");
        release_tx.send(()).unwrap();
        holder.await.unwrap().unwrap();
        let moved = tokio::time::timeout(std::time::Duration::from_secs(5), moving)
            .await
            .expect("the move proceeds once the lock is released")
            .unwrap();
        assert_eq!(moved.parent_id, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn two_opposite_moves_at_once_cannot_close_a_loop() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-race").await;
        let usecase = use_case(pool);
        let a = create(&usecase, org_id, None, "A").await;
        let b = create(&usecase, org_id, None, "B").await;

        let (a_under_b, b_under_a) = tokio::join!(
            usecase.update_workflow_folder(move_to(org_id, a.id, Some(b.id))),
            usecase.update_workflow_folder(move_to(org_id, b.id, Some(a.id))),
        );

        let outcomes = [a_under_b, b_under_a];
        let succeeded = outcomes.iter().filter(|outcome| outcome.is_ok()).count();
        let refused = outcomes
            .into_iter()
            .filter_map(Result::err)
            .map(refusal_of)
            .collect::<Vec<_>>();
        assert_eq!(succeeded, 1);
        assert_eq!(refused, vec![Some(FolderRefusal::WouldCycle)]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_an_empty_folder_succeeds_and_a_missing_one_is_not_found() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-delete").await;
        let usecase = use_case(pool);
        let folder = create(&usecase, org_id, None, "Empty").await;

        usecase
            .delete_workflow_folder(org_id, folder.id)
            .await
            .unwrap();
        let again = usecase
            .delete_workflow_folder(org_id, folder.id)
            .await
            .expect_err("already gone");

        assert!(
            usecase
                .list_workflow_folders(org_id)
                .await
                .unwrap()
                .is_empty()
        );
        assert!(matches!(again, CoreError::NotFound));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_folder_with_a_sub_folder_or_a_workflow_cannot_be_deleted_until_emptied() {
        let pool = automation_pool().await;
        let org_id = seed_organization(&pool, "folder-not-empty").await;
        let usecase = use_case(pool);
        let parent = create(&usecase, org_id, None, "Parent").await;
        let child = create(&usecase, org_id, Some(parent.id), "Child").await;
        let workflow = usecase
            .create_workflow_in_state(
                CreateWorkflowCommand {
                    org_id,
                    name: "Filed".to_owned(),
                    description: None,
                },
                WorkflowInitialState {
                    folder_id: Some(child.id),
                    ..WorkflowInitialState::default()
                },
            )
            .await
            .unwrap();

        let with_child = usecase
            .delete_workflow_folder(org_id, parent.id)
            .await
            .expect_err("holds a sub-folder");
        let with_workflow = usecase
            .delete_workflow_folder(org_id, child.id)
            .await
            .expect_err("holds a workflow");
        assert_eq!(refusal_of(with_child), Some(FolderRefusal::NotEmpty));
        assert_eq!(refusal_of(with_workflow), Some(FolderRefusal::NotEmpty));

        usecase
            .update_workflow(UpdateWorkflowCommand {
                org_id,
                id: workflow.id,
                name: None,
                description: None,
                enabled: None,
                folder_id: Some(None),
                tags: None,
                archived: None,
            })
            .await
            .unwrap();
        usecase
            .delete_workflow_folder(org_id, child.id)
            .await
            .unwrap();
        usecase
            .delete_workflow_folder(org_id, parent.id)
            .await
            .unwrap();
    }
}
