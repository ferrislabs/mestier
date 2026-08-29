use auth::Identity;
use axum::{Extension, body::Bytes, extract::State, http::HeaderMap};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use http::header::CONTENT_TYPE;
use mestier_core::{FacturXParser, ImportSupplierInvoiceOutcome, UploadFileCommand};

use crate::{
    paths::OrganizationSupplierInvoicesImportPath,
    require_org_membership,
    response::{ImportSupplierInvoiceResponse, TotalsMismatchResponse},
};

/// Imports a Factur-X file as a `Received` supplier invoice (#337), storing
/// the original alongside it (#339: "the file is stored, not only parsed" —
/// unlike `handlers-files`' own upload endpoint, this one both uploads and
/// records in a single call, since a caller here never has a reason to keep
/// the bytes around for a second, separate request the way an already-known
/// domain object attaching a photo does).
///
/// A file that fails to parse is not a 4xx: #337's binding rule keeps it,
/// with the reason, so the response body's own `outcome` tag is what tells
/// the two cases apart, not the status code — see
/// [`ImportSupplierInvoiceResponse`].
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/supplier-invoices/import",
    operation_id = "importSupplierInvoice",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body(content = Vec<u8>, content_type = "application/octet-stream"),
    responses(
        (status = 200, description = "File stored but could not be parsed — see the response's reason", body = inline(DataEnvelope<ImportSupplierInvoiceResponse>)),
        (status = 201, description = "Supplier invoice created", body = inline(DataEnvelope<ImportSupplierInvoiceResponse>)),
        (status = 400, description = "Empty body"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "This supplier invoice was already imported"),
        (status = 413, description = "Payload too large"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrganizationSupplierInvoicesImportPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<ImportSupplierInvoiceResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    if body.is_empty() {
        return Err(ApiError::Validation("file body cannot be empty".to_owned()));
    }

    if body.len() as u64 > state.args.file_storage.max_upload_bytes {
        return Err(ApiError::UnprocessableEntity(format!(
            "file exceeds max upload size of {} bytes",
            state.args.file_storage.max_upload_bytes
        )));
    }

    let mime_type = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_owned();

    let stored = state
        .file_storage
        .upload(UploadFileCommand {
            mime_type: mime_type.clone(),
            bytes: body.to_vec(),
            folder: Some("supplier-invoices".to_owned()),
        })
        .await?;

    let outcome = state
        .usecase
        .import_supplier_invoice(
            path.organization_id,
            body.to_vec(),
            &FacturXParser,
            stored.key,
            mime_type,
        )
        .await?;

    match outcome {
        ImportSupplierInvoiceOutcome::Created {
            invoice,
            totals_mismatch,
        } => Ok(Response::Created(ImportSupplierInvoiceResponse::Created {
            invoice: Box::new((*invoice).into()),
            totals_mismatch: totals_mismatch.map(TotalsMismatchResponse::from),
        })),
        ImportSupplierInvoiceOutcome::ParseFailed { reason } => {
            Ok(Response::OK(ImportSupplierInvoiceResponse::ParseFailed {
                reason: reason.to_string(),
            }))
        }
    }
}
