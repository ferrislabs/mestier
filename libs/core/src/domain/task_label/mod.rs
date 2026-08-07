use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TaskLabelId(pub Uuid);

impl FromStr for TaskLabelId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TaskLabelId)
    }
}

impl Display for TaskLabelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A tag attached to a task, absorbing what used to be its "nature" —
/// meeting, trip, training session. Not a column, not an enum: extensible
/// without a migration, and it follows the issue-label model the planning
/// remodel explicitly invokes. See the planning module design doc.
///
/// Nothing here distinguishes a preset (seeded by
/// [`crate::domain::organization::service::OrganizationService::create_organization`])
/// from a label created by hand — the user may rename or delete either.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskLabel {
    pub id: TaskLabelId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// The three labels seeded transactionally with every new organization,
/// alongside its default roles — `(name, color)`. Colors are `#RRGGBB`,
/// chosen to be readable and distinct from one another. Not privileged in
/// any way once created: same table, same constraints, same CRUD as a
/// label created by hand.
pub const PRESET_TASK_LABELS: [(&str, &str); 3] = [
    ("Réunion", "#2563EB"),
    ("Déplacement", "#059669"),
    ("Formation", "#D97706"),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_label_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = TaskLabelId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn task_label_id_rejects_invalid_uuid() {
        assert!(TaskLabelId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn task_label_id_displays_as_its_uuid() {
        let uuid = Uuid::new_v4();
        let id = TaskLabelId(uuid);

        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn preset_task_labels_has_exactly_the_three_documented_names() {
        let names: Vec<&str> = PRESET_TASK_LABELS.iter().map(|(name, _)| *name).collect();

        assert_eq!(names, vec!["Réunion", "Déplacement", "Formation"]);
    }

    #[test]
    fn preset_task_labels_use_well_formed_hex_colors() {
        for (name, color) in PRESET_TASK_LABELS {
            assert_eq!(
                color.len(),
                7,
                "{name}'s color must be a 7-character #RRGGBB value"
            );
            assert!(color.starts_with('#'), "{name}'s color must start with #");
            assert!(
                color[1..].chars().all(|c| c.is_ascii_hexdigit()),
                "{name}'s color must be hex digits after #"
            );
        }
    }

    #[test]
    fn preset_task_labels_use_distinct_colors() {
        let colors: std::collections::HashSet<&str> =
            PRESET_TASK_LABELS.iter().map(|(_, color)| *color).collect();

        assert_eq!(
            colors.len(),
            PRESET_TASK_LABELS.len(),
            "every preset must be visually distinguishable from the others"
        );
    }
}
