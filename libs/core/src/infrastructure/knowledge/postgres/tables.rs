use chrono::{DateTime, Utc};
use common::{CoreError, UserId};
use mestier_macros::repository;
use serde_json::Value;
use uuid::Uuid;

use crate::{
    OrganizationId,
    domain::knowledge::tables::{
        ColumnConfig, ColumnKind, ColumnType, KnowledgeTable, KnowledgeTableId, TableColumn,
        TableColumnId, TableDescription, TableItemName, TableView, TableViewId, ViewConfig,
        ViewKind, ports::KnowledgeTableRepository,
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

fn corrupt(what: &str) -> CoreError {
    CoreError::Internal(format!("a stored knowledge table {what} is unreadable"))
}

struct TableRecord {
    id: Uuid,
    org_id: Uuid,
    name: String,
    description: Option<String>,
    created_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<TableRecord> for KnowledgeTable {
    type Error = CoreError;

    fn try_from(record: TableRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: KnowledgeTableId(record.id),
            organization_id: OrganizationId(record.org_id),
            name: TableItemName::try_from(record.name.as_str()).map_err(|_| corrupt("name"))?,
            description: TableDescription::parse(record.description.as_deref())
                .map_err(|_| corrupt("description"))?,
            created_by: record.created_by.map(UserId),
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}

struct ColumnRecord {
    id: Uuid,
    table_id: Uuid,
    org_id: Uuid,
    name: String,
    kind: String,
    config: Value,
    position: i32,
    is_primary: bool,
    created_at: DateTime<Utc>,
}

impl TryFrom<ColumnRecord> for TableColumn {
    type Error = CoreError;

    fn try_from(record: ColumnRecord) -> Result<Self, Self::Error> {
        let kind = record
            .kind
            .parse::<ColumnKind>()
            .map_err(|()| corrupt("column kind"))?;
        let config: ColumnConfig =
            serde_json::from_value(record.config).map_err(|_| corrupt("column config"))?;
        Ok(Self {
            id: TableColumnId(record.id),
            table_id: KnowledgeTableId(record.table_id),
            organization_id: OrganizationId(record.org_id),
            name: TableItemName::try_from(record.name.as_str())
                .map_err(|_| corrupt("column name"))?,
            column_type: ColumnType::from_config(kind, &config)
                .map_err(|_| corrupt("column config"))?,
            position: record.position,
            is_primary: record.is_primary,
            created_at: record.created_at,
        })
    }
}

struct ViewRecord {
    id: Uuid,
    table_id: Uuid,
    org_id: Uuid,
    name: String,
    kind: String,
    config: Value,
    position: i32,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<ViewRecord> for TableView {
    type Error = CoreError;

    fn try_from(record: ViewRecord) -> Result<Self, Self::Error> {
        Ok(Self {
            id: TableViewId(record.id),
            table_id: KnowledgeTableId(record.table_id),
            organization_id: OrganizationId(record.org_id),
            name: TableItemName::try_from(record.name.as_str())
                .map_err(|_| corrupt("view name"))?,
            kind: record
                .kind
                .parse::<ViewKind>()
                .map_err(|()| corrupt("view kind"))?,
            config: serde_json::from_value::<ViewConfig>(record.config)
                .map_err(|_| corrupt("view config"))?,
            position: record.position,
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}

fn config_of<T: serde::Serialize>(config: &T) -> Result<Value, CoreError> {
    serde_json::to_value(config).map_err(|error| CoreError::Internal(error.to_string()))
}

#[repository(domain = KnowledgeTable, backend = Postgres)]
pub struct PgKnowledgeTableRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgKnowledgeTableRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> KnowledgeTableRepository for PgKnowledgeTableRepository<'tx> {
    async fn lock_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query(
            "SELECT pg_advisory_xact_lock(hashtextextended('knowledge_tables:' || $1::text, 0))",
        )
        .bind(organization_id.0)
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn lock_table(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeTableId,
    ) -> Result<bool, CoreError> {
        let mut tx = self.tx.lock().await;
        let locked = sqlx::query_scalar!(
            r#"SELECT 1 AS "locked!" FROM knowledge_tables WHERE id = $1 AND org_id = $2 FOR UPDATE"#,
            id.0,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(locked.is_some())
    }

    async fn count_tables(&mut self, organization_id: OrganizationId) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM knowledge_tables WHERE org_id = $1"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(u64::try_from(count).unwrap_or_default())
    }

    async fn insert_table(&mut self, table: &KnowledgeTable) -> Result<KnowledgeTable, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            TableRecord,
            r#"
            INSERT INTO knowledge_tables (id, org_id, name, description, created_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, name, description, created_by, created_at, updated_at
            "#,
            table.id.0,
            table.organization_id.0,
            table.name.as_str(),
            table.description.as_ref().map(TableDescription::as_str),
            table.created_by.map(|id| id.0),
            table.created_at,
            table.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .try_into()
    }

    async fn find_table(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeTableId,
    ) -> Result<Option<KnowledgeTable>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            TableRecord,
            r#"
            SELECT id, org_id, name, description, created_by, created_at, updated_at
            FROM knowledge_tables
            WHERE id = $1 AND org_id = $2
            "#,
            id.0,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .map(KnowledgeTable::try_from)
        .transpose()
    }

    async fn list_tables(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<KnowledgeTable>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            TableRecord,
            r#"
            SELECT id, org_id, name, description, created_by, created_at, updated_at
            FROM knowledge_tables
            WHERE org_id = $1
            ORDER BY lower(name), id
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(KnowledgeTable::try_from)
        .collect()
    }

    async fn update_table(&mut self, table: &KnowledgeTable) -> Result<KnowledgeTable, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            TableRecord,
            r#"
            UPDATE knowledge_tables
            SET name = $3, description = $4, updated_at = $5
            WHERE id = $1 AND org_id = $2
            RETURNING id, org_id, name, description, created_by, created_at, updated_at
            "#,
            table.id.0,
            table.organization_id.0,
            table.name.as_str(),
            table.description.as_ref().map(TableDescription::as_str),
            table.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?
        .try_into()
    }

    async fn delete_table(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeTableId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            "DELETE FROM knowledge_tables WHERE id = $1 AND org_id = $2",
            id.0,
            organization_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    async fn insert_column(&mut self, column: &TableColumn) -> Result<TableColumn, CoreError> {
        let config = config_of(&column.column_type.config())?;
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            ColumnRecord,
            r#"
            INSERT INTO knowledge_table_columns (id, table_id, org_id, name, kind, config, position, is_primary, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, table_id, org_id, name, kind, config, position, is_primary, created_at
            "#,
            column.id.0,
            column.table_id.0,
            column.organization_id.0,
            column.name.as_str(),
            column.kind().as_str(),
            config,
            column.position,
            column.is_primary,
            column.created_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .try_into()
    }

    async fn columns(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
    ) -> Result<Vec<TableColumn>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            ColumnRecord,
            r#"
            SELECT id, table_id, org_id, name, kind, config, position, is_primary, created_at
            FROM knowledge_table_columns
            WHERE table_id = $1 AND org_id = $2
            ORDER BY position, created_at, id
            "#,
            table_id.0,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(TableColumn::try_from)
        .collect()
    }

    async fn update_column(&mut self, column: &TableColumn) -> Result<TableColumn, CoreError> {
        let config = config_of(&column.column_type.config())?;
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            ColumnRecord,
            r#"
            UPDATE knowledge_table_columns
            SET name = $4, kind = $5, config = $6
            WHERE id = $1 AND table_id = $2 AND org_id = $3
            RETURNING id, table_id, org_id, name, kind, config, position, is_primary, created_at
            "#,
            column.id.0,
            column.table_id.0,
            column.organization_id.0,
            column.name.as_str(),
            column.kind().as_str(),
            config,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?
        .try_into()
    }

    async fn delete_column(
        &mut self,
        organization_id: OrganizationId,
        id: TableColumnId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            "DELETE FROM knowledge_table_columns WHERE id = $1 AND org_id = $2 AND NOT is_primary",
            id.0,
            organization_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }

    async fn reorder_columns(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        order: &[TableColumnId],
    ) -> Result<(), CoreError> {
        let ids: Vec<Uuid> = order.iter().map(|id| id.0).collect();
        let mut tx = self.tx.lock().await;
        sqlx::query!(
            r#"
            UPDATE knowledge_table_columns c
            SET position = (o.rank - 1)::int
            FROM UNNEST($3::uuid[]) WITH ORDINALITY AS o(id, rank)
            WHERE c.id = o.id AND c.table_id = $1 AND c.org_id = $2
            "#,
            table_id.0,
            organization_id.0,
            &ids,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(())
    }

    async fn insert_view(&mut self, view: &TableView) -> Result<TableView, CoreError> {
        let config = config_of(&view.config)?;
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            ViewRecord,
            r#"
            INSERT INTO knowledge_table_views (id, table_id, org_id, name, kind, config, position, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, table_id, org_id, name, kind, config, position, created_at, updated_at
            "#,
            view.id.0,
            view.table_id.0,
            view.organization_id.0,
            view.name.as_str(),
            view.kind.as_str(),
            config,
            view.position,
            view.created_at,
            view.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .try_into()
    }

    async fn views(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
    ) -> Result<Vec<TableView>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            ViewRecord,
            r#"
            SELECT id, table_id, org_id, name, kind, config, position, created_at, updated_at
            FROM knowledge_table_views
            WHERE table_id = $1 AND org_id = $2
            ORDER BY position, created_at, id
            "#,
            table_id.0,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(TableView::try_from)
        .collect()
    }

    async fn update_view(&mut self, view: &TableView) -> Result<TableView, CoreError> {
        let config = config_of(&view.config)?;
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            ViewRecord,
            r#"
            UPDATE knowledge_table_views
            SET name = $4, kind = $5, config = $6, updated_at = $7
            WHERE id = $1 AND table_id = $2 AND org_id = $3
            RETURNING id, table_id, org_id, name, kind, config, position, created_at, updated_at
            "#,
            view.id.0,
            view.table_id.0,
            view.organization_id.0,
            view.name.as_str(),
            view.kind.as_str(),
            config,
            view.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?
        .try_into()
    }

    async fn delete_view(
        &mut self,
        organization_id: OrganizationId,
        id: TableViewId,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            "DELETE FROM knowledge_table_views WHERE id = $1 AND org_id = $2",
            id.0,
            organization_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }
        Ok(())
    }
}
