use std::collections::{HashMap, HashSet};
use std::fmt;

use chrono::{DateTime, Utc};
use common::{CoreError, OrganizationId};
use uuid::Uuid;

pub const MAX_FOLDER_NAME_LENGTH: usize = 80;
pub const MAX_FOLDERS_PER_ORGANIZATION: usize = 500;
pub const MAX_FOLDER_DEPTH: usize = 8;

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
    TooMany,
    TooDeep,
}

impl FolderRefusal {
    const ALL: [Self; 6] = [
        Self::NotEmpty,
        Self::WouldCycle,
        Self::NameTaken,
        Self::Missing,
        Self::TooMany,
        Self::TooDeep,
    ];

    pub fn message(self) -> &'static str {
        match self {
            Self::NotEmpty => "the folder is not empty",
            Self::WouldCycle => "a folder cannot be moved into itself or one of its descendants",
            Self::NameTaken => "a folder with this name already exists at this level",
            Self::Missing => "the folder does not exist in this organization",
            Self::TooMany => "an organization holds at most 500 folders",
            Self::TooDeep => "folders nest at most 8 levels deep",
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

#[derive(Debug, Clone)]
pub struct FolderTree {
    parents: HashMap<Uuid, Option<Uuid>>,
    children: HashMap<Uuid, Vec<Uuid>>,
}

impl FolderTree {
    pub fn new(folders: &[WorkflowFolder]) -> Self {
        let mut parents = HashMap::with_capacity(folders.len());
        let mut children: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
        for folder in folders {
            parents.insert(folder.id, folder.parent_id);
            if let Some(parent_id) = folder.parent_id {
                children.entry(parent_id).or_default().push(folder.id);
            }
        }
        Self { parents, children }
    }

    pub fn would_cycle(&self, folder_id: Uuid, new_parent_id: Uuid) -> bool {
        let mut cursor = Some(new_parent_id);
        let mut steps = 0;

        while let Some(current) = cursor {
            if current == folder_id || steps > self.parents.len() {
                return true;
            }
            steps += 1;
            cursor = self.parents.get(&current).copied().flatten();
        }

        false
    }

    pub fn depth_of(&self, folder_id: Uuid) -> usize {
        let mut depth = 1;
        let mut cursor = self.parents.get(&folder_id).copied().flatten();

        while let Some(current) = cursor {
            depth += 1;
            if depth > self.parents.len() + 1 {
                break;
            }
            cursor = self.parents.get(&current).copied().flatten();
        }

        depth
    }

    pub fn height_below(&self, folder_id: Uuid) -> usize {
        let mut visited = HashSet::new();
        let mut frontier = vec![(folder_id, 0usize)];
        let mut height = 0;

        while let Some((current, level)) = frontier.pop() {
            if !visited.insert(current) {
                continue;
            }
            height = height.max(level);
            for &child in self.children.get(&current).into_iter().flatten() {
                frontier.push((child, level + 1));
            }
        }

        height
    }

    pub fn check_creation(&self, parent_id: Option<Uuid>) -> Result<(), FolderRefusal> {
        if self.parents.len() >= MAX_FOLDERS_PER_ORGANIZATION {
            return Err(FolderRefusal::TooMany);
        }
        let Some(parent_id) = parent_id else {
            return Ok(());
        };
        if !self.parents.contains_key(&parent_id) {
            return Err(FolderRefusal::Missing);
        }
        if self.depth_of(parent_id) + 1 > MAX_FOLDER_DEPTH {
            return Err(FolderRefusal::TooDeep);
        }

        Ok(())
    }

    pub fn check_move(
        &self,
        folder_id: Uuid,
        new_parent_id: Option<Uuid>,
    ) -> Result<(), FolderRefusal> {
        let Some(new_parent_id) = new_parent_id else {
            return Ok(());
        };
        if !self.parents.contains_key(&new_parent_id) {
            return Err(FolderRefusal::Missing);
        }
        if self.would_cycle(folder_id, new_parent_id) {
            return Err(FolderRefusal::WouldCycle);
        }
        if self.depth_of(new_parent_id) + 1 + self.height_below(folder_id) > MAX_FOLDER_DEPTH {
            return Err(FolderRefusal::TooDeep);
        }

        Ok(())
    }
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

    fn chain(length: usize) -> Vec<WorkflowFolder> {
        let mut folders: Vec<WorkflowFolder> = Vec::with_capacity(length);
        for level in 0..length {
            let parent_id = level.checked_sub(1).map(|previous| folders[previous].id);
            folders.push(folder(generate_uuid_v7(), parent_id));
        }
        folders
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

        assert!(FolderTree::new(&[folder(only, None)]).would_cycle(only, only));
    }

    #[test]
    fn a_folder_cannot_move_under_its_own_descendant() {
        let (root, child, grandchild) =
            (generate_uuid_v7(), generate_uuid_v7(), generate_uuid_v7());
        let tree = FolderTree::new(&[
            folder(root, None),
            folder(child, Some(root)),
            folder(grandchild, Some(child)),
        ]);

        assert!(tree.would_cycle(root, grandchild));
        assert!(tree.would_cycle(root, child));
    }

    #[test]
    fn a_folder_can_move_to_a_sibling_branch_or_a_shallower_level() {
        let (root, left, right, leaf) = (
            generate_uuid_v7(),
            generate_uuid_v7(),
            generate_uuid_v7(),
            generate_uuid_v7(),
        );
        let tree = FolderTree::new(&[
            folder(root, None),
            folder(left, Some(root)),
            folder(right, Some(root)),
            folder(leaf, Some(left)),
        ]);

        assert!(!tree.would_cycle(leaf, right));
        assert!(!tree.would_cycle(leaf, root));
        assert!(!tree.would_cycle(left, right));
    }

    #[test]
    fn an_already_corrupt_loop_above_the_target_is_refused_instead_of_looping() {
        let (a, b, moved) = (generate_uuid_v7(), generate_uuid_v7(), generate_uuid_v7());
        let tree = FolderTree::new(&[folder(a, Some(b)), folder(b, Some(a)), folder(moved, None)]);

        assert!(tree.would_cycle(moved, a));
    }

    #[test]
    fn a_root_folder_sits_at_depth_one_and_each_level_adds_one() {
        let folders = chain(3);
        let tree = FolderTree::new(&folders);

        assert_eq!(tree.depth_of(folders[0].id), 1);
        assert_eq!(tree.depth_of(folders[2].id), 3);
    }

    #[test]
    fn the_height_below_a_folder_is_the_longest_path_down() {
        let folders = chain(4);
        let tree = FolderTree::new(&folders);

        assert_eq!(tree.height_below(folders[0].id), 3);
        assert_eq!(tree.height_below(folders[2].id), 1);
        assert_eq!(tree.height_below(folders[3].id), 0);
    }

    #[test]
    fn a_folder_can_be_created_at_the_eighth_level_and_not_the_ninth() {
        let seven = chain(MAX_FOLDER_DEPTH - 1);
        let eight = chain(MAX_FOLDER_DEPTH);

        assert_eq!(
            FolderTree::new(&seven).check_creation(Some(seven[MAX_FOLDER_DEPTH - 2].id)),
            Ok(())
        );
        assert_eq!(
            FolderTree::new(&eight).check_creation(Some(eight[MAX_FOLDER_DEPTH - 1].id)),
            Err(FolderRefusal::TooDeep)
        );
    }

    #[test]
    fn a_root_can_always_be_created_below_the_count_cap() {
        assert_eq!(FolderTree::new(&[]).check_creation(None), Ok(()));
    }

    #[test]
    fn a_folder_is_refused_once_the_organization_holds_the_maximum() {
        let full: Vec<WorkflowFolder> = (0..MAX_FOLDERS_PER_ORGANIZATION)
            .map(|_| folder(generate_uuid_v7(), None))
            .collect();
        let one_short = &full[..MAX_FOLDERS_PER_ORGANIZATION - 1];

        assert_eq!(FolderTree::new(one_short).check_creation(None), Ok(()));
        assert_eq!(
            FolderTree::new(&full).check_creation(None),
            Err(FolderRefusal::TooMany)
        );
    }

    #[test]
    fn creating_under_an_unknown_parent_is_refused() {
        assert_eq!(
            FolderTree::new(&[]).check_creation(Some(generate_uuid_v7())),
            Err(FolderRefusal::Missing)
        );
    }

    #[test]
    fn moving_a_deep_subtree_is_refused_when_it_would_pass_the_eighth_level() {
        let target_chain = chain(5);
        let subtree = chain(4);
        let mut folders = target_chain.clone();
        folders.extend(subtree.iter().cloned());
        let tree = FolderTree::new(&folders);

        assert_eq!(
            tree.check_move(subtree[0].id, Some(target_chain[4].id)),
            Err(FolderRefusal::TooDeep),
            "5 + 4 levels do not fit in 8"
        );
        assert_eq!(
            tree.check_move(subtree[0].id, Some(target_chain[3].id)),
            Ok(()),
            "4 + 4 levels fit exactly"
        );
    }

    #[test]
    fn moving_to_the_root_is_always_allowed_depth_wise() {
        let folders = chain(MAX_FOLDER_DEPTH);

        assert_eq!(
            FolderTree::new(&folders).check_move(folders[3].id, None),
            Ok(())
        );
    }

    #[test]
    fn a_move_reports_a_loop_before_a_depth_problem() {
        let folders = chain(3);

        assert_eq!(
            FolderTree::new(&folders).check_move(folders[0].id, Some(folders[2].id)),
            Err(FolderRefusal::WouldCycle)
        );
    }

    #[test]
    fn a_move_under_an_unknown_parent_is_refused() {
        let folders = chain(2);

        assert_eq!(
            FolderTree::new(&folders).check_move(folders[0].id, Some(generate_uuid_v7())),
            Err(FolderRefusal::Missing)
        );
    }

    #[test]
    fn the_caps_are_stated_in_their_refusal_messages() {
        assert!(
            FolderRefusal::TooMany
                .message()
                .contains(&MAX_FOLDERS_PER_ORGANIZATION.to_string())
        );
        assert!(
            FolderRefusal::TooDeep
                .message()
                .contains(&MAX_FOLDER_DEPTH.to_string())
        );
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
