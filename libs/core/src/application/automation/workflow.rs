use chrono::Utc;
use common::{CoreError, OrganizationId, generate_uuid_v7};
use mestier_macros::transactional;
use uuid::Uuid;

use crate::{
    application::MestierUseCase,
    domain::automation::{
        connector::connector_catalogue,
        ports::{CredentialRepository, WorkflowRepository},
        workflow::{
            CreateWorkflowCommand, GraphError, SaveWorkflowVersionCommand, UpdateWorkflowCommand,
            Workflow, WorkflowVersion, validate_graph,
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
    #[transactional(workflow)]
    pub async fn create_workflow(
        &self,
        command: CreateWorkflowCommand,
    ) -> Result<Workflow, CoreError> {
        let mut repository = workflow_repository;
        let now = Utc::now();
        let workflow = Workflow {
            id: generate_uuid_v7(),
            org_id: command.org_id,
            name: command.name,
            description: command.description,
            enabled: true,
            current_version_id: None,
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
        repository.list_by_organization(org_id).await
    }

    #[transactional(workflow)]
    pub async fn update_workflow(
        &self,
        command: UpdateWorkflowCommand,
    ) -> Result<Workflow, CoreError> {
        let mut repository = workflow_repository;
        let existing = repository
            .find_by_id(command.org_id, command.id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let updated = Workflow {
            name: command.name.unwrap_or(existing.name),
            description: command.description.unwrap_or(existing.description),
            enabled: command.enabled.unwrap_or(existing.enabled),
            updated_at: Utc::now(),
            ..existing
        };

        repository.update(command.org_id, &updated).await
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
    #[transactional(workflow, credential)]
    pub async fn save_workflow_version(
        &self,
        command: SaveWorkflowVersionCommand,
    ) -> Result<WorkflowVersion, CoreError> {
        let mut workflows = workflow_repository;
        let mut credentials = credential_repository;

        workflows
            .find_by_id(command.org_id, command.workflow_id)
            .await?
            .ok_or(CoreError::NotFound)?;

        let catalogue = connector_catalogue();
        let organization_credentials = credentials.list_by_organization(command.org_id).await?;

        validate_graph(&command.graph, &catalogue, &organization_credentials)
            .map_err(|errors| CoreError::Conflict(format_graph_errors(&errors)))?;

        workflows
            .insert_version(
                command.org_id,
                command.workflow_id,
                &command.graph,
                command.created_by,
            )
            .await
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
    use crate::domain::automation::workflow::{Graph, PlacedConnector};
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
            }],
            edges: Vec::new(),
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
                name: Some("New name".to_string()),
                description: Some(None),
                enabled: Some(false),
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
                name: Some("Nope".to_string()),
                description: None,
                enabled: None,
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
            }],
            edges: Vec::new(),
        };

        let error = usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph: invalid_graph,
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
        });
        let error = usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id,
                workflow_id: created.id,
                graph,
                created_by: None,
            })
            .await
            .expect_err("a stranger's credential must not validate");

        assert!(matches!(error, CoreError::Conflict(_)));
    }
}
