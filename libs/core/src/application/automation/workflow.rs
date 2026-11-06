use chrono::Utc;
use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::transactional;
use uuid::Uuid;

use crate::{
    application::MestierUseCase,
    application::automation::folder::ensure_folder_exists,
    domain::automation::{
        catalogue::event_catalogue,
        connector::connector_catalogue,
        ports::{
            CollectionRepository, CredentialRepository, SubscriptionRepository, WorkflowRepository,
        },
        workflow::{
            ArchiveScope, CreateWorkflowCommand, GraphError, SaveWorkflowVersionCommand,
            UpdateWorkflowCommand, Workflow, WorkflowDescription, WorkflowInitialState,
            WorkflowName, WorkflowVersion, validate_graph,
        },
    },
};

/// One line per [`GraphError`], joined for [`CoreError::Conflict`]. The
/// structure (which connector, which field) survives inside each message —
/// see `GraphError`'s `Display` impls — even though the wire type here is a
/// single string; a handler wanting the errors apart can still split on the
/// separator, and no handler exists yet in this ticket's scope to need more.
fn format_graph_errors(errors: &[GraphError]) -> String {
    errors
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}

impl MestierUseCase {
    pub async fn create_workflow(
        &self,
        command: CreateWorkflowCommand,
    ) -> Result<Workflow, CoreError> {
        self.create_workflow_in_state(command, WorkflowInitialState::default())
            .await
    }

    #[transactional(workflow, workflow_folder)]
    pub async fn create_workflow_in_state(
        &self,
        command: CreateWorkflowCommand,
        initial_state: WorkflowInitialState,
    ) -> Result<Workflow, CoreError> {
        let mut repository = workflow_repository;
        let mut folders = workflow_folder_repository;
        let name = WorkflowName::try_from(command.name)
            .map_err(|error| CoreError::Conflict(error.to_string()))?;
        let description = command
            .description
            .map(WorkflowDescription::try_from)
            .transpose()
            .map_err(|error| CoreError::Conflict(error.to_string()))?;
        if let Some(folder_id) = initial_state.folder_id {
            ensure_folder_exists(&mut folders, command.org_id, folder_id).await?;
        }

        let now = Utc::now();
        let workflow = Workflow {
            id: generate_uuid_v7(),
            org_id: command.org_id,
            name: name.into_string(),
            description: description.map(WorkflowDescription::into_string),
            enabled: initial_state.enabled,
            current_version_id: None,
            layout: None,
            folder_id: initial_state.folder_id,
            tags: initial_state.tags,
            archived_at: None,
            created_at: now,
            updated_at: now,
        };

        repository.insert(&workflow).await
    }

    #[transactional(workflow)]
    pub async fn find_workflow(
        &self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<Workflow>, CoreError> {
        let mut repository = workflow_repository;
        repository.find_by_id(org_id, id).await
    }

    #[transactional(workflow)]
    pub async fn list_workflows(&self, org_id: OrganizationId) -> Result<Vec<Workflow>, CoreError> {
        let mut repository = workflow_repository;
        repository
            .list_by_organization(org_id, ArchiveScope::All)
            .await
    }

    #[transactional(workflow)]
    pub async fn list_workflows_by_scope(
        &self,
        org_id: OrganizationId,
        scope: ArchiveScope,
    ) -> Result<Vec<Workflow>, CoreError> {
        let mut repository = workflow_repository;
        repository.list_by_organization(org_id, scope).await
    }

    #[transactional(workflow, workflow_folder)]
    pub async fn update_workflow(
        &self,
        command: UpdateWorkflowCommand,
    ) -> Result<Workflow, CoreError> {
        let mut repository = workflow_repository;
        let mut folders = workflow_folder_repository;
        let existing = repository
            .find_by_id_for_update(command.org_id, command.id)
            .await?
            .ok_or(CoreError::NotFound)?;
        if let Some(Some(folder_id)) = command.folder_id {
            ensure_folder_exists(&mut folders, command.org_id, folder_id).await?;
        }

        let org_id = command.org_id;
        let updated = existing.apply(command, Utc::now())?;

        repository.update(org_id, &updated).await
    }

    #[transactional(workflow)]
    pub async fn delete_workflow(&self, org_id: OrganizationId, id: Uuid) -> Result<(), CoreError> {
        let mut repository = workflow_repository;
        repository
            .find_by_id(org_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;

        repository.delete(org_id, id).await
    }

    /// Validates the graph against the connector catalogue and the
    /// organization's own credentials, then — only if it is valid —
    /// persists it as a new immutable version and moves
    /// `current_version_id` to it, all inside one transaction (#199). A run
    /// (#200) reading the previous version therefore never observes a
    /// half-written one.
    #[transactional(workflow, credential, subscription, collection)]
    pub async fn save_workflow_version(
        &self,
        command: SaveWorkflowVersionCommand,
    ) -> Result<WorkflowVersion, CoreError> {
        let mut workflows = workflow_repository;
        let mut credentials = credential_repository;
        let mut subscriptions = subscription_repository;
        let mut collections = collection_repository;

        workflows
            .find_by_id(command.org_id, command.workflow_id)
            .await?
            .ok_or(CoreError::NotFound)?
            .ensure_not_archived()?;

        let catalogue = connector_catalogue();
        let events = event_catalogue();
        let organization_credentials = credentials.list_by_organization(command.org_id).await?;
        let referenced_collections = collections
            .find_many(command.org_id, &command.graph.referenced_collection_ids())
            .await?;

        validate_graph(
            &command.graph,
            &catalogue,
            &organization_credentials,
            &events,
            &referenced_collections,
        )
        .map_err(|errors| CoreError::Conflict(format_graph_errors(&errors)))?;

        let version = workflows
            .insert_version(
                command.org_id,
                command.workflow_id,
                &command.graph,
                command.created_by,
            )
            .await?;

        subscriptions
            .replace_workflow_subscriptions(
                command.org_id,
                command.workflow_id,
                &command.graph.triggers,
            )
            .await?;

        if let Some(layout) = command.layout.as_ref() {
            workflows
                .set_layout(command.org_id, command.workflow_id, Some(layout))
                .await?;
        }

        Ok(version)
    }

    #[transactional(workflow)]
    pub async fn find_workflow_version(
        &self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        version: i32,
    ) -> Result<Option<WorkflowVersion>, CoreError> {
        let mut repository = workflow_repository;
        repository.find_version(org_id, workflow_id, version).await
    }

    #[transactional(workflow)]
    pub async fn find_workflow_version_by_id(
        &self,
        org_id: OrganizationId,
        workflow_version_id: Uuid,
    ) -> Result<Option<WorkflowVersion>, CoreError> {
        let mut repository = workflow_repository;
        repository
            .find_version_by_id(org_id, workflow_version_id)
            .await
    }

    #[transactional(workflow)]
    pub async fn list_workflow_versions(
        &self,
        org_id: OrganizationId,
        workflow_id: Uuid,
    ) -> Result<Vec<WorkflowVersion>, CoreError> {
        let mut repository = workflow_repository;
        repository.list_versions(org_id, workflow_id).await
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::default_authorizer;
    use crate::application::test_support::automation_pool;
    use crate::domain::automation::folder::{CreateFolderCommand, FolderName, FolderRefusal};
    use crate::domain::automation::workflow::{
        Edge, Graph, PlacedConnector, PlacedTrigger, TriggerKind, WorkflowRefusal, WorkflowTags,
    };
    use crate::infrastructure::automation::webhook::secret::SecretCipher;
    use crate::infrastructure::realtime::EventHub;

    async fn make_pool() -> PgPool {
        automation_pool().await
    }

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

    /// A credential is sealed with a cipher, so only the one test that
    /// creates a real credential (to prove a cross-organization reference is
    /// refused) needs this; every other test's `use_case` has none.
    fn use_case_with_cipher(pool: PgPool) -> MestierUseCase {
        use_case(pool).with_cipher(Some(Arc::new(SecretCipher::new(&[7u8; 32]).unwrap())))
    }

    fn valid_graph() -> Graph {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), serde_json::json!("{{ true }}"));
        Graph {
            connectors: vec![PlacedConnector {
                id: "c1".to_string(),
                kind: "flow.condition".to_string(),
                version: 1,
                credential_id: None,
                config,
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }],
            edges: vec![Edge {
                from: "t1".to_string(),
                to: "c1".to_string(),
                branch: None,
            }],
            triggers: vec![PlacedTrigger {
                id: "t1".to_string(),
                kind: TriggerKind::Manual,
                name: None,
                description: None,
            }],
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_a_workflow_starts_enabled_with_no_current_version() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "create").await;
        let usecase = use_case(pool);

        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Quote accepted to CRM".to_string(),
                description: Some("Creates the customer in the CRM".to_string()),
            })
            .await
            .unwrap();

        assert_eq!(created.org_id, org_id);
        assert!(created.enabled);
        assert_eq!(created.current_version_id, None);

        let found = usecase.find_workflow(org_id, created.id).await.unwrap();
        assert_eq!(found, Some(created));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn finding_a_workflow_from_another_organization_reads_back_absent() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "find-owner").await;
        let stranger_org = seed_organization(&pool, "find-stranger").await;
        let usecase = use_case(pool);

        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Mine".to_string(),
                description: None,
            })
            .await
            .unwrap();

        let found = usecase
            .find_workflow(stranger_org, created.id)
            .await
            .unwrap();

        assert_eq!(found, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn listing_only_returns_the_requesting_organizations_workflows() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "list").await;
        let other_org = seed_organization(&pool, "list-other").await;
        let usecase = use_case(pool);

        usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Mine".to_string(),
                description: None,
            })
            .await
            .unwrap();
        usecase
            .create_workflow(CreateWorkflowCommand {
                org_id: other_org,
                name: "Not mine".to_string(),
                description: None,
            })
            .await
            .unwrap();

        let listed = usecase.list_workflows(org_id).await.unwrap();

        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "Mine");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_can_rename_disable_and_clear_the_description() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "update").await;
        let usecase = use_case(pool);

        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Old name".to_string(),
                description: Some("Old description".to_string()),
            })
            .await
            .unwrap();

        let updated = usecase
            .update_workflow(UpdateWorkflowCommand {
                org_id,
                id: created.id,
                name: Some(WorkflowName::try_from("New name".to_string()).unwrap()),
                description: Some(None),
                enabled: Some(false),
                folder_id: None,
                tags: None,
                archived: None,
            })
            .await
            .unwrap();

        assert_eq!(updated.name, "New name");
        assert_eq!(updated.description, None);
        assert!(!updated.enabled);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_a_missing_workflow_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "update-missing").await;
        let usecase = use_case(pool);

        let error = usecase
            .update_workflow(UpdateWorkflowCommand {
                org_id,
                id: generate_uuid_v7(),
                name: Some(WorkflowName::try_from("Nope".to_string()).unwrap()),
                description: None,
                enabled: None,
                folder_id: None,
                tags: None,
                archived: None,
            })
            .await
            .expect_err("there is nothing to update");

        assert!(matches!(error, CoreError::NotFound));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_workflow_removes_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete").await;
        let usecase = use_case(pool);

        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Name".to_string(),
                description: None,
            })
            .await
            .unwrap();

        usecase.delete_workflow(org_id, created.id).await.unwrap();

        assert_eq!(
            usecase.find_workflow(org_id, created.id).await.unwrap(),
            None
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_a_missing_workflow_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete-missing").await;
        let usecase = use_case(pool);

        let error = usecase
            .delete_workflow(org_id, generate_uuid_v7())
            .await
            .expect_err("there is nothing to delete");

        assert!(matches!(error, CoreError::NotFound));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_valid_graph_creates_a_version_and_moves_current() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "save-version").await;
        let usecase = use_case(pool);

        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Name".to_string(),
                description: None,
            })
            .await
            .unwrap();

        let version = usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph: valid_graph(),
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();

        assert_eq!(version.version, 1);
        assert_eq!(version.graph, valid_graph());

        let reloaded = usecase
            .find_workflow(org_id, created.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(reloaded.current_version_id, Some(version.id));
    }

    async fn subscribed_trigger_ids(pool: &PgPool, workflow_id: Uuid) -> Vec<String> {
        let mut ids: Vec<String> = sqlx::query_scalar!(
            r#"SELECT trigger_id AS "trigger_id!" FROM automation.subscription
               WHERE kind = 'workflow' AND target_id = $1"#,
            workflow_id,
        )
        .fetch_all(pool)
        .await
        .unwrap();
        ids.sort();
        ids
    }

    fn graph_with_two_event_triggers() -> Graph {
        let mut config = serde_json::Map::new();
        config.insert("predicate".to_string(), serde_json::json!("{{ true }}"));
        Graph {
            connectors: vec![
                PlacedConnector {
                    id: "c1".to_string(),
                    kind: "flow.condition".to_string(),
                    version: 1,
                    credential_id: None,
                    config: config.clone(),
                    name: None,
                    description: None,
                    disabled: false,
                    operation: None,
                },
                PlacedConnector {
                    id: "c2".to_string(),
                    kind: "flow.condition".to_string(),
                    version: 1,
                    credential_id: None,
                    config,
                    name: None,
                    description: None,
                    disabled: false,
                    operation: None,
                },
            ],
            edges: vec![
                Edge {
                    from: "t1".to_string(),
                    to: "c1".to_string(),
                    branch: None,
                },
                Edge {
                    from: "t2".to_string(),
                    to: "c2".to_string(),
                    branch: None,
                },
            ],
            triggers: vec![
                PlacedTrigger {
                    id: "t1".to_string(),
                    kind: TriggerKind::Events(vec!["quote.accepted".to_string()]),
                    name: None,
                    description: None,
                },
                PlacedTrigger {
                    id: "t2".to_string(),
                    kind: TriggerKind::Events(vec!["quote.declined".to_string()]),
                    name: None,
                    description: None,
                },
            ],
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_version_with_two_event_triggers_writes_two_subscription_rows() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "subs-two-triggers").await;
        let usecase = use_case(pool.clone());
        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Name".to_string(),
                description: None,
            })
            .await
            .unwrap();

        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph: graph_with_two_event_triggers(),
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();

        assert_eq!(
            subscribed_trigger_ids(&pool, created.id).await,
            vec!["t1".to_string(), "t2".to_string()]
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn removing_a_trigger_and_saving_removes_its_subscription_row() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "subs-remove-trigger").await;
        let usecase = use_case(pool.clone());
        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Name".to_string(),
                description: None,
            })
            .await
            .unwrap();
        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph: graph_with_two_event_triggers(),
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();

        let mut narrowed = graph_with_two_event_triggers();
        narrowed.connectors.remove(1);
        narrowed.edges.remove(1);
        narrowed.triggers.remove(1);
        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph: narrowed,
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();

        assert_eq!(
            subscribed_trigger_ids(&pool, created.id).await,
            vec!["t1".to_string()],
            "t2's row must be gone in the same transaction that dropped it from the graph"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_manual_only_graph_writes_no_subscription_row() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "subs-manual-only").await;
        let usecase = use_case(pool.clone());
        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Name".to_string(),
                description: None,
            })
            .await
            .unwrap();

        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph: valid_graph(),
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();

        assert!(subscribed_trigger_ids(&pool, created.id).await.is_empty());
    }

    /// The acceptance criterion at the use-case boundary: saving a second
    /// edit does not touch the first version at all.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_second_edit_leaves_the_previous_version_identical() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "second-edit").await;
        let usecase = use_case(pool);

        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Name".to_string(),
                description: None,
            })
            .await
            .unwrap();

        let version1 = usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph: valid_graph(),
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();

        let mut second_graph = valid_graph();
        second_graph.connectors[0]
            .config
            .insert("predicate".to_string(), serde_json::json!("{{ false }}"));
        let version2 = usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph: second_graph,
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();

        assert_eq!(version2.version, 2);

        let reread_version1 = usecase
            .find_workflow_version(org_id, created.id, 1)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(reread_version1, version1);

        let versions = usecase
            .list_workflow_versions(org_id, created.id)
            .await
            .unwrap();
        assert_eq!(versions, vec![version1, version2]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_version_for_a_missing_workflow_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "save-missing").await;
        let usecase = use_case(pool);

        let error = usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: generate_uuid_v7(),
                graph: valid_graph(),
                layout: None,
                created_by: None,
            })
            .await
            .expect_err("there is no workflow to version");

        assert!(matches!(error, CoreError::NotFound));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_an_invalid_graph_is_refused_and_names_the_connector() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "save-invalid").await;
        let usecase = use_case(pool);

        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Name".to_string(),
                description: None,
            })
            .await
            .unwrap();

        let invalid_graph = Graph {
            connectors: vec![PlacedConnector {
                id: "c1".to_string(),
                kind: "not.a.real.kind".to_string(),
                version: 1,
                credential_id: None,
                config: serde_json::Map::new(),
                name: None,
                description: None,
                disabled: false,
                operation: None,
            }],
            edges: Vec::new(),
            triggers: Vec::new(),
        };

        let error = usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph: invalid_graph,
                layout: None,
                created_by: None,
            })
            .await
            .expect_err("an unknown connector kind is refused");

        assert!(matches!(error, CoreError::Conflict(_)));
        assert!(format!("{error}").contains("c1"), "{error}");
        assert!(format!("{error}").contains("not.a.real.kind"), "{error}");

        // Nothing was persisted: the workflow still has no current version.
        let reloaded = usecase
            .find_workflow(org_id, created.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(reloaded.current_version_id, None);
    }

    /// A graph referencing a credential from another organization is
    /// refused the same way as one referencing no credential at all: the
    /// list handed to `validate_graph` is already scoped to the caller's
    /// organization, so a stranger's credential is simply not in it.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_graph_referencing_a_credential_from_another_organization_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "save-cross-org-cred").await;
        let stranger_org = seed_organization(&pool, "save-cross-org-stranger").await;
        let usecase = use_case_with_cipher(pool);

        let (stranger_credential, _secret) = usecase
            .create_credential(
                crate::domain::automation::credential::CreateCredentialCommand {
                    org_id: stranger_org,
                    kind: "bearer_token".to_string(),
                    name: "Stranger token".to_string(),
                    origin: crate::domain::automation::credential::CredentialOrigin::Supplied,
                    data: Some(serde_json::json!({ "token": "abc" })),
                },
            )
            .await
            .unwrap();

        let created = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id,
                name: "Name".to_string(),
                description: None,
            })
            .await
            .unwrap();

        let mut graph = valid_graph();
        graph.connectors.push(PlacedConnector {
            id: "c2".to_string(),
            kind: "flow.condition".to_string(),
            version: 1,
            credential_id: Some(stranger_credential.id),
            config: {
                let mut m = serde_json::Map::new();
                m.insert("predicate".to_string(), serde_json::json!("{{ true }}"));
                m
            },
            name: None,
            description: None,
            disabled: false,
            operation: None,
        });
        let error = usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph,
                layout: None,
                created_by: None,
            })
            .await
            .expect_err("a stranger's credential must not validate");

        assert!(matches!(error, CoreError::Conflict(_)));
    }

    fn plain_command(org_id: OrganizationId, name: &str) -> CreateWorkflowCommand {
        CreateWorkflowCommand {
            org_id,
            name: name.to_string(),
            description: Some("Original description".to_string()),
        }
    }

    fn untouched(workflow: &Workflow) -> UpdateWorkflowCommand {
        UpdateWorkflowCommand {
            org_id: workflow.org_id,
            id: workflow.id,
            name: None,
            description: None,
            enabled: None,
            folder_id: None,
            tags: None,
            archived: None,
        }
    }

    fn untouched_for(org_id: OrganizationId, id: Uuid) -> UpdateWorkflowCommand {
        UpdateWorkflowCommand {
            org_id,
            id,
            name: None,
            description: None,
            enabled: None,
            folder_id: None,
            tags: None,
            archived: None,
        }
    }

    fn tags(raw: &[&str]) -> WorkflowTags {
        WorkflowTags::try_from(raw.iter().map(|tag| (*tag).to_owned()).collect::<Vec<_>>()).unwrap()
    }

    async fn create_folder(usecase: &MestierUseCase, org_id: OrganizationId, name: &str) -> Uuid {
        usecase
            .create_workflow_folder(CreateFolderCommand {
                org_id,
                name: FolderName::try_from(name.to_owned()).unwrap(),
                parent_id: None,
            })
            .await
            .unwrap()
            .id
    }

    fn is_archived_refusal(error: &CoreError) -> bool {
        matches!(error, CoreError::Conflict(message)
            if WorkflowRefusal::from_conflict(message) == Some(WorkflowRefusal::Archived))
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_can_be_created_disabled_in_a_folder_with_tags() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "create-state").await;
        let usecase = use_case(pool);
        let folder_id = create_folder(&usecase, org_id, "Facturation").await;

        let created = usecase
            .create_workflow_in_state(
                plain_command(org_id, "Draft"),
                WorkflowInitialState {
                    enabled: false,
                    folder_id: Some(folder_id),
                    tags: tags(&["CRM", "relance"]),
                },
            )
            .await
            .unwrap();

        assert!(!created.enabled);
        assert_eq!(created.folder_id, Some(folder_id));
        assert_eq!(created.tags, tags(&["CRM", "relance"]));
        assert_eq!(created.archived_at, None);
        assert_eq!(
            usecase.find_workflow(org_id, created.id).await.unwrap(),
            Some(created)
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_with_the_default_state_is_enabled_unfiled_and_untagged() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "create-default").await;
        let usecase = use_case(pool);

        let created = usecase
            .create_workflow(plain_command(org_id, "Plain"))
            .await
            .unwrap();

        assert!(created.enabled);
        assert_eq!(created.folder_id, None);
        assert!(created.tags.is_empty());
        assert_eq!(created.archived_at, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn creating_in_a_missing_or_foreign_folder_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "create-folder").await;
        let stranger = seed_organization(&pool, "create-folder-stranger").await;
        let usecase = use_case(pool);
        let foreign_folder = create_folder(&usecase, stranger, "Theirs").await;

        for folder_id in [generate_uuid_v7(), foreign_folder] {
            let error = usecase
                .create_workflow_in_state(
                    plain_command(org_id, "Misfiled"),
                    WorkflowInitialState {
                        folder_id: Some(folder_id),
                        ..WorkflowInitialState::default()
                    },
                )
                .await
                .expect_err("the folder is not in this organization");

            assert!(matches!(&error, CoreError::Conflict(m)
                if FolderRefusal::from_conflict(m) == Some(FolderRefusal::Missing)));
        }
        assert!(usecase.list_workflows(org_id).await.unwrap().is_empty());
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updated_at_moves_for_a_rename_or_a_new_description_and_for_nothing_else() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "updated-at").await;
        let usecase = use_case(pool);
        let folder_id = create_folder(&usecase, org_id, "Folder").await;
        let created = usecase
            .create_workflow(plain_command(org_id, "Name"))
            .await
            .unwrap();

        let reclassified = usecase
            .update_workflow(UpdateWorkflowCommand {
                enabled: Some(false),
                folder_id: Some(Some(folder_id)),
                tags: Some(tags(&["a", "b"])),
                ..untouched(&created)
            })
            .await
            .unwrap();
        let archived = usecase
            .update_workflow(UpdateWorkflowCommand {
                archived: Some(true),
                ..untouched(&created)
            })
            .await
            .unwrap();
        let unarchived = usecase
            .update_workflow(UpdateWorkflowCommand {
                archived: Some(false),
                ..untouched(&created)
            })
            .await
            .unwrap();
        let renamed = usecase
            .update_workflow(UpdateWorkflowCommand {
                name: Some(WorkflowName::try_from("Other".to_string()).unwrap()),
                ..untouched(&created)
            })
            .await
            .unwrap();
        let redescribed = usecase
            .update_workflow(UpdateWorkflowCommand {
                description: Some(None),
                ..untouched(&created)
            })
            .await
            .unwrap();

        assert_eq!(reclassified.updated_at, created.updated_at);
        assert_eq!(archived.updated_at, created.updated_at);
        assert_eq!(unarchived.updated_at, created.updated_at);
        assert!(renamed.updated_at > created.updated_at);
        assert!(redescribed.updated_at > renamed.updated_at);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_new_version_moves_updated_at() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "updated-at-version").await;
        let usecase = use_case(pool);
        let created = usecase
            .create_workflow(plain_command(org_id, "Name"))
            .await
            .unwrap();

        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph: valid_graph(),
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();

        let reloaded = usecase
            .find_workflow(org_id, created.id)
            .await
            .unwrap()
            .unwrap();
        assert!(reloaded.updated_at > created.updated_at);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn archiving_stamps_the_instant_disables_and_unarchiving_leaves_it_disabled() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "archive").await;
        let usecase = use_case(pool);
        let created = usecase
            .create_workflow(plain_command(org_id, "Name"))
            .await
            .unwrap();
        assert!(created.enabled);

        let archived = usecase
            .update_workflow(UpdateWorkflowCommand {
                archived: Some(true),
                ..untouched(&created)
            })
            .await
            .unwrap();
        assert!(archived.archived_at.is_some());
        assert!(!archived.enabled);

        let unarchived = usecase
            .update_workflow(UpdateWorkflowCommand {
                archived: Some(false),
                ..untouched(&created)
            })
            .await
            .unwrap();
        assert_eq!(unarchived.archived_at, None);
        assert!(!unarchived.enabled);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_archived_workflow_cannot_be_enabled_but_a_disabled_one_can() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "archive-enable").await;
        let usecase = use_case(pool);
        let created = usecase
            .create_workflow(plain_command(org_id, "Name"))
            .await
            .unwrap();
        usecase
            .update_workflow(UpdateWorkflowCommand {
                archived: Some(true),
                ..untouched(&created)
            })
            .await
            .unwrap();

        let refused = usecase
            .update_workflow(UpdateWorkflowCommand {
                enabled: Some(true),
                ..untouched(&created)
            })
            .await
            .expect_err("archived workflows stay disabled");
        assert!(is_archived_refusal(&refused));

        let restored = usecase
            .update_workflow(UpdateWorkflowCommand {
                archived: Some(false),
                enabled: Some(true),
                ..untouched(&created)
            })
            .await
            .unwrap();
        assert!(restored.enabled);
        assert_eq!(restored.archived_at, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_update_waits_for_a_concurrent_one_and_never_undoes_its_archive() {
        use crate::domain::automation::ports::WorkflowRepository;
        use crate::infrastructure::automation::postgres::PgWorkflowRepository;
        use crate::infrastructure::postgres::with_tx;

        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "update-race").await;
        let usecase = use_case(pool.clone());
        let created = usecase
            .create_workflow(plain_command(org_id, "Name"))
            .await
            .unwrap();
        let (locked_tx, locked_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel::<()>();
        let archiver = tokio::spawn({
            let pool = pool.clone();
            let id = created.id;
            async move {
                with_tx(&pool, async |tx| {
                    let mut repo = PgWorkflowRepository::new(&tx);
                    let current = repo
                        .find_by_id_for_update(org_id, id)
                        .await?
                        .expect("the workflow exists");
                    locked_tx.send(()).ok();
                    release_rx.await.ok();
                    let archived = current.apply(
                        UpdateWorkflowCommand {
                            archived: Some(true),
                            ..untouched_for(org_id, id)
                        },
                        chrono::Utc::now(),
                    )?;
                    repo.update(org_id, &archived).await
                })
                .await
            }
        });
        locked_rx.await.unwrap();

        let mut renaming = Box::pin(usecase.update_workflow(UpdateWorkflowCommand {
            name: Some(WorkflowName::try_from("Renamed".to_string()).unwrap()),
            ..untouched(&created)
        }));
        let waiting =
            tokio::time::timeout(std::time::Duration::from_millis(300), &mut renaming).await;

        assert!(
            waiting.is_err(),
            "the second update must wait for the first"
        );
        release_tx.send(()).unwrap();
        archiver.await.unwrap().unwrap();
        let renamed = tokio::time::timeout(std::time::Duration::from_secs(5), renaming)
            .await
            .expect("the waiting update proceeds once the first commits")
            .unwrap();
        assert_eq!(renamed.name, "Renamed");
        assert!(
            renamed.archived_at.is_some(),
            "the rename was computed from a stale copy and unarchived the workflow"
        );
        assert!(!renamed.enabled);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn listing_hides_archived_workflows_unless_asked_for_them() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "list-scope").await;
        let usecase = use_case(pool);
        let active = usecase
            .create_workflow(plain_command(org_id, "Active"))
            .await
            .unwrap();
        let archived = usecase
            .create_workflow(plain_command(org_id, "Archived"))
            .await
            .unwrap();
        usecase
            .update_workflow(UpdateWorkflowCommand {
                archived: Some(true),
                ..untouched(&archived)
            })
            .await
            .unwrap();

        let ids =
            |workflows: Vec<Workflow>| workflows.into_iter().map(|w| w.id).collect::<Vec<_>>();
        let only_active = usecase
            .list_workflows_by_scope(org_id, ArchiveScope::Active)
            .await
            .unwrap();
        let only_archived = usecase
            .list_workflows_by_scope(org_id, ArchiveScope::Archived)
            .await
            .unwrap();
        let everything = usecase.list_workflows(org_id).await.unwrap();

        assert_eq!(ids(only_active), vec![active.id]);
        assert_eq!(ids(only_archived), vec![archived.id]);
        assert_eq!(everything.len(), 2);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn moving_a_workflow_between_folders_and_back_to_the_root() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "move-folder").await;
        let usecase = use_case(pool);
        let folder_id = create_folder(&usecase, org_id, "Folder").await;
        let created = usecase
            .create_workflow(plain_command(org_id, "Name"))
            .await
            .unwrap();

        let filed = usecase
            .update_workflow(UpdateWorkflowCommand {
                folder_id: Some(Some(folder_id)),
                ..untouched(&created)
            })
            .await
            .unwrap();
        let kept = usecase
            .update_workflow(UpdateWorkflowCommand {
                tags: Some(tags(&["x"])),
                ..untouched(&created)
            })
            .await
            .unwrap();
        let rooted = usecase
            .update_workflow(UpdateWorkflowCommand {
                folder_id: Some(None),
                ..untouched(&created)
            })
            .await
            .unwrap();

        assert_eq!(filed.folder_id, Some(folder_id));
        assert_eq!(kept.folder_id, Some(folder_id));
        assert_eq!(rooted.folder_id, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn moving_a_workflow_into_a_foreign_folder_is_refused() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "move-foreign").await;
        let stranger = seed_organization(&pool, "move-foreign-stranger").await;
        let usecase = use_case(pool);
        let foreign_folder = create_folder(&usecase, stranger, "Theirs").await;
        let created = usecase
            .create_workflow(plain_command(org_id, "Name"))
            .await
            .unwrap();

        let error = usecase
            .update_workflow(UpdateWorkflowCommand {
                folder_id: Some(Some(foreign_folder)),
                ..untouched(&created)
            })
            .await
            .expect_err("the folder belongs to someone else");

        assert!(matches!(&error, CoreError::Conflict(m)
            if FolderRefusal::from_conflict(m) == Some(FolderRefusal::Missing)));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_version_of_an_archived_workflow_is_refused_until_it_is_unarchived() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "archive-save").await;
        let usecase = use_case(pool);
        let created = usecase
            .create_workflow(plain_command(org_id, "Name"))
            .await
            .unwrap();
        usecase
            .update_workflow(UpdateWorkflowCommand {
                archived: Some(true),
                ..untouched(&created)
            })
            .await
            .unwrap();
        let save = |graph| SaveWorkflowVersionCommand {
            org_id,
            workflow_id: created.id,
            graph,
            layout: None,
            created_by: None,
        };

        let refused = usecase
            .save_workflow_version(save(valid_graph()))
            .await
            .expect_err("archived workflows are frozen");
        assert!(is_archived_refusal(&refused));
        assert!(
            usecase
                .list_workflow_versions(org_id, created.id)
                .await
                .unwrap()
                .is_empty()
        );

        usecase
            .update_workflow(UpdateWorkflowCommand {
                archived: Some(false),
                ..untouched(&created)
            })
            .await
            .unwrap();
        let saved = usecase
            .save_workflow_version(save(valid_graph()))
            .await
            .unwrap();
        assert_eq!(saved.version, 1);
    }

    mod operation_nodes {
        use serde_json::{Map, json};

        use super::*;
        use crate::domain::automation::collection::{
            CollectionDraft, CollectionDraftInput, CollectionSource, HttpMethod,
            ImportCollectionCommand, Operation, Parameter, ParameterLocation,
            ReplaceCollectionCommand,
        };
        use crate::domain::automation::workflow::OperationRef;

        fn draft(path: &str, required: bool) -> CollectionDraft {
            CollectionDraft::new(CollectionDraftInput {
                title: "Addresses".to_owned(),
                version: "1.0.0".to_owned(),
                description: String::new(),
                base_url: "https://api.example.com".to_owned(),
                source: CollectionSource::Url {
                    url: "https://api.example.com/openapi.json".to_owned(),
                },
                security: Vec::new(),
                operations: vec![Operation {
                    id: "getAddress".to_owned(),
                    method: HttpMethod::Get,
                    path: path.to_owned(),
                    summary: String::new(),
                    tag: String::new(),
                    parameters: vec![Parameter {
                        name: "limit".to_owned(),
                        location: ParameterLocation::Query,
                        required,
                        description: String::new(),
                        schema: json!({ "type": "integer" }),
                    }],
                    request_body: None,
                    response: None,
                    response_status: None,
                }],
                schemas: Map::new(),
            })
            .unwrap()
        }

        fn operation_graph(collection_id: Uuid, config: serde_json::Value) -> Graph {
            let mut graph = valid_graph();
            graph.connectors[0] = PlacedConnector {
                id: "c1".to_string(),
                kind: "collection.operation".to_string(),
                version: 1,
                credential_id: None,
                config: config.as_object().cloned().unwrap(),
                name: None,
                description: None,
                disabled: false,
                operation: Some(OperationRef {
                    collection_id,
                    operation_id: "getAddress".to_owned(),
                }),
            };
            graph
        }

        async fn save(
            usecase: &MestierUseCase,
            org_id: OrganizationId,
            workflow_id: Uuid,
            graph: Graph,
        ) -> Result<WorkflowVersion, CoreError> {
            usecase
                .save_workflow_version(SaveWorkflowVersionCommand {
                    org_id,
                    workflow_id,
                    graph,
                    layout: None,
                    created_by: None,
                })
                .await
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_node_on_a_stored_operation_saves_and_one_on_a_gone_collection_does_not() {
            let pool = automation_pool().await;
            let org_id = seed_organization(&pool, "save-operation").await;
            let usecase = use_case(pool);
            let workflow = usecase
                .create_workflow(CreateWorkflowCommand {
                    org_id,
                    name: "Uses a collection".to_string(),
                    description: None,
                })
                .await
                .unwrap();
            let collection = usecase
                .import_collection(ImportCollectionCommand {
                    org_id,
                    draft: draft("/addresses", false),
                })
                .await
                .unwrap();

            let saved = save(
                &usecase,
                org_id,
                workflow.id,
                operation_graph(collection.id, json!({ "query": { "limit": 3 } })),
            )
            .await;
            assert!(saved.is_ok(), "{saved:?}");

            usecase
                .delete_collection(org_id, collection.id)
                .await
                .unwrap();
            let refused = save(
                &usecase,
                org_id,
                workflow.id,
                operation_graph(collection.id, json!({ "query": { "limit": 3 } })),
            )
            .await;

            let Err(CoreError::Conflict(message)) = refused else {
                panic!("expected a conflict, got {refused:?}");
            };
            assert!(message.contains("c1"), "{message}");
            assert!(message.contains(&collection.id.to_string()), "{message}");
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_collection_of_another_organization_is_not_a_known_collection() {
            let pool = automation_pool().await;
            let org_id = seed_organization(&pool, "save-operation-own").await;
            let stranger = seed_organization(&pool, "save-operation-stranger").await;
            let usecase = use_case(pool);
            let workflow = usecase
                .create_workflow(CreateWorkflowCommand {
                    org_id,
                    name: "Borrows".to_string(),
                    description: None,
                })
                .await
                .unwrap();
            let foreign = usecase
                .import_collection(ImportCollectionCommand {
                    org_id: stranger,
                    draft: draft("/addresses", false),
                })
                .await
                .unwrap();

            let refused = save(
                &usecase,
                org_id,
                workflow.id,
                operation_graph(foreign.id, json!({})),
            )
            .await;

            let Err(CoreError::Conflict(message)) = refused else {
                panic!("expected a conflict, got {refused:?}");
            };
            assert!(message.contains("unknown collection"), "{message}");
        }

        #[tokio::test]
        #[ignore = "requires live postgres"]
        async fn a_required_parameter_added_later_fails_the_next_save_not_the_stored_version() {
            let pool = automation_pool().await;
            let org_id = seed_organization(&pool, "save-operation-update").await;
            let usecase = use_case(pool);
            let workflow = usecase
                .create_workflow(CreateWorkflowCommand {
                    org_id,
                    name: "Follows the collection".to_string(),
                    description: None,
                })
                .await
                .unwrap();
            let collection = usecase
                .import_collection(ImportCollectionCommand {
                    org_id,
                    draft: draft("/addresses", false),
                })
                .await
                .unwrap();
            let graph = operation_graph(collection.id, json!({}));
            let first = save(&usecase, org_id, workflow.id, graph.clone())
                .await
                .unwrap();

            usecase
                .replace_collection(ReplaceCollectionCommand {
                    org_id,
                    id: collection.id,
                    draft: draft("/addresses", true),
                })
                .await
                .unwrap();
            let refused = save(&usecase, org_id, workflow.id, graph).await;

            let Err(CoreError::Conflict(message)) = refused else {
                panic!("expected a conflict, got {refused:?}");
            };
            assert!(message.contains("query.limit"), "{message}");
            let current = usecase
                .find_workflow_version_by_id(org_id, first.id)
                .await
                .unwrap();
            assert_eq!(current.map(|version| version.version), Some(1));
        }
    }
}
