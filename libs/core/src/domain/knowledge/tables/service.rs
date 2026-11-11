use std::collections::HashSet;

use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};

use crate::{
    OrganizationId,
    domain::knowledge::tables::{
        CreateKnowledgeTableCommand, CreateTableColumnCommand, CreateTableViewCommand,
        DEFAULT_VIEW_NAME, KnowledgeTable, KnowledgeTableId, MAX_COLUMNS, MAX_TABLES, MAX_VIEWS,
        PRIMARY_COLUMN_NAME, ReorderTableColumnsCommand, TableColumn, TableColumnId,
        TableDescription, TableError, TableItemName, TableOutcome, TableSchema, TableView,
        TableViewId, UpdateKnowledgeTableCommand, UpdateTableColumnCommand, UpdateTableViewCommand,
        ViewConfig, ViewKind,
        column::{ColumnKind, ColumnType, Conversion},
        csv::ColumnPlan,
        error::{Stop, settle},
        ports::{KnowledgeTableRepository, KnowledgeTableRowRepository},
        value::CellValue,
    },
};

fn next_position(positions: impl Iterator<Item = i32>) -> i32 {
    positions.max().map_or(0, |last| last + 1)
}

pub struct KnowledgeTableService<T, R>
where
    T: KnowledgeTableRepository,
    R: KnowledgeTableRowRepository,
{
    tables: T,
    rows: R,
}

impl<T, R> KnowledgeTableService<T, R>
where
    T: KnowledgeTableRepository,
    R: KnowledgeTableRowRepository,
{
    pub fn new(tables: T, rows: R) -> Self {
        Self { tables, rows }
    }

    pub fn into_parts(self) -> (T, R) {
        (self.tables, self.rows)
    }

    pub async fn table(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeTableId,
    ) -> Result<KnowledgeTable, CoreError> {
        self.tables
            .find_table(organization_id, id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    async fn lock(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeTableId,
    ) -> Result<(), CoreError> {
        if self.tables.lock_table(organization_id, id).await? {
            Ok(())
        } else {
            Err(CoreError::NotFound)
        }
    }

    pub async fn schema(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeTableId,
    ) -> Result<TableSchema, CoreError> {
        let table = self.table(organization_id, id).await?;
        let columns = self.tables.columns(organization_id, id).await?;
        let views = self.tables.views(organization_id, id).await?;
        Ok(TableSchema {
            table,
            columns,
            views,
        })
    }

    pub async fn list(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<KnowledgeTable>, CoreError> {
        self.tables.list_tables(organization_id).await
    }

    pub async fn create_table(
        &mut self,
        command: CreateKnowledgeTableCommand,
        now: DateTime<Utc>,
    ) -> Result<TableOutcome<TableSchema>, CoreError> {
        settle(self.try_create_table(command, &[], now).await)
    }

    pub async fn create_table_with_columns(
        &mut self,
        command: CreateKnowledgeTableCommand,
        columns: &[ColumnPlan],
        now: DateTime<Utc>,
    ) -> Result<TableOutcome<TableSchema>, CoreError> {
        settle(self.try_create_table(command, columns, now).await)
    }

    async fn try_create_table(
        &mut self,
        command: CreateKnowledgeTableCommand,
        columns: &[ColumnPlan],
        now: DateTime<Utc>,
    ) -> Result<TableSchema, Stop> {
        let organization_id = command.organization_id;
        let name = TableItemName::try_from(command.name.as_str())?;
        let description = TableDescription::parse(command.description.as_deref())?;
        if columns.len() > MAX_COLUMNS {
            return Err(TableError::TooManyColumns.into());
        }
        let (primary_name, others) = match columns.split_first() {
            Some((first, others)) => (first.name.clone(), others),
            None => (TableItemName::try_from(PRIMARY_COLUMN_NAME)?, &[][..]),
        };
        self.tables.lock_organization(organization_id).await?;
        if self.tables.count_tables(organization_id).await? >= MAX_TABLES {
            return Err(TableError::TooManyTables.into());
        }
        self.ensure_table_name_free(organization_id, &name, None)
            .await?;
        let table = self
            .tables
            .insert_table(&KnowledgeTable {
                id: KnowledgeTableId(generate_uuid_v7()),
                organization_id,
                name,
                description,
                created_by: command.user_id,
                created_at: now,
                updated_at: now,
            })
            .await?;
        let primary = self
            .tables
            .insert_column(&TableColumn {
                id: TableColumnId(generate_uuid_v7()),
                table_id: table.id,
                organization_id,
                name: primary_name,
                column_type: ColumnType::Text,
                position: 0,
                is_primary: true,
                created_at: now,
            })
            .await?;
        let mut inserted = vec![primary];
        for (position, plan) in (1..).zip(others) {
            inserted.push(
                self.tables
                    .insert_column(&TableColumn {
                        id: TableColumnId(generate_uuid_v7()),
                        table_id: table.id,
                        organization_id,
                        name: plan.name.clone(),
                        column_type: plan.column_type.clone(),
                        position,
                        is_primary: false,
                        created_at: now,
                    })
                    .await?,
            );
        }
        let view = self
            .tables
            .insert_view(&TableView {
                id: TableViewId(generate_uuid_v7()),
                table_id: table.id,
                organization_id,
                name: TableItemName::try_from(DEFAULT_VIEW_NAME)?,
                kind: ViewKind::Grid,
                config: ViewConfig::default(),
                position: 0,
                created_at: now,
                updated_at: now,
            })
            .await?;
        Ok(TableSchema {
            table,
            columns: inserted,
            views: vec![view],
        })
    }

    async fn ensure_table_name_free(
        &mut self,
        organization_id: OrganizationId,
        name: &TableItemName,
        except: Option<KnowledgeTableId>,
    ) -> Result<(), Stop> {
        let taken = self
            .tables
            .list_tables(organization_id)
            .await?
            .iter()
            .any(|table| Some(table.id) != except && table.name.same_as(name.as_str()));
        if taken {
            return Err(TableError::TableNameTaken.into());
        }
        Ok(())
    }

    pub async fn update_table(
        &mut self,
        command: UpdateKnowledgeTableCommand,
        now: DateTime<Utc>,
    ) -> Result<TableOutcome<KnowledgeTable>, CoreError> {
        settle(self.try_update_table(command, now).await)
    }

    async fn try_update_table(
        &mut self,
        command: UpdateKnowledgeTableCommand,
        now: DateTime<Utc>,
    ) -> Result<KnowledgeTable, Stop> {
        let organization_id = command.organization_id;
        let mut table = self.table(organization_id, command.id).await?;
        if let Some(name) = &command.name {
            let name = TableItemName::try_from(name.as_str())?;
            self.ensure_table_name_free(organization_id, &name, Some(table.id))
                .await?;
            table.name = name;
        }
        if let Some(description) = &command.description {
            table.description = TableDescription::parse(description.as_deref())?;
        }
        table.updated_at = now;
        Ok(self.tables.update_table(&table).await?)
    }

    pub async fn delete_table(
        &mut self,
        organization_id: OrganizationId,
        id: KnowledgeTableId,
    ) -> Result<(), CoreError> {
        self.table(organization_id, id).await?;
        self.tables.delete_table(organization_id, id).await
    }

    pub async fn create_column(
        &mut self,
        command: CreateTableColumnCommand,
        now: DateTime<Utc>,
    ) -> Result<TableOutcome<TableColumn>, CoreError> {
        settle(self.try_create_column(command, now).await)
    }

    async fn try_create_column(
        &mut self,
        command: CreateTableColumnCommand,
        now: DateTime<Utc>,
    ) -> Result<TableColumn, Stop> {
        let organization_id = command.organization_id;
        self.lock(organization_id, command.table_id).await?;
        let name = TableItemName::try_from(command.name.as_str())?;
        let column_type = ColumnType::from_config(command.kind, &command.config)?;
        let columns = self
            .tables
            .columns(organization_id, command.table_id)
            .await?;
        if columns.len() >= MAX_COLUMNS {
            return Err(TableError::TooManyColumns.into());
        }
        if columns
            .iter()
            .any(|column| column.name.same_as(name.as_str()))
        {
            return Err(TableError::ColumnNameTaken.into());
        }
        Ok(self
            .tables
            .insert_column(&TableColumn {
                id: TableColumnId(generate_uuid_v7()),
                table_id: command.table_id,
                organization_id,
                name,
                column_type,
                position: next_position(columns.iter().map(|column| column.position)),
                is_primary: false,
                created_at: now,
            })
            .await?)
    }

    pub async fn update_column(
        &mut self,
        command: UpdateTableColumnCommand,
    ) -> Result<TableOutcome<TableColumn>, CoreError> {
        settle(self.try_update_column(command).await)
    }

    async fn try_update_column(
        &mut self,
        command: UpdateTableColumnCommand,
    ) -> Result<TableColumn, Stop> {
        let organization_id = command.organization_id;
        let table_id = command.table_id;
        self.lock(organization_id, table_id).await?;
        let columns = self.tables.columns(organization_id, table_id).await?;
        let mut column = columns
            .iter()
            .find(|column| column.id == command.id)
            .cloned()
            .ok_or(CoreError::NotFound)?;
        if let Some(name) = &command.name {
            let name = TableItemName::try_from(name.as_str())?;
            if columns
                .iter()
                .any(|other| other.id != column.id && other.name.same_as(name.as_str()))
            {
                return Err(TableError::ColumnNameTaken.into());
            }
            column.name = name;
        }

        let before = column.column_type.clone();
        let mut rewritten = Vec::new();
        if command.kind.is_some() || command.config.is_some() {
            let kind = command.kind.unwrap_or(before.kind());
            let config = command.config.clone().unwrap_or_else(|| before.config());
            let target = ColumnType::from_config(kind, &config)?;
            if column.is_primary && target.kind() != ColumnKind::Text {
                return Err(TableError::PrimaryColumnLocked.into());
            }
            let conversion = before.conversion_to(&target)?;
            if conversion != Conversion::Unchanged {
                let stored = self
                    .rows
                    .values_of(organization_id, table_id, column.id)
                    .await?;
                let references: Vec<_> = stored
                    .iter()
                    .filter_map(|(_, value)| {
                        CellValue::parse(&before, value)
                            .ok()
                            .flatten()
                            .and_then(|cell| cell.reference(&before))
                    })
                    .collect::<HashSet<_>>()
                    .into_iter()
                    .collect();
                let labels = if references.is_empty() {
                    Default::default()
                } else {
                    self.rows.labels(organization_id, &references).await?
                };
                rewritten = before.convert_stored(conversion, &stored, &labels)?;
            }
            column.column_type = target;
        }

        if !rewritten.is_empty() {
            self.rows
                .rewrite_values(organization_id, table_id, column.id, &rewritten)
                .await?;
        }
        let removed = match (column.column_type.options(), before.options()) {
            (Some(after), Some(before)) => after.removed_from(before),
            _ => Vec::new(),
        };
        let updated = self.tables.update_column(&column).await?;
        for option in removed {
            self.rows
                .remove_option(organization_id, table_id, column.id, updated.kind(), option)
                .await?;
        }
        let columns: Vec<TableColumn> = columns
            .into_iter()
            .map(|other| {
                if other.id == updated.id {
                    updated.clone()
                } else {
                    other
                }
            })
            .collect();
        self.reconcile_views(organization_id, table_id, &columns)
            .await?;
        Ok(updated)
    }

    pub async fn delete_column(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        id: TableColumnId,
    ) -> Result<TableOutcome<()>, CoreError> {
        settle(self.try_delete_column(organization_id, table_id, id).await)
    }

    async fn try_delete_column(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        id: TableColumnId,
    ) -> Result<(), Stop> {
        self.lock(organization_id, table_id).await?;
        let mut columns = self.tables.columns(organization_id, table_id).await?;
        let column = columns
            .iter()
            .find(|column| column.id == id)
            .ok_or(CoreError::NotFound)?;
        if column.is_primary {
            return Err(TableError::PrimaryColumnLocked.into());
        }
        self.tables.delete_column(organization_id, id).await?;
        self.rows
            .remove_column(organization_id, table_id, id)
            .await?;
        columns.retain(|column| column.id != id);
        self.reconcile_views(organization_id, table_id, &columns)
            .await?;
        Ok(())
    }

    pub async fn reorder_columns(
        &mut self,
        command: ReorderTableColumnsCommand,
    ) -> Result<TableOutcome<Vec<TableColumn>>, CoreError> {
        settle(self.try_reorder_columns(command).await)
    }

    async fn try_reorder_columns(
        &mut self,
        command: ReorderTableColumnsCommand,
    ) -> Result<Vec<TableColumn>, Stop> {
        let organization_id = command.organization_id;
        self.table(organization_id, command.table_id).await?;
        let columns = self
            .tables
            .columns(organization_id, command.table_id)
            .await?;
        let listed: HashSet<TableColumnId> = command.order.iter().copied().collect();
        let known: HashSet<TableColumnId> = columns.iter().map(|column| column.id).collect();
        if listed.len() != command.order.len() || listed != known {
            return Err(TableError::ColumnOrderMismatch.into());
        }
        let primary = columns.iter().find(|column| column.is_primary);
        if command.order.first() != primary.map(|column| &column.id) {
            return Err(TableError::PrimaryColumnFirst.into());
        }
        self.tables
            .reorder_columns(organization_id, command.table_id, &command.order)
            .await?;
        Ok(self
            .tables
            .columns(organization_id, command.table_id)
            .await?)
    }

    pub async fn create_view(
        &mut self,
        command: CreateTableViewCommand,
        now: DateTime<Utc>,
    ) -> Result<TableOutcome<TableView>, CoreError> {
        settle(self.try_create_view(command, now).await)
    }

    async fn try_create_view(
        &mut self,
        command: CreateTableViewCommand,
        now: DateTime<Utc>,
    ) -> Result<TableView, Stop> {
        let organization_id = command.organization_id;
        self.lock(organization_id, command.table_id).await?;
        let name = TableItemName::try_from(command.name.as_str())?;
        let views = self.tables.views(organization_id, command.table_id).await?;
        if views.len() >= MAX_VIEWS {
            return Err(TableError::TooManyViews.into());
        }
        if views.iter().any(|view| view.name.same_as(name.as_str())) {
            return Err(TableError::ViewNameTaken.into());
        }
        let columns = self
            .tables
            .columns(organization_id, command.table_id)
            .await?;
        let config = command.config.check(command.kind, &columns)?;
        Ok(self
            .tables
            .insert_view(&TableView {
                id: TableViewId(generate_uuid_v7()),
                table_id: command.table_id,
                organization_id,
                name,
                kind: command.kind,
                config,
                position: next_position(views.iter().map(|view| view.position)),
                created_at: now,
                updated_at: now,
            })
            .await?)
    }

    pub async fn update_view(
        &mut self,
        command: UpdateTableViewCommand,
        now: DateTime<Utc>,
    ) -> Result<TableOutcome<TableView>, CoreError> {
        settle(self.try_update_view(command, now).await)
    }

    async fn try_update_view(
        &mut self,
        command: UpdateTableViewCommand,
        now: DateTime<Utc>,
    ) -> Result<TableView, Stop> {
        let organization_id = command.organization_id;
        self.table(organization_id, command.table_id).await?;
        let views = self.tables.views(organization_id, command.table_id).await?;
        let mut view = views
            .iter()
            .find(|view| view.id == command.id)
            .cloned()
            .ok_or(CoreError::NotFound)?;
        if let Some(name) = &command.name {
            let name = TableItemName::try_from(name.as_str())?;
            if views
                .iter()
                .any(|other| other.id != view.id && other.name.same_as(name.as_str()))
            {
                return Err(TableError::ViewNameTaken.into());
            }
            view.name = name;
        }
        if command.kind.is_some() || command.config.is_some() {
            let kind = command.kind.unwrap_or(view.kind);
            let config = command.config.as_ref().unwrap_or(&view.config);
            let columns = self
                .tables
                .columns(organization_id, command.table_id)
                .await?;
            view.config = config.check(kind, &columns)?;
            view.kind = kind;
        }
        view.updated_at = now;
        Ok(self.tables.update_view(&view).await?)
    }

    pub async fn delete_view(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        id: TableViewId,
    ) -> Result<TableOutcome<()>, CoreError> {
        settle(self.try_delete_view(organization_id, table_id, id).await)
    }

    async fn try_delete_view(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        id: TableViewId,
    ) -> Result<(), Stop> {
        self.table(organization_id, table_id).await?;
        let views = self.tables.views(organization_id, table_id).await?;
        if !views.iter().any(|view| view.id == id) {
            return Err(CoreError::NotFound.into());
        }
        if views.len() <= 1 {
            return Err(TableError::LastView.into());
        }
        Ok(self.tables.delete_view(organization_id, id).await?)
    }

    async fn reconcile_views(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        columns: &[TableColumn],
    ) -> Result<(), CoreError> {
        for view in self.tables.views(organization_id, table_id).await? {
            let (kind, config) = view.config.reconcile(view.kind, columns);
            if kind != view.kind || config != view.config {
                self.tables
                    .update_view(&TableView {
                        kind,
                        config,
                        updated_at: Utc::now(),
                        ..view
                    })
                    .await?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::*;
    use crate::{
        KnowledgeResourceType,
        domain::knowledge::tables::{
            ColumnConfig, SelectOptionInput, SelectOptions, TableRowId,
            ports::{MockKnowledgeTableRepository, MockKnowledgeTableRowRepository},
        },
    };

    fn org() -> OrganizationId {
        OrganizationId(generate_uuid_v7())
    }

    fn actor() -> authz::Subject {
        authz::Subject::new(authz::SubjectKind::User, "someone")
    }

    fn table(organization_id: OrganizationId) -> KnowledgeTable {
        KnowledgeTable {
            id: KnowledgeTableId(generate_uuid_v7()),
            organization_id,
            name: TableItemName::try_from("Chantiers").unwrap(),
            description: None,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn column(table: &KnowledgeTable, name: &str, column_type: ColumnType) -> TableColumn {
        TableColumn {
            id: TableColumnId(generate_uuid_v7()),
            table_id: table.id,
            organization_id: table.organization_id,
            name: TableItemName::try_from(name).unwrap(),
            column_type,
            position: 1,
            is_primary: false,
            created_at: Utc::now(),
        }
    }

    fn primary(table: &KnowledgeTable) -> TableColumn {
        TableColumn {
            position: 0,
            is_primary: true,
            ..column(table, PRIMARY_COLUMN_NAME, ColumnType::Text)
        }
    }

    fn view(table: &KnowledgeTable) -> TableView {
        TableView {
            id: TableViewId(generate_uuid_v7()),
            table_id: table.id,
            organization_id: table.organization_id,
            name: TableItemName::try_from(DEFAULT_VIEW_NAME).unwrap(),
            kind: ViewKind::Grid,
            config: ViewConfig::default(),
            position: 0,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    fn finding(tables: &mut MockKnowledgeTableRepository, found: &KnowledgeTable) {
        tables
            .expect_lock_table()
            .returning(|_, _| Box::pin(async { Ok(true) }));
        let found = found.clone();
        tables.expect_find_table().returning(move |_, _| {
            let found = found.clone();
            Box::pin(async move { Ok(Some(found)) })
        });
    }

    fn with_columns(tables: &mut MockKnowledgeTableRepository, columns: Vec<TableColumn>) {
        tables.expect_columns().returning(move |_, _| {
            let columns = columns.clone();
            Box::pin(async move { Ok(columns) })
        });
    }

    fn with_views(tables: &mut MockKnowledgeTableRepository, views: Vec<TableView>) {
        tables.expect_views().returning(move |_, _| {
            let views = views.clone();
            Box::pin(async move { Ok(views) })
        });
    }

    #[tokio::test]
    async fn a_new_table_has_a_primary_text_column_and_a_grid() {
        let mut tables = MockKnowledgeTableRepository::new();
        let mut sequence = mockall::Sequence::new();
        tables
            .expect_lock_organization()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|_| Box::pin(async { Ok(()) }));
        tables
            .expect_count_tables()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|_| Box::pin(async { Ok(0) }));
        tables
            .expect_list_tables()
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));
        tables.expect_insert_table().returning(|table| {
            let table = table.clone();
            Box::pin(async move { Ok(table) })
        });
        tables.expect_insert_column().returning(|column| {
            let column = column.clone();
            Box::pin(async move { Ok(column) })
        });
        tables.expect_insert_view().returning(|view| {
            let view = view.clone();
            Box::pin(async move { Ok(view) })
        });

        let schema = KnowledgeTableService::new(tables, MockKnowledgeTableRowRepository::new())
            .create_table(
                CreateKnowledgeTableCommand {
                    actor: actor(),
                    organization_id: org(),
                    user_id: None,
                    name: " Chantiers ".to_owned(),
                    description: None,
                },
                Utc::now(),
            )
            .await
            .unwrap()
            .unwrap();
        assert_eq!(schema.table.name.as_str(), "Chantiers");
        assert_eq!(schema.columns.len(), 1);
        assert!(schema.columns[0].is_primary);
        assert_eq!(schema.columns[0].name.as_str(), PRIMARY_COLUMN_NAME);
        assert_eq!(schema.columns[0].column_type, ColumnType::Text);
        assert_eq!(schema.views[0].kind, ViewKind::Grid);
    }

    #[tokio::test]
    async fn an_imported_table_takes_its_primary_column_and_the_others_from_the_plan() {
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
        tables.expect_insert_column().times(3).returning(|column| {
            let column = column.clone();
            Box::pin(async move { Ok(column) })
        });
        tables.expect_insert_view().returning(|view| {
            let view = view.clone();
            Box::pin(async move { Ok(view) })
        });
        let plan = |name: &str, column_type: ColumnType| ColumnPlan {
            name: TableItemName::try_from(name).unwrap(),
            column_type,
        };

        let schema = KnowledgeTableService::new(tables, MockKnowledgeTableRowRepository::new())
            .create_table_with_columns(
                CreateKnowledgeTableCommand {
                    actor: actor(),
                    organization_id: org(),
                    user_id: None,
                    name: "Articles".to_owned(),
                    description: None,
                },
                &[
                    plan("Désignation", ColumnType::Text),
                    plan("Prix", ColumnType::Currency),
                    plan("Livré", ColumnType::Checkbox),
                ],
                Utc::now(),
            )
            .await
            .unwrap()
            .unwrap();

        let columns: Vec<_> = schema
            .columns
            .iter()
            .map(|column| {
                (
                    column.name.as_str(),
                    column.kind(),
                    column.position,
                    column.is_primary,
                )
            })
            .collect();
        assert_eq!(
            columns,
            vec![
                ("Désignation", ColumnKind::Text, 0, true),
                ("Prix", ColumnKind::Currency, 1, false),
                ("Livré", ColumnKind::Checkbox, 2, false),
            ]
        );
    }

    #[tokio::test]
    async fn the_two_hundred_and_first_table_is_refused() {
        let mut tables = MockKnowledgeTableRepository::new();
        let mut sequence = mockall::Sequence::new();
        tables
            .expect_lock_organization()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|_| Box::pin(async { Ok(()) }));
        tables
            .expect_count_tables()
            .times(1)
            .in_sequence(&mut sequence)
            .returning(|_| Box::pin(async { Ok(MAX_TABLES) }));
        let outcome = KnowledgeTableService::new(tables, MockKnowledgeTableRowRepository::new())
            .create_table(
                CreateKnowledgeTableCommand {
                    actor: actor(),
                    organization_id: org(),
                    user_id: None,
                    name: "Encore".to_owned(),
                    description: None,
                },
                Utc::now(),
            )
            .await
            .unwrap();
        assert_eq!(outcome, Err(TableError::TooManyTables));
    }

    #[tokio::test]
    async fn the_fifty_first_column_and_a_twin_name_are_refused() {
        let organization_id = org();
        let stored = table(organization_id);
        let mut full: Vec<TableColumn> = (0..MAX_COLUMNS)
            .map(|n| column(&stored, &format!("C{n}"), ColumnType::Text))
            .collect();
        full[0] = primary(&stored);
        let mut tables = MockKnowledgeTableRepository::new();
        finding(&mut tables, &stored);
        with_columns(&mut tables, full);
        let command = |name: &str| CreateTableColumnCommand {
            actor: actor(),
            organization_id,
            table_id: stored.id,
            name: name.to_owned(),
            kind: ColumnKind::Text,
            config: ColumnConfig::default(),
        };
        let mut service =
            KnowledgeTableService::new(tables, MockKnowledgeTableRowRepository::new());
        assert_eq!(
            service
                .create_column(command("Statut"), Utc::now())
                .await
                .unwrap(),
            Err(TableError::TooManyColumns)
        );

        let mut tables = MockKnowledgeTableRepository::new();
        finding(&mut tables, &stored);
        with_columns(
            &mut tables,
            vec![
                primary(&stored),
                column(&stored, "Statut", ColumnType::Text),
            ],
        );
        let mut service =
            KnowledgeTableService::new(tables, MockKnowledgeTableRowRepository::new());
        assert_eq!(
            service
                .create_column(command("statut"), Utc::now())
                .await
                .unwrap(),
            Err(TableError::ColumnNameTaken)
        );
    }

    #[tokio::test]
    async fn the_primary_column_is_never_deleted_nor_retyped() {
        let organization_id = org();
        let stored = table(organization_id);
        let first = primary(&stored);
        let mut tables = MockKnowledgeTableRepository::new();
        finding(&mut tables, &stored);
        with_columns(&mut tables, vec![first.clone()]);
        let mut service =
            KnowledgeTableService::new(tables, MockKnowledgeTableRowRepository::new());
        assert_eq!(
            service
                .delete_column(organization_id, stored.id, first.id)
                .await
                .unwrap(),
            Err(TableError::PrimaryColumnLocked)
        );
        assert_eq!(
            service
                .update_column(UpdateTableColumnCommand {
                    actor: actor(),
                    organization_id,
                    table_id: stored.id,
                    id: first.id,
                    name: None,
                    kind: Some(ColumnKind::Number),
                    config: None,
                })
                .await
                .unwrap(),
            Err(TableError::PrimaryColumnLocked)
        );
        assert_eq!(
            service
                .update_column(UpdateTableColumnCommand {
                    actor: actor(),
                    organization_id,
                    table_id: stored.id,
                    id: first.id,
                    name: None,
                    kind: Some(ColumnKind::LongText),
                    config: None,
                })
                .await
                .unwrap(),
            Err(TableError::PrimaryColumnLocked)
        );
    }

    #[tokio::test]
    async fn the_primary_column_stays_first() {
        let organization_id = org();
        let stored = table(organization_id);
        let first = primary(&stored);
        let other = column(&stored, "Statut", ColumnType::Checkbox);
        let mut tables = MockKnowledgeTableRepository::new();
        finding(&mut tables, &stored);
        with_columns(&mut tables, vec![first.clone(), other.clone()]);
        let mut service =
            KnowledgeTableService::new(tables, MockKnowledgeTableRowRepository::new());
        let reorder = |order: Vec<TableColumnId>| ReorderTableColumnsCommand {
            actor: actor(),
            organization_id,
            table_id: stored.id,
            order,
        };
        assert_eq!(
            service
                .reorder_columns(reorder(vec![other.id, first.id]))
                .await
                .unwrap(),
            Err(TableError::PrimaryColumnFirst)
        );
        assert_eq!(
            service
                .reorder_columns(reorder(vec![first.id]))
                .await
                .unwrap(),
            Err(TableError::ColumnOrderMismatch)
        );
        assert_eq!(
            service
                .reorder_columns(reorder(vec![first.id, other.id, other.id]))
                .await
                .unwrap(),
            Err(TableError::ColumnOrderMismatch)
        );
    }

    #[tokio::test]
    async fn the_last_view_is_kept_and_the_twenty_first_refused() {
        let organization_id = org();
        let stored = table(organization_id);
        let only = view(&stored);
        let mut tables = MockKnowledgeTableRepository::new();
        finding(&mut tables, &stored);
        with_views(&mut tables, vec![only.clone()]);
        let mut service =
            KnowledgeTableService::new(tables, MockKnowledgeTableRowRepository::new());
        assert_eq!(
            service
                .delete_view(organization_id, stored.id, only.id)
                .await
                .unwrap(),
            Err(TableError::LastView)
        );

        let mut tables = MockKnowledgeTableRepository::new();
        finding(&mut tables, &stored);
        with_views(
            &mut tables,
            (0..MAX_VIEWS)
                .map(|n| TableView {
                    id: TableViewId(generate_uuid_v7()),
                    name: TableItemName::try_from(format!("V{n}").as_str()).unwrap(),
                    ..only.clone()
                })
                .collect(),
        );
        let mut service =
            KnowledgeTableService::new(tables, MockKnowledgeTableRowRepository::new());
        assert_eq!(
            service
                .create_view(
                    CreateTableViewCommand {
                        actor: actor(),
                        organization_id,
                        table_id: stored.id,
                        name: "Une de trop".to_owned(),
                        kind: ViewKind::Grid,
                        config: ViewConfig::default(),
                    },
                    Utc::now()
                )
                .await
                .unwrap(),
            Err(TableError::TooManyViews)
        );
    }

    fn options(labels: &[&str]) -> Vec<SelectOptionInput> {
        labels
            .iter()
            .map(|label| SelectOptionInput {
                id: None,
                label: (*label).to_owned(),
                color: None,
            })
            .collect()
    }

    #[tokio::test]
    async fn a_refused_conversion_writes_nothing() {
        let organization_id = org();
        let stored = table(organization_id);
        let parsed = SelectOptions::parse(&options(&["A", "B"])).unwrap();
        let [a, b] = [parsed.as_slice()[0].id, parsed.as_slice()[1].id];
        let tags = column(&stored, "Tags", ColumnType::MultiSelect(parsed));
        let mut tables = MockKnowledgeTableRepository::new();
        finding(&mut tables, &stored);
        with_columns(&mut tables, vec![primary(&stored), tags.clone()]);
        tables.expect_update_column().never();
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_values_of().returning(move |_, _, _| {
            let both: Value = json!([a.to_string(), b.to_string()]);
            Box::pin(async move { Ok(vec![(TableRowId(generate_uuid_v7()), both)]) })
        });
        rows.expect_rewrite_values().never();
        rows.expect_remove_option().never();

        let outcome = KnowledgeTableService::new(tables, rows)
            .update_column(UpdateTableColumnCommand {
                actor: actor(),
                organization_id,
                table_id: stored.id,
                id: tags.id,
                name: None,
                kind: Some(ColumnKind::SingleSelect),
                config: None,
            })
            .await
            .unwrap();
        assert_eq!(outcome, Err(TableError::ConversionLosesValue));
    }

    #[tokio::test]
    async fn a_number_to_date_is_refused_before_reading_rows() {
        let organization_id = org();
        let stored = table(organization_id);
        let amount = column(
            &stored,
            "Quantité",
            ColumnType::from_config(ColumnKind::Number, &ColumnConfig::default()).unwrap(),
        );
        let mut tables = MockKnowledgeTableRepository::new();
        finding(&mut tables, &stored);
        with_columns(&mut tables, vec![primary(&stored), amount.clone()]);
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_values_of().never();
        let outcome = KnowledgeTableService::new(tables, rows)
            .update_column(UpdateTableColumnCommand {
                actor: actor(),
                organization_id,
                table_id: stored.id,
                id: amount.id,
                name: None,
                kind: Some(ColumnKind::Date),
                config: None,
            })
            .await
            .unwrap();
        assert_eq!(outcome, Err(TableError::ConversionRefused));
    }

    #[tokio::test]
    async fn a_removed_option_leaves_the_rows_and_a_removed_column_its_key() {
        let organization_id = org();
        let stored = table(organization_id);
        let before = SelectOptions::parse(&options(&["En cours", "Terminé"])).unwrap();
        let kept = before.as_slice()[0].clone();
        let gone = before.as_slice()[1].id;
        let status = column(&stored, "Statut", ColumnType::SingleSelect(before));
        let link = column(
            &stored,
            "Client",
            ColumnType::Link(KnowledgeResourceType::Customer),
        );
        let mut tables = MockKnowledgeTableRepository::new();
        finding(&mut tables, &stored);
        with_columns(
            &mut tables,
            vec![primary(&stored), status.clone(), link.clone()],
        );
        with_views(&mut tables, vec![view(&stored)]);
        tables.expect_update_column().returning(|column| {
            let column = column.clone();
            Box::pin(async move { Ok(column) })
        });
        tables
            .expect_delete_column()
            .returning(|_, _| Box::pin(async { Ok(()) }));
        let mut rows = MockKnowledgeTableRowRepository::new();
        rows.expect_remove_option()
            .withf(move |_, _, _, kind, option| {
                *kind == ColumnKind::SingleSelect && *option == gone
            })
            .times(1)
            .returning(|_, _, _, _, _| Box::pin(async { Ok(2) }));
        let link_id = link.id;
        rows.expect_remove_column()
            .withf(move |_, _, column| *column == link_id)
            .times(1)
            .returning(|_, _, _| Box::pin(async { Ok(3) }));

        let mut service = KnowledgeTableService::new(tables, rows);
        service
            .update_column(UpdateTableColumnCommand {
                actor: actor(),
                organization_id,
                table_id: stored.id,
                id: status.id,
                name: None,
                kind: None,
                config: Some(ColumnConfig {
                    options: Some(vec![SelectOptionInput {
                        id: Some(kept.id),
                        label: kept.label,
                        color: None,
                    }]),
                    ..ColumnConfig::default()
                }),
            })
            .await
            .unwrap()
            .unwrap();
        service
            .delete_column(organization_id, stored.id, link.id)
            .await
            .unwrap()
            .unwrap();
    }
}
