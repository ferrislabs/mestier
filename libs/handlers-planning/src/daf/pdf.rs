use auth::Identity;
use axum::{
    Extension,
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use doc_render::TypstDafRenderer;
use handlers::{ApiError, AppState};
use http::{
    StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use mestier_core::{BundledFile, DafDocument, DafRenderer, DoeRenderError, FileBundle};

use crate::{
    daf::{DafPdfPath, require_daf},
    doe::pdf::{content_disposition, off_the_executor},
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/dafs/{daf_id}/pdf",
    operation_id = "exportDafPdf",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("daf_id" = mestier_core::DafId, Path, description = "DAF identifier"),
    ),
    responses(
        (status = 200, description = "The DAF as a PDF, generated on demand: the sheet then its attachments", content_type = "application/pdf"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DAF not found"),
        (status = 409, description = "A file the DAF references is no longer in storage"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DafPdfPath {
        organization_id,
        daf_id,
    }: DafPdfPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response, ApiError> {
    let daf = require_daf(&state, &identity, organization_id, daf_id).await?;
    let document = state.usecase.daf_document(organization_id, daf_id).await?;
    let files = load_files(&state, &document).await?;

    let filename = single_filename(daf.number.get(), daf.designation.as_str());
    let bytes = render_off_the_executor(document, files).await?;

    Ok(pdf_response(&filename, bytes))
}

pub(crate) fn pdf_response(filename: &str, bytes: Vec<u8>) -> Response {
    (
        StatusCode::OK,
        [
            (CONTENT_TYPE, "application/pdf".to_owned()),
            (CONTENT_DISPOSITION, content_disposition(filename)),
        ],
        Body::from(bytes),
    )
        .into_response()
}

/// Bytes of stored files one export may load. Every file is held in memory
/// for the duration of the render.
const MAX_BUNDLE_BYTES: usize = 256 * 1024 * 1024;

pub(crate) async fn load_files(
    state: &AppState,
    document: &DafDocument,
) -> Result<FileBundle, ApiError> {
    let mut bundle = FileBundle::default();
    let mut total_bytes = 0usize;
    let decoration_keys: Vec<&str> = document.style.decoration_image_keys().collect();
    for key in document.file_keys() {
        let object = match state.file_storage.get(key).await {
            Ok(object) => object,
            Err(error) => match ApiError::from(error) {
                ApiError::NotFound if decoration_keys.contains(&key) => continue,
                ApiError::NotFound => return Err(missing_file(document, key)),
                other => return Err(other),
            },
        };
        total_bytes += object.bytes.len();
        if total_bytes > MAX_BUNDLE_BYTES {
            return Err(ApiError::Conflict(format!(
                "this DAF export is too large: its files exceed {} MB",
                MAX_BUNDLE_BYTES / (1024 * 1024)
            )));
        }
        bundle.insert(
            key.to_owned(),
            BundledFile {
                mime_type: object.mime_type,
                bytes: object.bytes,
            },
        );
    }
    Ok(bundle)
}

fn owner_of(document: &DafDocument, key: &str) -> Option<String> {
    document
        .sheets
        .iter()
        .find(|sheet| {
            sheet.photo_key.as_deref() == Some(key)
                || sheet
                    .attachments
                    .iter()
                    .any(|attachment| attachment.key == key)
        })
        .map(|sheet| format!("{} {}", sheet.label, sheet.designation))
}

fn missing_file(document: &DafDocument, key: &str) -> ApiError {
    match owner_of(document, key) {
        Some(owner) => {
            ApiError::Conflict(format!("file `{key}` of {owner} is no longer in storage"))
        }
        None => ApiError::Conflict(format!("file `{key}` is no longer in storage")),
    }
}

pub(crate) async fn render_off_the_executor(
    document: DafDocument,
    files: FileBundle,
) -> Result<Vec<u8>, ApiError> {
    off_the_executor(move || render_with(&TypstDafRenderer, &document, &files)).await
}

fn render_with<R: DafRenderer>(
    renderer: &R,
    document: &DafDocument,
    files: &FileBundle,
) -> Result<Vec<u8>, ApiError> {
    renderer
        .render(document, files)
        .map_err(|error| render_error(document, error))
}

fn render_error(document: &DafDocument, error: DoeRenderError) -> ApiError {
    match error {
        DoeRenderError::MissingFile { key } => missing_file(document, &key),
        DoeRenderError::UnsupportedFile { key, .. } => {
            let place = owner_of(document, &key)
                .map(|owner| format!(" of {owner}"))
                .unwrap_or_default();
            ApiError::Conflict(format!(
                "file `{key}`{place} is not an image or a PDF and cannot be placed in a DAF"
            ))
        }
        DoeRenderError::TooLarge(reason) => {
            ApiError::Conflict(format!("this DAF export is too large: {reason}"))
        }
        other => {
            tracing::error!(error = %other, "DAF rendering failed");
            ApiError::Internal
        }
    }
}

fn clean(text: &str) -> String {
    text.chars()
        .map(|c| {
            if c.is_control() || matches!(c, '"' | '\\' | '/' | ':' | '*' | '?' | '<' | '>' | '|') {
                ' '
            } else {
                c
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn single_filename(number: u16, designation: &str) -> String {
    format!("DAF-{number:02}-{}.pdf", clean(designation))
}

pub(crate) fn dossier_filename(title: &str) -> String {
    format!("DAF-{}.pdf", clean(title))
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use mestier_core::{DafAttachment, DafSheet, DocumentStyle, DoeStyle};

    use super::*;

    fn document_with_attachment(key: &str) -> DafDocument {
        DafDocument {
            title: "Parc".to_owned(),
            issuer: "Treyve".to_owned(),
            summary: None,
            sheets: vec![DafSheet {
                label: "D.A.F n°09".to_owned(),
                designation: "Ganivelle".to_owned(),
                price_ref: None,
                supplier: None,
                installer: None,
                specs: None,
                observations: None,
                photo_key: None,
                attachments: vec![DafAttachment {
                    key: key.to_owned(),
                    name: "Fiche technique".to_owned(),
                }],
                project_owner: None,
                project_manager: None,
                issued_on: NaiveDate::from_ymd_opt(2026, 11, 18).unwrap(),
            }],
            style: DocumentStyle::from(&DoeStyle::default()),
        }
    }

    struct Failing(DoeRenderError);

    impl DafRenderer for Failing {
        fn render(&self, _: &DafDocument, _: &FileBundle) -> Result<Vec<u8>, DoeRenderError> {
            Err(self.0.clone())
        }
    }

    #[test]
    fn a_missing_file_is_a_conflict_naming_the_daf() {
        let document = document_with_attachment("sheets/a.pdf");
        let renderer = Failing(DoeRenderError::MissingFile {
            key: "sheets/a.pdf".to_owned(),
        });

        let error = render_with(&renderer, &document, &FileBundle::default()).unwrap_err();

        let ApiError::Conflict(message) = error else {
            panic!("expected a conflict");
        };
        assert!(message.contains("sheets/a.pdf"), "{message}");
        assert!(message.contains("D.A.F n°09 Ganivelle"), "{message}");
    }

    #[test]
    fn a_typesetting_failure_is_internal() {
        let document = document_with_attachment("k");
        let renderer = Failing(DoeRenderError::Typeset("boom".to_owned()));

        let error = render_with(&renderer, &document, &FileBundle::default()).unwrap_err();

        assert!(matches!(error, ApiError::Internal));
    }

    #[test]
    fn filenames_are_sanitized() {
        assert_eq!(single_filename(9, "Ganivelle"), "DAF-09-Ganivelle.pdf");
        assert_eq!(
            single_filename(120, "Jeux \"x\" 8/9"),
            "DAF-120-Jeux x 8 9.pdf"
        );
        assert_eq!(dossier_filename("Parc Montgroux"), "DAF-Parc Montgroux.pdf");
    }
}
