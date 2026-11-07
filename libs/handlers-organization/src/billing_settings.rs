use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{
    BillingSettings, BillingSettingsInput, ReplaceBillingSettingsCommand, application::policy,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    billing_error::BillingError, paths::OrganizationBillingSettingsPath, require_org_membership,
    resolve_identity_user,
};

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct BillingSettingsResponse {
    pub payment_terms_days: i32,
    pub quote_validity_days: i32,
    pub default_vat_rate_bp: i32,
    pub late_penalty: String,
    pub recovery_fee_cents: i32,
    pub invoice_footer: String,
    pub number_padding: i32,
    pub iban: Option<String>,
    pub bic: Option<String>,
}

impl From<BillingSettings> for BillingSettingsResponse {
    fn from(settings: BillingSettings) -> Self {
        Self {
            payment_terms_days: settings.payment_terms_days.get(),
            quote_validity_days: settings.quote_validity_days.get(),
            default_vat_rate_bp: settings.default_vat_rate.basis_points(),
            late_penalty: settings.late_penalty.as_str().to_owned(),
            recovery_fee_cents: settings.recovery_fee.get(),
            invoice_footer: settings.invoice_footer,
            number_padding: settings.number_padding.as_i32(),
            iban: settings.iban,
            bic: settings.bic,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct ReplaceBillingSettingsRequest {
    pub payment_terms_days: i64,
    pub quote_validity_days: i64,
    pub default_vat_rate_bp: i64,
    pub late_penalty: String,
    pub recovery_fee_cents: i64,
    pub invoice_footer: String,
    pub number_padding: i64,
    #[serde(default)]
    pub iban: Option<String>,
    #[serde(default)]
    pub bic: Option<String>,
}

impl From<ReplaceBillingSettingsRequest> for BillingSettingsInput {
    fn from(request: ReplaceBillingSettingsRequest) -> Self {
        Self {
            payment_terms_days: request.payment_terms_days,
            quote_validity_days: request.quote_validity_days,
            default_vat_rate_bp: request.default_vat_rate_bp,
            late_penalty: request.late_penalty,
            recovery_fee_cents: request.recovery_fee_cents,
            invoice_footer: request.invoice_footer,
            number_padding: request.number_padding,
            iban: request.iban,
            bic: request.bic,
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/billing-settings",
    operation_id = "getOrganizationBillingSettings",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Billing settings, the defaults when none were saved", body = inline(DataEnvelope<BillingSettingsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn get_handler(
    OrganizationBillingSettingsPath { organization_id }: OrganizationBillingSettingsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<BillingSettingsResponse>, BillingError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let settings = state.usecase.get_billing_settings(organization_id).await?;

    Ok(Response::OK(settings.into()))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/billing-settings",
    operation_id = "replaceOrganizationBillingSettings",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = ReplaceBillingSettingsRequest,
    responses(
        (status = 200, description = "Billing settings replaced", body = inline(DataEnvelope<BillingSettingsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden, requires MANAGE_ORG"),
        (status = 404, description = "Organization not found"),
        (status = 422, description = "A setting is out of its bounds, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn put_handler(
    OrganizationBillingSettingsPath { organization_id }: OrganizationBillingSettingsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ReplaceBillingSettingsRequest>,
) -> Result<Response<BillingSettingsResponse>, BillingError> {
    let user = resolve_identity_user(&state, &identity).await?;
    let iam_roles = match &identity {
        Identity::User(u) => u.roles.clone(),
        Identity::Client(c) => c.roles.clone(),
    };
    let actor = policy::user_subject(user.id, iam_roles);
    let settings = BillingSettings::try_from(BillingSettingsInput::from(payload))?;

    let stored = state
        .usecase
        .acting_as(user.id)
        .replace_billing_settings(ReplaceBillingSettingsCommand {
            actor,
            id: organization_id,
            settings,
        })
        .await?;

    Ok(Response::OK(stored.into()))
}
