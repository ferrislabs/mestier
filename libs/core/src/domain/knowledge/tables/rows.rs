use std::collections::{BTreeMap, HashMap, HashSet};

use chrono::{DateTime, Utc};
use common::{CoreError, UserId, generate_uuid_v7};
use events::EventEmitter;

use crate::{
    OrganizationId,
    domain::knowledge::tables::{
        CellLabel, CreateTableRowCommand, KnowledgeTable, KnowledgeTableId, ListTableRowsCommand,
        MAX_DELETED_ROWS, MAX_ROW_PAGE, MAX_ROWS, ResolvedRow, RowBatch, RowPage, RowSelection,
        RowUpdate, TableColumn, TableColumnId, TableError, TableOutcome, TableRow, TableRowId,
        TableViewId, UpdateTableRowCommand,
        csv::{self, ColumnMapping, CsvSheet, ImportReport, InvalidCsvCell},
        error::{Stop, settle},
        events::{TableRowCreated, TableRowUpdated, changed_columns},
        filter::RowQuery,
        ports::{CellReferenceCheck, KnowledgeTableRepository, KnowledgeTableRowRepository},
        value::{CellError, CellReference, CellValue, RowPatch, RowValues},
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportTargets {
    ByName,
    Positional(Vec<TableColumnId>),
}

#[derive(Debug, Clone)]
pub struct RowImport<'a> {
    pub organization_id: OrganizationId,
    pub table_id: KnowledgeTableId,
    pub user_id: Option<UserId>,
    pub sheet: &'a CsvSheet,
    pub targets: ImportTargets,
    pub now: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableExport {
    pub table_name: String,
    pub view_name: String,
    pub bytes: Vec<u8>,
}

pub struct KnowledgeTableRowService<T, R, E>
where
    T: KnowledgeTableRepository,
    R: KnowledgeTableRowRepository,
    E: EventEmitter,
{
    tables: T,
    rows: R,
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
        self.table(organization_id, table_id).await?;
        self.tables.columns(organization_id, table_id).await
    }

    async fn table(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
    ) -> Result<KnowledgeTable, CoreError> {
        self.tables
            .find_table(organization_id, table_id)
            .await?
            .ok_or(CoreError::NotFound)
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
        let table = self.table(organization_id, command.table_id).await?;
        let resolved = self
            .resolve(organization_id, &columns, vec![row])
            .await?
            .pop()
            .ok_or(Stop::Failed(CoreError::NotFound))?;
        self.emitter.emit(
            organization_id,
            &TableRowCreated {
                table: &table,
                columns: &columns,
                row: &resolved,
            },
        )?;
        Ok(resolved)
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
        let table = self.table(organization_id, command.table_id).await?;
        let columns = self
            .tables
            .columns(organization_id, command.table_id)
            .await?;
        let before = self
            .rows
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
        let resolved = self
            .resolve(organization_id, &columns, vec![row])
            .await?
            .pop()
            .ok_or(Stop::Failed(CoreError::NotFound))?;
        let changed = changed_columns(&columns, &before.values, &resolved.row.values);
        if !changed.is_empty() {
            self.emitter.emit(
                organization_id,
                &TableRowUpdated {
                    table: &table,
                    columns: &columns,
                    row: &resolved,
                    changed,
                },
            )?;
        }
        Ok(resolved)
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
        let mut checked = HashSet::new();
        for (_, values) in &batch.rows {
            for (column, reference) in values.references(&columns) {
                if checked.insert(reference)
                    && !references.exists(organization_id, reference).await?
                {
                    return Err(TableError::InvalidCell {
                        column,
                        reason: CellError::UnknownTarget,
                    }
                    .into());
                }
            }
        }
        Ok(self.rows.insert_many(batch).await?)
    }

    pub async fn import_rows<C>(
        &mut self,
        import: RowImport<'_>,
        references: &mut C,
    ) -> Result<TableOutcome<ImportReport>, CoreError>
    where
        C: CellReferenceCheck,
    {
        settle(self.try_import_rows(import, references).await)
    }

    async fn try_import_rows<C>(
        &mut self,
        import: RowImport<'_>,
        references: &mut C,
    ) -> Result<ImportReport, Stop>
    where
        C: CellReferenceCheck,
    {
        let organization_id = import.organization_id;
        let columns = self.lock(organization_id, import.table_id).await?;
        let mapping = match &import.targets {
            ImportTargets::ByName => ColumnMapping::by_name(import.sheet, &columns)?,
            ImportTargets::Positional(ids) => ColumnMapping::by_position(ids),
        };
        let mut plan = csv::convert(import.sheet, &columns, &mapping);
        let mut known: HashMap<CellReference, bool> = HashMap::new();
        for row in &mut plan.rows {
            for (column_id, reference) in row.values.references(&columns) {
                let exists = match known.get(&reference) {
                    Some(exists) => *exists,
                    None => {
                        let exists = references.exists(organization_id, reference).await?;
                        known.insert(reference, exists);
                        exists
                    }
                };
                if !exists {
                    row.values.remove(column_id);
                    plan.invalid_cells.push(InvalidCsvCell {
                        row: row.line,
                        column: columns
                            .iter()
                            .find(|column| column.id == column_id)
                            .map(|column| column.name.as_str().to_owned())
                            .unwrap_or_default(),
                        value: reference.id().to_string(),
                        reason: CellError::UnknownTarget,
                    });
                }
            }
        }
        let batch = RowBatch {
            organization_id,
            table_id: import.table_id,
            created_by: import.user_id,
            created_at: import.now,
            rows: plan
                .rows
                .into_iter()
                .map(|row| (TableRowId(generate_uuid_v7()), row.values))
                .collect(),
        };
        let created = self.try_insert_rows(&batch, references).await?;
        Ok(ImportReport::new(
            created,
            mapping.ignored().to_vec(),
            plan.invalid_cells,
        ))
    }

    pub async fn export_view(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        view_id: TableViewId,
    ) -> Result<TableOutcome<TableExport>, CoreError> {
        settle(
            self.try_export_view(organization_id, table_id, view_id)
                .await,
        )
    }

    async fn try_export_view(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        view_id: TableViewId,
    ) -> Result<TableExport, Stop> {
        let table = self.table(organization_id, table_id).await?;
        let columns = self.tables.columns(organization_id, table_id).await?;
        let view = self
            .tables
            .views(organization_id, table_id)
            .await?
            .into_iter()
            .find(|view| view.id == view_id)
            .ok_or(CoreError::NotFound)?;
        let mut rows = Vec::new();
        let mut offset = 0;
        loop {
            let query = RowQuery::build(
                &columns,
                &view.config.filters,
                &view.config.sorts,
                MAX_ROW_PAGE,
                offset,
            )?;
            let (page, total) = self.rows.list(organization_id, table_id, &query).await?;
            let fetched = u64::try_from(page.len()).unwrap_or(u64::MAX);
            rows.extend(self.resolve(organization_id, &columns, page).await?);
            offset = offset.saturating_add(fetched);
            if fetched == 0 || offset >= total || offset >= MAX_ROWS {
                break;
            }
        }
        let visible: Vec<&TableColumn> = columns
            .iter()
            .filter(|column| !view.config.hidden.contains(&column.id))
            .collect();
        let bytes =
            csv::write(&visible, &rows).map_err(|error| CoreError::Internal(error.to_string()))?;
        Ok(TableExport {
            table_name: table.name.into_inner(),
            view_name: view.name.into_inner(),
            bytes,
        })
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
    use serde_json::{Value, json};

    use super::*;
    use crate::{
        KnowledgeResource, KnowledgeResourceType,
        domain::knowledge::tables::{
            KnowledgeTable, TableColumnId, TableItemName, TableView, ViewConfig, ViewKind,
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

    fn named(table_id: KnowledgeTableId, name: &str, column_type: ColumnType) -> TableColumn {
        TableColumn {
            name: TableItemName::try_from(name).unwrap(),
            is_primary: name == "Nom",
            ..column(table_id, column_type)
        }
    }

    fn stored(table_id: KnowledgeTableId, values: serde_json::Map<String, Value>) -> TableRow {
        TableRow {
            id: TableRowId(generate_uuid_v7()),
            table_id,
            organization_id: org(),
            values,
            created_by: None,
            updated_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn rows_holding(row: TableRow) -> MockKnowledgeTableRowRepository {
        let mut rows = MockKnowledgeTableRowRepository::new();
        let found = row.clone();
        rows.expect_find().returning(move |_, _, _| {
            let found = found.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
        rows.expect_patch().returning(move |update| {
            let mut patched = row.clone();
            patched.values.extend(update.set.clone());
            for key in &update.cleared {
                patched.values.remove(key);
            }
            Box::pin(async move { Ok(Some(patched)) })
        });
        rows
    }

    fn update(
        table_id: KnowledgeTableId,
        id: TableRowId,
        values: Vec<(TableColumnId, Value)>,
    ) -> UpdateTableRowCommand {
        UpdateTableRowCommand {
            actor: actor(),
            organization_id: org(),
            table_id,
            id,
            user_id: None,
            values: values.into_iter().collect(),
        }
    }

    #[tokio::test]
    async fn creating_a_row_emits_one_created_event_naming_its_fields() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let name = named(table_id, "Nom", ColumnType::Text);
        let done = named(table_id, "Traité", ColumnType::Checkbox);
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_count()
            .returning(|_, _| Box::pin(async { Ok(0) }));
        rows.expect_insert().times(1).returning(|row| {
            let row = row.clone();
            Box::pin(async move { Ok(row) })
        });
        let emitter = RecordingEmitter::default();
        let mut service = KnowledgeTableRowService::new(
            tables_with(vec![name.clone(), done.clone()]),
            rows,
            &emitter,
        );

        let created = service
            .create_row(
                CreateTableRowCommand {
                    actor: actor(),
                    organization_id: org(),
                    table_id,
                    user_id: None,
                    values: BTreeMap::from([(name.id, json!("Villa")), (done.id, json!(true))]),
                },
                &mut MockCellReferenceCheck::new(),
                Utc::now(),
            )
            .await
            .unwrap()
            .unwrap();

        let event = emitter.only("table_row.created");
        assert_eq!(emitter.recorded().len(), 1);
        assert_eq!(event.version, 1);
        assert_eq!(event.payload["table"]["name"], json!("Chantiers"));
        assert_eq!(event.payload["row"]["id"], json!(created.row.id.0));
        assert_eq!(event.payload["row"]["title"], json!("Villa"));
        assert_eq!(
            event.payload["row"]["fields"],
            json!({"Nom": "Villa", "Traité": "Oui"})
        );
    }

    #[tokio::test]
    async fn updating_two_cells_emits_one_updated_event_naming_both() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let name = named(table_id, "Nom", ColumnType::Text);
        let done = named(table_id, "Traité", ColumnType::Checkbox);
        let note = named(table_id, "Note", ColumnType::LongText);
        let row = stored(
            table_id,
            serde_json::Map::from_iter([
                (name.key(), json!("Villa")),
                (note.key(), json!("à revoir")),
            ]),
        );
        let emitter = RecordingEmitter::default();
        let mut service = KnowledgeTableRowService::new(
            tables_with(vec![name.clone(), done.clone(), note.clone()]),
            rows_holding(row.clone()),
            &emitter,
        );

        service
            .update_row(
                update(
                    table_id,
                    row.id,
                    vec![
                        (name.id, json!("Villa")),
                        (done.id, json!(true)),
                        (note.id, Value::Null),
                    ],
                ),
                &mut MockCellReferenceCheck::new(),
                Utc::now(),
            )
            .await
            .unwrap()
            .unwrap();

        let event = emitter.only("table_row.updated");
        assert_eq!(emitter.recorded().len(), 1);
        assert_eq!(event.payload["changed"], json!(["Traité", "Note"]));
        assert_eq!(
            event.payload["row"]["fields"],
            json!({"Nom": "Villa", "Traité": "Oui", "Note": null})
        );
    }

    #[tokio::test]
    async fn an_update_that_changes_nothing_emits_nothing() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let name = named(table_id, "Nom", ColumnType::Text);
        let done = named(table_id, "Traité", ColumnType::Checkbox);
        let row = stored(
            table_id,
            serde_json::Map::from_iter([(name.key(), json!("Villa"))]),
        );
        let emitter = RecordingEmitter::default();
        let mut service = KnowledgeTableRowService::new(
            tables_with(vec![name.clone(), done.clone()]),
            rows_holding(row.clone()),
            &emitter,
        );

        let outcome = service
            .update_row(
                update(
                    table_id,
                    row.id,
                    vec![(name.id, json!(" Villa ")), (done.id, Value::Null)],
                ),
                &mut MockCellReferenceCheck::new(),
                Utc::now(),
            )
            .await
            .unwrap();

        assert!(outcome.is_ok());
        assert!(emitter.recorded().is_empty());
    }

    #[tokio::test]
    async fn a_bulk_insert_emits_nothing() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_count()
            .returning(|_, _| Box::pin(async { Ok(0) }));
        rows.expect_insert_many().times(1).returning(|batch| {
            let added = batch.rows.len() as u64;
            Box::pin(async move { Ok(added) })
        });
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

        assert_eq!(outcome, Ok(2));
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

    fn sheet(text: &str) -> CsvSheet {
        CsvSheet::read(text.as_bytes()).unwrap()
    }

    fn import(table_id: KnowledgeTableId, sheet: &CsvSheet) -> RowImport<'_> {
        RowImport {
            organization_id: org(),
            table_id,
            user_id: None,
            sheet,
            targets: ImportTargets::ByName,
            now: Utc::now(),
        }
    }

    #[tokio::test]
    async fn an_import_reports_dead_targets_and_unknown_columns_and_emits_nothing() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let name = named(table_id, "Nom", ColumnType::Text);
        let client = named(
            table_id,
            "Client",
            ColumnType::Link(KnowledgeResourceType::Customer),
        );
        let known = generate_uuid_v7();
        let gone = generate_uuid_v7();
        let client_id = client.id;
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_count()
            .returning(|_, _| Box::pin(async { Ok(0) }));
        rows.expect_insert_many()
            .times(1)
            .withf(move |batch| {
                batch.rows.len() == 2
                    && batch.rows[0].1.get(client_id).is_some()
                    && batch.rows[1].1.get(client_id).is_none()
            })
            .returning(|batch| {
                let added = batch.rows.len() as u64;
                Box::pin(async move { Ok(added) })
            });
        let mut references = MockCellReferenceCheck::new();
        references.expect_exists().returning(move |_, reference| {
            let found = reference.id() == known;
            Box::pin(async move { Ok(found) })
        });
        let emitter = RecordingEmitter::default();
        let mut service =
            KnowledgeTableRowService::new(tables_with(vec![name, client]), rows, &emitter);
        let read = sheet(&format!(
            "Nom;Client;Remarque\nVilla;{known};a\nGrange;{gone};b\n"
        ));

        let report = service
            .import_rows(import(table_id, &read), &mut references)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(report.rows_created, 2);
        assert_eq!(report.ignored_columns, ["Remarque"]);
        assert_eq!(
            report.invalid_cells,
            vec![InvalidCsvCell {
                row: 3,
                column: "Client".to_owned(),
                value: gone.to_string(),
                reason: CellError::UnknownTarget,
            }]
        );
        assert!(emitter.recorded().is_empty());
    }

    #[tokio::test]
    async fn an_import_past_the_row_cap_writes_nothing() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_count()
            .returning(|_, _| Box::pin(async { Ok(MAX_ROWS - 1) }));
        rows.expect_insert_many().never();
        let emitter = RecordingEmitter::default();
        let mut service = KnowledgeTableRowService::new(
            tables_with(vec![named(table_id, "Nom", ColumnType::Text)]),
            rows,
            &emitter,
        );
        let read = sheet("Nom\nA\nB\n");

        let outcome = service
            .import_rows(import(table_id, &read), &mut MockCellReferenceCheck::new())
            .await
            .unwrap();

        assert_eq!(outcome, Err(TableError::TooManyRows));
    }

    #[tokio::test]
    async fn an_export_pages_through_its_view_and_keeps_its_visible_columns() {
        let table_id = KnowledgeTableId(generate_uuid_v7());
        let name = named(table_id, "Nom", ColumnType::Text);
        let secret = named(table_id, "Secret", ColumnType::Text);
        let view = TableView {
            id: TableViewId(generate_uuid_v7()),
            table_id,
            organization_id: org(),
            name: TableItemName::try_from("Grille").unwrap(),
            kind: ViewKind::Grid,
            config: ViewConfig {
                hidden: vec![secret.id],
                ..ViewConfig::default()
            },
            position: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        let view_id = view.id;
        let mut tables = tables_with(vec![name.clone(), secret.clone()]);
        tables.expect_views().returning(move |_, _| {
            let views = vec![view.clone()];
            Box::pin(async move { Ok(views) })
        });
        let mut rows = MockKnowledgeTableRowRepository::new();
        let (name_key, secret_key) = (name.key(), secret.key());
        rows.expect_list().times(2).returning(move |_, _, query| {
            let page: Vec<TableRow> = (query.offset..(query.offset + query.limit).min(150))
                .map(|n| {
                    stored(
                        table_id,
                        serde_json::Map::from_iter([
                            (name_key.clone(), json!(format!("Ligne {n}"))),
                            (secret_key.clone(), json!("x")),
                        ]),
                    )
                })
                .collect();
            Box::pin(async move { Ok((page, 150)) })
        });
        let emitter = RecordingEmitter::default();
        let mut service = KnowledgeTableRowService::new(tables, rows, &emitter);

        let export = service
            .export_view(org(), table_id, view_id)
            .await
            .unwrap()
            .unwrap();

        let text = std::str::from_utf8(&export.bytes[csv::UTF8_BOM.len()..]).unwrap();
        let lines: Vec<&str> = text.split("\r\n").filter(|line| !line.is_empty()).collect();
        assert_eq!(lines.len(), 151);
        assert_eq!(lines[0], "Nom");
        assert_eq!(lines[150], "Ligne 149");
        assert_eq!(export.table_name, "Chantiers");
        assert_eq!(export.view_name, "Grille");
    }
}
