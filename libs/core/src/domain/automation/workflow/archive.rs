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
}

impl WorkflowRefusal {
    const ALL: [Self; 1] = [Self::Archived];

    pub fn message(self) -> &'static str {
        match self {
            Self::Archived => "the workflow is archived",
        }
    }

    pub fn from_conflict(message: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|refusal| refusal.message() == message)
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
    fn an_unrelated_conflict_is_not_a_refusal() {
        assert_eq!(WorkflowRefusal::from_conflict("slug taken"), None);
    }
}
