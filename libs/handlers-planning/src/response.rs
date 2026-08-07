use chrono::{DateTime, Utc};
use mestier_core::{
    CustomerContextId, CustomerId, Employee, EmployeeId, OrganizationId, QuoteId, Task, TaskId,
    TaskStatus, UserId,
};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TaskResponse {
    pub id: TaskId,
    pub organization_id: OrganizationId,
    pub parent_task_id: Option<TaskId>,
    pub title: String,
    pub description: Option<String>,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
    pub all_day: bool,
    pub status: TaskStatus,
    pub blocks_availability: bool,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub quote_id: Option<QuoteId>,
    /// The complete set of currently assigned employees — mirrors the
    /// `PATCH` contract, where `assignees` is always the full list.
    pub employee_ids: Vec<EmployeeId>,
    /// The number of direct children — only `GET /tasks` computes this (one
    /// grouped query per page, see `TaskRepository::count_children`); every
    /// other endpoint leaves it `None` rather than pay for an extra query
    /// or report a stale/wrong `0`.
    pub child_count: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Task> for TaskResponse {
    fn from(value: Task) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            parent_task_id: value.parent_task_id,
            title: value.title,
            description: value.description,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            all_day: value.all_day,
            status: value.status,
            blocks_availability: value.blocks_availability,
            customer_id: value.customer_id,
            customer_context_id: value.customer_context_id,
            quote_id: value.quote_id,
            employee_ids: value
                .assignments
                .into_iter()
                .map(|assignment| assignment.employee_id)
                .collect(),
            child_count: None,
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
    pub last_name: String,
    /// `null` means "not provided" — see `Employee::first_name`.
    pub first_name: Option<String>,
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
            last_name: value.last_name,
            first_name: value.first_name,
            hourly_rate_cents: value.hourly_rate_cents,
            weekly_contract_minutes: value.weekly_contract_minutes,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

/// Response body for the transactional `PATCH`: the task as it stands
/// after reparenting/reschedule/reassignment, plus every employee record
/// that had to be provisioned on the fly for a `member` assignee.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PatchTaskResponse {
    pub task: TaskResponse,
    pub created_employees: Vec<EmployeeResponse>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use mestier_core::TaskAssignment;

    // `uuid` is not a direct dependency of `handlers-planning` (a
    // `Cargo.toml` this workstream does not own), so fixture ids are parsed
    // from literal strings via `FromStr` rather than generated.
    fn task() -> Task {
        let now = Utc::now();
        let id: TaskId = "11111111-1111-1111-1111-111111111111".parse().unwrap();
        let organization_id: OrganizationId =
            "22222222-2222-2222-2222-222222222222".parse().unwrap();
        let employee_id: EmployeeId = "33333333-3333-3333-3333-333333333333".parse().unwrap();
        Task {
            id,
            organization_id,
            parent_task_id: None,
            title: "Toiture".to_owned(),
            description: None,
            starts_at: Some(now),
            ends_at: Some(now + chrono::Duration::hours(2)),
            all_day: false,
            status: TaskStatus::Planned,
            blocks_availability: true,
            customer_id: Some("44444444-4444-4444-4444-444444444444".parse().unwrap()),
            customer_context_id: Some("55555555-5555-5555-5555-555555555555".parse().unwrap()),
            quote_id: None,
            assignments: vec![TaskAssignment {
                id: "66666666-6666-6666-6666-666666666666".parse().unwrap(),
                organization_id,
                task_id: id,
                employee_id,
                created_at: now,
            }],
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn task_response_flattens_assignments_to_employee_ids() {
        let source = task();
        let expected_employee_id = source.assignments[0].employee_id;

        let response: TaskResponse = source.into();

        assert_eq!(response.employee_ids, vec![expected_employee_id]);
    }

    #[test]
    fn task_response_leaves_child_count_unpopulated_by_default() {
        let response: TaskResponse = task().into();

        assert_eq!(
            response.child_count, None,
            "only the GET /tasks list handler populates child_count"
        );
    }
}
