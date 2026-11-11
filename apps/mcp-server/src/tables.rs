use http::request::Parts;
use rmcp::{
    handler::server::{tool::Extension, wrapper::Parameters},
    model::CallToolResult,
    schemars, tool, tool_router,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};

use crate::{
    api::Fault,
    session::{Session, uuid},
    tools::{MestierTools, finish, json_result},
};

pub const DEFAULT_ROWS: u64 = 50;
pub const MAX_ROWS: u64 = 100;
pub const DELETED_TARGET: &str = "élément supprimé";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Kind {
    Text,
    LongText,
    Number,
    Currency,
    Date,
    Checkbox,
    SingleSelect,
    MultiSelect,
    Link,
    File,
    Member,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct ColumnSetup {
    #[serde(default)]
    pub options: Vec<Choice>,
    #[serde(default)]
    pub link_kind: Option<String>,
    #[serde(default)]
    pub decimals: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct Column {
    pub id: String,
    pub name: String,
    pub kind: Kind,
    #[serde(default)]
    pub config: ColumnSetup,
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Debug, Default, Deserialize)]
pub struct ViewSetup {
    #[serde(default)]
    pub filters: Vec<Value>,
    #[serde(default)]
    pub sorts: Vec<Value>,
    #[serde(default)]
    pub group_by: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct View {
    pub id: String,
    pub name: String,
    pub kind: String,
    #[serde(default)]
    pub config: ViewSetup,
}

#[derive(Debug, Deserialize)]
pub struct Table {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub columns: Vec<Column>,
    #[serde(default)]
    pub views: Vec<View>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ListTablesArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct GetTableArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the table (id from list_tables).")]
    pub table_id: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct ListRowsArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the table (id from list_tables).")]
    pub table_id: String,
    #[schemars(
        description = "UUID of a saved view of the table (see get_table): its filters and sorts apply. Omit for every row, oldest first."
    )]
    pub view_id: Option<String>,
    #[schemars(
        description = "Text to look for, case-insensitive, in the primary column (the row title)."
    )]
    pub search: Option<String>,
    #[schemars(description = "Rows to return. Default 50, at most 100.")]
    pub limit: Option<u64>,
    #[schemars(description = "Rows to skip, for the next page. Default 0.")]
    pub offset: Option<u64>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct CreateRowArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the table (id from list_tables).")]
    pub table_id: String,
    #[schemars(
        description = "Cell values keyed by column name, case-insensitive, as listed by get_table. Omitted columns stay empty."
    )]
    #[serde(default)]
    pub values: Map<String, Value>,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
#[schemars(crate = "rmcp::schemars")]
pub struct UpdateRowArgs {
    #[schemars(
        description = "Slug of the organization. Optional when the user belongs to exactly one organization; see list_organizations."
    )]
    pub organization: Option<String>,
    #[schemars(description = "UUID of the table (id from list_tables).")]
    pub table_id: String,
    #[schemars(description = "UUID of the row (id from list_table_rows).")]
    pub row_id: String,
    #[schemars(
        description = "The cells to change, keyed by column name, case-insensitive. Other cells keep their value; null empties a cell."
    )]
    pub values: Map<String, Value>,
}

pub fn row_limit(requested: Option<u64>) -> u64 {
    requested.unwrap_or(DEFAULT_ROWS).clamp(1, MAX_ROWS)
}

fn listing<'a>(names: impl Iterator<Item = &'a str>) -> String {
    names.collect::<Vec<_>>().join(", ")
}

fn euros(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let magnitude = cents.unsigned_abs();
    format!("{sign}{},{:02} €", magnitude / 100, magnitude % 100)
}

fn option_id<'a>(column: &'a Column, raw: &Value) -> Result<&'a str, Fault> {
    let Some(given) = raw.as_str().map(str::trim) else {
        return Err(Fault::new(format!(
            "la colonne « {} » attend le libellé d'une option",
            column.name
        )));
    };
    let wanted = given.to_lowercase();
    column
        .config
        .options
        .iter()
        .find(|choice| choice.label.to_lowercase() == wanted || choice.id == given)
        .map(|choice| choice.id.as_str())
        .ok_or_else(|| {
            Fault::new(format!(
                "option inconnue « {given} » pour la colonne « {} » ; options : {}",
                column.name,
                listing(
                    column
                        .config
                        .options
                        .iter()
                        .map(|choice| choice.label.as_str())
                )
            ))
        })
}

fn option_label<'a>(column: &'a Column, raw: &Value) -> Option<&'a str> {
    let id = raw.as_str()?;
    column
        .config
        .options
        .iter()
        .find(|choice| choice.id == id)
        .map(|choice| choice.label.as_str())
}

fn cell(column: &Column, raw: Value) -> Result<Value, Fault> {
    if raw.is_null() {
        return Ok(raw);
    }
    Ok(match column.kind {
        Kind::SingleSelect => Value::from(option_id(column, &raw)?),
        Kind::MultiSelect => match raw {
            Value::Array(items) => Value::Array(
                items
                    .iter()
                    .map(|item| option_id(column, item).map(Value::from))
                    .collect::<Result<_, _>>()?,
            ),
            single => Value::Array(vec![Value::from(option_id(column, &single)?)]),
        },
        Kind::Link => match raw {
            Value::String(id) => json!({"kind": column.config.link_kind, "id": id}),
            other => other,
        },
        _ => raw,
    })
}

pub fn api_values(
    columns: &[Column],
    values: Map<String, Value>,
) -> Result<Map<String, Value>, Fault> {
    let mut mapped = Map::new();
    for (name, raw) in values {
        let given = name.trim();
        let wanted = given.to_lowercase();
        let column = columns
            .iter()
            .find(|column| column.name.to_lowercase() == wanted)
            .ok_or_else(|| {
                Fault::new(format!(
                    "colonne inconnue « {given} » ; colonnes de la table : {}",
                    listing(columns.iter().map(|column| column.name.as_str()))
                ))
            })?;
        if mapped.contains_key(&column.id) {
            return Err(Fault::new(format!("colonne « {given} » donnée deux fois")));
        }
        mapped.insert(column.id.clone(), cell(column, raw)?);
    }
    Ok(mapped)
}

fn shown(column: &Column, raw: Value, mut label: Value) -> Value {
    match column.kind {
        Kind::Currency => match raw.as_i64() {
            Some(cents) => Value::from(euros(cents)),
            None => raw,
        },
        Kind::SingleSelect => match option_label(column, &raw) {
            Some(text) => Value::from(text),
            None => raw,
        },
        Kind::MultiSelect => match raw {
            Value::Array(items) => Value::Array(
                items
                    .into_iter()
                    .map(|item| match option_label(column, &item) {
                        Some(text) => Value::from(text),
                        None => item,
                    })
                    .collect(),
            ),
            other => other,
        },
        Kind::Link | Kind::File | Kind::Member => {
            let id = match raw {
                Value::Object(mut link) => link.remove("id").unwrap_or_default(),
                other => other,
            };
            let text = if label["missing"].as_bool() == Some(true) {
                Value::from(DELETED_TARGET)
            } else {
                label["label"].take()
            };
            json!({"id": id, "label": text})
        }
        Kind::Text | Kind::LongText | Kind::Number | Kind::Date | Kind::Checkbox => raw,
    }
}

pub fn render_row(columns: &[Column], mut row: Value) -> Value {
    let mut values = match row["values"].take() {
        Value::Object(values) => values,
        _ => Map::new(),
    };
    let mut labels = row["labels"].take();
    let mut fields = Map::new();
    for column in columns {
        match values.remove(&column.id) {
            None | Some(Value::Null) => {}
            Some(raw) => {
                let label = labels[column.id.as_str()].take();
                fields.insert(column.name.clone(), shown(column, raw, label));
            }
        }
    }
    json!({"id": row["id"].take(), "fields": fields})
}

fn describe(table: &Table) -> Value {
    let columns: Vec<Value> = table
        .columns
        .iter()
        .map(|column| {
            let mut out = json!({
                "id": column.id,
                "name": column.name,
                "kind": column.kind,
                "primary": column.is_primary,
            });
            match column.kind {
                Kind::SingleSelect | Kind::MultiSelect => {
                    out["options"] = column
                        .config
                        .options
                        .iter()
                        .map(|choice| choice.label.as_str())
                        .collect::<Vec<_>>()
                        .into();
                }
                Kind::Link => out["link_kind"] = json!(column.config.link_kind),
                Kind::Number => out["decimals"] = json!(column.config.decimals),
                _ => {}
            }
            out
        })
        .collect();
    let views: Vec<Value> = table
        .views
        .iter()
        .map(|view| {
            let mut out = json!({"id": view.id, "name": view.name, "kind": view.kind});
            if let Some(column) = view
                .config
                .group_by
                .as_deref()
                .and_then(|id| table.columns.iter().find(|column| column.id == id))
            {
                out["group_by"] = json!(column.name);
            }
            out
        })
        .collect();
    json!({
        "id": table.id,
        "name": table.name,
        "description": table.description,
        "columns": columns,
        "views": views,
    })
}

fn values_body(values: Map<String, Value>) -> Value {
    let mut body = Map::new();
    body.insert("values".to_owned(), Value::Object(values));
    Value::Object(body)
}

impl Session<'_> {
    async fn table(&self, org: &str, table_id: &str) -> Result<Table, Fault> {
        let mut body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/knowledge/tables/{table_id}"),
                &[],
            )
            .await?;
        serde_json::from_value(body["data"].take())
            .map_err(|_| Fault::new("table illisible dans la réponse de l'API Mestier"))
    }

    pub async fn list_tables(&self, args: &ListTablesArgs) -> Result<Value, Fault> {
        let org = self.organization_id(args.organization.as_deref()).await?;
        let body = self
            .api
            .get(
                self.token,
                &format!("organizations/{org}/knowledge/tables"),
                &[],
            )
            .await?;
        let tables: Vec<Value> = body["data"]
            .as_array()
            .map(|items| {
                items
                    .iter()
                    .map(|table| {
                        json!({"id": table["id"], "name": table["name"], "description": table["description"]})
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok(Value::Array(tables))
    }

    pub async fn get_table(&self, args: &GetTableArgs) -> Result<Value, Fault> {
        let table_id = uuid(&args.table_id)?;
        let org = self.organization_id(args.organization.as_deref()).await?;
        Ok(describe(&self.table(&org, &table_id).await?))
    }

    pub async fn list_table_rows(&self, args: &ListRowsArgs) -> Result<Value, Fault> {
        let table_id = uuid(&args.table_id)?;
        let view_id = args.view_id.as_deref().map(uuid).transpose()?;
        let org = self.organization_id(args.organization.as_deref()).await?;
        let table = self.table(&org, &table_id).await?;
        let view =
            match view_id.as_deref() {
                Some(id) => Some(table.views.iter().find(|view| view.id == id).ok_or_else(
                    || {
                        Fault::new(format!(
                            "vue introuvable « {id} » ; vues de la table : {}",
                            listing(table.views.iter().map(|view| view.name.as_str()))
                        ))
                    },
                )?),
                None => None,
            };
        let limit = row_limit(args.limit);
        let offset = args.offset.unwrap_or(0);
        let path = format!("organizations/{org}/knowledge/tables/{table_id}/rows");
        let search = args
            .search
            .as_deref()
            .map(str::trim)
            .filter(|text| !text.is_empty());
        let mut body = match search {
            None => {
                let mut query = vec![("limit", limit.to_string()), ("offset", offset.to_string())];
                if let Some(id) = view_id {
                    query.push(("view_id", id));
                }
                self.api.get(self.token, &path, &query).await?
            }
            Some(text) => {
                let primary = table
                    .columns
                    .iter()
                    .find(|column| column.is_primary)
                    .ok_or_else(|| Fault::new("la table n'a pas de colonne principale"))?;
                let contains =
                    json!({"column_id": primary.id, "operator": "contains", "value": text});
                let mut filters: Vec<&Value> = view
                    .map(|view| view.config.filters.iter().collect())
                    .unwrap_or_default();
                filters.push(&contains);
                let sorts: &[Value] = view.map_or(&[], |view| &view.config.sorts);
                self.api
                    .post(
                        self.token,
                        &format!("{path}/query"),
                        &json!({"filters": filters, "sorts": sorts, "limit": limit, "offset": offset}),
                    )
                    .await?
            }
        };
        let data = &mut body["data"];
        let rows: Vec<Value> = match data["rows"].take() {
            Value::Array(rows) => rows
                .into_iter()
                .map(|row| render_row(&table.columns, row))
                .collect(),
            _ => Vec::new(),
        };
        Ok(json!({
            "table": {"id": table.id, "name": table.name},
            "view": view.map(|view| view.name.as_str()),
            "rows": rows,
            "total": data["total"],
            "limit": data["limit"],
            "offset": data["offset"],
        }))
    }

    pub async fn create_table_row(&self, args: CreateRowArgs) -> Result<Value, Fault> {
        let table_id = uuid(&args.table_id)?;
        let org = self.organization_id(args.organization.as_deref()).await?;
        let table = self.table(&org, &table_id).await?;
        let values = api_values(&table.columns, args.values)?;
        let mut body = self
            .api
            .post(
                self.token,
                &format!("organizations/{org}/knowledge/tables/{table_id}/rows"),
                &values_body(values),
            )
            .await?;
        Ok(render_row(&table.columns, body["data"].take()))
    }

    pub async fn update_table_row(&self, args: UpdateRowArgs) -> Result<Value, Fault> {
        let table_id = uuid(&args.table_id)?;
        let row_id = uuid(&args.row_id)?;
        if args.values.is_empty() {
            return Err(Fault::new("aucune modification demandée"));
        }
        let org = self.organization_id(args.organization.as_deref()).await?;
        let table = self.table(&org, &table_id).await?;
        let values = api_values(&table.columns, args.values)?;
        let mut body = self
            .api
            .patch(
                self.token,
                &format!("organizations/{org}/knowledge/tables/{table_id}/rows/{row_id}"),
                &values_body(values),
            )
            .await?;
        Ok(render_row(&table.columns, body["data"].take()))
    }
}

#[tool_router(router = tables_router, vis = "pub(crate)")]
impl MestierTools {
    #[tool(
        description = "List the tables of the organization's knowledge base: tables the users built themselves (site tracking, inventories, supplier contacts...). Returns [{id, name, description}]. Read a table's columns and views with get_table and its rows with list_table_rows."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "list_tables"))]
    async fn list_tables(
        &self,
        Parameters(args): Parameters<ListTablesArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_tables") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { Ok(json_result(&s.list_tables(&args).await?)) }.await)
    }

    #[tool(
        description = "Read the structure of a knowledge base table. Returns {id, name, description, columns, views}. Each column is {id, name, kind, primary} plus options (the labels of a single_select or multi_select column), link_kind (customer, project or quote, for a link column) or decimals (for a number column). kind is one of text, long_text, number, currency, date, checkbox, single_select, multi_select, link, file, member. The primary column is the title of a row. Each view is {id, name, kind (grid or kanban), group_by (the column a kanban groups by)}. Columns, options and views cannot be changed through these tools."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "get_table"))]
    async fn get_table(
        &self,
        Parameters(args): Parameters<GetTableArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "get_table") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { Ok(json_result(&s.get_table(&args).await?)) }.await)
    }

    #[tool(
        description = "List the rows of a knowledge base table, at most 100 per call (default 50), from offset. With view_id the rows are filtered and sorted as in that saved view; without it, oldest first. search keeps the rows whose primary column contains the text, case-insensitive, on top of the view's filters. Returns {table, view, rows, total, limit, offset}. Each row is {id, fields}: fields is keyed by column name, empty cells are left out, and holds the displayed value: text as is, number as a number, currency as text in euros such as \"1250,50 €\" (written back in cents), date as \"YYYY-MM-DD\", checkbox true or false, single_select the option label, multi_select a list of labels, link, file and member as {id, label} (label \"élément supprimé\" when the target was deleted)."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "list_table_rows"))]
    async fn list_table_rows(
        &self,
        Parameters(args): Parameters<ListRowsArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "list_table_rows") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { Ok(json_result(&s.list_table_rows(&args).await?)) }.await)
    }

    #[tool(
        description = "Add a row to a knowledge base table. values is keyed by column name, case-insensitive, as listed by get_table; omitted columns stay empty. Formats: text a string of at most 1000 characters, long_text at most 20000; number a JSON number; currency an integer amount in euro cents (1250,50 € is 125050); date \"YYYY-MM-DD\"; checkbox true or false; single_select the label of an existing option; multi_select a list of option labels; link the UUID of a customer, project or quote, of the kind set by the column; file the UUID of a file article of the knowledge base; member the user id of a member of the organization; null leaves the cell empty. Options and columns cannot be created here. Returns the row as list_table_rows renders it."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "create_table_row"))]
    async fn create_table_row(
        &self,
        Parameters(args): Parameters<CreateRowArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "create_table_row") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { Ok(json_result(&s.create_table_row(args).await?)) }.await)
    }

    #[tool(
        description = "Change cells of one row of a knowledge base table. Only the columns given in values change, keyed by column name, case-insensitive; the other cells keep their value and null empties a cell. Value formats are those of create_table_row: currency in euro cents, date \"YYYY-MM-DD\", options by label, link, file and member by id. Rows cannot be deleted. Returns the row as list_table_rows renders it."
    )]
    #[tracing::instrument(name = "mcp.tool", skip_all, fields(tool = "update_table_row"))]
    async fn update_table_row(
        &self,
        Parameters(args): Parameters<UpdateRowArgs>,
        Extension(parts): Extension<Parts>,
    ) -> CallToolResult {
        let s = match self.session(&parts, "update_table_row") {
            Ok(s) => s,
            Err(refusal) => return refusal,
        };
        finish(async { Ok(json_result(&s.update_table_row(args).await?)) }.await)
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Map, Value, json};

    use super::*;

    fn columns() -> Vec<Column> {
        serde_json::from_value(json!([
            {"id": "c-name", "name": "Nom", "kind": "text", "config": {}, "is_primary": true},
            {"id": "c-status", "name": "Statut", "kind": "single_select",
             "config": {"options": [{"id": "o-on", "label": "En cours"}, {"id": "o-done", "label": "Terminé"}]}},
            {"id": "c-lots", "name": "Lots", "kind": "multi_select",
             "config": {"options": [{"id": "o-lawn", "label": "Gazon"}]}},
            {"id": "c-client", "name": "Client", "kind": "link", "config": {"link_kind": "project"}},
            {"id": "c-file", "name": "Plan", "kind": "file", "config": {}},
            {"id": "c-amount", "name": "Montant", "kind": "currency", "config": {}}
        ]))
        .unwrap()
    }

    fn values(raw: Value) -> Map<String, Value> {
        raw.as_object().unwrap().clone()
    }

    #[test]
    fn column_names_are_matched_whatever_the_case_and_options_by_label() {
        let mapped = api_values(
            &columns(),
            values(json!({
                " NOM ": "Haie", "statut": "TERMINÉ", "lots": "gazon",
                "Client": "p-1", "plan": "f-1", "Montant": 1200
            })),
        )
        .unwrap();
        assert_eq!(
            Value::Object(mapped),
            json!({
                "c-name": "Haie", "c-status": "o-done", "c-lots": ["o-lawn"],
                "c-client": {"kind": "project", "id": "p-1"}, "c-file": "f-1", "c-amount": 1200
            })
        );
    }

    #[test]
    fn an_option_id_is_accepted_and_null_clears_the_cell() {
        let mapped = api_values(
            &columns(),
            values(
                json!({"Statut": "o-on", "Lots": null, "Client": {"kind": "project", "id": "p-2"}}),
            ),
        )
        .unwrap();
        assert_eq!(
            Value::Object(mapped),
            json!({"c-status": "o-on", "c-lots": null, "c-client": {"kind": "project", "id": "p-2"}})
        );
    }

    #[test]
    fn an_unknown_column_names_the_columns_of_the_table() {
        let fault = api_values(&columns(), values(json!({"Couleur": "vert"}))).unwrap_err();
        assert_eq!(
            fault.message,
            "colonne inconnue « Couleur » ; colonnes de la table : Nom, Statut, Lots, Client, Plan, Montant"
        );
    }

    #[test]
    fn an_unknown_option_names_the_options_of_the_column() {
        let fault =
            api_values(&columns(), values(json!({"Lots": ["Gazon", "Taille"]}))).unwrap_err();
        assert_eq!(
            fault.message,
            "option inconnue « Taille » pour la colonne « Lots » ; options : Gazon"
        );
        let fault = api_values(&columns(), values(json!({"Statut": 3}))).unwrap_err();
        assert_eq!(
            fault.message,
            "la colonne « Statut » attend le libellé d'une option"
        );
    }

    #[test]
    fn a_column_given_twice_is_refused() {
        let fault = api_values(&columns(), values(json!({"Nom": "a", "nom": "b"}))).unwrap_err();
        assert_eq!(fault.message, "colonne « nom » donnée deux fois");
    }

    #[test]
    fn a_row_is_rendered_by_column_name_and_a_deleted_target_says_so() {
        let row = render_row(
            &columns(),
            json!({
                "id": "r-1",
                "values": {"c-name": "Haie", "c-status": "o-on", "c-lots": ["o-lawn"],
                           "c-client": {"kind": "project", "id": "p-1"}, "c-file": "f-1",
                           "c-amount": -5},
                "labels": {"c-client": {"label": null, "missing": true},
                           "c-file": {"label": "plan.pdf", "missing": false}}
            }),
        );
        assert_eq!(
            row,
            json!({"id": "r-1", "fields": {
                "Nom": "Haie", "Statut": "En cours", "Lots": ["Gazon"],
                "Client": {"id": "p-1", "label": "élément supprimé"},
                "Plan": {"id": "f-1", "label": "plan.pdf"}, "Montant": "-0,05 €"
            }})
        );
    }

    #[test]
    fn the_row_limit_defaults_and_is_capped() {
        assert_eq!(row_limit(None), 50);
        assert_eq!(row_limit(Some(500)), 100);
        assert_eq!(row_limit(Some(0)), 1);
    }
}
