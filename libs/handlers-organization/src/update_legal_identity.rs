use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{UpdateLegalIdentityCommand, VatStatus, application::policy};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::OrganizationLegalIdentityPath, resolve_identity_user, response::OrganizationResponse,
};

/// Mirrors `VatStatusResponse` on the way in: a client can send a subject
/// status or a not-subject status, never a blank that means either (#311).
#[derive(Debug, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum VatStatusRequest {
    Subject { vat_number: String },
    NotSubject { basis: String },
}

impl From<VatStatusRequest> for VatStatus {
    fn from(value: VatStatusRequest) -> Self {
        match value {
            VatStatusRequest::Subject { vat_number } => Self::Subject { vat_number },
            VatStatusRequest::NotSubject { basis } => Self::NotSubject { basis },
        }
    }
}

/// Replaces the whole legal-identity block. A field left out of the
/// payload (or sent as `null`) is cleared, not skipped: the settings
/// section (#311) is a single form and always resends every field it owns,
/// so there is no "leave unchanged" case to represent here.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateLegalIdentityRequest {
    #[serde(default)]
    pub legal_name: Option<String>,
    #[serde(default)]
    pub legal_form: Option<String>,
    #[serde(default)]
    pub registration_number: Option<String>,
    #[serde(default)]
    pub vat_status: Option<VatStatusRequest>,
    #[serde(default)]
    pub share_capital_cents: Option<i64>,
    #[serde(default)]
    pub address_line1: Option<String>,
    #[serde(default)]
    pub address_line2: Option<String>,
    #[serde(default)]
    pub address_postal_code: Option<String>,
    #[serde(default)]
    pub address_city: Option<String>,
    #[serde(default)]
    pub address_country: Option<String>,
    #[serde(default)]
    pub contact_email: Option<String>,
    #[serde(default)]
    pub contact_phone: Option<String>,
    #[serde(default)]
    pub insurance_mention: Option<String>,
    /// Not optional, unlike the fields above: a VAT regime choice is
    /// always either debits or encaissements. Defaults to `false`
    /// (encaissements) so existing clients that do not yet send this field
    /// keep today's behaviour rather than failing to deserialize.
    #[serde(default)]
    pub vat_on_debits: bool,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/legal-identity",
    operation_id = "updateOrganizationLegalIdentity",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = UpdateLegalIdentityRequest,
    responses(
        (status = 200, description = "Legal identity updated", body = inline(DataEnvelope<OrganizationResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn handler(
    OrganizationLegalIdentityPath { organization_id }: OrganizationLegalIdentityPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateLegalIdentityRequest>,
) -> Result<Response<OrganizationResponse>, ApiError> {
    let user = resolve_identity_user(&state, &identity).await?;
    let iam_roles = match &identity {
        Identity::User(u) => u.roles.clone(),
        Identity::Client(c) => c.roles.clone(),
    };
    let actor = policy::user_subject(user.id, iam_roles);

    let organization = state
        .usecase
        .acting_as(user.id)
        .update_legal_identity(UpdateLegalIdentityCommand {
            actor,
            id: organization_id,
            legal_name: payload.legal_name,
            legal_form: payload.legal_form,
            registration_number: payload.registration_number,
            vat_status: payload.vat_status.map(Into::into),
            share_capital_cents: payload.share_capital_cents,
            address_line1: payload.address_line1,
            address_line2: payload.address_line2,
            address_postal_code: payload.address_postal_code,
            address_city: payload.address_city,
            address_country: payload.address_country,
            contact_email: payload.contact_email,
            contact_phone: payload.contact_phone,
            insurance_mention: payload.insurance_mention,
            vat_on_debits: payload.vat_on_debits,
        })
        .await?;

    Ok(Response::OK(organization.into()))
}
