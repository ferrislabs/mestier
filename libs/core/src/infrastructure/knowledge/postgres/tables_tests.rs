use std::collections::BTreeMap;

use chrono::Utc;
use common::{CoreError, UserId, generate_uuid_v7};
use serde_json::{Map, Value, json};
use sqlx::PgPool;

use crate::{
    KnowledgeResource, KnowledgeResourceType, OrganizationId,
    application::test_support::dev_pool,
    domain::knowledge::tables::{
        ColumnConfig, ColumnKind, CreateKnowledgeTableCommand, CreateTableColumnCommand,
        CreateTableRowCommand, CreateTableViewCommand, FilterOperator, KnowledgeTableId,
        MAX_ROW_PAGE, MAX_ROWS, RowBatch, RowFilter, RowPatch, RowQuery, RowSort, RowUpdate,
        SelectOptionInput, SortDirection, TableColumn, TableError, TableRow, TableRowId,
        TableSchema, UpdateTableColumnCommand, ViewConfig, ViewKind,
        ports::MockCellReferenceCheck,
        ports::{KnowledgeTableRepository, KnowledgeTableRowRepository},
        rows::KnowledgeTableRowService,
        service::KnowledgeTableService,
        value::CellReference,
    },
    infrastructure::{
        knowledge::postgres::{PgKnowledgeTableRepository, PgKnowledgeTableRowRepository},
        postgres::{SharedTx, with_tx},
    },
};

async fn seed_organization(tx: &SharedTx<'_>) -> OrganizationId {
    let mut guard = tx.lock().await;
    let user_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO users (id, email, username, display_name, sub)
           VALUES ($1, $2, $3, $4, $5)"#,
        user_id,
        format!("owner-{user_id}@example.com"),
        format!("owner-{user_id}"),
        "Owner",
        format!("sub-{user_id}"),
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    let org_id = generate_uuid_v7();
    sqlx::query!(
        r#"INSERT INTO organizations (id, name, slug, owner_id) VALUES ($1, $2, $3, $4)"#,
        org_id,
        "Test Org",
        format!("test-org-{org_id}"),
        user_id,
    )
    .execute(&mut ***guard)
    .await
    .unwrap();

    OrganizationId(org_id)
}

async fn run<F>(work: F)
where
    F: AsyncFnOnce(SharedTx<'_>) -> Result<(), CoreError>,
{
    let pool: PgPool = dev_pool().await;
    let outcome = with_tx(&pool, async |tx| {
        work(tx).await?;
        Err::<(), _>(CoreError::Internal("rollback".into()))
    })
    .await;
    assert!(matches!(outcome, Err(CoreError::Internal(message)) if message == "rollback"));
}

fn actor() -> authz::Subject {
    authz::Subject::new(authz::SubjectKind::User, "someone")
}

async fn create_table(tx: &SharedTx<'_>, organization_id: OrganizationId) -> TableSchema {
    KnowledgeTableService::new(
        PgKnowledgeTableRepository::new(tx),
        PgKnowledgeTableRowRepository::new(tx),
    )
    .create_table(
        CreateKnowledgeTableCommand {
            actor: actor(),
            organization_id,
            user_id: None,
            name: "Chantiers".to_owned(),
            description: None,
        },
        Utc::now(),
    )
    .await
    .unwrap()
    .unwrap()
}

async fn add_column(
    tx: &SharedTx<'_>,
    schema: &TableSchema,
    name: &str,
    kind: ColumnKind,
    config: ColumnConfig,
) -> TableColumn {
    KnowledgeTableService::new(
        PgKnowledgeTableRepository::new(tx),
        PgKnowledgeTableRowRepository::new(tx),
    )
    .create_column(
        CreateTableColumnCommand {
            actor: actor(),
            organization_id: schema.table.organization_id,
            table_id: schema.table.id,
            name: name.to_owned(),
            kind,
            config,
        },
        Utc::now(),
    )
    .await
    .unwrap()
    .unwrap()
}

fn options(labels: &[&str]) -> ColumnConfig {
    ColumnConfig {
        options: Some(
            labels
                .iter()
                .map(|label| SelectOptionInput {
                    id: None,
                    label: (*label).to_owned(),
                    color: None,
                })
                .collect(),
        ),
        ..ColumnConfig::default()
    }
}

fn option_id(column: &TableColumn, index: usize) -> String {
    column.column_type.options().unwrap().as_slice()[index]
        .id
        .to_string()
}

async fn insert_row(
    tx: &SharedTx<'_>,
    schema: &TableSchema,
    columns: &[TableColumn],
    values: Vec<(&TableColumn, Value)>,
) -> TableRow {
    let raw = values
        .into_iter()
        .map(|(column, value)| (column.id, value))
        .collect::<BTreeMap<_, _>>();
    let patch = RowPatch::parse(columns, &raw).unwrap();
    let now = Utc::now();
    PgKnowledgeTableRowRepository::new(tx)
        .insert(&TableRow {
            id: TableRowId(generate_uuid_v7()),
            table_id: schema.table.id,
            organization_id: schema.table.organization_id,
            values: patch.set.to_json(),
            created_by: None,
            updated_by: None,
            created_at: now,
            updated_at: now,
        })
        .await
        .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn two_patches_on_two_cells_keep_both_values() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let schema = create_table(&tx, org).await;
        let name = schema.columns[0].clone();
        let done = add_column(
            &tx,
            &schema,
            "Fait",
            ColumnKind::Checkbox,
            ColumnConfig::default(),
        )
        .await;
        let note = add_column(
            &tx,
            &schema,
            "Note",
            ColumnKind::LongText,
            ColumnConfig::default(),
        )
        .await;
        let columns = [name.clone(), done.clone(), note.clone()];
        let row = insert_row(
            &tx,
            &schema,
            &columns,
            vec![(&name, json!("Villa")), (&note, json!("à revoir"))],
        )
        .await;

        let mut rows = PgKnowledgeTableRowRepository::new(&tx);
        let update = |set: Map<String, Value>, cleared: Vec<String>| RowUpdate {
            organization_id: org,
            table_id: schema.table.id,
            id: row.id,
            set,
            cleared,
            updated_by: None,
            updated_at: Utc::now(),
        };
        rows.patch(&update(
            Map::from_iter([(done.key(), json!(true))]),
            Vec::new(),
        ))
        .await?
        .unwrap();
        let after = rows
            .patch(&update(
                Map::from_iter([(name.key(), json!("Villa Dupont"))]),
                vec![note.key()],
            ))
            .await?
            .unwrap();
        assert_eq!(
            after.values,
            Map::from_iter([
                (name.key(), json!("Villa Dupont")),
                (done.key(), json!(true))
            ])
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn deleting_a_column_or_an_option_clears_it_from_every_row() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let schema = create_table(&tx, org).await;
        let name = schema.columns[0].clone();
        let status = add_column(
            &tx,
            &schema,
            "Statut",
            ColumnKind::SingleSelect,
            options(&["En cours", "Terminé"]),
        )
        .await;
        let tags = add_column(
            &tx,
            &schema,
            "Tags",
            ColumnKind::MultiSelect,
            options(&["A", "B"]),
        )
        .await;
        let columns = [name.clone(), status.clone(), tags.clone()];
        let first = insert_row(
            &tx,
            &schema,
            &columns,
            vec![
                (&name, json!("A")),
                (&status, json!(option_id(&status, 0))),
                (&tags, json!([option_id(&tags, 0), option_id(&tags, 1)])),
            ],
        )
        .await;
        let second = insert_row(
            &tx,
            &schema,
            &columns,
            vec![
                (&name, json!("B")),
                (&status, json!(option_id(&status, 1))),
                (&tags, json!([option_id(&tags, 0)])),
            ],
        )
        .await;

        let mut service = KnowledgeTableService::new(
            PgKnowledgeTableRepository::new(&tx),
            PgKnowledgeTableRowRepository::new(&tx),
        );
        let keep_second = |column: &TableColumn, index: usize| {
            let option = &column.column_type.options().unwrap().as_slice()[index];
            let config = ColumnConfig {
                options: Some(vec![SelectOptionInput {
                    id: Some(option.id),
                    label: option.label.clone(),
                    color: None,
                }]),
                ..ColumnConfig::default()
            };
            UpdateTableColumnCommand {
                actor: actor(),
                organization_id: org,
                table_id: schema.table.id,
                id: column.id,
                name: None,
                kind: None,
                config: Some(config),
            }
        };
        service
            .update_column(keep_second(&status, 1))
            .await?
            .unwrap();
        service.update_column(keep_second(&tags, 1)).await?.unwrap();

        let mut rows = PgKnowledgeTableRowRepository::new(&tx);
        let first = rows.find(org, schema.table.id, first.id).await?.unwrap();
        let second = rows.find(org, schema.table.id, second.id).await?.unwrap();
        assert!(!first.values.contains_key(&status.key()));
        assert_eq!(first.values[&tags.key()], json!([option_id(&tags, 1)]));
        assert_eq!(second.values[&status.key()], json!(option_id(&status, 1)));
        assert!(!second.values.contains_key(&tags.key()));

        let kept_filter = RowFilter {
            column_id: name.id,
            operator: FilterOperator::Contains,
            value: Some(json!("A")),
        };
        let view = service
            .create_view(
                CreateTableViewCommand {
                    actor: actor(),
                    organization_id: org,
                    table_id: schema.table.id,
                    name: "Par statut".to_owned(),
                    kind: ViewKind::Kanban,
                    config: ViewConfig {
                        filters: vec![
                            kept_filter.clone(),
                            RowFilter {
                                column_id: status.id,
                                operator: FilterOperator::Is,
                                value: Some(json!(option_id(&status, 1))),
                            },
                        ],
                        sorts: vec![RowSort {
                            column_id: status.id,
                            direction: SortDirection::Asc,
                        }],
                        group_by: Some(status.id),
                        ..ViewConfig::default()
                    },
                },
                Utc::now(),
            )
            .await?
            .unwrap();
        assert_eq!(view.config.filters.len(), 2);

        service
            .delete_column(org, schema.table.id, status.id)
            .await?
            .unwrap();
        let reconciled = PgKnowledgeTableRepository::new(&tx)
            .views(org, schema.table.id)
            .await?
            .into_iter()
            .find(|stored| stored.id == view.id)
            .unwrap();
        assert_eq!(reconciled.config.filters, vec![kept_filter]);
        assert!(reconciled.config.sorts.is_empty());
        assert_eq!(reconciled.config.group_by, None);
        assert_eq!(reconciled.kind, ViewKind::Grid);
        for id in [first.id, second.id] {
            let row = rows.find(org, schema.table.id, id).await?.unwrap();
            assert!(!row.values.contains_key(&status.key()));
            assert!(row.values.contains_key(&name.key()));
        }
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_number_column_becomes_text_and_a_refused_conversion_changes_nothing() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let schema = create_table(&tx, org).await;
        let name = schema.columns[0].clone();
        let quantity = add_column(
            &tx,
            &schema,
            "Quantité",
            ColumnKind::Number,
            ColumnConfig::default(),
        )
        .await;
        let tags = add_column(
            &tx,
            &schema,
            "Tags",
            ColumnKind::MultiSelect,
            options(&["A", "B"]),
        )
        .await;
        let columns = [name.clone(), quantity.clone(), tags.clone()];
        let row = insert_row(
            &tx,
            &schema,
            &columns,
            vec![
                (&quantity, json!(12.5)),
                (&tags, json!([option_id(&tags, 0), option_id(&tags, 1)])),
            ],
        )
        .await;

        let mut service = KnowledgeTableService::new(
            PgKnowledgeTableRepository::new(&tx),
            PgKnowledgeTableRowRepository::new(&tx),
        );
        let retype = |column: &TableColumn, kind| UpdateTableColumnCommand {
            actor: actor(),
            organization_id: org,
            table_id: schema.table.id,
            id: column.id,
            name: None,
            kind: Some(kind),
            config: None,
        };
        let refused = service
            .update_column(retype(&tags, ColumnKind::SingleSelect))
            .await?;
        assert_eq!(refused, Err(TableError::ConversionLosesValue));
        service
            .update_column(retype(&quantity, ColumnKind::Text))
            .await?
            .unwrap();

        let stored = PgKnowledgeTableRowRepository::new(&tx)
            .find(org, schema.table.id, row.id)
            .await?
            .unwrap();
        assert_eq!(stored.values[&quantity.key()], json!("12,5"));
        assert_eq!(stored.values[&tags.key()], row.values[&tags.key()]);
        let columns = PgKnowledgeTableRepository::new(&tx)
            .columns(org, schema.table.id)
            .await?;
        let tags_after = columns.iter().find(|column| column.id == tags.id).unwrap();
        assert_eq!(tags_after.kind(), ColumnKind::MultiSelect);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn rows_filter_sort_and_page_on_typed_values() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let schema = create_table(&tx, org).await;
        let name = schema.columns[0].clone();
        let amount = add_column(
            &tx,
            &schema,
            "Montant",
            ColumnKind::Currency,
            ColumnConfig::default(),
        )
        .await;
        let status = add_column(
            &tx,
            &schema,
            "Statut",
            ColumnKind::SingleSelect,
            options(&["En cours", "Terminé"]),
        )
        .await;
        let start = add_column(
            &tx,
            &schema,
            "Début",
            ColumnKind::Date,
            ColumnConfig::default(),
        )
        .await;
        let columns = vec![name.clone(), amount.clone(), status.clone(), start.clone()];
        let running = option_id(&status, 0);
        let done = option_id(&status, 1);
        for (label, cents, state, date) in [
            ("A", 250_000, &running, "2026-03-01"),
            ("B", 50_000, &running, "2026-04-01"),
            ("C", 300_000, &done, "2026-05-01"),
            ("D", 150_000, &running, "2026-06-01"),
            ("E", 120_000, &running, "2026-01-15"),
        ] {
            insert_row(
                &tx,
                &schema,
                &columns,
                vec![
                    (&name, json!(label)),
                    (&amount, json!(cents)),
                    (&status, json!(state)),
                    (&start, json!(date)),
                ],
            )
            .await;
        }
        insert_row(
            &tx,
            &schema,
            &columns,
            vec![(&name, json!("F")), (&status, json!(running))],
        )
        .await;

        let filters = [
            RowFilter {
                column_id: amount.id,
                operator: FilterOperator::Gt,
                value: Some(json!(100_000)),
            },
            RowFilter {
                column_id: status.id,
                operator: FilterOperator::Is,
                value: Some(json!(running)),
            },
        ];
        let sorts = [RowSort {
            column_id: start.id,
            direction: SortDirection::Desc,
        }];
        let mut rows = PgKnowledgeTableRowRepository::new(&tx);
        let titles = |page: &[TableRow]| -> Vec<String> {
            page.iter()
                .map(|row| row.title(&columns).unwrap().to_owned())
                .collect()
        };

        let query = RowQuery::build(&columns, &filters, &sorts, 2, 0).unwrap();
        let (first, total) = rows.list(org, schema.table.id, &query).await?;
        assert_eq!(total, 3);
        assert_eq!(titles(&first), vec!["D", "A"]);
        let query = RowQuery::build(&columns, &filters, &sorts, 2, 2).unwrap();
        let (second, _) = rows.list(org, schema.table.id, &query).await?;
        assert_eq!(titles(&second), vec!["E"]);

        let by_status = [RowSort {
            column_id: status.id,
            direction: SortDirection::Desc,
        }];
        let query = RowQuery::build(&columns, &[], &by_status, MAX_ROW_PAGE, 0).unwrap();
        let (all, total) = rows.list(org, schema.table.id, &query).await?;
        assert_eq!(total, 6);
        assert_eq!(titles(&all)[0], "C");

        let empty_amount = [RowFilter {
            column_id: amount.id,
            operator: FilterOperator::IsEmpty,
            value: None,
        }];
        let query = RowQuery::build(&columns, &empty_amount, &[], MAX_ROW_PAGE, 0).unwrap();
        assert_eq!(
            titles(&rows.list(org, schema.table.id, &query).await?.0),
            vec!["F"]
        );

        let not_fifty = [RowFilter {
            column_id: amount.id,
            operator: FilterOperator::Ne,
            value: Some(json!(50_000)),
        }];
        let query = RowQuery::build(&columns, &not_fifty, &[], MAX_ROW_PAGE, 0).unwrap();
        let mut different = titles(&rows.list(org, schema.table.id, &query).await?.0);
        different.sort();
        assert_eq!(different, vec!["A", "C", "D", "E", "F"]);

        let contains = [RowFilter {
            column_id: name.id,
            operator: FilterOperator::Contains,
            value: Some(json!("'; DROP TABLE x; --")),
        }];
        let query = RowQuery::build(&columns, &contains, &[], MAX_ROW_PAGE, 0).unwrap();
        assert_eq!(rows.list(org, schema.table.id, &query).await?.1, 0);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_column_and_a_view_are_deleted_only_through_their_own_table() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let schema = create_table(&tx, org).await;
        let elsewhere = KnowledgeTableService::new(
            PgKnowledgeTableRepository::new(&tx),
            PgKnowledgeTableRowRepository::new(&tx),
        )
        .create_table(
            CreateKnowledgeTableCommand {
                actor: actor(),
                organization_id: org,
                user_id: None,
                name: "Autre".to_owned(),
                description: None,
            },
            Utc::now(),
        )
        .await
        .unwrap()
        .unwrap();
        let note = add_column(
            &tx,
            &schema,
            "Note",
            ColumnKind::Text,
            ColumnConfig::default(),
        )
        .await;

        let mut tables = PgKnowledgeTableRepository::new(&tx);
        assert!(matches!(
            tables.delete_column(org, elsewhere.table.id, note.id).await,
            Err(CoreError::NotFound)
        ));
        assert_eq!(tables.columns(org, schema.table.id).await?.len(), 2);
        assert!(matches!(
            tables
                .delete_view(org, elsewhere.table.id, schema.views[0].id)
                .await,
            Err(CoreError::NotFound)
        ));
        assert_eq!(tables.views(org, schema.table.id).await?.len(), 1);
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn another_organization_reaches_no_table_and_no_row() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let other = seed_organization(&tx).await;
        let schema = create_table(&tx, org).await;
        let name = schema.columns[0].clone();
        let row = insert_row(
            &tx,
            &schema,
            std::slice::from_ref(&name),
            vec![(&name, json!("Villa"))],
        )
        .await;

        let mut tables = PgKnowledgeTableRepository::new(&tx);
        assert!(tables.find_table(other, schema.table.id).await?.is_none());
        assert!(tables.columns(other, schema.table.id).await?.is_empty());
        assert!(tables.views(other, schema.table.id).await?.is_empty());
        assert!(matches!(
            tables.delete_column(other, schema.table.id, name.id).await,
            Err(CoreError::NotFound)
        ));
        assert!(matches!(
            tables
                .delete_view(other, schema.table.id, schema.views[0].id)
                .await,
            Err(CoreError::NotFound)
        ));

        let mut rows = PgKnowledgeTableRowRepository::new(&tx);
        assert!(rows.find(other, schema.table.id, row.id).await?.is_none());
        let query = RowQuery::build(std::slice::from_ref(&name), &[], &[], 10, 0).unwrap();
        assert_eq!(rows.list(other, schema.table.id, &query).await?.1, 0);
        assert!(
            rows.patch(&RowUpdate {
                organization_id: other,
                table_id: schema.table.id,
                id: row.id,
                set: Map::from_iter([(name.key(), json!("Volé"))]),
                cleared: Vec::new(),
                updated_by: None,
                updated_at: Utc::now(),
            })
            .await?
            .is_none()
        );
        assert_eq!(rows.delete(other, schema.table.id, &[row.id]).await?, 0);
        assert_eq!(
            rows.remove_column(other, schema.table.id, name.id).await?,
            0
        );
        assert_eq!(
            rows.find(org, schema.table.id, row.id)
                .await?
                .unwrap()
                .values,
            row.values
        );
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn names_are_unique_whatever_the_case_and_rows_insert_in_bulk() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let schema = create_table(&tx, org).await;
        let mut service = KnowledgeTableService::new(
            PgKnowledgeTableRepository::new(&tx),
            PgKnowledgeTableRowRepository::new(&tx),
        );
        let twin = service
            .create_table(
                CreateKnowledgeTableCommand {
                    actor: actor(),
                    organization_id: org,
                    user_id: None,
                    name: "chantiers".to_owned(),
                    description: None,
                },
                Utc::now(),
            )
            .await?;
        assert_eq!(twin, Err(TableError::TableNameTaken));

        let name = schema.columns[0].clone();
        let mut renamed = name.clone();
        renamed.name = crate::domain::knowledge::tables::TableItemName::try_from("Statut").unwrap();
        let mut tables = PgKnowledgeTableRepository::new(&tx);
        tables.update_column(&renamed).await?;

        let rows: Vec<_> = (0..250)
            .map(|n| {
                let raw = BTreeMap::from([(name.id, json!(format!("Ligne {n}")))]);
                (
                    TableRowId(generate_uuid_v7()),
                    RowPatch::parse(std::slice::from_ref(&renamed), &raw)
                        .unwrap()
                        .set,
                )
            })
            .collect();
        let inserted = PgKnowledgeTableRowRepository::new(&tx)
            .insert_many(&RowBatch {
                organization_id: org,
                table_id: schema.table.id,
                created_by: None,
                created_at: Utc::now(),
                rows,
            })
            .await?;
        assert_eq!(inserted, 250);
        assert_eq!(
            PgKnowledgeTableRowRepository::new(&tx)
                .count(org, schema.table.id)
                .await?,
            250
        );

        let mut clash = renamed.clone();
        clash.id = crate::domain::knowledge::tables::TableColumnId(generate_uuid_v7());
        clash.is_primary = false;
        clash.name = crate::domain::knowledge::tables::TableItemName::try_from("statut").unwrap();
        clash.position = 1;
        assert!(matches!(
            tables.insert_column(&clash).await,
            Err(CoreError::Conflict(_))
        ));
        Ok(())
    })
    .await;
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn a_deleted_customer_has_no_label_while_a_live_one_does() {
    run(async |tx| {
        let org = seed_organization(&tx).await;
        let alive = generate_uuid_v7();
        let gone = generate_uuid_v7();
        {
            let mut guard = tx.lock().await;
            for (id, deleted) in [(alive, false), (gone, true)] {
                sqlx::query(
                    "INSERT INTO customers (id, org_id, name, deleted_at)
                     VALUES ($1, $2, 'Dupont SARL', CASE WHEN $3 THEN now() END)",
                )
                .bind(id)
                .bind(org.0)
                .bind(deleted)
                .execute(&mut ***guard)
                .await
                .unwrap();
            }
        }
        let reference = |id| {
            CellReference::Resource(KnowledgeResource {
                resource_type: KnowledgeResourceType::Customer,
                id,
            })
        };
        let labels = PgKnowledgeTableRowRepository::new(&tx)
            .labels(
                org,
                &[
                    reference(alive),
                    reference(gone),
                    CellReference::Member(UserId(generate_uuid_v7())),
                ],
            )
            .await?;
        assert_eq!(
            labels.get(&reference(alive)).map(String::as_str),
            Some("Dupont SARL")
        );
        assert!(!labels.contains_key(&reference(gone)));
        assert_eq!(labels.len(), 1);
        Ok(())
    })
    .await;
}

async fn committed_table() -> (PgPool, OrganizationId, KnowledgeTableId) {
    let pool = dev_pool().await;
    let (organization_id, table_id) = with_tx(&pool, async |tx| {
        let org = seed_organization(&tx).await;
        let schema = create_table(&tx, org).await;
        let rows: Vec<_> = (1..MAX_ROWS)
            .map(|_| (TableRowId(generate_uuid_v7()), Default::default()))
            .collect();
        PgKnowledgeTableRowRepository::new(&tx)
            .insert_many(&RowBatch {
                organization_id: org,
                table_id: schema.table.id,
                created_by: None,
                created_at: Utc::now(),
                rows,
            })
            .await?;
        Ok((org, schema.table.id))
    })
    .await
    .unwrap();
    (pool, organization_id, table_id)
}

async fn add_one_row(
    pool: &PgPool,
    organization_id: OrganizationId,
    table_id: KnowledgeTableId,
    hold: std::time::Duration,
) -> Result<(), TableError> {
    with_tx(pool, async |tx| {
        let emitter = events::testing::RecordingEmitter::default();
        let outcome = KnowledgeTableRowService::new(
            PgKnowledgeTableRepository::new(&tx),
            PgKnowledgeTableRowRepository::new(&tx),
            &emitter,
        )
        .create_row(
            CreateTableRowCommand {
                actor: actor(),
                organization_id,
                table_id,
                user_id: None,
                values: BTreeMap::new(),
            },
            &mut MockCellReferenceCheck::new(),
            Utc::now(),
        )
        .await?;
        tokio::time::sleep(hold).await;
        Ok(outcome.map(|_| ()))
    })
    .await
    .unwrap()
}

#[tokio::test]
#[ignore = "requires live postgres"]
async fn two_concurrent_rows_on_the_last_free_slot_leave_exactly_the_cap() {
    let (pool, org, table_id) = committed_table().await;
    let first = {
        let pool = pool.clone();
        tokio::spawn(async move {
            add_one_row(&pool, org, table_id, std::time::Duration::from_millis(300)).await
        })
    };
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    let second = add_one_row(&pool, org, table_id, std::time::Duration::ZERO).await;
    let first = first.await.unwrap();

    let stored: i64 =
        sqlx::query_scalar("SELECT count(*) FROM knowledge_table_rows WHERE table_id = $1")
            .bind(table_id.0)
            .fetch_one(&pool)
            .await
            .unwrap();
    let owner: uuid::Uuid = sqlx::query_scalar("SELECT owner_id FROM organizations WHERE id = $1")
        .bind(org.0)
        .fetch_one(&pool)
        .await
        .unwrap();
    for statement in [
        "DELETE FROM knowledge_tables WHERE org_id = $1",
        "DELETE FROM organizations WHERE id = $1",
    ] {
        sqlx::query(statement)
            .bind(org.0)
            .execute(&pool)
            .await
            .unwrap();
    }
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(owner)
        .execute(&pool)
        .await
        .unwrap();
    assert_eq!(first, Ok(()));
    assert_eq!(second, Err(TableError::TooManyRows));
    assert_eq!(u64::try_from(stored).unwrap(), MAX_ROWS);
}

static MIGRATION_SCRATCH: tokio::sync::OnceCell<String> = tokio::sync::OnceCell::const_new();

#[tokio::test]
#[ignore = "requires live postgres"]
async fn the_tables_migration_reverts_and_applies_again() {
    const PREVIOUS: i64 = 20261118000001;
    let pool = crate::application::test_support::scratch_pool(
        "mestier_tablesmig_test_",
        &MIGRATION_SCRATCH,
    )
    .await;
    let migrator = sqlx::migrate::Migrator::new(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../migrations"),
    )
    .await
    .unwrap();
    let present = async |pool: &PgPool| -> Vec<Option<String>> {
        let mut found = Vec::new();
        for table in [
            "knowledge_tables",
            "knowledge_table_columns",
            "knowledge_table_rows",
            "knowledge_table_views",
        ] {
            found.push(
                sqlx::query_scalar("SELECT to_regclass($1)::text")
                    .bind(table)
                    .fetch_one(pool)
                    .await
                    .unwrap(),
            );
        }
        found
    };
    assert!(present(&pool).await.iter().all(Option::is_some));
    migrator.undo(&pool, PREVIOUS).await.unwrap();
    assert!(present(&pool).await.iter().all(Option::is_none));
    migrator.run(&pool).await.unwrap();
    assert!(present(&pool).await.iter().all(Option::is_some));
    migrator.undo(&pool, PREVIOUS).await.unwrap();
    assert!(present(&pool).await.iter().all(Option::is_none));
    migrator.run(&pool).await.unwrap();
}
