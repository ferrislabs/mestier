use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::error::SurveyError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Visibility {
    Public,
    Members,
}

impl Visibility {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Public => "PUBLIC",
            Self::Members => "MEMBERS",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
pub struct QuestionId(pub Uuid);

impl FromStr for QuestionId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(QuestionId)
    }
}

impl Display for QuestionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QuestionKind {
    Text,
    LongText,
    Number,
    SingleChoice,
    MultipleChoice,
    Date,
    Datetime,
    Phone,
    Email,
    Photo,
}

impl QuestionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "TEXT",
            Self::LongText => "LONG_TEXT",
            Self::Number => "NUMBER",
            Self::SingleChoice => "SINGLE_CHOICE",
            Self::MultipleChoice => "MULTIPLE_CHOICE",
            Self::Date => "DATE",
            Self::Datetime => "DATETIME",
            Self::Phone => "PHONE",
            Self::Email => "EMAIL",
            Self::Photo => "PHOTO",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct EmailAddress(String);

impl EmailAddress {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for EmailAddress {
    type Error = SurveyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let normalized = value.trim().to_lowercase();
        let Some((local, domain)) = normalized.split_once('@') else {
            return Err(SurveyError::EmailInvalid);
        };
        let well_formed = !local.is_empty()
            && !domain.is_empty()
            && !domain.contains('@')
            && !normalized.chars().any(char::is_whitespace);
        if well_formed {
            Ok(Self(normalized))
        } else {
            Err(SurveyError::EmailInvalid)
        }
    }
}

impl TryFrom<String> for EmailAddress {
    type Error = SurveyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct PhoneNumber(String);

impl PhoneNumber {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for PhoneNumber {
    type Error = SurveyError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        let mut normalized = String::with_capacity(trimmed.len());
        let mut digits = 0usize;
        for (index, c) in trimmed.char_indices() {
            match c {
                '0'..='9' => {
                    normalized.push(c);
                    digits += 1;
                }
                '+' if index == 0 => normalized.push(c),
                ' ' | '-' | '(' | ')' => {}
                _ => return Err(SurveyError::PhoneInvalid),
            }
        }
        if (6..=15).contains(&digits) {
            Ok(Self(normalized))
        } else {
            Err(SurveyError::PhoneInvalid)
        }
    }
}

impl TryFrom<String> for PhoneNumber {
    type Error = SurveyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Answer {
    Text(String),
    Number(f64),
    Choice(String),
    Choices(Vec<String>),
    Date(NaiveDate),
    DateTime(DateTime<Utc>),
    Phone(PhoneNumber),
    Email(EmailAddress),
    Photos(Vec<String>),
}

impl Answer {
    pub fn is_blank(&self) -> bool {
        match self {
            Self::Text(text) => text.trim().is_empty(),
            Self::Choices(options) => options.is_empty(),
            Self::Photos(photos) => photos.is_empty(),
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String")]
pub struct PublicSlug(String);

impl PublicSlug {
    pub fn new(value: String) -> Result<Self, SurveyError> {
        let well_formed = (8..=64).contains(&value.len())
            && value
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_');
        if well_formed {
            Ok(Self(value))
        } else {
            Err(SurveyError::PublicSlugInvalid)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PublicSlug {
    type Error = SurveyError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
