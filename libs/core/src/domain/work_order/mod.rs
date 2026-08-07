use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{CustomerContextId, CustomerId, EmployeeId, OrganizationId, QuoteId, UserId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct WorkOrderId(pub Uuid);

impl FromStr for WorkOrderId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(WorkOrderId)
    }
}

impl Display for WorkOrderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct AssignmentId(pub Uuid);

impl FromStr for AssignmentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(AssignmentId)
    }
}

impl Display for AssignmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkOrderStatus {
    Planned,
    InProgress,
    Done,
    Cancelled,
}

impl WorkOrderStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "PLANNED",
            Self::InProgress => "IN_PROGRESS",
            Self::Done => "DONE",
            Self::Cancelled => "CANCELLED",
        }
    }
}

impl Display for WorkOrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for WorkOrderStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PLANNED" => Ok(Self::Planned),
            "IN_PROGRESS" => Ok(Self::InProgress),
            "DONE" => Ok(Self::Done),
            "CANCELLED" => Ok(Self::Cancelled),
            other => Err(format!("invalid work order status `{other}`")),
        }
    }
}

/// A reference to a plannable resource, as carried by the `PATCH` payload's
/// `assignees` list (see the planning module design doc).
///
/// `Employee` points at an existing employee record. `Member` points at an
/// organization member who may not have one yet — resolving it is what
/// triggers on-the-fly employee-record creation in
/// [`service::WorkOrderService::patch_work_order`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssigneeRef {
    Employee(EmployeeId),
    Member(UserId),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub id: AssignmentId,
    pub organization_id: OrganizationId,
    pub work_order_id: WorkOrderId,
    pub employee_id: EmployeeId,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkOrder {
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
    /// The complete set of employees currently assigned. Always loaded and
    /// persisted as a whole — see the `PATCH` contract: `assignees` is the
    /// full list, never a delta.
    pub assignments: Vec<Assignment>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn work_order_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = WorkOrderId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn work_order_id_rejects_invalid_uuid() {
        assert!(WorkOrderId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn assignment_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = AssignmentId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn assignment_id_rejects_invalid_uuid() {
        assert!(AssignmentId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn work_order_status_parses_known_values() {
        assert_eq!(
            "PLANNED".parse::<WorkOrderStatus>().unwrap(),
            WorkOrderStatus::Planned
        );
        assert_eq!(
            "IN_PROGRESS".parse::<WorkOrderStatus>().unwrap(),
            WorkOrderStatus::InProgress
        );
        assert_eq!(
            "DONE".parse::<WorkOrderStatus>().unwrap(),
            WorkOrderStatus::Done
        );
        assert_eq!(
            "CANCELLED".parse::<WorkOrderStatus>().unwrap(),
            WorkOrderStatus::Cancelled
        );
    }

    #[test]
    fn work_order_status_rejects_unknown_values() {
        assert!("SCHEDULED".parse::<WorkOrderStatus>().is_err());
    }

    #[test]
    fn work_order_status_round_trips_through_as_str() {
        for status in [
            WorkOrderStatus::Planned,
            WorkOrderStatus::InProgress,
            WorkOrderStatus::Done,
            WorkOrderStatus::Cancelled,
        ] {
            assert_eq!(status.as_str().parse::<WorkOrderStatus>().unwrap(), status);
        }
    }
}
