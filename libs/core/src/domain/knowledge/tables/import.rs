use chrono::{DateTime, Utc};
use common::CoreError;
use events::EventEmitter;

use crate::domain::knowledge::tables::{
    CreateKnowledgeTableCommand, ImportKnowledgeTableCommand, ImportTableRowsCommand, TableOutcome,
    TableSchema,
    csv::{CsvSheet, ImportReport, plan_columns},
    ports::{CellReferenceCheck, KnowledgeTableRepository, KnowledgeTableRowRepository},
    rows::{ImportTargets, KnowledgeTableRowService, RowImport},
    service::KnowledgeTableService,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ImportedTable {
    pub schema: TableSchema,
    pub report: ImportReport,
}

pub async fn import_new_table<T, R, E, C>(
    tables: T,
    rows: R,
    emitter: E,
    references: &mut C,
    command: ImportKnowledgeTableCommand,
    now: DateTime<Utc>,
) -> Result<TableOutcome<ImportedTable>, CoreError>
where
    T: KnowledgeTableRepository,
    R: KnowledgeTableRowRepository,
    E: EventEmitter,
    C: CellReferenceCheck,
{
    let sheet = match CsvSheet::read(&command.file) {
        Ok(sheet) => sheet,
        Err(error) => return Ok(Err(error)),
    };
    let plan = match plan_columns(&sheet) {
        Ok(plan) => plan,
        Err(error) => return Ok(Err(error)),
    };
    let organization_id = command.organization_id;
    let mut service = KnowledgeTableService::new(tables, rows);
    let schema = match service
        .create_table_with_columns(
            CreateKnowledgeTableCommand {
                actor: command.actor,
                organization_id,
                user_id: command.user_id,
                name: command.name,
                description: None,
            },
            &plan,
            now,
        )
        .await?
    {
        Ok(schema) => schema,
        Err(error) => return Ok(Err(error)),
    };
    let (tables, rows) = service.into_parts();
    let imported = KnowledgeTableRowService::new(tables, rows, emitter)
        .import_rows(
            RowImport {
                organization_id,
                table_id: schema.table.id,
                user_id: command.user_id,
                sheet: &sheet,
                targets: ImportTargets::Positional(
                    schema.columns.iter().map(|column| column.id).collect(),
                ),
                now,
            },
            references,
        )
        .await?;
    match imported {
        Ok(report) => Ok(Ok(ImportedTable { schema, report })),
        Err(error) => Err(CoreError::Internal(format!(
            "a new table refused its imported rows: {error}"
        ))),
    }
}

pub async fn import_into_table<T, R, E, C>(
    tables: T,
    rows: R,
    emitter: E,
    references: &mut C,
    command: ImportTableRowsCommand,
    now: DateTime<Utc>,
) -> Result<TableOutcome<ImportReport>, CoreError>
where
    T: KnowledgeTableRepository,
    R: KnowledgeTableRowRepository,
    E: EventEmitter,
    C: CellReferenceCheck,
{
    let sheet = match CsvSheet::read(&command.file) {
        Ok(sheet) => sheet,
        Err(error) => return Ok(Err(error)),
    };
    KnowledgeTableRowService::new(tables, rows, emitter)
        .import_rows(
            RowImport {
                organization_id: command.organization_id,
                table_id: command.table_id,
                user_id: command.user_id,
                sheet: &sheet,
                targets: ImportTargets::ByName,
                now,
            },
            references,
        )
        .await
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use bytes::Bytes;
    use common::generate_uuid_v7;
    use events::testing::RecordingEmitter;
    use serde_json::{Map, Value, json};

    use super::*;
    use crate::{
        OrganizationId,
        domain::knowledge::tables::{
            ColumnKind, TableColumn, TableError,
            ports::{
                MockCellReferenceCheck, MockKnowledgeTableRepository,
                MockKnowledgeTableRowRepository,
            },
        },
    };

    fn command(file: Bytes) -> ImportKnowledgeTableCommand {
        ImportKnowledgeTableCommand {
            actor: authz::Subject::new(authz::SubjectKind::User, "someone"),
            organization_id: OrganizationId(generate_uuid_v7()),
            user_id: None,
            name: "Articles".to_owned(),
            file,
        }
    }

    #[tokio::test]
    async fn a_french_excel_file_becomes_a_typed_table_without_any_event() {
        let created: Arc<Mutex<Vec<TableColumn>>> = Arc::default();
        let mut tables = MockKnowledgeTableRepository::new();
        tables
            .expect_lock_organization()
            .returning(|_| Box::pin(async { Ok(()) }));
        tables
            .expect_count_tables()
            .returning(|_| Box::pin(async { Ok(0) }));
        tables
            .expect_list_tables()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        tables.expect_insert_table().returning(|table| {
            let table = table.clone();
            Box::pin(async move { Ok(table) })
        });
        tables.expect_insert_view().returning(|view| {
            let view = view.clone();
            Box::pin(async move { Ok(view) })
        });
        tables
            .expect_lock_table()
            .returning(|_, _| Box::pin(async { Ok(true) }));
        let pushed = Arc::clone(&created);
        tables.expect_insert_column().returning(move |column| {
            pushed.lock().unwrap().push(column.clone());
            let column = column.clone();
            Box::pin(async move { Ok(column) })
        });
        let listed = Arc::clone(&created);
        tables.expect_columns().returning(move |_, _| {
            let columns = listed.lock().unwrap().clone();
            Box::pin(async move { Ok(columns) })
        });
        let inserted: Arc<Mutex<Vec<Map<String, Value>>>> = Arc::default();
        let kept = Arc::clone(&inserted);
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_count()
            .returning(|_, _| Box::pin(async { Ok(0) }));
        rows.expect_insert_many().times(1).returning(move |batch| {
            kept.lock()
                .unwrap()
                .extend(batch.rows.iter().map(|(_, values)| values.to_json()));
            let added = batch.rows.len() as u64;
            Box::pin(async move { Ok(added) })
        });
        let emitter = RecordingEmitter::default();
        let file = Bytes::from_static(
            b"D\xE9signation;Prix unitaire;Livr\xE9 le\r\nVis \xE0 b\xE9ton;12,50;05/11/2026\r\n",
        );

        let imported = import_new_table(
            tables,
            rows,
            &emitter,
            &mut MockCellReferenceCheck::new(),
            command(file),
            Utc::now(),
        )
        .await
        .unwrap()
        .unwrap();

        let columns: Vec<_> = imported
            .schema
            .columns
            .iter()
            .map(|column| (column.name.as_str(), column.kind(), column.is_primary))
            .collect();
        assert_eq!(
            columns,
            vec![
                ("Désignation", ColumnKind::Text, true),
                ("Prix unitaire", ColumnKind::Number, false),
                ("Livré le", ColumnKind::Date, false),
            ]
        );
        let keys: Vec<String> = imported
            .schema
            .columns
            .iter()
            .map(TableColumn::key)
            .collect();
        assert_eq!(
            inserted.lock().unwrap().as_slice(),
            [Map::from_iter([
                (keys[0].clone(), json!("Vis à béton")),
                (keys[1].clone(), json!(12.5)),
                (keys[2].clone(), json!("2026-11-05")),
            ])]
        );
        assert_eq!(imported.report.rows_created, 1);
        assert!(emitter.recorded().is_empty());
    }

    #[tokio::test]
    async fn a_file_of_fifty_one_columns_creates_nothing() {
        let mut tables = MockKnowledgeTableRepository::new();
        tables.expect_insert_table().never();
        let header: String = (0..51).map(|n| format!("C{n};")).collect();

        let outcome = import_new_table(
            tables,
            MockKnowledgeTableRowRepository::new(),
            &RecordingEmitter::default(),
            &mut MockCellReferenceCheck::new(),
            command(Bytes::from(header)),
            Utc::now(),
        )
        .await
        .unwrap();

        assert_eq!(outcome, Err(TableError::CsvTooManyColumns));
    }
}
