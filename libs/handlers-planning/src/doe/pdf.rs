use auth::Identity;
use axum::{
    Extension,
    body::Body,
    extract::State,
    response::{IntoResponse, Response},
};
use std::{
    sync::{Arc, LazyLock},
    time::Duration,
};

use doc_render::TypstDoeRenderer;
use handlers::{ApiError, AppState};
use http::{
    StatusCode,
    header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use mestier_core::{
    BundledFile, DoeDocument, DoeRenderError, DoeRenderer, FileBundle, NumberedContent,
};
use tokio::sync::Semaphore;

use crate::doe::{DoePdfPath, require_doe};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/does/{doe_id}/pdf",
    operation_id = "exportDoePdf",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("doe_id" = mestier_core::DoeId, Path, description = "DOE identifier"),
    ),
    responses(
        (status = 200, description = "The DOE as a PDF, generated on demand", content_type = "application/pdf"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DOE not found"),
        (status = 409, description = "A file the DOE references is no longer in storage"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DoePdfPath {
        organization_id,
        doe_id,
    }: DoePdfPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response, ApiError> {
    let doe = require_doe(&state, &identity, organization_id, doe_id).await?;
    let document = state.usecase.doe_document(doe_id).await?;
    let files = load_files(&state, &document).await?;

    let filename = pdf_filename(&doe.title, doe.revision.as_str());
    let bytes = render_off_the_executor(document, files).await?;

    Ok((
        StatusCode::OK,
        [
            (CONTENT_TYPE, "application/pdf".to_owned()),
            (CONTENT_DISPOSITION, content_disposition(&filename)),
        ],
        Body::from(bytes),
    )
        .into_response())
}

/// Bytes of stored files one export may load. Every file is held in memory
/// for the duration of the render.
const MAX_BUNDLE_BYTES: usize = 256 * 1024 * 1024;

async fn load_files(state: &AppState, document: &DoeDocument) -> Result<FileBundle, ApiError> {
    let mut bundle = FileBundle::default();
    let mut total_bytes = 0usize;
    for key in document.file_keys() {
        let object =
            state
                .file_storage
                .get(key)
                .await
                .map_err(|error| match ApiError::from(error) {
                    ApiError::NotFound => missing_file(document, key),
                    other => other,
                })?;
        total_bytes += object.bytes.len();
        if total_bytes > MAX_BUNDLE_BYTES {
            return Err(ApiError::Conflict(format!(
                "this DOE is too large to export: its files exceed {} MB",
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

fn missing_file(document: &DoeDocument, key: &str) -> ApiError {
    match owner_of(document, key) {
        Some(owner) => {
            ApiError::Conflict(format!("file `{key}` of {owner} is no longer in storage"))
        }
        None => ApiError::Conflict(format!("file `{key}` is no longer in storage")),
    }
}

fn owner_of(document: &DoeDocument, key: &str) -> Option<String> {
    document
        .sections
        .iter()
        .find_map(|section| match &section.content {
            NumberedContent::Supplies { supplies, .. } => supplies.iter().find_map(|numbered| {
                let referenced = numbered.supply.photos.iter().any(|photo| photo.key == key)
                    || numbered.supply.datasheet_keys.iter().any(|k| k == key);
                referenced.then(|| format!("supply {} {}", numbered.number, numbered.heading))
            }),
            NumberedContent::Photos { photos, .. } => photos
                .iter()
                .any(|photo| photo.key == key)
                .then(|| format!("section {} {}", section.number, section.title)),
            NumberedContent::Plans { plans } => plans
                .iter()
                .any(|plan| plan.key == key)
                .then(|| format!("section {} {}", section.number, section.title)),
            NumberedContent::BlankPages { .. } => None,
        })
}

/// Exports rendering at the same time, across the whole process. Rendering
/// is CPU-bound and holds every file of a DOE in memory, so an unbounded
/// number of parallel exports is how one organization slows down all others.
const CONCURRENT_RENDERS: usize = 2;

/// Past this the caller gets an error. The blocking task cannot be cancelled
/// and finishes on its own, still holding its permit, so the limit above
/// keeps counting it.
const RENDER_TIMEOUT: Duration = Duration::from_secs(120);

static RENDER_PERMITS: LazyLock<Arc<Semaphore>> =
    LazyLock::new(|| Arc::new(Semaphore::new(CONCURRENT_RENDERS)));

async fn render_off_the_executor(
    document: DoeDocument,
    files: FileBundle,
) -> Result<Vec<u8>, ApiError> {
    let permit = RENDER_PERMITS
        .clone()
        .acquire_owned()
        .await
        .map_err(|_| ApiError::Internal)?;
    let task = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        render_with(&TypstDoeRenderer, &document, &files)
    });
    match tokio::time::timeout(RENDER_TIMEOUT, task).await {
        Ok(joined) => joined.map_err(|error| {
            tracing::error!(error = %error, "DOE rendering task failed");
            ApiError::Internal
        })?,
        Err(_) => {
            tracing::warn!("DOE rendering exceeded {RENDER_TIMEOUT:?}");
            Err(ApiError::Internal)
        }
    }
}

fn render_with<R: DoeRenderer>(
    renderer: &R,
    document: &DoeDocument,
    files: &FileBundle,
) -> Result<Vec<u8>, ApiError> {
    renderer
        .render(document, files)
        .map_err(|error| match error {
            DoeRenderError::MissingFile { key } => missing_file(document, &key),
            DoeRenderError::UnsupportedFile { key, .. } => {
                let place = owner_of(document, &key)
                    .map(|owner| format!(" of {owner}"))
                    .unwrap_or_default();
                ApiError::Conflict(format!(
                    "file `{key}`{place} is not an image or a PDF and cannot be placed in a DOE"
                ))
            }
            DoeRenderError::TooLarge(reason) => {
                ApiError::Conflict(format!("this DOE is too large to export: {reason}"))
            }
            other => {
                tracing::error!(error = %other, "DOE rendering failed");
                ApiError::Internal
            }
        })
}

fn pdf_filename(title: &str, revision: &str) -> String {
    let clean = |text: &str| {
        text.chars()
            .map(|c| {
                if c.is_control()
                    || matches!(c, '"' | '\\' | '/' | ':' | '*' | '?' | '<' | '>' | '|')
                {
                    ' '
                } else {
                    c
                }
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    format!("DOE - {} - {}.pdf", clean(title), clean(revision))
}

fn content_disposition(filename: &str) -> String {
    let ascii: String = filename
        .chars()
        .map(|c| if c.is_ascii() { c } else { '_' })
        .collect();
    let mut encoded = String::new();
    for byte in filename.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    format!("attachment; filename=\"{ascii}\"; filename*=UTF-8''{encoded}")
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use mestier_core::{DoeFile, DoeSupply, HexColor, NumberedSection, NumberedSupply};
    use uuid::Uuid;

    use super::*;

    fn document_with_supply_photo(key: &str) -> DoeDocument {
        DoeDocument {
            title: "Avenue".to_owned(),
            place: None,
            lot: None,
            issued_on: NaiveDate::from_ymd_opt(2026, 10, 7).unwrap(),
            author: None,
            revision: "A".to_owned(),
            parties: Vec::new(),
            sections: vec![NumberedSection {
                number: 2,
                title: "Mobilier".to_owned(),
                content: NumberedContent::Supplies {
                    maker_label: "Fabricant".to_owned(),
                    supplies: vec![NumberedSupply {
                        number: "2.1".to_owned(),
                        heading: "Banc Garden".to_owned(),
                        supply: DoeSupply {
                            id: Uuid::new_v4(),
                            source_product_id: None,
                            title: None,
                            designation: "Banc Garden".to_owned(),
                            dpgf_ref: None,
                            destination: None,
                            supplier: None,
                            maker: None,
                            specs: None,
                            photos: vec![DoeFile {
                                key: key.to_owned(),
                                caption: None,
                            }],
                            datasheet_keys: Vec::new(),
                        },
                    }],
                },
            }],
            accent: HexColor::mestier_default(),
            logo_key: None,
        }
    }

    struct Failing(DoeRenderError);

    impl DoeRenderer for Failing {
        fn render(&self, _: &DoeDocument, _: &FileBundle) -> Result<Vec<u8>, DoeRenderError> {
            Err(self.0.clone())
        }
    }

    #[test]
    fn a_missing_file_is_a_conflict_naming_the_supply() {
        let document = document_with_supply_photo("photos/banc.jpg");
        let renderer = Failing(DoeRenderError::MissingFile {
            key: "photos/banc.jpg".to_owned(),
        });

        let error = render_with(&renderer, &document, &FileBundle::default()).unwrap_err();

        let ApiError::Conflict(message) = error else {
            panic!("expected a conflict");
        };
        assert!(message.contains("photos/banc.jpg"), "{message}");
        assert!(message.contains("2.1 Banc Garden"), "{message}");
    }

    #[test]
    fn an_unsupported_file_is_a_conflict_without_its_type() {
        let document = document_with_supply_photo("photos/banc.tiff");
        let renderer = Failing(DoeRenderError::UnsupportedFile {
            key: "photos/banc.tiff".to_owned(),
            mime_type: "image/tiff".to_owned(),
        });

        let error = render_with(&renderer, &document, &FileBundle::default()).unwrap_err();

        let ApiError::Conflict(message) = error else {
            panic!("expected a conflict");
        };
        assert!(message.contains("2.1 Banc Garden"), "{message}");
        assert!(!message.contains("image/tiff"), "{message}");
    }

    #[test]
    fn a_too_large_document_is_a_conflict() {
        let document = document_with_supply_photo("k");
        let renderer = Failing(DoeRenderError::TooLarge("pages".to_owned()));

        let error = render_with(&renderer, &document, &FileBundle::default()).unwrap_err();

        assert!(matches!(error, ApiError::Conflict(_)));
    }

    #[test]
    fn a_typesetting_failure_is_internal() {
        let document = document_with_supply_photo("k");
        let renderer = Failing(DoeRenderError::Typeset("boom".to_owned()));

        let error = render_with(&renderer, &document, &FileBundle::default()).unwrap_err();

        assert!(matches!(error, ApiError::Internal));
    }

    #[test]
    fn the_filename_is_sanitized() {
        assert_eq!(
            pdf_filename("Avenue \"du\" 8/mai", "A"),
            "DOE - Avenue du 8 mai - A.pdf"
        );
    }

    #[test]
    fn content_disposition_is_a_valid_ascii_header() {
        let value = content_disposition("DOE - Écoquartier - B.pdf");

        assert!(http::HeaderValue::from_str(&value).is_ok());
        assert!(value.contains("filename=\"DOE - _coquartier - B.pdf\""));
        assert!(value.contains("filename*=UTF-8''DOE%20-%20%C3%89coquartier%20-%20B.pdf"));
    }
}
