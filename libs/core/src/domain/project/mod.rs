//! The subject a cost is reported against.
//!
//! Before this module a "chantier" was not an entity: it was a root
//! [`crate::Task`] carrying a customer, and profitability grouped tasks on
//! `COALESCE(parent_task_id, id)`. That worked only because the hierarchy is
//! capped at two levels, and it left no home at all for work that bills nobody
//! — a weekly team meeting, an afternoon of admin. Both cost real money.
//!
//! A project is deliberately thin. It names a subject, optionally points at a
//! customer and a quote, and that is all. Everything expensive about it lives
//! on the tasks attached to it: their windows, their assignees, their expenses.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{CustomerContextId, CustomerId, OrganizationId, QuoteId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ProjectId(pub Uuid);

impl FromStr for ProjectId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ProjectId)
    }
}

impl Display for ProjectId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub id: ProjectId,
    pub organization_id: OrganizationId,
    pub name: String,
    /// Absent for internal work. Not a missing value to be filled in later:
    /// a project with no customer is the whole reason this entity exists, and
    /// [`Self::is_internal`] reads it as the deliberate state it is.
    pub customer_id: Option<CustomerId>,
    /// Requires `customer_id`, never the other way round — same relaxed
    /// pairing as `tasks` since the `relax_task_customer_context_pairing`
    /// migration.
    pub customer_context_id: Option<CustomerContextId>,
    /// The quote this project is measured against. Absent means no margin can
    /// be stated, which is a different thing from a margin of zero.
    pub quote_id: Option<QuoteId>,
    /// Set when the project is no longer worked. It stops appearing in
    /// pickers, and its cost history stays readable: what a job cost last
    /// spring does not stop being true when the job ends.
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Project {
    /// Whether this project bills nobody. Internal projects are what let a
    /// recurring meeting or an afternoon of admin carry a cost.
    pub fn is_internal(&self) -> bool {
        self.customer_id.is_none()
    }

    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project(customer_id: Option<CustomerId>) -> Project {
        let now = Utc::now();

        Project {
            id: ProjectId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            name: "Réunion hebdo".to_owned(),
            customer_id,
            customer_context_id: None,
            quote_id: None,
            archived_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn project_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = ProjectId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn project_id_rejects_invalid_uuid() {
        assert!(ProjectId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn a_project_without_a_customer_is_internal() {
        assert!(project(None).is_internal());
    }

    #[test]
    fn a_project_with_a_customer_is_not_internal() {
        assert!(!project(Some(CustomerId(Uuid::new_v4()))).is_internal());
    }
}
