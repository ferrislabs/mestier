use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{
    CreateCustomerCommand, CustomerContactId, CustomerContextId, CustomerKind, CustomerProfile,
    CustomerStatus, LeadSource, MemberId, RegistrationNumber, payment_terms_of,
};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    customer_error::CustomerFailure, paths::CustomersPath, require_org_membership,
    response::CustomerResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCustomerRequest {
    pub status: CustomerStatus,
    pub name: String,
    #[serde(default)]
    pub registration_number: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub kind: CustomerKind,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub vat_number: Option<String>,
    #[serde(default)]
    pub owner_member_id: Option<MemberId>,
    #[serde(default)]
    pub lead_source: Option<LeadSource>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub payment_terms_days: Option<i64>,
    #[serde(default)]
    pub billing_contact_id: Option<CustomerContactId>,
    #[serde(default)]
    pub billing_address_id: Option<CustomerContextId>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/customers",
    operation_id = "createCustomer",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateCustomerRequest,
    responses(
        (status = 201, description = "Customer created", body = inline(DataEnvelope<CustomerResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Customer conflict"),
        (status = 422, description = "A field is refused, the field is named in details"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CustomersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateCustomerRequest>,
) -> Result<Response<CustomerResponse>, CustomerFailure> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;
    let registration_number = payload
        .registration_number
        .map(RegistrationNumber::try_from)
        .transpose()?;
    let payment_terms = payment_terms_of(payload.payment_terms_days)?;

    let customer = state
        .usecase
        .acting_as(user_id)
        .create_customer(CreateCustomerCommand {
            actor,
            organization_id: path.organization_id,
            status: payload.status,
            name: payload.name,
            registration_number,
            phone: payload.phone,
            email: payload.email,
            profile: CustomerProfile {
                kind: payload.kind,
                website: payload.website,
                vat_number: payload.vat_number,
                owner_member_id: payload.owner_member_id,
                lead_source: payload.lead_source,
                notes: payload.notes,
                payment_terms,
                billing_contact_id: payload.billing_contact_id,
                billing_address_id: payload.billing_address_id,
            },
        })
        .await??;

    Ok(Response::Created(customer.into()))
}
