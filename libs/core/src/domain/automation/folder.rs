use std::fmt;

use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use uuid::Uuid;

pub const MAX_FOLDER_NAME_LENGTH: usize = 80;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FolderName(String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum FolderNameError {
    #[error("a folder name cannot be empty")]
    Empty,
    #[error("a folder name holds at most {MAX_FOLDER_NAME_LENGTH} characters")]
    TooLong,
}

impl FolderName {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<String> for FolderName {
    type Error = FolderNameError;

    fn try_from(raw: String) -> Result<Self, Self::Error> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(FolderNameError::Empty);
        }
        if trimmed.chars().count() > MAX_FOLDER_NAME_LENGTH {
            return Err(FolderNameError::TooLong);
        }
        if trimmed.len() == raw.len() {
            return Ok(Self(raw));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowFolder {
    pub id: Uuid,
    pub org_id: OrganizationId,
    pub parent_id: Option<Uuid>,
    pub name: FolderName,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateFolderCommand {
    pub org_id: OrganizationId,
    pub name: FolderName,
    pub parent_id: Option<Uuid>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateFolderCommand {
    pub org_id: OrganizationId,
    pub id: Uuid,
    pub name: Option<FolderName>,
    pub parent_id: Option<Option<Uuid>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FolderRefusal {
    NotEmpty,
    WouldCycle,
    NameTaken,
    Missing,
}

impl FolderRefusal {
    const ALL: [Self; 4] = [
        Self::NotEmpty,
        Self::WouldCycle,
        Self::NameTaken,
        Self::Missing,
    ];

    pub fn message(self) -> &'static str {
        match self {
            Self::NotEmpty => "the folder is not empty",
            Self::WouldCycle => "a folder cannot be moved into itself or one of its descendants",
            Self::NameTaken => "a folder with this name already exists at this level",
            Self::Missing => "the folder does not exist in this organization",
        }
    }

    pub fn from_conflict(message: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|refusal| refusal.message() == message)
    }
}

impl fmt::Display for FolderRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.message())
    }
}

impl std::error::Error for FolderRefusal {}

impl From<FolderRefusal> for CoreError {
    fn from(refusal: FolderRefusal) -> Self {
        Self::Conflict(refusal.message().to_owned())
    }
}

pub fn would_cycle(folders: &[WorkflowFolder], folder_id: Uuid, new_parent_id: Uuid) -> bool {
    let mut cursor = Some(new_parent_id);
    let mut steps = 0;

    while let Some(current) = cursor {
        if current == folder_id || steps > folders.len() {
            return true;
        }
        steps += 1;
        cursor = folders
            .iter()
            .find(|folder| folder.id == current)
            .and_then(|folder| folder.parent_id);
    }

    false
}

#[cfg(test)]
mod tests {
    use common::generate_uuid_v7;

    use super::*;

    fn name(raw: &str) -> FolderName {
        FolderName::try_from(raw.to_owned()).expect("valid name")
    }

    fn folder(id: Uuid, parent_id: Option<Uuid>) -> WorkflowFolder {
        WorkflowFolder {
            id,
            org_id: OrganizationId(Uuid::nil()),
            parent_id,
            name: name("Folder"),
            created_at: DateTime::<Utc>::from_timestamp(1_700_000_000, 0).expect("valid instant"),
        }
    }

    #[test]
    fn a_name_is_trimmed() {
        assert_eq!(name("  Facturation ").as_str(), "Facturation");
    }

    #[test]
    fn a_blank_name_is_refused() {
        assert_eq!(
            FolderName::try_from("   ".to_owned()),
            Err(FolderNameError::Empty)
        );
        assert_eq!(
            FolderName::try_from(String::new()),
            Err(FolderNameError::Empty)
        );
    }

    #[test]
    fn eighty_characters_pass_and_eighty_one_do_not() {
        assert!(FolderName::try_from("é".repeat(80)).is_ok());
        assert_eq!(
            FolderName::try_from("é".repeat(81)),
            Err(FolderNameError::TooLong)
        );
    }

    #[test]
    fn the_length_limit_applies_after_trimming() {
        assert!(FolderName::try_from(format!(" {} ", "a".repeat(80))).is_ok());
    }

    #[test]
    fn a_folder_cannot_become_its_own_parent() {
        let only = generate_uuid_v7();

        assert!(would_cycle(&[folder(only, None)], only, only));
    }

    #[test]
    fn a_folder_cannot_move_under_its_own_descendant() {
        let (root, child, grandchild) =
            (generate_uuid_v7(), generate_uuid_v7(), generate_uuid_v7());
        let tree = [
            folder(root, None),
            folder(child, Some(root)),
            folder(grandchild, Some(child)),
        ];

        assert!(would_cycle(&tree, root, grandchild));
        assert!(would_cycle(&tree, root, child));
    }

    #[test]
    fn a_folder_can_move_to_a_sibling_branch_or_a_shallower_level() {
        let (root, left, right, leaf) = (
            generate_uuid_v7(),
            generate_uuid_v7(),
            generate_uuid_v7(),
            generate_uuid_v7(),
        );
        let tree = [
            folder(root, None),
            folder(left, Some(root)),
            folder(right, Some(root)),
            folder(leaf, Some(left)),
        ];

        assert!(!would_cycle(&tree, leaf, right));
        assert!(!would_cycle(&tree, leaf, root));
        assert!(!would_cycle(&tree, left, right));
    }

    #[test]
    fn an_already_corrupt_loop_above_the_target_is_refused_instead_of_looping() {
        let (a, b, moved) = (generate_uuid_v7(), generate_uuid_v7(), generate_uuid_v7());
        let tree = [folder(a, Some(b)), folder(b, Some(a)), folder(moved, None)];

        assert!(would_cycle(&tree, moved, a));
    }

    #[test]
    fn a_refusal_survives_the_trip_through_a_conflict() {
        for refusal in FolderRefusal::ALL {
            let CoreError::Conflict(message) = CoreError::from(refusal) else {
                panic!("a refusal must become a conflict");
            };

            assert_eq!(FolderRefusal::from_conflict(&message), Some(refusal));
        }
    }

    #[test]
    fn an_unrelated_conflict_is_not_a_refusal() {
        assert_eq!(FolderRefusal::from_conflict("slug taken"), None);
    }
}
