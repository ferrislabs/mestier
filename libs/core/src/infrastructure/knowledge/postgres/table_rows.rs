use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::{CoreError, UserId};
use mestier_macros::repository;
use serde_json::{Map, Value};
use sqlx::{Postgres, QueryBuilder};
use uuid::Uuid;

use crate::{
    KnowledgeArticleId, KnowledgeResource, KnowledgeResourceType, OrganizationId,
    domain::knowledge::tables::{
        ColumnKind, KnowledgeTableId, RowBatch, RowCondition, RowQuery, RowUpdate, SelectOptionId,
        SortDirection, TableColumnId, TableRow, TableRowId,
        filter::{Comparison, SortKey},
        ports::KnowledgeTableRowRepository,
        value::CellReference,
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[derive(sqlx::FromRow)]
struct RowRecord {
    id: Uuid,
    table_id: Uuid,
    org_id: Uuid,
    values: Value,
    created_by: Option<Uuid>,
    updated_by: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<RowRecord> for TableRow {
    type Error = CoreError;

    fn try_from(record: RowRecord) -> Result<Self, Self::Error> {
        let Value::Object(values) = record.values else {
            return Err(CoreError::Internal(
                "a stored table row holds no object".into(),
            ));
        };
        Ok(Self {
            id: TableRowId(record.id),
            table_id: KnowledgeTableId(record.table_id),
            organization_id: OrganizationId(record.org_id),
            values,
            created_by: record.created_by.map(UserId),
            updated_by: record.updated_by.map(UserId),
            created_at: record.created_at,
            updated_at: record.updated_at,
        })
    }
}

const ROW_COLUMNS: &str = r#"SELECT id, table_id, org_id, "values", created_by, updated_by, created_at, updated_at FROM knowledge_table_rows WHERE "#;

fn operator(comparison: Comparison) -> &'static str {
    match comparison {
        Comparison::Eq => " = ",
        Comparison::Ne => " IS DISTINCT FROM ",
        Comparison::Lt => " < ",
        Comparison::Le => " <= ",
        Comparison::Gt => " > ",
        Comparison::Ge => " >= ",
    }
}

fn push_number(builder: &mut QueryBuilder<'_, Postgres>, key: &str) {
    builder.push(r#"(CASE WHEN jsonb_typeof("values" -> "#);
    builder.push_bind(key.to_owned());
    builder.push(r#") = 'number' THEN ("values" ->> "#);
    builder.push_bind(key.to_owned());
    builder.push(")::numeric END)");
}

fn push_date(builder: &mut QueryBuilder<'_, Postgres>, key: &str) {
    builder.push(r#"(CASE WHEN jsonb_typeof("values" -> "#);
    builder.push_bind(key.to_owned());
    builder.push(r#") = 'string' THEN ("values" ->> "#);
    builder.push_bind(key.to_owned());
    builder.push(")::date END)");
}

fn push_text(builder: &mut QueryBuilder<'_, Postgres>, key: &str) {
    builder.push(r#"("values" ->> "#);
    builder.push_bind(key.to_owned());
    builder.push(")");
}

fn push_scope(
    builder: &mut QueryBuilder<'_, Postgres>,
    organization_id: OrganizationId,
    table_id: KnowledgeTableId,
    query: &RowQuery,
) {
    builder.push("org_id = ");
    builder.push_bind(organization_id.0);
    builder.push(" AND table_id = ");
    builder.push_bind(table_id.0);
    for filter in &query.filters {
        let key = filter.column.to_string();
        builder.push(" AND ");
        match &filter.condition {
            RowCondition::Empty => {
                builder.push(r#"NOT ("values" ? "#);
                builder.push_bind(key);
                builder.push(")");
            }
            RowCondition::NotEmpty => {
                builder.push(r#"("values" ? "#);
                builder.push_bind(key);
                builder.push(")");
            }
            RowCondition::TextContains(text) => {
                builder.push("strpos(lower(");
                push_text(builder, &key);
                builder.push("), lower(");
                builder.push_bind(text.clone());
                builder.push(")) > 0");
            }
            RowCondition::TextIs(text) => {
                builder.push("lower(");
                push_text(builder, &key);
                builder.push(") = lower(");
                builder.push_bind(text.clone());
                builder.push(")");
            }
            RowCondition::NumberCompare(comparison, number) => {
                push_number(builder, &key);
                builder.push(operator(*comparison));
                builder.push_bind(number.clone());
                builder.push("::numeric");
            }
            RowCondition::DateCompare(comparison, date) => {
                push_date(builder, &key);
                builder.push(operator(*comparison));
                builder.push_bind(*date);
            }
            RowCondition::Checked(checked) => {
                builder.push(r#"("values" -> "#);
                builder.push_bind(key);
                builder.push(if *checked {
                    ") = 'true'::jsonb"
                } else {
                    ") IS DISTINCT FROM 'true'::jsonb"
                });
            }
            RowCondition::OptionIs(id) | RowCondition::IdIs(id) => {
                push_text(builder, &key);
                builder.push(" = ");
                builder.push_bind(id.clone());
            }
            RowCondition::OptionIsNot(id) => {
                push_text(builder, &key);
                builder.push(" IS DISTINCT FROM ");
                builder.push_bind(id.clone());
            }
            RowCondition::OptionAnyOf(ids) => {
                push_text(builder, &key);
                builder.push(" = ANY(");
                builder.push_bind(ids.clone());
                builder.push("::text[])");
            }
            RowCondition::ListAnyOf(ids) => {
                builder.push(r#"("values" -> "#);
                builder.push_bind(key);
                builder.push(") ?| ");
                builder.push_bind(ids.clone());
                builder.push("::text[]");
            }
            RowCondition::ListAllOf(ids) => {
                builder.push(r#"("values" -> "#);
                builder.push_bind(key);
                builder.push(") ?& ");
                builder.push_bind(ids.clone());
                builder.push("::text[]");
            }
            RowCondition::LinkIs(id) => {
                builder.push(r#"("values" -> "#);
                builder.push_bind(key);
                builder.push(" ->> 'id') = ");
                builder.push_bind(id.clone());
            }
        }
    }
}

fn push_order(builder: &mut QueryBuilder<'_, Postgres>, query: &RowQuery) {
    builder.push(" ORDER BY ");
    for sort in &query.sorts {
        let key = sort.column.to_string();
        match &sort.key {
            SortKey::Text => {
                builder.push("lower(");
                push_text(builder, &key);
                builder.push(")");
            }
            SortKey::Number => push_number(builder, &key),
            SortKey::Date => push_date(builder, &key),
            SortKey::Checkbox => {
                builder.push(r#"(CASE WHEN jsonb_typeof("values" -> "#);
                builder.push_bind(key.clone());
                builder.push(r#") = 'boolean' THEN ("values" ->> "#);
                builder.push_bind(key);
                builder.push(")::boolean END)");
            }
            SortKey::OptionRank(ids) => {
                builder.push("array_position(");
                builder.push_bind(ids.clone());
                builder.push("::text[], ");
                push_text(builder, &key);
                builder.push(")");
            }
            SortKey::FirstOptionRank(ids) => {
                builder.push("array_position(");
                builder.push_bind(ids.clone());
                builder.push(r#"::text[], ("values" -> "#);
                builder.push_bind(key);
                builder.push(" ->> 0))");
            }
        }
        builder.push(match sort.direction {
            SortDirection::Asc => " ASC NULLS LAST, ",
            SortDirection::Desc => " DESC NULLS LAST, ",
        });
    }
    builder.push("created_at ASC, id ASC");
}

#[repository(domain = KnowledgeTableRow, backend = Postgres)]
pub struct PgKnowledgeTableRowRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgKnowledgeTableRowRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> KnowledgeTableRowRepository for PgKnowledgeTableRowRepository<'tx> {
    async fn count(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
    ) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;
        let count = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM knowledge_table_rows WHERE table_id = $1 AND org_id = $2"#,
            table_id.0,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(u64::try_from(count).unwrap_or_default())
    }

    async fn insert(&mut self, row: &TableRow) -> Result<TableRow, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            RowRecord,
            r#"
            INSERT INTO knowledge_table_rows (id, table_id, org_id, "values", created_by, updated_by, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, table_id, org_id, "values", created_by, updated_by, created_at, updated_at
            "#,
            row.id.0,
            row.table_id.0,
            row.organization_id.0,
            Value::Object(row.values.clone()),
            row.created_by.map(|id| id.0),
            row.updated_by.map(|id| id.0),
            row.created_at,
            row.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .try_into()
    }

    async fn insert_many(&mut self, batch: &RowBatch) -> Result<u64, CoreError> {
        if batch.rows.is_empty() {
            return Ok(0);
        }
        let ids: Vec<Uuid> = batch.rows.iter().map(|(id, _)| id.0).collect();
        let values: Vec<Value> = batch
            .rows
            .iter()
            .map(|(_, values)| Value::Object(values.to_json()))
            .collect();
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            INSERT INTO knowledge_table_rows (id, table_id, org_id, "values", created_by, updated_by, created_at, updated_at)
            SELECT u.id, $1, $2, u.values, $3, $3, $4, $4
            FROM UNNEST($5::uuid[], $6::jsonb[]) AS u(id, values)
            "#,
            batch.table_id.0,
            batch.organization_id.0,
            batch.created_by.map(|id| id.0),
            batch.created_at,
            &ids,
            &values,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(result.rows_affected())
    }

    async fn find(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        id: TableRowId,
    ) -> Result<Option<TableRow>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            RowRecord,
            r#"
            SELECT id, table_id, org_id, "values", created_by, updated_by, created_at, updated_at
            FROM knowledge_table_rows
            WHERE id = $1 AND table_id = $2 AND org_id = $3
            "#,
            id.0,
            table_id.0,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .map(TableRow::try_from)
        .transpose()
    }

    async fn patch(&mut self, update: &RowUpdate) -> Result<Option<TableRow>, CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query_as!(
            RowRecord,
            r#"
            UPDATE knowledge_table_rows
            SET "values" = ("values" || $4::jsonb) - $5::text[], updated_by = $6, updated_at = $7
            WHERE id = $1 AND table_id = $2 AND org_id = $3
            RETURNING id, table_id, org_id, "values", created_by, updated_by, created_at, updated_at
            "#,
            update.id.0,
            update.table_id.0,
            update.organization_id.0,
            Value::Object(update.set.clone()),
            &update.cleared,
            update.updated_by.map(|id| id.0),
            update.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .map(TableRow::try_from)
        .transpose()
    }

    async fn delete(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        ids: &[TableRowId],
    ) -> Result<u64, CoreError> {
        let ids: Vec<Uuid> = ids.iter().map(|id| id.0).collect();
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            "DELETE FROM knowledge_table_rows WHERE table_id = $1 AND org_id = $2 AND id = ANY($3)",
            table_id.0,
            organization_id.0,
            &ids,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(result.rows_affected())
    }

    async fn list(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        query: &RowQuery,
    ) -> Result<(Vec<TableRow>, u64), CoreError> {
        let limit = i64::try_from(query.limit).unwrap_or(i64::MAX);
        let offset = i64::try_from(query.offset).unwrap_or(i64::MAX);
        let mut select = QueryBuilder::<Postgres>::new(ROW_COLUMNS);
        push_scope(&mut select, organization_id, table_id, query);
        push_order(&mut select, query);
        select.push(" LIMIT ");
        select.push_bind(limit);
        select.push(" OFFSET ");
        select.push_bind(offset);

        let mut count =
            QueryBuilder::<Postgres>::new("SELECT COUNT(*) FROM knowledge_table_rows WHERE ");
        push_scope(&mut count, organization_id, table_id, query);

        let mut tx = self.tx.lock().await;
        let records = select
            .build_query_as::<RowRecord>()
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        let total: i64 = count
            .build_query_scalar()
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        let rows = records
            .into_iter()
            .map(TableRow::try_from)
            .collect::<Result<_, _>>()?;
        Ok((rows, u64::try_from(total).unwrap_or_default()))
    }

    async fn remove_column(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        column: TableColumnId,
    ) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE knowledge_table_rows SET "values" = "values" - $3::text
            WHERE table_id = $1 AND org_id = $2 AND "values" ? $3
            "#,
            table_id.0,
            organization_id.0,
            column.to_string(),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(result.rows_affected())
    }

    async fn remove_option(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        column: TableColumnId,
        kind: ColumnKind,
        option: SelectOptionId,
    ) -> Result<u64, CoreError> {
        let mut tx = self.tx.lock().await;
        let result = if kind == ColumnKind::MultiSelect {
            sqlx::query!(
                r#"
                UPDATE knowledge_table_rows
                SET "values" = CASE
                    WHEN jsonb_array_length(("values" -> $3::text) - $4::text) = 0 THEN "values" - $3::text
                    ELSE jsonb_set("values", ARRAY[$3::text], ("values" -> $3::text) - $4::text)
                END
                WHERE table_id = $1 AND org_id = $2 AND ("values" -> $3::text) ? $4::text
                "#,
                table_id.0,
                organization_id.0,
                column.to_string(),
                option.to_string(),
            )
            .execute(&mut ***tx)
            .await
        } else {
            sqlx::query!(
                r#"
                UPDATE knowledge_table_rows SET "values" = "values" - $3::text
                WHERE table_id = $1 AND org_id = $2 AND "values" ->> $3::text = $4::text
                "#,
                table_id.0,
                organization_id.0,
                column.to_string(),
                option.to_string(),
            )
            .execute(&mut ***tx)
            .await
        }
        .map_err(map_sqlx_error)?;
        Ok(result.rows_affected())
    }

    async fn values_of(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        column: TableColumnId,
    ) -> Result<Vec<(TableRowId, Value)>, CoreError> {
        let mut tx = self.tx.lock().await;
        let records = sqlx::query!(
            r#"
            SELECT id, "values" -> $3::text AS "value!"
            FROM knowledge_table_rows
            WHERE table_id = $1 AND org_id = $2 AND "values" ? $3::text
            FOR UPDATE
            "#,
            table_id.0,
            organization_id.0,
            column.to_string(),
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(records
            .into_iter()
            .map(|record| (TableRowId(record.id), record.value))
            .collect())
    }

    async fn rewrite_values(
        &mut self,
        organization_id: OrganizationId,
        table_id: KnowledgeTableId,
        column: TableColumnId,
        values: &[(TableRowId, Value)],
    ) -> Result<u64, CoreError> {
        let rewritten: Map<String, Value> = values
            .iter()
            .map(|(id, value)| (id.to_string(), value.clone()))
            .collect();
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE knowledge_table_rows r
            SET "values" = jsonb_set(r."values", ARRAY[$3::text], u.value)
            FROM jsonb_each($4::jsonb) AS u(id, value)
            WHERE r.id = u.id::uuid AND r.table_id = $1 AND r.org_id = $2
            "#,
            table_id.0,
            organization_id.0,
            column.to_string(),
            Value::Object(rewritten),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;
        Ok(result.rows_affected())
    }

    async fn labels(
        &mut self,
        organization_id: OrganizationId,
        references: &[CellReference],
    ) -> Result<HashMap<CellReference, String>, CoreError> {
        let mut customers = Vec::new();
        let mut projects = Vec::new();
        let mut quotes = Vec::new();
        let mut files = Vec::new();
        let mut members = Vec::new();
        for reference in references {
            match reference {
                CellReference::Resource(resource) => match resource.resource_type {
                    KnowledgeResourceType::Customer => customers.push(resource.id),
                    KnowledgeResourceType::Project => projects.push(resource.id),
                    KnowledgeResourceType::Quote => quotes.push(resource.id),
                },
                CellReference::File(id) => files.push(id.0),
                CellReference::Member(id) => members.push(id.0),
            }
        }
        let resource =
            |resource_type, id| CellReference::Resource(KnowledgeResource { resource_type, id });
        let mut labels = HashMap::new();
        let mut tx = self.tx.lock().await;
        if !customers.is_empty() {
            for record in sqlx::query!(
                "SELECT id, name FROM customers WHERE org_id = $1 AND id = ANY($2) AND deleted_at IS NULL",
                organization_id.0,
                &customers,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            {
                labels.insert(
                    resource(KnowledgeResourceType::Customer, record.id),
                    record.name,
                );
            }
        }
        if !projects.is_empty() {
            for record in sqlx::query!(
                "SELECT id, name FROM projects WHERE org_id = $1 AND id = ANY($2)",
                organization_id.0,
                &projects,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            {
                labels.insert(
                    resource(KnowledgeResourceType::Project, record.id),
                    record.name,
                );
            }
        }
        if !quotes.is_empty() {
            for record in sqlx::query!(
                r#"
                SELECT id, COALESCE(reference || ' ' || title, title) AS "label!"
                FROM quotes WHERE org_id = $1 AND id = ANY($2) AND deleted_at IS NULL
                "#,
                organization_id.0,
                &quotes,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            {
                labels.insert(
                    resource(KnowledgeResourceType::Quote, record.id),
                    record.label,
                );
            }
        }
        if !files.is_empty() {
            for record in sqlx::query!(
                "SELECT id, title FROM knowledge_articles WHERE org_id = $1 AND id = ANY($2) AND kind = 'file'",
                organization_id.0,
                &files,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            {
                labels.insert(
                    CellReference::File(KnowledgeArticleId(record.id)),
                    record.title,
                );
            }
        }
        if !members.is_empty() {
            for record in sqlx::query!(
                r#"
                SELECT user_id AS "user_id!", btrim(COALESCE(first_name || ' ', '') || last_name) AS "label!"
                FROM organization_members
                WHERE organization_id = $1 AND user_id = ANY($2) AND deleted_at IS NULL
                "#,
                organization_id.0,
                &members,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            {
                labels.insert(
                    CellReference::Member(UserId(record.user_id)),
                    record.label,
                );
            }
        }
        Ok(labels)
    }
}
