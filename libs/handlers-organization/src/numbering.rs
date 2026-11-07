use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{Datelike, Utc};
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{NumberSeries, SeriesNumbering, SetNextNumberCommand, application::policy};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    billing_error::BillingError,
    paths::{OrganizationNumberingPath, OrganizationNumberingSeriesPath},
    require_org_membership, resolve_identity_user,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum NumberSeriesKind {
    Quote,
    Invoice,
    CreditNote,
}

impl std::fmt::Display for NumberSeriesKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(NumberSeries::from(*self).as_str())
    }
}

impl From<NumberSeriesKind> for NumberSeries {
    fn from(kind: NumberSeriesKind) -> Self {
        match kind {
            NumberSeriesKind::Quote => Self::Quote,
            NumberSeriesKind::Invoice => Self::Invoice,
            NumberSeriesKind::CreditNote => Self::CreditNote,
        }
    }
}

impl From<NumberSeries> for NumberSeriesKind {
    fn from(series: NumberSeries) -> Self {
        match series {
            NumberSeries::Quote => Self::Quote,
            NumberSeries::Invoice => Self::Invoice,
            NumberSeries::CreditNote => Self::CreditNote,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct SeriesNumberingResponse {
    pub kind: NumberSeriesKind,
    pub prefix: String,
    pub year: i32,
    pub last_assigned_number: i32,
    pub next_number: i32,
    pub next_reference: String,
    pub padding: i32,
}

impl From<SeriesNumbering> for SeriesNumberingResponse {
    fn from(numbering: SeriesNumbering) -> Self {
        Self {
            kind: numbering.series.into(),
            next_number: numbering.next_number(),
            next_reference: numbering.next_reference(),
            prefix: numbering.prefix,
            year: numbering.year,
            last_assigned_number: numbering.last_assigned,
            padding: numbering.padding.as_i32(),
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetNextNumberRequest {
    pub next_number: i64,
}

fn current_year() -> i32 {
    Utc::now().year()
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/numbering",
    operation_id = "getOrganizationNumbering",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Prefix, next number and padding of each series for the current year", body = inline(DataEnvelope<Vec<SeriesNumberingResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Organization not found"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn get_handler(
    OrganizationNumberingPath { organization_id }: OrganizationNumberingPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<SeriesNumberingResponse>>, BillingError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let numbering = state
        .usecase
        .get_numbering(organization_id, current_year())
        .await?
        .into_iter()
        .map(SeriesNumberingResponse::from)
        .collect();

    Ok(Response::OK(numbering))
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/numbering/{kind}",
    operation_id = "setOrganizationNextNumber",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("kind" = NumberSeriesKind, Path, description = "Document series"),
    ),
    request_body = SetNextNumberRequest,
    responses(
        (status = 200, description = "Counter moved forward for the current year", body = inline(DataEnvelope<SeriesNumberingResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden, requires MANAGE_ORG"),
        (status = 404, description = "Organization not found"),
        (status = 422, description = "E_NUMBER_NOT_FORWARD: the number is at or below the last assigned one"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn put_handler(
    OrganizationNumberingSeriesPath {
        organization_id,
        kind,
    }: OrganizationNumberingSeriesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SetNextNumberRequest>,
) -> Result<Response<SeriesNumberingResponse>, BillingError> {
    let user = resolve_identity_user(&state, &identity).await?;
    let iam_roles = match &identity {
        Identity::User(u) => u.roles.clone(),
        Identity::Client(c) => c.roles.clone(),
    };
    let actor = policy::user_subject(user.id, iam_roles);

    let numbering = state
        .usecase
        .acting_as(user.id)
        .set_next_number(SetNextNumberCommand {
            actor,
            id: organization_id,
            series: kind.into(),
            year: current_year(),
            next_number: payload.next_number,
        })
        .await??;

    Ok(Response::OK(numbering.into()))
}
