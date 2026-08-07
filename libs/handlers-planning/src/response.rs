use chrono::{DateTime, Utc};
use mestier_core::{
    CustomerContextId, CustomerId, Employee, EmployeeId, EquipmentId, OrganizationId, QuoteId,
    UserId, WorkOrder, WorkOrderId, WorkOrderStatus,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkOrderResponse {
    pub id: WorkOrderId,
    pub organization_id: OrganizationId,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub quote_id: Option<QuoteId>,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub all_day: bool,
    pub status: WorkOrderStatus,
    pub title: Option<String>,
    pub note: Option<String>,
    /// The complete set of currently assigned employees — mirrors the
    /// `PATCH` contract, where `assignees` is always the full list.
    pub employee_ids: Vec<EmployeeId>,
    /// The complete set of currently assigned equipment — mirrors the
    /// `PATCH` contract, where `equipment` is always the full list.
    pub equipment_ids: Vec<EquipmentId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WorkOrder> for WorkOrderResponse {
    fn from(value: WorkOrder) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
            quote_id: value.quote_id,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            all_day: value.all_day,
            status: value.status,
            title: value.title,
            note: value.note,
            employee_ids: value
                .assignments
                .into_iter()
                .map(|assignment| assignment.employee_id)
                .collect(),
            equipment_ids: value
                .equipment
                .into_iter()
                .map(|link| link.equipment_id)
                .collect(),
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// A minimal employee projection for `created_employees`.
///
/// Duplicated from `handlers-reference::response::EmployeeResponse` rather
/// than shared: `handlers-planning` has no dependency on `handlers-reference`
/// (and adding one is a `Cargo.toml` change outside this workstream's
/// files), so each HTTP adapter crate owns its own response DTOs. The shape
/// tracks `Employee` field for field.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct EmployeeResponse {
    pub id: EmployeeId,
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub name: String,
    /// `null` means the rate is not set yet; `0` means genuinely free.
    pub hourly_rate_cents: Option<i32>,
    pub weekly_contract_minutes: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Employee> for EmployeeResponse {
    fn from(value: Employee) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            user_id: value.user_id,
            name: value.name,
            hourly_rate_cents: value.hourly_rate_cents,
            weekly_contract_minutes: value.weekly_contract_minutes,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// Response body for the transactional `PATCH`: the work order as it stands
/// after reschedule/reassignment, plus every employee record that had to be
/// provisioned on the fly for a `member` assignee.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PatchWorkOrderResponse {
    pub work_order: WorkOrderResponse,
    pub created_employees: Vec<EmployeeResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mestier_core::Assignment;

    // `uuid` is not a direct dependency of `handlers-planning` (a
    // `Cargo.toml` this workstream does not own), so fixture ids are parsed
    // from literal strings via `FromStr` rather than generated.
    fn work_order() -> WorkOrder {
        let now = Utc::now();
        let id: WorkOrderId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let organization_id: OrganizationId =
            "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let employee_id: EmployeeId = "33333333-3333-3333-3333-333333333333".parse().unwrap();
        WorkOrder {
            id,
            organization_id,
            customer_id: "44444444-4444-4444-4444-444444444444".parse().unwrap(),
            customer_context_id: "55555555-5555-5555-5555-555555555555".parse().unwrap(),
            quote_id: None,
            starts_at: now,
            ends_at: now + chrono::Duration::hours(2),
            all_day: false,
            status: WorkOrderStatus::Planned,
            title: Some("Toiture".to_owned()),
            note: None,
            assignments: vec![Assignment {
                id: "66666666-6666-6666-6666-666666666666".parse().unwrap(),
                organization_id,
                work_order_id: id,
                employee_id,
                created_at: now,
            }],
            equipment: Vec::new(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn work_order_response_flattens_assignments_to_employee_ids() {
        let source = work_order();
        let expected_employee_id = source.assignments[0].employee_id;

        let response: WorkOrderResponse = source.into();

        assert_eq!(response.employee_ids, vec![expected_employee_id]);
    }
}
