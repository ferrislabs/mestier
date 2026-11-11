use std::collections::{BTreeMap, HashSet};

use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    OrganizationId,
    domain::knowledge::tables::{
        CellLabel, CreateTableRowCommand, KnowledgeTableId, ListTableRowsCommand, MAX_DELETED_ROWS,
        MAX_ROWS, ResolvedRow, RowBatch, RowPage, RowSelection, RowUpdate, TableColumn, TableError,
        TableOutcome, TableRow, TableRowId, UpdateTableRowCommand,
        error::{Stop, settle},
        filter::RowQuery,
        ports::{CellReferenceCheck, KnowledgeTableRepository, KnowledgeTableRowRepository},
        value::{CellError, CellValue, RowPatch, RowValues},
    },
};

pub struct KnowledgeTableRowService<T, R, E>
where
    T: KnowledgeTableRepository,
    R: KnowledgeTableRowRepository,
    E: EventEmitter,
{
    tables: T,
    rows: R,
    #[expect(dead_code)]
    emitter: E,
}

impl<T, R, E> KnowledgeTableRowService<T, R, E>
where
    T: KnowledgeTableRepository,
    R: KnowledgeTableRowRepository,
    E: EventEmitter,
{
    pub fn new(tables: T, rows: R, emitter: E) -> Self {
        Self {
            tables,
            rows,
            emitter,
        }
    }

    pub async fn columns(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
    ) -> Result<Vec<TableColumn>, CoreError> {
        self.tables
            .find_table(organization_id, table_id)
            .await?
            .ok_or(CoreError::NotFound)?;
        self.tables.columns(organization_id, table_id).await
    }

    async fn lock(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
    ) -> Result<Vec<TableColumn>, CoreError> {
        if !self.tables.lock_table(organization_id, table_id).await? {
            return Err(CoreError::NotFound);
        }
        self.tables.columns(organization_id, table_id).await
    }

    async fn check_references<C>(
        organization_id: OrganizationId,
        columns: &[TableColumn],
        values: &RowValues,
        references: &mut C,
    ) -> Result<(), Stop>
    where
        C: CellReferenceCheck,
    {
        for (column, reference) in values.references(columns) {
            if !references.exists(organization_id, reference).await? {
                return Err(TableError::InvalidCell {
                    column,
                    reason: CellError::UnknownTarget,
                }
                .into());
            }
        }
        Ok(())
    }

    pub async fn create_row<C>(
        &mut self,
        command: CreateTableRowCommand,
        references: &mut C,
        now: DateTime<Utc>,
    ) -> Result<TableOutcome<ResolvedRow>, CoreError>
    where
        C: CellReferenceCheck,
    {
        settle(self.try_create_row(command, references, now).await)
    }

    async fn try_create_row<C>(
        &mut self,
        command: CreateTableRowCommand,
        references: &mut C,
        now: DateTime<Utc>,
    ) -> Result<ResolvedRow, Stop>
    where
        C: CellReferenceCheck,
    {
        let organization_id = command.organization_id;
        let columns = self.lock(organization_id, command.table_id).await?;
        let patch = RowPatch::parse(&columns, &command.values)?;
        Self::check_references(organization_id, &columns, &patch.set, references).await?;
        if self.rows.count(organization_id, command.table_id).await? >= MAX_ROWS {
            return Err(TableError::TooManyRows.into());
        }
        let row = self
            .rows
            .insert(&TableRow {
                id: TableRowId(generate_uuid_v7()),
                table_id: command.table_id,
                organization_id,
                values: patch.set.to_json(),
                created_by: command.user_id,
                updated_by: command.user_id,
                created_at: now,
                updated_at: now,
            })
            .await?;
        let mut resolved = self.resolve(organization_id, &columns, vec![row]).await?;
        resolved.pop().ok_or(Stop::Failed(CoreError::NotFound))
    }

    pub async fn update_row<C>(
        &mut self,
        command: UpdateTableRowCommand,
        references: &mut C,
        now: DateTime<Utc>,
    ) -> Result<TableOutcome<ResolvedRow>, CoreError>
    where
        C: CellReferenceCheck,
    {
        settle(self.try_update_row(command, references, now).await)
    }

    async fn try_update_row<C>(
        &mut self,
        command: UpdateTableRowCommand,
        references: &mut C,
        now: DateTime<Utc>,
    ) -> Result<ResolvedRow, Stop>
    where
        C: CellReferenceCheck,
    {
        let organization_id = command.organization_id;
        let columns = self.columns(organization_id, command.table_id).await?;
        self.rows
            .find(organization_id, command.table_id, command.id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let patch = RowPatch::parse(&columns, &command.values)?;
        Self::check_references(organization_id, &columns, &patch.set, references).await?;
        let row = self
            .rows
            .patch(&RowUpdate {
                organization_id,
                table_id: command.table_id,
                id: command.id,
                set: patch.set.to_json(),
                cleared: patch.cleared_keys(),
                updated_by: command.user_id,
                updated_at: now,
            })
            .await?
            .ok_or(CoreError::NotFound)?;
        let mut resolved = self.resolve(organization_id, &columns, vec![row]).await?;
        resolved.pop().ok_or(Stop::Failed(CoreError::NotFound))
    }

    pub async fn insert_rows<C>(
        &mut self,
        batch: &RowBatch,
        references: &mut C,
    ) -> Result<TableOutcome<u64>, CoreError>
    where
        C: CellReferenceCheck,
    {
        settle(self.try_insert_rows(batch, references).await)
    }

    async fn try_insert_rows<C>(
        &mut self,
        batch: &RowBatch,
        references: &mut C,
    ) -> Result<u64, Stop>
    where
        C: CellReferenceCheck,
    {
        let organization_id = batch.organization_id;
        let columns = self.lock(organization_id, batch.table_id).await?;
        let stored = self.rows.count(organization_id, batch.table_id).await?;
        let added = u64::try_from(batch.rows.len()).unwrap_or(u64::MAX);
        if stored.saturating_add(added) > MAX_ROWS {
            return Err(TableError::TooManyRows.into());
        }
        for (_, values) in &batch.rows {
            Self::check_references(organization_id, &columns, values, references).await?;
        }
        Ok(self.rows.insert_many(batch).await?)
    }

    pub async fn delete_rows(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        ids: &[TableRowId],
    ) -> Result<TableOutcome<u64>, CoreError> {
        if ids.len() > MAX_DELETED_ROWS {
            return Ok(Err(TableError::TooManyRowsAtOnce));
        }
        self.columns(organization_id, table_id).await?;
        self.rows
            .delete(organization_id, table_id, ids)
            .await
            .map(Ok)
    }

    pub async fn get_row(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        id: TableRowId,
    ) -> Result<ResolvedRow, CoreError> {
        let columns = self.columns(organization_id, table_id).await?;
        let row = self
            .rows
            .find(organization_id, table_id, id)
            .await?
            .ok_or(CoreError::NotFound)?;
        let mut resolved = self.resolve(organization_id, &columns, vec![row]).await?;
        resolved.pop().ok_or(CoreError::NotFound)
    }

    pub async fn list_rows(
        &mut self,
        command: ListTableRowsCommand,
    ) -> Result<TableOutcome<RowPage>, CoreError> {
        settle(self.try_list_rows(command).await)
    }

    async fn try_list_rows(&mut self, command: ListTableRowsCommand) -> Result<RowPage, Stop> {
        let organization_id = command.organization_id;
        let columns = self.columns(organization_id, command.table_id).await?;
        let query = match &command.selection {
            RowSelection::View(None) => {
                RowQuery::build(&columns, &[], &[], command.limit, command.offset)?
            }
            RowSelection::View(Some(view_id)) => {
                let view = self
                    .tables
                    .views(organization_id, command.table_id)
                    .await?
                    .into_iter()
                    .find(|view| view.id == *view_id)
                    .ok_or(CoreError::NotFound)?;
                RowQuery::build(
                    &columns,
                    &view.config.filters,
                    &view.config.sorts,
                    command.limit,
                    command.offset,
                )?
            }
            RowSelection::AdHoc { filters, sorts } => {
                RowQuery::build(&columns, filters, sorts, command.limit, command.offset)?
            }
        };
        let (rows, total) = self
            .rows
            .list(organization_id, command.table_id, &query)
            .await?;
        Ok(RowPage {
            rows: self.resolve(organization_id, &columns, rows).await?,
            total,
            limit: query.limit,
            offset: query.offset,
        })
    }

    pub async fn resolve(
        &mut self,
        organization_id: OrganizationId,
        columns: &[TableColumn],
        rows: Vec<TableRow>,
    ) -> Result<Vec<ResolvedRow>, CoreError> {
        let cells = |row: &TableRow| -> Vec<_> {
            columns
                .iter()
                .filter(|column| column.kind().is_reference())
                .filter_map(|column| {
                    let reference = row
                        .values
                        .get(&column.key())
                        .and_then(|raw| CellValue::parse(&column.column_type, raw).ok().flatten())
                        .and_then(|cell| cell.reference(&column.column_type));
                    row.values
                        .contains_key(&column.key())
                        .then_some((column.id, reference))
                })
                .collect()
        };
        let wanted: Vec<_> = rows
            .iter()
            .flat_map(cells)
            .filter_map(|(_, reference)| reference)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();
        let labels = if wanted.is_empty() {
            Default::default()
        } else {
            self.rows.labels(organization_id, &wanted).await?
        };
        Ok(rows
            .into_iter()
            .map(|row| {
                let resolved: BTreeMap<_, _> = cells(&row)
                    .into_iter()
                    .map(|(column, reference)| {
                        let label = reference
                            .and_then(|reference| labels.get(&reference))
                            .map_or(CellLabel::Missing, |label| CellLabel::Found(label.clone()));
                        (column, label)
                    })
                    .collect();
                ResolvedRow {
                    row,
                    labels: resolved,
                }
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use events::testing::RecordingEmitter;
    use serde_json::json;

    use super::*;
    use crate::{
        KnowledgeResource, KnowledgeResourceType,
        domain::knowledge::tables::{
            KnowledgeTable, TableColumnId, TableItemName,
            column::ColumnType,
            ports::{
                MockCellReferenceCheck, MockKnowledgeTableRepository,
                MockKnowledgeTableRowRepository,
            },
            value::CellReference,
        },
    };

    fn org() -> OrganizationId {
        OrganizationId(generate_uuid_v7())
    }

    fn actor() -> authz::Subject {
        authz::Subject::new(authz::SubjectKind::User, "someone")
    }

    fn column(table_id: KnowledgeTableId, column_type: ColumnType) -> TableColumn {
        TableColumn {
            id: TableColumnId(generate_uuid_v7()),
            table_id,
            organization_id: org(),
            name: TableItemName::try_from("Colonne").unwrap(),
            column_type,
            position: 0,
            is_primary: false,
            created_at: Utc::now(),
        }
    }

    fn tables_with(columns: Vec<TableColumn>) -> MockKnowledgeTableRepository {
        let mut tables = MockKnowledgeTableRepository::new();
        tables
            .expect_lock_table()
            .returning(|_, _| Box::pin(async { Ok(true) }));
        tables.expect_find_table().returning(|organization_id, id| {
            Box::pin(async move {
                Ok(Some(KnowledgeTable {
                    id,
                    organization_id,
                    name: TableItemName::try_from("Chantiers").unwrap(),
                    description: None,
                    created_by: None,
                    created_at: Utc::now(),
                    updated_at: Utc::now(),
                }))
            })
        });
        tables.expect_columns().returning(move |_, _| {
            let columns = columns.clone();
            Box::pin(async move { Ok(columns) })
        });
        tables
    }

    #[tokio::test]
    async fn a_link_to_another_organization_is_refused_and_nothing_is_written() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let client = column(table_id, ColumnType::Link(KnowledgeResourceType::Customer));
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_insert().never();
        let mut references = MockCellReferenceCheck::new();
        references
            .expect_exists()
            .returning(|_, _| Box::pin(async { Ok(false) }));
        let emitter = RecordingEmitter::default();
        let mut service =
            KnowledgeTableRowService::new(tables_with(vec![client.clone()]), rows, &emitter);
        let outcome = service
            .create_row(
                CreateTableRowCommand {
                    actor: actor(),
                    organization_id: org(),
                    table_id,
                    user_id: None,
                    values: BTreeMap::from([(
                        client.id,
                        json!({"kind": "customer", "id": generate_uuid_v7()}),
                    )]),
                },
                &mut references,
                Utc::now(),
            )
            .await
            .unwrap();
        assert_eq!(
            outcome,
            Err(TableError::InvalidCell {
                column: client.id,
                reason: CellError::UnknownTarget
            })
        );
    }

    #[tokio::test]
    async fn the_ten_thousand_and_first_row_is_refused() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_count()
            .returning(|_, _| Box::pin(async { Ok(MAX_ROWS) }));
        rows.expect_insert().never();
        let emitter = RecordingEmitter::default();
        let mut service = KnowledgeTableRowService::new(tables_with(Vec::new()), rows, &emitter);
        let outcome = service
            .create_row(
                CreateTableRowCommand {
                    actor: actor(),
                    organization_id: org(),
                    table_id,
                    user_id: None,
                    values: BTreeMap::new(),
                },
                &mut MockCellReferenceCheck::new(),
                Utc::now(),
            )
            .await
            .unwrap();
        assert_eq!(outcome, Err(TableError::TooManyRows));
    }

    #[tokio::test]
    async fn a_bulk_insert_never_passes_ten_thousand_rows() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_count()
            .returning(|_, _| Box::pin(async { Ok(MAX_ROWS - 1) }));
        rows.expect_insert_many().never();
        let emitter = RecordingEmitter::default();
        let mut service = KnowledgeTableRowService::new(tables_with(Vec::new()), rows, &emitter);
        let batch = RowBatch {
            organization_id: org(),
            table_id,
            created_by: None,
            created_at: Utc::now(),
            rows: vec![
                (TableRowId(generate_uuid_v7()), RowValues::default()),
                (TableRowId(generate_uuid_v7()), RowValues::default()),
            ],
        };
        let outcome = service
            .insert_rows(&batch, &mut MockCellReferenceCheck::new())
            .await
            .unwrap();
        assert_eq!(outcome, Err(TableError::TooManyRows));
        assert!(emitter.recorded().is_empty());
    }

    #[tokio::test]
    async fn a_dead_target_reads_as_missing() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let client = column(table_id, ColumnType::Link(KnowledgeResourceType::Customer));
        let alive = generate_uuid_v7();
        let dead = generate_uuid_v7();
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_labels().times(1).returning(move |_, _| {
            Box::pin(async move {
                Ok(std::collections::HashMap::from([(
                    CellReference::Resource(KnowledgeResource {
                        resource_type: KnowledgeResourceType::Customer,
                        id: alive,
                    }),
                    "Dupont SARL".to_owned(),
                )]))
            })
        });
        let emitter = RecordingEmitter::default();
        let mut service =
            KnowledgeTableRowService::new(MockKnowledgeTableRepository::new(), rows, &emitter);
        let row = |id| TableRow {
            id: TableRowId(generate_uuid_v7()),
            table_id,
            organization_id: org(),
            values: serde_json::Map::from_iter([(
                client.key(),
                json!({"kind": "customer", "id": id}),
            )]),
            created_by: None,
            updated_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let resolved = service
            .resolve(
                org(),
                std::slice::from_ref(&client),
                vec![row(alive), row(dead)],
            )
            .await
            .unwrap();
        assert_eq!(
            resolved[0].labels[&client.id],
            CellLabel::Found("Dupont SARL".to_owned())
        );
        assert_eq!(resolved[1].labels[&client.id], CellLabel::Missing);
    }
}
