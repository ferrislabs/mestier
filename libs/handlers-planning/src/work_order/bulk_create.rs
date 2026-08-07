use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{
    BulkCreateWorkOrderItem, BulkCreateWorkOrdersCommand, CustomerContextId, CustomerId,
    EquipmentId, QuoteId,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    require_org_membership,
    response::{EmployeeResponse, WorkOrderResponse},
    work_order::{BulkWorkOrdersPath, require_work_order_targets, update::AssigneeRefRequest},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkCreateWorkOrderItemRequest {
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    #[serde(default)]
    pub quote_id: Option<QuoteId>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

/// Shared schedule + assignees + equipment applied to every customer/context
/// item — one transaction creates N work orders.
#[derive(Debug, Deserialize, ToSchema)]
pub struct BulkCreateWorkOrdersRequest {
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub assignees: Vec<AssigneeRefRequest>,
    #[serde(default)]
    pub equipment_ids: Vec<EquipmentId>,
    pub items: Vec<BulkCreateWorkOrderItemRequest>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct BulkCreateWorkOrdersResponse {
    pub work_orders: Vec<WorkOrderResponse>,
    pub created_employees: Vec<EmployeeResponse>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/work-orders/bulk",
    operation_id = "bulkCreateWorkOrders",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = BulkCreateWorkOrdersRequest,
    responses(
        (status = 201, description = "Work orders created", body = inline(DataEnvelope<BulkCreateWorkOrdersResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer, context, equipment, employee or member not found"),
        (status = 409, description = "Work order conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: BulkWorkOrdersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<BulkCreateWorkOrdersRequest>,
) -> Result<Response<BulkCreateWorkOrdersResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    for item in &payload.items {
        require_work_order_targets(
            &state,
            path.organization_id,
            item.customer_id,
            item.customer_context_id,
            item.quote_id,
        )
        .await?;
    }

    let (work_orders, created_employees) = state
        .usecase
        .bulk_create_work_orders(BulkCreateWorkOrdersCommand {
            organization_id: path.organization_id,
            starts_at: payload.starts_at,
            ends_at: payload.ends_at,
            all_day: payload.all_day,
            assignees: payload
                .assignees
                .into_iter()
                .map(Into::into)
                .collect(),
            equipment_ids: payload.equipment_ids,
            items: payload
                .items
                .into_iter()
                .map(|item| BulkCreateWorkOrderItem {
                    customer_id: item.customer_id,
                    customer_context_id: item.customer_context_id,
                    quote_id: item.quote_id,
                    title: item.title,
                    note: item.note,
                })
                .collect(),
        })
        .await?;

    Ok(Response::Created(BulkCreateWorkOrdersResponse {
        work_orders: work_orders.into_iter().map(Into::into).collect(),
        created_employees: created_employees.into_iter().map(Into::into).collect(),
    }))
}
