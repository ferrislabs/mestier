use auth::Identity;
use axum::{
    Extension, Json,
    body::Bytes,
    extract::{Multipart, State, multipart::MultipartError},
    response::IntoResponse,
};
use handlers::{ApiError, AppState, Response, require_org_membership, resolve_actor};
use http::{
    HeaderValue, StatusCode,
    header::{CACHE_CONTROL, CONTENT_DISPOSITION, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS},
};
use mestier_core::{
    ExportTableViewCommand, ImportKnowledgeTableCommand, ImportTableRowsCommand, MAX_CSV_BYTES,
    TableError,
};
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use utoipa::ToSchema;

use crate::tables::{
    TableImportPath, TableRowsImportPath, ViewExportPath, refused,
    response::{ImportedKnowledgeTableResponse, TableImportReportResponse},
};

pub const MAX_IMPORT_BODY_BYTES: usize = MAX_CSV_BYTES + 64 * 1024;

#[derive(Debug, ToSchema)]
pub struct ImportKnowledgeTableForm {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
    pub name: Option<String>,
}

#[derive(Debug, ToSchema)]
pub struct ImportTableRowsForm {
    #[schema(value_type = String, format = Binary)]
    pub file: Vec<u8>,
}

#[derive(Debug)]
pub enum ImportError {
    TooLarge,
    Api(ApiError),
}

impl From<ApiError> for ImportError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl From<common::CoreError> for ImportError {
    fn from(error: common::CoreError) -> Self {
        Self::Api(error.into())
    }
}

impl From<TableError> for ImportError {
    fn from(error: TableError) -> Self {
        match error {
            TableError::CsvTooLarge => Self::TooLarge,
            other => Self::Api(refused(other)),
        }
    }
}

impl From<MultipartError> for ImportError {
    fn from(error: MultipartError) -> Self {
        if error.status() == StatusCode::PAYLOAD_TOO_LARGE {
            Self::TooLarge
        } else {
            Self::Api(ApiError::BadRequest(error.body_text()))
        }
    }
}

impl IntoResponse for ImportError {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Api(error) => error.into_response(),
            Self::TooLarge => (
                StatusCode::PAYLOAD_TOO_LARGE,
                Json(serde_json::json!({
                    "code": "E_PAYLOAD_TOO_LARGE",
                    "message": TableError::CsvTooLarge.to_string(),
                    "status": 413,
                })),
            )
                .into_response(),
        }
    }
}

struct ImportForm {
    file: Bytes,
    file_name: Option<String>,
    name: Option<String>,
}

async fn read_form(mut multipart: Multipart) -> Result<ImportForm, ImportError> {
    let mut file = None;
    let mut file_name = None;
    let mut name = None;
    while let Some(field) = multipart.next_field().await? {
        match field.name() {
            Some("file") => {
                file_name = field.file_name().map(str::to_owned);
                file = Some(field.bytes().await?);
            }
            Some("name") => name = Some(field.text().await?),
            _ => {}
        }
    }
    let file = file.ok_or_else(|| ApiError::Validation("file: required".to_owned()))?;
    if file.len() > MAX_CSV_BYTES {
        return Err(ImportError::TooLarge);
    }
    Ok(ImportForm {
        file,
        file_name,
        name,
    })
}

fn stem(file_name: &str) -> &str {
    let base = file_name.rsplit(['/', '\\']).next().unwrap_or(file_name);
    match base.rsplit_once('.') {
        Some((stem, _)) if !stem.trim().is_empty() => stem,
        _ => base,
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/import",
    operation_id = "importKnowledgeTable",
    tag = crate::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    request_body(content = ImportKnowledgeTableForm, content_type = "multipart/form-data", description = "`file`: the CSV (`;`, `,` or tab, UTF-8 or Windows-1252); `name`: the table name, the file name without its extension when absent"),
    responses(
        (status = 201, description = "The table, one column per header with a guessed type (number, date, checkbox, text or long text; the first column is the primary text column), its rows, and the import report. No row event is emitted", body = inline(handlers::DataEnvelope<ImportedKnowledgeTableResponse>)),
        (status = 400, description = "The body is not a multipart form"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "A table already has this name, whatever the case"),
        (status = 413, description = "The file exceeds 5 MB"),
        (status = 422, description = "No file, no header line, more than 50 columns or 10000 rows, an invalid name, or 200 tables already; nothing is created"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn import_table(
    TableImportPath { organization_id }: TableImportPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    multipart: Multipart,
) -> Result<Response<ImportedKnowledgeTableResponse>, ImportError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let form = read_form(multipart).await?;
    let name = form
        .name
        .filter(|name| !name.trim().is_empty())
        .or_else(|| form.file_name.as_deref().map(|file| stem(file).to_owned()))
        .ok_or_else(|| ApiError::UnprocessableEntity("name: required".to_owned()))?;
    let imported = state
        .usecase
        .acting_as(user_id)
        .import_knowledge_table(ImportKnowledgeTableCommand {
            actor,
            organization_id,
            user_id: Some(user_id),
            name,
            file: form.file,
        })
        .await??;
    Ok(Response::Created(imported.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/import",
    operation_id = "importTableRows",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
    ),
    request_body(content = ImportTableRowsForm, content_type = "multipart/form-data", description = "`file`: the CSV; its headers are matched to the columns by name, whatever the case"),
    responses(
        (status = 200, description = "The import report: rows created, ignored columns, and the invalid cells left empty. No row event is emitted", body = inline(handlers::DataEnvelope<TableImportReportResponse>)),
        (status = 400, description = "The body is not a multipart form"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table not found, or held by another organization"),
        (status = 413, description = "The file exceeds 5 MB"),
        (status = 422, description = "No file, no header line, no header matching a column, or more rows than the file or the table may hold; nothing is written"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn import_rows(
    TableRowsImportPath {
        organization_id,
        table_id,
    }: TableRowsImportPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    multipart: Multipart,
) -> Result<Response<TableImportReportResponse>, ImportError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let form = read_form(multipart).await?;
    let report = state
        .usecase
        .acting_as(user_id)
        .import_table_rows(ImportTableRowsCommand {
            actor,
            organization_id,
            table_id,
            user_id: Some(user_id),
            file: form.file,
        })
        .await??;
    Ok(Response::OK(report.into()))
}

fn folded(character: char) -> Option<&'static str> {
    Some(match character {
        'à' | 'â' | 'ä' | 'á' => "a",
        'À' | 'Â' | 'Ä' | 'Á' => "A",
        'é' | 'è' | 'ê' | 'ë' => "e",
        'É' | 'È' | 'Ê' | 'Ë' => "E",
        'î' | 'ï' | 'í' => "i",
        'Î' | 'Ï' | 'Í' => "I",
        'ô' | 'ö' | 'ó' => "o",
        'Ô' | 'Ö' | 'Ó' => "O",
        'ù' | 'û' | 'ü' | 'ú' => "u",
        'Ù' | 'Û' | 'Ü' | 'Ú' => "U",
        'ç' => "c",
        'Ç' => "C",
        'œ' => "oe",
        'Œ' => "OE",
        'æ' => "ae",
        'Æ' => "AE",
        _ => return None,
    })
}

fn ascii_fallback(name: &str) -> String {
    let mut fallback = String::with_capacity(name.len());
    for character in name.chars() {
        match folded(character) {
            Some(letters) => fallback.push_str(letters),
            None if character.is_ascii_alphanumeric()
                || matches!(character, ' ' | '-' | '_' | '.' | '(' | ')') =>
            {
                fallback.push(character);
            }
            None => fallback.push('_'),
        }
    }
    fallback
}

pub fn attachment(table: &str, view: &str) -> String {
    let file_name = format!("{table} - {view}.csv").replace(['/', '\\'], "_");
    format!(
        "attachment; filename=\"{}\"; filename*=UTF-8''{}",
        ascii_fallback(&file_name),
        utf8_percent_encode(&file_name, NON_ALPHANUMERIC)
    )
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/knowledge/tables/{table_id}/views/{view_id}/export",
    operation_id = "exportTableView",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("table_id" = mestier_core::KnowledgeTableId, Path, description = "Table identifier"),
        ("view_id" = mestier_core::TableViewId, Path, description = "View identifier"),
    ),
    responses(
        (status = 200, description = "The rows of the saved view (its filters, sorts and visible columns, in column order) as CSV for a French Excel: `;`, UTF-8 with a byte order mark, CRLF, decimal comma, amounts in euros without symbol, dates JJ/MM/AAAA, Oui/Non, choices, links, files and members by their label", content_type = "text/csv", body = String),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Table or view not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn export_view(
    ViewExportPath {
        organization_id,
        table_id,
        view_id,
    }: ViewExportPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<impl IntoResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;
    let export = state
        .usecase
        .export_table_view(ExportTableViewCommand {
            actor,
            organization_id,
            table_id,
            view_id,
        })
        .await?
        .map_err(refused)?;
    let disposition = HeaderValue::try_from(attachment(&export.table_name, &export.view_name))
        .map_err(|_| ApiError::Internal)?;
    Ok((
        StatusCode::OK,
        [
            (
                CONTENT_TYPE,
                HeaderValue::from_static("text/csv; charset=utf-8"),
            ),
            (CONTENT_DISPOSITION, disposition),
            (X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff")),
            (CACHE_CONTROL, HeaderValue::from_static("private, no-store")),
        ],
        export.bytes,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_attachment_names_the_table_and_the_view_with_an_ascii_fallback() {
        assert_eq!(
            attachment("Matériaux", "Écarts / à voir"),
            "attachment; filename=\"Materiaux - Ecarts _ a voir.csv\"; filename*=UTF-8''Mat%C3%A9riaux%20%2D%20%C3%89carts%20%5F%20%C3%A0%20voir%2Ecsv"
        );
        assert_eq!(
            attachment("Devis \"2026\"", "Grille"),
            "attachment; filename=\"Devis _2026_ - Grille.csv\"; filename*=UTF-8''Devis%20%222026%22%20%2D%20Grille%2Ecsv"
        );
    }

    #[test]
    fn a_file_name_loses_its_folder_and_extension() {
        assert_eq!(stem("C:\\Exports\\Articles.csv"), "Articles");
        assert_eq!(stem("stock.2026.csv"), "stock.2026");
        assert_eq!(stem(".csv"), ".csv");
        assert_eq!(stem("Articles"), "Articles");
    }
}
