use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{UserId, domain::organization::OrganizationId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ToSchema)]
pub struct MemberId(pub Uuid);

impl FromStr for MemberId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(MemberId)
    }
}

impl Display for MemberId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Member {
    pub id: MemberId,
    pub organization_id: OrganizationId,
    /// `None` = free seat: nobody occupies it yet. A seat can stay that way
    /// indefinitely (an apprentice, a temp, a seasonal hire planned without
    /// access yet). The invitation flow (#184) is what fills it.
    pub user_id: Option<UserId>,
    pub last_name: String,
    /// `None` = not provided, distinct from an empty string.
    pub first_name: Option<String>,
    /// `None` for as long as the seat has never been occupied.
    pub joined_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl Member {
    /// The one display format for a member, everywhere: "{last_name}
    /// {first_name}". See [`format_person_name`].
    pub fn display_name(&self) -> String {
        format_person_name(&self.last_name, self.first_name.as_deref())
    }
}

/// "{last_name} {first_name}" — last name first, then first name, in that
/// order (not the conventional first-then-last). When `first_name` is
/// absent or blank, the last name alone, with no trailing whitespace.
///
/// The one formatting function, called from every layer that needs to show
/// a person's name — domain, handler DTOs, front — so the format lives in
/// exactly one place instead of being hand-concatenated on each screen.
/// Used by both [`Member`] and [`crate::domain::employee::Employee`]: a
/// person's display name is the same shape whether it comes from a
/// standalone seat or an HR profile attached to one.
pub fn format_person_name(last_name: &str, first_name: Option<&str>) -> String {
    let last = last_name.trim();
    let first = first_name.map(str::trim).filter(|value| !value.is_empty());

    match first {
        Some(first) => format!("{last} {first}"),
        None => last.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = MemberId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn member_id_rejects_invalid_uuid() {
        assert!(MemberId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn format_person_name_joins_last_and_first_name_in_that_order() {
        assert_eq!(
            format_person_name("Bonnal", Some("Baptiste")),
            "Bonnal Baptiste"
        );
    }

    #[test]
    fn format_person_name_falls_back_to_the_last_name_alone_without_a_trailing_space() {
        assert_eq!(format_person_name("Bonnal", None), "Bonnal");
    }

    #[test]
    fn format_person_name_treats_a_blank_first_name_as_absent() {
        assert_eq!(format_person_name("Bonnal", Some("   ")), "Bonnal");
    }

    #[test]
    fn format_person_name_trims_stray_whitespace_on_both_parts() {
        assert_eq!(
            format_person_name("  Bonnal  ", Some("  Baptiste  ")),
            "Bonnal Baptiste"
        );
    }

    #[test]
    fn member_display_name_delegates_to_format_person_name() {
        let now = Utc::now();
        let member = Member {
            id: MemberId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            user_id: None,
            last_name: "Parmantier".to_owned(),
            first_name: Some("Baptiste".to_owned()),
            joined_at: None,
            created_at: now,
            deleted_at: None,
        };

        assert_eq!(member.display_name(), "Parmantier Baptiste");
    }
}
