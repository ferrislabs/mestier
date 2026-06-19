use chrono::{DateTime, Utc};
pub use common::UserId;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub username: String,
    pub name: String,
    pub sub: String,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn user_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = UserId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn user_id_rejects_invalid_uuid() {
        let parsed = UserId::from_str("not-a-uuid");

        assert!(parsed.is_err());
    }
}
