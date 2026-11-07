use serde::{Deserialize, Deserializer};

/// Distinguishes "the key is absent" (leave the field unchanged) from "the
/// key is present" (apply it, `null` included) — same convention as
/// `handlers-planning::task::update::deserialize_present`.
pub(crate) fn deserialize_present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

pub(crate) fn invalid(error: impl std::fmt::Display) -> handlers::ApiError {
    handlers::ApiError::Validation(error.to_string())
}
