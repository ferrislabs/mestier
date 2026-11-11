use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::{ActionContext, failure};
use crate::domain::knowledge::tables::{
    CreateTableRowCommand, FilterOperator, GetKnowledgeTableCommand, KnowledgeTableId,
    ListTableRowsCommand, ResolvedRow, RowFilter, RowSelection, SelectOptions, TableColumn,
    TableColumnId, TableError, TableRowId, UpdateTableRowCommand, column::ColumnType,
    events::row_payload, name::MAX_NAME_CHARS,
};

const FOUND_ROWS: u64 = 50;

pub(super) async fn run(kind: &str, context: &ActionContext<'_>) -> Option<Result<Value, String>> {
    Some(match kind {
        "mestier.table_row.create" => create_row(context).await,
        "mestier.table_row.update" => update_row(context).await,
        "mestier.table_row.find" => find_rows(context).await,
        _ => return None,
    })
}

fn table_id(context: &ActionContext<'_>) -> Result<KnowledgeTableId, String> {
    context.uuid("table_id").map(KnowledgeTableId)
}

fn named_values<'a>(context: &'a ActionContext<'_>) -> Result<&'a Map<String, Value>, String> {
    match context.config.get("values") {
        Some(Value::Object(values)) => Ok(values),
        _ => Err("`values` must resolve to an object".to_owned()),
    }
}

async fn columns(
    context: &ActionContext<'_>,
    table_id: KnowledgeTableId,
) -> Result<Vec<TableColumn>, String> {
    context
        .usecase
        .get_knowledge_table(GetKnowledgeTableCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            id: table_id,
        })
        .await
        .map(|schema| schema.columns)
        .map_err(failure)
}

fn column_named<'a>(columns: &'a [TableColumn], name: &str) -> Result<&'a TableColumn, String> {
    columns
        .iter()
        .find(|column| column.name.same_as(name))
        .ok_or_else(|| format!("`{}`: no such column in the table", name.trim()))
}

fn option_id(options: &SelectOptions, raw: &Value) -> Value {
    raw.as_str()
        .map(|label| label.trim().to_lowercase())
        .and_then(|label| {
            options
                .as_slice()
                .iter()
                .find(|option| option.label.to_lowercase() == label)
        })
        .map_or_else(
            || raw.clone(),
            |option| Value::String(option.id.to_string()),
        )
}

fn typed(column_type: &ColumnType, raw: &Value) -> Value {
    match (column_type, raw) {
        (ColumnType::SingleSelect(options), _) => option_id(options, raw),
        (ColumnType::MultiSelect(options), Value::Array(items)) => {
            Value::Array(items.iter().map(|item| option_id(options, item)).collect())
        }
        (ColumnType::MultiSelect(options), Value::String(_)) => {
            Value::Array(vec![option_id(options, raw)])
        }
        (
            ColumnType::Number(_) | ColumnType::Currency | ColumnType::Checkbox,
            Value::String(text),
        ) => serde_json::from_str::<Value>(text.trim())
            .ok()
            .filter(|parsed| parsed.is_number() || parsed.is_boolean())
            .unwrap_or_else(|| raw.clone()),
        _ => raw.clone(),
    }
}

fn by_column_id(
    columns: &[TableColumn],
    named: &Map<String, Value>,
) -> Result<BTreeMap<TableColumnId, Value>, String> {
    named
        .iter()
        .map(|(name, raw)| {
            let column = column_named(columns, name)?;
            Ok((column.id, typed(&column.column_type, raw)))
        })
        .collect()
}

fn refusal(columns: &[TableColumn], error: TableError) -> String {
    let name_of = |id: TableColumnId| {
        columns
            .iter()
            .find(|column| column.id == id)
            .map_or_else(|| id.to_string(), |column| column.name.as_str().to_owned())
    };
    match error {
        TableError::InvalidCell { column, reason } => {
            format!("`values.{}`: {reason}", name_of(column))
        }
        TableError::UnknownColumn(column) => {
            format!("`values.{}`: no such column in the table", name_of(column))
        }
        TableError::InvalidFilter(column) | TableError::InvalidFilterValue(column) => {
            format!("`value` does not fit the column `{}`", name_of(column))
        }
        other => other.to_string(),
    }
}

fn row_json(columns: &[TableColumn], resolved: &ResolvedRow) -> Value {
    let mut output = row_payload(columns, resolved);
    if let Value::Object(fields) = &mut output {
        fields.insert("table_id".to_owned(), json!(resolved.row.table_id.0));
        fields.insert("created_at".to_owned(), json!(resolved.row.created_at));
        fields.insert("updated_at".to_owned(), json!(resolved.row.updated_at));
    }
    output
}

async fn create_row(context: &ActionContext<'_>) -> Result<Value, String> {
    let table_id = table_id(context)?;
    let named = named_values(context)?;
    let columns = columns(context, table_id).await?;
    let values = by_column_id(&columns, named)?;

    context
        .usecase
        .create_table_row(CreateTableRowCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            table_id,
            user_id: None,
            values,
        })
        .await
        .map_err(failure)?
        .map(|row| row_json(&columns, &row))
        .map_err(|error| refusal(&columns, error))
}

async fn update_row(context: &ActionContext<'_>) -> Result<Value, String> {
    let table_id = table_id(context)?;
    let id = context.uuid("row_id").map(TableRowId)?;
    let named = named_values(context)?;
    let columns = columns(context, table_id).await?;
    let values = by_column_id(&columns, named)?;

    context
        .usecase
        .update_table_row(UpdateTableRowCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            table_id,
            id,
            user_id: None,
            values,
        })
        .await
        .map_err(failure)?
        .map(|row| row_json(&columns, &row))
        .map_err(|error| refusal(&columns, error))
}

fn equals(column: &TableColumn, raw: &Value) -> Result<RowFilter, String> {
    let blank = raw.is_null() || raw.as_str().is_some_and(|text| text.trim().is_empty());
    let value = typed(&column.column_type, raw);
    let operator = match &column.column_type {
        ColumnType::Checkbox => match value {
            Value::Bool(true) => FilterOperator::Checked,
            Value::Bool(false) | Value::Null => FilterOperator::Unchecked,
            _ if blank => FilterOperator::Unchecked,
            _ => return Err("`value` must be true or false for a checkbox column".to_owned()),
        },
        _ if blank => FilterOperator::IsEmpty,
        ColumnType::Number(_) | ColumnType::Currency => FilterOperator::Eq,
        ColumnType::Date => FilterOperator::On,
        ColumnType::MultiSelect(_) => FilterOperator::AnyOf,
        ColumnType::Text
        | ColumnType::LongText
        | ColumnType::SingleSelect(_)
        | ColumnType::Link(_)
        | ColumnType::File
        | ColumnType::Member => FilterOperator::Is,
    };
    let compared = !blank && !matches!(column.column_type, ColumnType::Checkbox);
    Ok(RowFilter {
        column_id: column.id,
        operator,
        value: compared.then_some(value),
    })
}

async fn find_rows(context: &ActionContext<'_>) -> Result<Value, String> {
    let table_id = table_id(context)?;
    let name = context.filled("column", MAX_NAME_CHARS)?;
    let raw = context.config.get("value").unwrap_or(&Value::Null);
    let columns = columns(context, table_id).await?;
    let filter = equals(column_named(&columns, name)?, raw)?;

    let page = context
        .usecase
        .list_table_rows(ListTableRowsCommand {
            actor: context.actor(),
            organization_id: context.organization_id,
            table_id,
            selection: RowSelection::AdHoc {
                filters: vec![filter],
                sorts: Vec::new(),
            },
            limit: FOUND_ROWS,
            offset: 0,
        })
        .await
        .map_err(failure)?
        .map_err(|error| refusal(&columns, error))?;

    Ok(Value::Array(
        page.rows
            .iter()
            .map(|row| row_json(&columns, row))
            .collect(),
    ))
}

#[cfg(test)]
mod knowledge {
    use authz::Subject;
    use chrono::Utc;
    use common::{OrganizationId, generate_uuid_v7};
    use serde_json::{Value, json};
    use sqlx::PgPool;

    use super::super::clients::tests_clients::{config, failure, produced, registry, run};
    use super::*;
    use crate::application::test_support::automation_pool;
    use crate::application::{MestierUseCase, default_authorizer};
    use crate::domain::automation::workflow::{
        CreateWorkflowCommand, Edge, Graph, PlacedConnector, PlacedTrigger,
        SaveWorkflowVersionCommand, TriggerKind,
    };
    use crate::domain::knowledge::tables::{
        ColumnConfig, ColumnKind, CreateKnowledgeTableCommand, CreateTableColumnCommand,
        GetTableRowCommand, RowBatch, RowValues, SelectOptionInput, TableItemName, TableSchema,
        ports::MockCellReferenceCheck, rows::KnowledgeTableRowService,
    };
    use crate::infrastructure::automation::TransactionalEventEmitter;
    use crate::infrastructure::automation::connectors::ConnectorRegistry;
    use crate::infrastructure::automation::connectors::test_support::assert_same_shape;
    use crate::infrastructure::automation::connectors::world_support;
    use crate::infrastructure::automation::postgres::dispatcher::DISPATCH_LOCK;
    use crate::infrastructure::automation::postgres::run::RUN_CLAIM_LOCK;
    use crate::infrastructure::automation::worker::WorkerSchedule;
    use crate::infrastructure::knowledge::postgres::{
        PgKnowledgeTableRepository, PgKnowledgeTableRowRepository,
    };
    use crate::infrastructure::postgres::with_tx_emitting;
    use crate::infrastructure::realtime::EventHub;

    fn column(name: &str, column_type: ColumnType) -> TableColumn {
        TableColumn {
            id: TableColumnId(generate_uuid_v7()),
            table_id: KnowledgeTableId(generate_uuid_v7()),
            organization_id: OrganizationId(generate_uuid_v7()),
            name: TableItemName::try_from(name).unwrap(),
            column_type,
            position: 0,
            is_primary: false,
            created_at: Utc::now(),
        }
    }

    fn status_column() -> TableColumn {
        let options = SelectOptions::parse(&[
            SelectOptionInput {
                id: None,
                label: "En cours".to_owned(),
                color: None,
            },
            SelectOptionInput {
                id: None,
                label: "Terminé".to_owned(),
                color: None,
            },
        ])
        .unwrap();
        column("Statut", ColumnType::MultiSelect(options))
    }

    #[test]
    fn values_by_name_are_matched_without_case_and_choices_by_label() {
        let done = column("Traité", ColumnType::Checkbox);
        let status = status_column();
        let ongoing = status.column_type.options().unwrap().as_slice()[0].id;
        let columns = [done.clone(), status.clone()];
        let named = Map::from_iter([
            ("traité".to_owned(), json!(true)),
            (" STATUT ".to_owned(), json!("en cours")),
        ]);

        let values = by_column_id(&columns, &named).unwrap();

        assert_eq!(values[&done.id], json!(true));
        assert_eq!(values[&status.id], json!([ongoing.to_string()]));
    }

    #[test]
    fn an_unknown_column_name_fails_the_step_naming_it() {
        let columns = [column("Traité", ColumnType::Checkbox)];
        let named = Map::from_iter([("Fini".to_owned(), json!(true))]);

        assert_eq!(
            by_column_id(&columns, &named),
            Err("`Fini`: no such column in the table".to_owned())
        );
    }

    #[test]
    fn a_refused_cell_is_reported_by_column_name() {
        let done = column("Traité", ColumnType::Checkbox);

        let message = refusal(
            std::slice::from_ref(&done),
            TableError::InvalidCell {
                column: done.id,
                reason: crate::domain::knowledge::tables::CellError::NotBoolean,
            },
        );

        assert_eq!(message, "`values.Traité`: expected true or false");
    }

    #[test]
    fn find_compares_with_the_equality_of_each_type() {
        let cases = [
            (
                column("Nom", ColumnType::Text),
                json!("Villa"),
                FilterOperator::Is,
            ),
            (
                column("Montant", ColumnType::Currency),
                json!("1200"),
                FilterOperator::Eq,
            ),
            (
                column("Le", ColumnType::Date),
                json!("2026-11-05"),
                FilterOperator::On,
            ),
            (status_column(), json!("En cours"), FilterOperator::AnyOf),
            (
                column("Nom", ColumnType::Text),
                json!(" "),
                FilterOperator::IsEmpty,
            ),
            (
                column("Traité", ColumnType::Checkbox),
                json!("true"),
                FilterOperator::Checked,
            ),
            (
                column("Traité", ColumnType::Checkbox),
                json!(false),
                FilterOperator::Unchecked,
            ),
        ];
        for (column, raw, operator) in cases {
            assert_eq!(equals(&column, &raw).unwrap().operator, operator, "{raw}");
        }
        assert_eq!(
            equals(&column("Montant", ColumnType::Currency), &json!("1200"))
                .unwrap()
                .value,
            Some(json!(1200))
        );
        assert!(equals(&column("Traité", ColumnType::Checkbox), &json!("peut-être")).is_err());
    }

    fn usecase(pool: &PgPool) -> MestierUseCase {
        MestierUseCase::new(pool.clone(), default_authorizer(), EventHub::new())
    }

    async fn chantiers(usecase: &MestierUseCase, org: OrganizationId) -> TableSchema {
        let schema = usecase
            .create_knowledge_table(CreateKnowledgeTableCommand {
                actor: Subject::system(),
                organization_id: org,
                user_id: None,
                name: "Chantiers".to_owned(),
                description: None,
            })
            .await
            .unwrap()
            .unwrap();
        for (name, kind, config) in [
            ("Traité", ColumnKind::Checkbox, ColumnConfig::default()),
            (
                "Statut",
                ColumnKind::SingleSelect,
                ColumnConfig {
                    options: Some(
                        ["En cours", "Terminé"]
                            .into_iter()
                            .map(|label| SelectOptionInput {
                                id: None,
                                label: label.to_owned(),
                                color: None,
                            })
                            .collect(),
                    ),
                    ..ColumnConfig::default()
                },
            ),
            ("Note", ColumnKind::LongText, ColumnConfig::default()),
        ] {
            usecase
                .create_table_column(CreateTableColumnCommand {
                    actor: Subject::system(),
                    organization_id: org,
                    table_id: schema.table.id,
                    name: name.to_owned(),
                    kind,
                    config,
                })
                .await
                .unwrap()
                .unwrap();
        }
        usecase
            .get_knowledge_table(GetKnowledgeTableCommand {
                actor: Subject::system(),
                organization_id: org,
                id: schema.table.id,
            })
            .await
            .unwrap()
    }

    async fn events_named(pool: &PgPool, org: OrganizationId, name: &str) -> Vec<Value> {
        sqlx::query_scalar::<_, Value>(
            "SELECT payload FROM automation.event WHERE org_id = $1 AND name = $2 ORDER BY occurred_at",
        )
        .bind(org.0)
        .bind(name)
        .fetch_all(pool)
        .await
        .unwrap()
    }

    async fn row_events(pool: &PgPool, org: OrganizationId) -> i64 {
        sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM automation.event WHERE org_id = $1 AND name LIKE 'table_row.%'",
        )
        .bind(org.0)
        .fetch_one(pool)
        .await
        .unwrap()
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_created_row_records_one_created_event_with_its_table_title_and_fields() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "tables-created").await;
        let schema = chantiers(&usecase(&pool), org).await;
        let registry = registry(&pool);

        let created = produced(
            run(
                &registry,
                "mestier.table_row.create",
                org,
                &config(&[
                    ("table_id", json!(schema.table.id.to_string())),
                    (
                        "values",
                        json!({"nom": "Villa Dupont", "Statut": "en cours", "Traité": false}),
                    ),
                ]),
            )
            .await,
        );

        assert_same_shape(&created, "mestier.table_row.create");
        let events = events_named(&pool, org, "table_row.created").await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["table"]["name"], json!("Chantiers"));
        assert_eq!(events[0]["row"]["id"], created["id"]);
        assert_eq!(events[0]["row"]["title"], json!("Villa Dupont"));
        assert_eq!(
            events[0]["row"]["fields"],
            json!({"Nom": "Villa Dupont", "Traité": "Non", "Statut": "En cours", "Note": null})
        );
        assert_eq!(row_events(&pool, org).await, 1);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn updating_two_cells_records_the_two_names_and_a_no_op_records_nothing() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "tables-updated").await;
        let schema = chantiers(&usecase(&pool), org).await;
        let registry = registry(&pool);
        let table_id = json!(schema.table.id.to_string());
        let created = produced(
            run(
                &registry,
                "mestier.table_row.create",
                org,
                &config(&[
                    ("table_id", table_id.clone()),
                    ("values", json!({"Nom": "Villa", "Note": "à revoir"})),
                ]),
            )
            .await,
        );
        let update = |values: Value| {
            config(&[
                ("table_id", table_id.clone()),
                ("row_id", created["id"].clone()),
                ("values", values),
            ])
        };

        let updated = produced(
            run(
                &registry,
                "mestier.table_row.update",
                org,
                &update(json!({"Nom": "Villa", "Traité": true, "Note": null})),
            )
            .await,
        );
        produced(
            run(
                &registry,
                "mestier.table_row.update",
                org,
                &update(json!({"Nom": " Villa ", "Traité": "true"})),
            )
            .await,
        );

        assert_same_shape(&updated, "mestier.table_row.update");
        assert_eq!(
            updated["fields"],
            json!({"Nom": "Villa", "Traité": "Oui", "Statut": null, "Note": null})
        );
        let events = events_named(&pool, org, "table_row.updated").await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["changed"], json!(["Traité", "Note"]));
        assert_eq!(row_events(&pool, org).await, 2);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn an_unknown_column_or_an_invalid_value_fails_the_step_and_writes_nothing() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "tables-refused").await;
        let schema = chantiers(&usecase(&pool), org).await;
        let registry = registry(&pool);
        let table_id = json!(schema.table.id.to_string());

        let unknown = failure(
            run(
                &registry,
                "mestier.table_row.create",
                org,
                &config(&[
                    ("table_id", table_id.clone()),
                    ("values", json!({"Nom": "Villa", "Fini": true})),
                ]),
            )
            .await,
        );
        let invalid = failure(
            run(
                &registry,
                "mestier.table_row.create",
                org,
                &config(&[
                    ("table_id", table_id),
                    ("values", json!({"Traité": "peut-être"})),
                ]),
            )
            .await,
        );

        assert_eq!(unknown, "`Fini`: no such column in the table");
        assert_eq!(invalid, "`values.Traité`: expected true or false");
        assert_eq!(row_events(&pool, org).await, 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn find_returns_the_rows_whose_status_is_ongoing() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "tables-find").await;
        let schema = chantiers(&usecase(&pool), org).await;
        let registry = registry(&pool);
        let table_id = json!(schema.table.id.to_string());
        for (name, status) in [
            ("Villa", "En cours"),
            ("Grange", "Terminé"),
            ("Atelier", "En cours"),
        ] {
            produced(
                run(
                    &registry,
                    "mestier.table_row.create",
                    org,
                    &config(&[
                        ("table_id", table_id.clone()),
                        ("values", json!({"Nom": name, "Statut": status})),
                    ]),
                )
                .await,
            );
        }

        let found = produced(
            run(
                &registry,
                "mestier.table_row.find",
                org,
                &config(&[
                    ("table_id", table_id),
                    ("column", json!("statut")),
                    ("value", json!("En cours")),
                ]),
            )
            .await,
        );

        assert_same_shape(&found, "mestier.table_row.find");
        let mut titles: Vec<&str> = found
            .as_array()
            .unwrap()
            .iter()
            .map(|row| row["title"].as_str().unwrap())
            .collect();
        titles.sort_unstable();
        assert_eq!(titles, ["Atelier", "Villa"]);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_bulk_insert_records_no_event() {
        let _guard = RUN_CLAIM_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "tables-bulk").await;
        let schema = chantiers(&usecase(&pool), org).await;
        let emitter = TransactionalEventEmitter::new(events::Actor::system(), None);
        let batch = RowBatch {
            organization_id: org,
            table_id: schema.table.id,
            created_by: None,
            created_at: Utc::now(),
            rows: (0..3)
                .map(|_| (TableRowId(generate_uuid_v7()), RowValues::default()))
                .collect(),
        };

        let inserted = with_tx_emitting(&pool, &emitter, async |tx| {
            KnowledgeTableRowService::new(
                PgKnowledgeTableRepository::new(&tx),
                PgKnowledgeTableRowRepository::new(&tx),
                &emitter,
            )
            .insert_rows(&batch, &mut MockCellReferenceCheck::new())
            .await
        })
        .await
        .unwrap();

        assert_eq!(inserted, Ok(3));
        assert_eq!(row_events(&pool, org).await, 0);
    }

    #[tokio::test]
    #[ignore = "requires live postgres"]
    async fn a_workflow_on_created_rows_ticks_the_row_once_and_does_not_retrigger_itself() {
        let _claim = RUN_CLAIM_LOCK.lock().await;
        let _dispatch = DISPATCH_LOCK.lock().await;
        let pool = automation_pool().await;
        let org = world_support::organization(&pool, "tables-workflow").await;
        let usecase = usecase(&pool);
        let schema = chantiers(&usecase, org).await;
        let workflow = usecase
            .create_workflow(CreateWorkflowCommand {
                org_id: org,
                name: "Cocher Traité".to_owned(),
                description: None,
            })
            .await
            .unwrap();
        let step = config(&[
            ("table_id", json!("{{ trigger.table.id }}")),
            ("row_id", json!("{{ trigger.row.id }}")),
            ("values", json!({"Traité": true})),
        ]);
        usecase
            .save_workflow_version(SaveWorkflowVersionCommand {
                org_id: org,
                workflow_id: workflow.id,
                graph: Graph {
                    connectors: vec![PlacedConnector {
                        id: "c1".to_owned(),
                        kind: "mestier.table_row.update".to_owned(),
                        version: 1,
                        credential_id: None,
                        config: step,
                        name: None,
                        description: None,
                        disabled: false,
                        operation: None,
                    }],
                    edges: vec![Edge {
                        from: "t1".to_owned(),
                        to: "c1".to_owned(),
                        branch: None,
                    }],
                    triggers: vec![PlacedTrigger {
                        id: "t1".to_owned(),
                        kind: TriggerKind::Events(vec![
                            "table_row.created".to_owned(),
                            "table_row.updated".to_owned(),
                        ]),
                        name: None,
                        description: None,
                    }],
                },
                layout: None,
                created_by: None,
            })
            .await
            .unwrap();
        let row = usecase
            .create_table_row(CreateTableRowCommand {
                actor: Subject::system(),
                organization_id: org,
                table_id: schema.table.id,
                user_id: None,
                values: BTreeMap::from([(schema.columns[0].id, json!("Villa"))]),
            })
            .await
            .unwrap()
            .unwrap();
        let connectors = ConnectorRegistry::new(usecase.clone());
        let runs = async || {
            sqlx::query_as::<_, (String,)>(
                "SELECT status FROM automation.run WHERE workflow_id = $1",
            )
            .bind(workflow.id)
            .fetch_all(&pool)
            .await
            .unwrap()
        };

        usecase.dispatch_pending_events(100).await.unwrap();
        usecase
            .run_engine_pass(&connectors, "worker-1", WorkerSchedule::default())
            .await
            .unwrap();
        usecase.dispatch_pending_events(100).await.unwrap();
        usecase
            .run_engine_pass(&connectors, "worker-1", WorkerSchedule::default())
            .await
            .unwrap();

        assert_eq!(runs().await, vec![("succeeded".to_owned(),)]);
        let ticked = usecase
            .get_table_row(GetTableRowCommand {
                actor: Subject::system(),
                organization_id: org,
                table_id: schema.table.id,
                id: row.row.id,
            })
            .await
            .unwrap();
        let done = schema
            .columns
            .iter()
            .find(|column| column.name.as_str() == "Traité")
            .unwrap();
        assert_eq!(ticked.row.values[&done.key()], json!(true));
        let emitted_by: Vec<String> = sqlx::query_scalar(
            "SELECT actor_kind FROM automation.event WHERE org_id = $1 AND name = 'table_row.updated'",
        )
        .bind(org.0)
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(emitted_by, ["automation"]);
    }
}
