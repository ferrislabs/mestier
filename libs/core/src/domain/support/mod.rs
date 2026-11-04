use std::{collections::HashSet, fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use common::CoreError;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{MemberId, OrganizationId};

pub use api_key::{
    GeneratedApiKey, PresentedApiKey, SupportApiKey, SupportApiKeyId, api_key_matches,
    generate_api_key, validate_api_key_name,
};

pub mod api_key;
pub mod assignment;
pub mod commands;
pub mod events;
pub mod ports;
pub mod thread_service;
pub mod ticket_service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TicketId(pub Uuid);

impl FromStr for TicketId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TicketId)
    }
}

impl Display for TicketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
pub struct TicketNumber(pub i32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TicketEntryId(pub Uuid);

impl FromStr for TicketEntryId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TicketEntryId)
    }
}

impl Display for TicketEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct AttachmentId(pub Uuid);

impl FromStr for AttachmentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(AttachmentId)
    }
}

impl Display for AttachmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum TicketStatus {
    New,
    InProgress,
    WaitingCustomer,
    Resolved,
}

impl TicketStatus {
    pub const ALL: [TicketStatus; 4] = [
        TicketStatus::New,
        TicketStatus::InProgress,
        TicketStatus::WaitingCustomer,
        TicketStatus::Resolved,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            TicketStatus::New => "new",
            TicketStatus::InProgress => "in_progress",
            TicketStatus::WaitingCustomer => "waiting_customer",
            TicketStatus::Resolved => "resolved",
        }
    }
}

impl FromStr for TicketStatus {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        TicketStatus::ALL
            .into_iter()
            .find(|status| status.as_str() == s)
            .ok_or_else(|| CoreError::Conflict(format!("unknown ticket status `{s}`")))
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum BadgeTone {
    #[default]
    Neutral,
    Brand,
    Success,
    Warning,
    Error,
}

impl BadgeTone {
    pub const ALL: [BadgeTone; 5] = [
        BadgeTone::Neutral,
        BadgeTone::Brand,
        BadgeTone::Success,
        BadgeTone::Warning,
        BadgeTone::Error,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            BadgeTone::Neutral => "neutral",
            BadgeTone::Brand => "brand",
            BadgeTone::Success => "success",
            BadgeTone::Warning => "warning",
            BadgeTone::Error => "error",
        }
    }
}

impl FromStr for BadgeTone {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        BadgeTone::ALL
            .into_iter()
            .find(|tone| tone.as_str() == s)
            .ok_or_else(|| CoreError::Conflict(format!("unknown badge tone `{s}`")))
    }
}

pub const MAX_BADGE_LABEL_CHARS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct Badge {
    label: String,
    tone: BadgeTone,
}

impl Badge {
    pub fn new(label: &str, tone: BadgeTone) -> Result<Self, CoreError> {
        let label = label.trim();
        if label.is_empty() {
            return Err(CoreError::Conflict(
                "badge label must not be empty".to_owned(),
            ));
        }
        if label.chars().count() > MAX_BADGE_LABEL_CHARS {
            return Err(CoreError::Conflict(format!(
                "badge label must be at most {MAX_BADGE_LABEL_CHARS} characters"
            )));
        }
        Ok(Self {
            label: label.to_owned(),
            tone,
        })
    }

    pub fn label(&self) -> &str {
        &self.label
    }

    pub fn tone(&self) -> BadgeTone {
        self.tone
    }
}

pub const MAX_CUSTOM_FIELD_KEYS: usize = 64;
pub const MAX_CUSTOM_FIELD_KEY_CHARS: usize = 64;
pub const MAX_CUSTOM_FIELDS_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Default, PartialEq, Serialize)]
#[serde(transparent)]
pub struct CustomFields(Map<String, Value>);

impl CustomFields {
    pub fn as_map(&self) -> &Map<String, Value> {
        &self.0
    }

    pub fn into_value(self) -> Value {
        Value::Object(self.0)
    }
}

impl TryFrom<Value> for CustomFields {
    type Error = CoreError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let Value::Object(map) = value else {
            return Err(CoreError::Conflict(
                "custom_fields must be a JSON object".to_owned(),
            ));
        };
        if map.len() > MAX_CUSTOM_FIELD_KEYS {
            return Err(CoreError::Conflict(format!(
                "custom_fields holds at most {MAX_CUSTOM_FIELD_KEYS} keys"
            )));
        }
        if let Some(key) = map
            .keys()
            .find(|key| key.is_empty() || key.chars().count() > MAX_CUSTOM_FIELD_KEY_CHARS)
        {
            return Err(CoreError::Conflict(format!(
                "custom_fields key `{key}` must be 1 to {MAX_CUSTOM_FIELD_KEY_CHARS} characters"
            )));
        }
        let size = serde_json::to_vec(&map)
            .map_err(|error| CoreError::Internal(error.to_string()))?
            .len();
        if size > MAX_CUSTOM_FIELDS_BYTES {
            return Err(CoreError::Conflict(format!(
                "custom_fields must be at most {MAX_CUSTOM_FIELDS_BYTES} bytes"
            )));
        }
        Ok(Self(map))
    }
}

pub const MAX_DISPLAY_FIELDS: usize = 8;
pub const MAX_DISPLAY_FIELD_CHARS: usize = 64;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct DisplayFields(Vec<String>);

impl DisplayFields {
    pub fn as_slice(&self) -> &[String] {
        &self.0
    }

    pub fn into_vec(self) -> Vec<String> {
        self.0
    }
}

impl TryFrom<Vec<String>> for DisplayFields {
    type Error = CoreError;

    fn try_from(keys: Vec<String>) -> Result<Self, Self::Error> {
        if keys.len() > MAX_DISPLAY_FIELDS {
            return Err(CoreError::Conflict(format!(
                "display fields hold at most {MAX_DISPLAY_FIELDS} keys"
            )));
        }
        if let Some(key) = keys
            .iter()
            .find(|key| key.is_empty() || key.chars().count() > MAX_DISPLAY_FIELD_CHARS)
        {
            return Err(CoreError::Conflict(format!(
                "display field `{key}` must be 1 to {MAX_DISPLAY_FIELD_CHARS} characters"
            )));
        }
        let mut seen = HashSet::with_capacity(keys.len());
        if let Some(key) = keys.iter().find(|key| !seen.insert(key.as_str())) {
            return Err(CoreError::Conflict(format!(
                "display field `{key}` is listed more than once"
            )));
        }
        Ok(Self(keys))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ticket {
    pub id: TicketId,
    pub organization_id: OrganizationId,
    pub number: TicketNumber,
    pub subject: String,
    pub body: String,
    pub status: TicketStatus,
    pub assignee: Option<MemberId>,
    pub assigned_at: Option<DateTime<Utc>>,
    pub custom_fields: CustomFields,
    pub badges: Vec<Badge>,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum EntryKind {
    ClientMessage,
    Reply,
    InternalNote,
    Draft,
}

impl EntryKind {
    pub const ALL: [EntryKind; 4] = [
        EntryKind::ClientMessage,
        EntryKind::Reply,
        EntryKind::InternalNote,
        EntryKind::Draft,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            EntryKind::ClientMessage => "client_message",
            EntryKind::Reply => "reply",
            EntryKind::InternalNote => "internal_note",
            EntryKind::Draft => "draft",
        }
    }

    pub fn is_client_visible(self) -> bool {
        matches!(self, EntryKind::ClientMessage | EntryKind::Reply)
    }
}

impl FromStr for EntryKind {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        EntryKind::ALL
            .into_iter()
            .find(|kind| kind.as_str() == s)
            .ok_or_else(|| CoreError::Conflict(format!("unknown entry kind `{s}`")))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attachment {
    pub id: AttachmentId,
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
    pub entry_id: TicketEntryId,
    pub file_key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocatedAttachment {
    pub attachment: Attachment,
    pub entry_kind: EntryKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TicketEntry {
    pub id: TicketEntryId,
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
    pub kind: EntryKind,
    pub author: Option<MemberId>,
    pub body: String,
    pub attachments: Vec<Attachment>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CreatedTicket {
    pub ticket: Ticket,
    pub created: bool,
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn every_status_round_trips_through_its_stored_value() {
        for status in TicketStatus::ALL {
            assert_eq!(TicketStatus::from_str(status.as_str()).unwrap(), status);
        }
    }

    #[test]
    fn stored_status_values_are_the_four_fixed_ones() {
        let values: Vec<&str> = TicketStatus::ALL.iter().map(|s| s.as_str()).collect();

        assert_eq!(
            values,
            ["new", "in_progress", "waiting_customer", "resolved"]
        );
    }

    #[test]
    fn an_unknown_status_is_refused() {
        assert!(TicketStatus::from_str("closed").is_err());
    }

    #[test]
    fn badge_tone_defaults_to_neutral_and_round_trips() {
        assert_eq!(BadgeTone::default(), BadgeTone::Neutral);
        for tone in BadgeTone::ALL {
            assert_eq!(BadgeTone::from_str(tone.as_str()).unwrap(), tone);
        }
    }

    #[test]
    fn badge_label_is_trimmed_and_must_not_be_blank() {
        assert_eq!(
            Badge::new("  High ", BadgeTone::Error).unwrap().label(),
            "High"
        );
        assert!(Badge::new("   ", BadgeTone::Neutral).is_err());
        assert!(Badge::new(&"x".repeat(65), BadgeTone::Neutral).is_err());
    }

    #[test]
    fn custom_fields_accept_an_object() {
        let fields = CustomFields::try_from(json!({ "team": "tech", "seats": 12 })).unwrap();

        assert_eq!(fields.as_map().get("team"), Some(&json!("tech")));
    }

    #[test]
    fn custom_fields_refuse_anything_but_an_object() {
        assert!(CustomFields::try_from(json!(["team"])).is_err());
        assert!(CustomFields::try_from(json!("team")).is_err());
        assert!(CustomFields::try_from(Value::Null).is_err());
    }

    #[test]
    fn custom_fields_refuse_more_than_64_keys() {
        let map: Map<String, Value> = (0..65).map(|i| (format!("k{i}"), json!(i))).collect();

        assert!(CustomFields::try_from(Value::Object(map)).is_err());
    }

    #[test]
    fn custom_fields_refuse_an_empty_or_too_long_key() {
        assert!(CustomFields::try_from(json!({ "": 1 })).is_err());
        let long_key = "k".repeat(65);
        assert!(CustomFields::try_from(json!({ long_key: 1 })).is_err());
    }

    #[test]
    fn custom_fields_refuse_more_than_16_kib() {
        let big = "x".repeat(MAX_CUSTOM_FIELDS_BYTES);

        assert!(CustomFields::try_from(json!({ "blob": big })).is_err());
    }

    fn keys(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    fn is_conflict<T: std::fmt::Debug>(outcome: Result<T, CoreError>) -> bool {
        matches!(outcome, Err(CoreError::Conflict(_)))
    }

    #[test]
    fn display_fields_keep_the_order_they_were_given() {
        let fields = DisplayFields::try_from(keys(&["team", "plan", "deployment_id"])).unwrap();

        assert_eq!(fields.as_slice(), keys(&["team", "plan", "deployment_id"]));
        assert_eq!(fields.into_vec(), keys(&["team", "plan", "deployment_id"]));
    }

    #[test]
    fn display_fields_default_to_an_empty_list() {
        assert!(DisplayFields::default().as_slice().is_empty());
        assert!(
            DisplayFields::try_from(Vec::new())
                .unwrap()
                .as_slice()
                .is_empty()
        );
    }

    #[test]
    fn display_fields_accept_eight_keys_and_refuse_nine() {
        let eight: Vec<String> = (0..MAX_DISPLAY_FIELDS).map(|i| format!("k{i}")).collect();
        let nine: Vec<String> = (0..=MAX_DISPLAY_FIELDS).map(|i| format!("k{i}")).collect();

        assert_eq!(DisplayFields::try_from(eight).unwrap().as_slice().len(), 8);
        assert!(is_conflict(DisplayFields::try_from(nine)));
    }

    #[test]
    fn display_fields_refuse_a_duplicate_key() {
        assert!(is_conflict(DisplayFields::try_from(keys(&[
            "team", "plan", "team"
        ]))));
    }

    #[test]
    fn display_fields_refuse_an_empty_key() {
        assert!(is_conflict(DisplayFields::try_from(keys(&["team", ""]))));
    }

    #[test]
    fn display_fields_accept_a_64_character_key_and_refuse_65() {
        let accepted = "k".repeat(MAX_DISPLAY_FIELD_CHARS);
        let refused = "k".repeat(MAX_DISPLAY_FIELD_CHARS + 1);

        assert!(DisplayFields::try_from(vec![accepted]).is_ok());
        assert!(is_conflict(DisplayFields::try_from(vec![refused])));
    }

    #[test]
    fn display_fields_count_characters_not_bytes() {
        let accented = "é".repeat(MAX_DISPLAY_FIELD_CHARS);
        let too_long = "é".repeat(MAX_DISPLAY_FIELD_CHARS + 1);

        assert!(accented.len() > MAX_DISPLAY_FIELD_CHARS);
        assert!(DisplayFields::try_from(vec![accented]).is_ok());
        assert!(is_conflict(DisplayFields::try_from(vec![too_long])));
    }

    #[test]
    fn display_fields_serialize_as_a_plain_array() {
        let fields = DisplayFields::try_from(keys(&["team", "plan"])).unwrap();

        assert_eq!(
            serde_json::to_value(&fields).unwrap(),
            json!(["team", "plan"])
        );
    }

    #[test]
    fn only_client_message_and_reply_are_client_visible() {
        let visible: Vec<EntryKind> = EntryKind::ALL
            .into_iter()
            .filter(|kind| kind.is_client_visible())
            .collect();

        assert_eq!(visible, [EntryKind::ClientMessage, EntryKind::Reply]);
    }

    #[test]
    fn every_entry_kind_round_trips_through_its_stored_value() {
        for kind in EntryKind::ALL {
            assert_eq!(EntryKind::from_str(kind.as_str()).unwrap(), kind);
        }
    }
}
