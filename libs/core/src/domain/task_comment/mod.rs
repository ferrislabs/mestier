use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{OrganizationId, TaskId, UserId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TaskCommentId(pub Uuid);

impl FromStr for TaskCommentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TaskCommentId)
    }
}

impl Display for TaskCommentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A single message in a task's comment thread — the annotable half of the
/// planning remodel's "a task resembles a GitHub issue" framing (see the
/// planning module design doc). The author is always a `user`, never an
/// `employee`: it is the person writing, and an employee without an account
/// cannot comment — the only place in the planning module where identity
/// matters more than worker role.
///
/// Deletion is logical: `deleted_at` set means the comment is gone from the
/// thread, but the row survives — unlike `task_label_links`'/
/// `task_assignments`' physical deletion, a conversation is worth keeping
/// once retracted from view.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskComment {
    pub id: TaskCommentId,
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub author_user_id: UserId,
    pub body: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_comment_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = TaskCommentId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn task_comment_id_rejects_invalid_uuid() {
        assert!(TaskCommentId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn task_comment_id_displays_as_its_uuid() {
        let uuid = Uuid::new_v4();
        let id = TaskCommentId(uuid);

        assert_eq!(id.to_string(), uuid.to_string());
    }
}
