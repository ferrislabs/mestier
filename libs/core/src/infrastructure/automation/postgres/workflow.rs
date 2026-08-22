use common::{CoreError, OrganizationId};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    domain::automation::{
        ports::WorkflowRepository,
        workflow::{Graph, Workflow, WorkflowReference, WorkflowVersion},
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = Workflow, backend = Postgres)]
pub struct PgWorkflowRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgWorkflowRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

struct WorkflowRow {
    id: Uuid,
    org_id: Uuid,
    name: String,
    description: Option<String>,
    enabled: bool,
    current_version_id: Option<Uuid>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<WorkflowRow> for Workflow {
    fn from(row: WorkflowRow) -> Self {
        Self {
            id: row.id,
            org_id: OrganizationId(row.org_id),
            name: row.name,
            description: row.description,
            enabled: row.enabled,
            current_version_id: row.current_version_id,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

struct WorkflowVersionRow {
    id: Uuid,
    workflow_id: Uuid,
    version: i32,
    graph: serde_json::Value,
    created_at: chrono::DateTime<chrono::Utc>,
    created_by: Option<Uuid>,
}

impl TryFrom<WorkflowVersionRow> for WorkflowVersion {
    type Error = CoreError;

    fn try_from(row: WorkflowVersionRow) -> Result<Self, Self::Error> {
        let graph: Graph = serde_json::from_value(row.graph)
            .map_err(|e| CoreError::Internal(format!("invalid workflow graph in database: {e}")))?;

        Ok(Self {
            id: row.id,
            workflow_id: row.workflow_id,
            version: row.version,
            graph,
            created_at: row.created_at,
            created_by: row.created_by,
        })
    }
}

impl<'tx> WorkflowRepository for PgWorkflowRepository<'tx> {
    async fn insert(&mut self, workflow: &Workflow) -> Result<Workflow, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WorkflowRow,
            r#"INSERT INTO automation.workflow
                   (id, org_id, name, description, enabled, current_version_id, created_at, updated_at)
               VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
               RETURNING id, org_id, name, description, enabled, current_version_id, created_at, updated_at"#,
            workflow.id,
            workflow.org_id.0,
            workflow.name,
            workflow.description,
            workflow.enabled,
            workflow.current_version_id,
            workflow.created_at,
            workflow.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(
        &mut self,
        org_id: OrganizationId,
        id: Uuid,
    ) -> Result<Option<Workflow>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WorkflowRow,
            r#"SELECT id, org_id, name, description, enabled, current_version_id, created_at, updated_at
               FROM automation.workflow WHERE org_id = $1 AND id = $2"#,
            org_id.0,
            id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Workflow::from))
    }

    async fn list_by_organization(
        &mut self,
        org_id: OrganizationId,
    ) -> Result<Vec<Workflow>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            WorkflowRow,
            r#"SELECT id, org_id, name, description, enabled, current_version_id, created_at, updated_at
               FROM automation.workflow
               WHERE org_id = $1
               ORDER BY name"#,
            org_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Workflow::from).collect())
    }

    async fn update(
        &mut self,
        org_id: OrganizationId,
        workflow: &Workflow,
    ) -> Result<Workflow, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WorkflowRow,
            r#"UPDATE automation.workflow
               SET name = $3, description = $4, enabled = $5, updated_at = $6
               WHERE org_id = $1 AND id = $2
               RETURNING id, org_id, name, description, enabled, current_version_id, created_at, updated_at"#,
            org_id.0,
            workflow.id,
            workflow.name,
            workflow.description,
            workflow.enabled,
            workflow.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Workflow::from).ok_or(CoreError::NotFound)
    }

    async fn delete(&mut self, org_id: OrganizationId, id: Uuid) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            "DELETE FROM automation.workflow WHERE org_id = $1 AND id = $2",
            org_id.0,
            id,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn insert_version(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        graph: &Graph,
        created_by: Option<Uuid>,
    ) -> Result<WorkflowVersion, CoreError> {
        let mut tx = self.tx.lock().await;

        let owned = sqlx::query_scalar!(
            "SELECT id FROM automation.workflow WHERE org_id = $1 AND id = $2",
            org_id.0,
            workflow_id,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        if owned.is_none() {
            return Err(CoreError::NotFound);
        }

        // Serializes concurrent saves of the same workflow so two editors
        // computing "next version" at once can never mint the same number —
        // the same technique `PgQuoteRepository::next_reference` uses for
        // its own per-organization counter. `1` is a namespace salt, kept
        // distinct from the quote repository's `0` so the two counters
        // never share a lock key by coincidence.
        sqlx::query!(
            "SELECT pg_advisory_xact_lock(hashtextextended($1::text, 1))",
            workflow_id.to_string(),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let next_version = sqlx::query_scalar!(
            r#"SELECT COALESCE(MAX(version), 0) + 1 AS "next_version!"
               FROM automation.workflow_version WHERE workflow_id = $1"#,
            workflow_id,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let graph_json = serde_json::to_value(graph).map_err(|e| {
            CoreError::Internal(format!("workflow graph cannot be serialized: {e}"))
        })?;

        let version_id = common::generate_uuid_v7();
        let now = chrono::Utc::now();
        let row = sqlx::query_as!(
            WorkflowVersionRow,
            r#"INSERT INTO automation.workflow_version
                   (id, workflow_id, version, graph, created_at, created_by)
               VALUES ($1, $2, $3, $4, $5, $6)
               RETURNING id, workflow_id, version, graph, created_at, created_by"#,
            version_id,
            workflow_id,
            next_version,
            graph_json,
            now,
            created_by,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        sqlx::query!(
            "UPDATE automation.workflow SET current_version_id = $2, updated_at = $3 WHERE id = $1",
            workflow_id,
            version_id,
            now,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.try_into()
    }

    async fn find_version(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
        version: i32,
    ) -> Result<Option<WorkflowVersion>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WorkflowVersionRow,
            r#"SELECT wv.id, wv.workflow_id, wv.version, wv.graph, wv.created_at, wv.created_by
               FROM automation.workflow_version wv
               JOIN automation.workflow w ON w.id = wv.workflow_id
               WHERE w.org_id = $1 AND wv.workflow_id = $2 AND wv.version = $3"#,
            org_id.0,
            workflow_id,
            version,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(WorkflowVersion::try_from).transpose()
    }

    async fn list_versions(
        &mut self,
        org_id: OrganizationId,
        workflow_id: Uuid,
    ) -> Result<Vec<WorkflowVersion>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            WorkflowVersionRow,
            r#"SELECT wv.id, wv.workflow_id, wv.version, wv.graph, wv.created_at, wv.created_by
               FROM automation.workflow_version wv
               JOIN automation.workflow w ON w.id = wv.workflow_id
               WHERE w.org_id = $1 AND wv.workflow_id = $2
               ORDER BY wv.version"#,
            org_id.0,
            workflow_id,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(WorkflowVersion::try_from).collect()
    }

    async fn workflows_referencing_credential(
        &mut self,
        org_id: OrganizationId,
        credential_id: Uuid,
    ) -> Result<Vec<WorkflowReference>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            WorkflowReference,
            r#"SELECT DISTINCT w.id, w.name
               FROM automation.workflow w
               JOIN automation.workflow_version wv ON wv.workflow_id = w.id
               WHERE w.org_id = $1
                 AND wv.graph @> jsonb_build_object(
                       'connectors',
                       jsonb_build_array(jsonb_build_object('credential_id', $2::text))
                     )
               ORDER BY w.name"#,
            org_id.0,
            credential_id.to_string(),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;
    use sqlx::PgPool;

    use super::*;
    use crate::application::test_support::automation_pool;
    use crate::application::test_support::now_storable;
    use crate::domain::automation::workflow::{Branch, Edge, PlacedConnector};
    use crate::infrastructure::postgres::with_tx;

    async fn make_pool() -> PgPool {
        automation_pool().await
    }

    async fn seed_user(pool: &PgPool, label: &str) -> Uuid {
        let id = generate_uuid_v7();
        sqlx::query!(
            r#"INSERT INTO users (id, email, username, display_name, sub)
               VALUES ($1, $2, $3, $4, $5)"#,
            id,
            format!("{label}-{id}@example.com"),
            format!("{label}-{id}"),
            "Some User",
            format!("sub-{label}-{id}"),
        )
        .execute(pool)
        .await
        .unwrap();
        id
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

    fn workflow(org_id: OrganizationId, name: &str) -> Workflow {
        let now = now_storable();
        Workflow {
            id: generate_uuid_v7(),
            org_id,
            name: name.to_owned(),
            description: None,
            enabled: true,
            current_version_id: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn graph_with_credential(credential_id: Option<Uuid>) -> Graph {
        Graph {
            connectors: vec![PlacedConnector {
                id: "c1".to_string(),
                kind: "odoo.create_partner".to_string(),
                version: 1,
                credential_id,
                config: serde_json::Map::new(),
            }],
            edges: Vec::new(),
        }
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_inserted_workflow_is_found_by_id_in_its_own_organization() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "insert").await;
        let written = workflow(org_id, "Quote to CRM");

        let inserted = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert(&written).await
        })
        .await
        .unwrap();

        assert_eq!(inserted, written);

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.find_by_id(org_id, written.id).await
        })
        .await
        .unwrap();

        assert_eq!(found, Some(written));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_from_another_organization_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "owner").await;
        let stranger_org = seed_organization(&pool, "stranger").await;
        let written = workflow(org_id, "Owner's workflow");

        with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert(&written).await
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.find_by_id(stranger_org, written.id).await
        })
        .await
        .unwrap();

        assert_eq!(
            found, None,
            "a workflow id from another organization must read back as absent"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn listing_only_returns_the_requesting_organizations_workflows() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "listed").await;
        let other_org = seed_organization(&pool, "not-listed").await;

        with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert(&workflow(org_id, "A")).await?;
            repo.insert(&workflow(org_id, "B")).await?;
            repo.insert(&workflow(other_org, "Not mine")).await
        })
        .await
        .unwrap();

        let listed = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.list_by_organization(org_id).await
        })
        .await
        .unwrap();

        assert_eq!(listed.len(), 2);
        assert!(listed.iter().all(|w| w.org_id == org_id));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_changes_name_description_and_enabled_without_touching_current_version() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "update").await;
        let written = workflow(org_id, "Old name");

        let inserted = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            let inserted = repo.insert(&written).await?;
            repo.insert_version(org_id, inserted.id, &graph_with_credential(None), None)
                .await?;
            repo.find_by_id(org_id, inserted.id).await
        })
        .await
        .unwrap()
        .unwrap();
        assert!(inserted.current_version_id.is_some());

        let updated = Workflow {
            name: "New name".to_string(),
            description: Some("A description".to_string()),
            enabled: false,
            ..inserted.clone()
        };

        let result = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.update(org_id, &updated).await
        })
        .await
        .unwrap();

        assert_eq!(result.name, "New name");
        assert_eq!(result.description, Some("A description".to_string()));
        assert!(!result.enabled);
        assert_eq!(
            result.current_version_id, inserted.current_version_id,
            "renaming must not move current_version_id"
        );
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_a_workflow_from_another_organization_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "target").await;
        let stranger_org = seed_organization(&pool, "attacker").await;
        let written = workflow(org_id, "Name");

        let inserted = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert(&written).await
        })
        .await
        .unwrap();

        let tampered = Workflow {
            name: "Renamed by a stranger".to_string(),
            ..inserted
        };

        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.update(stranger_org, &tampered).await
        })
        .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn deleting_removes_the_row() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "delete").await;
        let written = workflow(org_id, "Name");

        with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert(&written).await
        })
        .await
        .unwrap();

        with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.delete(org_id, written.id).await
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.find_by_id(org_id, written.id).await
        })
        .await
        .unwrap();

        assert_eq!(found, None);
    }

    /// The acceptance criterion the whole ticket rests on: saving a second
    /// edit inserts a new row and leaves the first one byte-for-byte as it
    /// was, and moves `current_version_id` forward to the new one.
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn saving_a_second_version_leaves_the_first_untouched_and_moves_current() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "versions").await;
        let inserted = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert(&workflow(org_id, "Versioned")).await
        })
        .await
        .unwrap();

        let graph_v1 = graph_with_credential(Some(Uuid::from_u128(1)));
        let version1 = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert_version(org_id, inserted.id, &graph_v1, None)
                .await
        })
        .await
        .unwrap();
        assert_eq!(version1.version, 1);
        assert_eq!(version1.graph, graph_v1);

        let creator = seed_user(&pool, "creator").await;
        let graph_v2 = graph_with_credential(Some(Uuid::from_u128(2)));
        let version2 = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert_version(org_id, inserted.id, &graph_v2, Some(creator))
                .await
        })
        .await
        .unwrap();
        assert_eq!(version2.version, 2);
        assert_eq!(version2.graph, graph_v2);
        assert_eq!(version2.created_by, Some(creator));

        // The first version, read back fresh, is exactly what it was before
        // the second save — nothing about it moved.
        let reread_version1 = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.find_version(org_id, inserted.id, 1).await
        })
        .await
        .unwrap()
        .unwrap();
        assert_eq!(reread_version1, version1);

        let current = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.find_by_id(org_id, inserted.id).await
        })
        .await
        .unwrap()
        .unwrap();
        assert_eq!(
            current.current_version_id,
            Some(version2.id),
            "current_version_id moves to the newest version"
        );

        let versions = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.list_versions(org_id, inserted.id).await
        })
        .await
        .unwrap();
        assert_eq!(versions, vec![version1, version2]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn inserting_a_version_for_an_unknown_workflow_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "no-workflow").await;

        let outcome = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert_version(org_id, generate_uuid_v7(), &Graph::default(), None)
                .await
        })
        .await;

        assert!(matches!(outcome, Err(CoreError::NotFound)));
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn finding_a_version_from_another_organization_is_not_found() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "version-owner").await;
        let stranger_org = seed_organization(&pool, "version-stranger").await;
        let inserted = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            let inserted = repo.insert(&workflow(org_id, "Mine")).await?;
            repo.insert_version(org_id, inserted.id, &Graph::default(), None)
                .await?;
            Ok::<_, CoreError>(inserted)
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.find_version(stranger_org, inserted.id, 1).await
        })
        .await
        .unwrap();

        assert_eq!(found, None);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn workflows_referencing_credential_names_the_workflow_whose_graph_contains_it() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "ref-owner").await;
        let credential_id = generate_uuid_v7();
        let referencing = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            let workflow = repo.insert(&workflow(org_id, "References it")).await?;
            repo.insert_version(
                org_id,
                workflow.id,
                &graph_with_credential(Some(credential_id)),
                None,
            )
            .await?;
            Ok::<_, CoreError>(workflow)
        })
        .await
        .unwrap();

        // A second workflow, in the same organization, that does not
        // reference the credential at all.
        with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            let workflow = repo
                .insert(&workflow(org_id, "Does not reference it"))
                .await?;
            repo.insert_version(org_id, workflow.id, &graph_with_credential(None), None)
                .await
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.workflows_referencing_credential(org_id, credential_id)
                .await
        })
        .await
        .unwrap();

        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, referencing.id);
        assert_eq!(found[0].name, "References it");
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn workflows_referencing_credential_ignores_another_organizations_workflow() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "ref-empty").await;
        let other_org = seed_organization(&pool, "ref-other").await;
        let credential_id = generate_uuid_v7();

        with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            let workflow = repo.insert(&workflow(other_org, "In another org")).await?;
            repo.insert_version(
                other_org,
                workflow.id,
                &graph_with_credential(Some(credential_id)),
                None,
            )
            .await
        })
        .await
        .unwrap();

        let found = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.workflows_referencing_credential(org_id, credential_id)
                .await
        })
        .await
        .unwrap();

        assert!(found.is_empty());
    }

    /// The `Each`/`After`/`Then`/`Else` branches must round-trip through the
    /// JSONB column, not only through `serde_json` in memory (`graph.rs`'s
    /// own test already covers that half).
    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_graph_with_edges_and_branches_round_trips_through_the_database() {
        let pool = make_pool().await;
        let org_id = seed_organization(&pool, "edges").await;
        let inserted = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert(&workflow(org_id, "With edges")).await
        })
        .await
        .unwrap();

        let graph = Graph {
            connectors: vec![
                PlacedConnector {
                    id: "c1".to_string(),
                    kind: "flow.condition".to_string(),
                    version: 1,
                    credential_id: None,
                    config: {
                        let mut m = serde_json::Map::new();
                        m.insert("predicate".to_string(), serde_json::json!("{{ true }}"));
                        m
                    },
                },
                PlacedConnector {
                    id: "c2".to_string(),
                    kind: "flow.condition".to_string(),
                    version: 1,
                    credential_id: None,
                    config: {
                        let mut m = serde_json::Map::new();
                        m.insert("predicate".to_string(), serde_json::json!("{{ false }}"));
                        m
                    },
                },
            ],
            edges: vec![Edge {
                from: "c1".to_string(),
                to: "c2".to_string(),
                branch: Some(Branch::Then),
            }],
        };

        let version = with_tx(&pool, async |tx| {
            let mut repo = PgWorkflowRepository::new(&tx);
            repo.insert_version(org_id, inserted.id, &graph, None).await
        })
        .await
        .unwrap();

        assert_eq!(version.graph, graph);
    }
}
