use std::fmt;

use common::CoreError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveScope {
    Active,
    Archived,
    All,
}

impl ArchiveScope {
    pub fn includes_active(self) -> bool {
        matches!(self, Self::Active | Self::All)
    }

    pub fn includes_archived(self) -> bool {
        matches!(self, Self::Archived | Self::All)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowRefusal {
    Archived,
    ConflictingFlags,
    WebhookPathTaken,
    TooManyWebhookRoutes,
    TooManySchedules,
}

impl WorkflowRefusal {
    const ALL: [Self; 5] = [
        Self::Archived,
        Self::ConflictingFlags,
        Self::WebhookPathTaken,
        Self::TooManyWebhookRoutes,
        Self::TooManySchedules,
    ];

    pub fn message(self) -> &'static str {
        match self {
            Self::Archived => "the workflow is archived",
            Self::ConflictingFlags => "a workflow cannot be archived and enabled in one request",
            Self::WebhookPathTaken => "a webhook path is already used by another workflow",
            Self::TooManyWebhookRoutes => "an organization holds at most 200 webhook routes",
            Self::TooManySchedules => "an organization holds at most 200 schedules",
        }
    }

    pub fn from_conflict(message: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|refusal| refusal.message() == message)
            .or_else(|| Self::webhook_path_taken_triggers(message).map(|_| Self::WebhookPathTaken))
    }

    pub fn webhook_path_taken(trigger_ids: &[String]) -> CoreError {
        let ids = serde_json::to_string(trigger_ids).unwrap_or_default();
        CoreError::Conflict(format!("{}: {ids}", Self::WebhookPathTaken.message()))
    }

    pub fn webhook_path_taken_triggers(message: &str) -> Option<Vec<String>> {
        let rest = message.strip_prefix(Self::WebhookPathTaken.message())?;
        if rest.is_empty() {
            return Some(Vec::new());
        }
        serde_json::from_str(rest.strip_prefix(": ")?).ok()
    }
}

impl fmt::Display for WorkflowRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for WorkflowRefusal {}

impl From<WorkflowRefusal> for CoreError {
    fn from(refusal: WorkflowRefusal) -> Self {
        Self::Conflict(refusal.message().to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        schedule::MAX_SCHEDULES_PER_ORGANIZATION, webhook::MAX_WEBHOOK_ROUTES_PER_ORGANIZATION,
    };
    use super::*;

    #[test]
    fn the_active_scope_excludes_archived_workflows() {
        assert!(ArchiveScope::Active.includes_active());
        assert!(!ArchiveScope::Active.includes_archived());
    }

    #[test]
    fn the_archived_scope_excludes_active_workflows() {
        assert!(!ArchiveScope::Archived.includes_active());
        assert!(ArchiveScope::Archived.includes_archived());
    }

    #[test]
    fn the_all_scope_includes_both() {
        assert!(ArchiveScope::All.includes_active());
        assert!(ArchiveScope::All.includes_archived());
    }

    #[test]
    fn a_refusal_survives_the_trip_through_a_conflict() {
        let conflict = CoreError::from(WorkflowRefusal::Archived);

        let CoreError::Conflict(message) = conflict else {
            panic!("a refusal must become a conflict");
        };

        assert_eq!(
            WorkflowRefusal::from_conflict(&message),
            Some(WorkflowRefusal::Archived)
        );
    }

    #[test]
    fn every_refusal_survives_the_trip_through_a_conflict() {
        for refusal in WorkflowRefusal::ALL {
            let CoreError::Conflict(message) = CoreError::from(refusal) else {
                panic!("a refusal must become a conflict");
            };

            assert_eq!(WorkflowRefusal::from_conflict(&message), Some(refusal));
        }
    }

    #[test]
    fn an_unrelated_conflict_is_not_a_refusal() {
        assert_eq!(WorkflowRefusal::from_conflict("slug taken"), None);
    }

    #[test]
    fn a_taken_path_names_the_triggers_that_hold_it_through_a_conflict() {
        let ids = vec![
            "t1".to_owned(),
            "with, comma".to_owned(),
            "quote\"d".to_owned(),
            "d\u{e9}clencheur".to_owned(),
        ];

        let CoreError::Conflict(message) = WorkflowRefusal::webhook_path_taken(&ids) else {
            panic!("a refusal must become a conflict");
        };

        assert_eq!(
            WorkflowRefusal::from_conflict(&message),
            Some(WorkflowRefusal::WebhookPathTaken)
        );
        assert_eq!(
            WorkflowRefusal::webhook_path_taken_triggers(&message),
            Some(ids)
        );
    }

    #[test]
    fn the_bare_message_of_a_taken_path_names_no_trigger() {
        let bare = WorkflowRefusal::WebhookPathTaken.message();

        assert_eq!(
            WorkflowRefusal::webhook_path_taken_triggers(bare),
            Some(Vec::new())
        );
        assert_eq!(
            WorkflowRefusal::webhook_path_taken_triggers("slug taken"),
            None
        );
    }

    #[test]
    fn a_taken_path_message_followed_by_anything_else_is_not_that_refusal() {
        let bare = WorkflowRefusal::WebhookPathTaken.message();

        for suffix in [
            ": not json",
            ": {\"a\":1}",
            " extra",
            ": [1, 2]",
            ":[\"t1\"]",
        ] {
            let message = format!("{bare}{suffix}");

            assert_eq!(
                WorkflowRefusal::webhook_path_taken_triggers(&message),
                None,
                "{suffix}"
            );
            assert_eq!(WorkflowRefusal::from_conflict(&message), None, "{suffix}");
        }
    }

    #[test]
    fn the_quota_messages_state_the_limits_the_domain_enforces() {
        assert!(
            WorkflowRefusal::TooManyWebhookRoutes
                .message()
                .contains(&MAX_WEBHOOK_ROUTES_PER_ORGANIZATION.to_string())
        );
        assert!(
            WorkflowRefusal::TooManySchedules
                .message()
                .contains(&MAX_SCHEDULES_PER_ORGANIZATION.to_string())
        );
    }
}
