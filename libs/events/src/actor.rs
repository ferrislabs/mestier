use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Who caused an event.
///
/// `User` and `Client` mirror the two variants of `auth::Identity`: a FerrisKey
/// client is an integration acting on its own behalf. Telling the two apart
/// matters the day an automation has to ignore what another automation caused.
///
/// An enum rather than a `kind` field beside an optional `id`: only `System`
/// has no identity, and a pair of fields would let a `User` without an id
/// compile. The invariant is worth more than the flatter shape.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Actor {
    User {
        id: Uuid,
    },
    Client {
        id: Uuid,
    },
    /// Scheduled jobs, migrations, and anything the product does on its own.
    System,
}

impl Actor {
    pub fn user(id: Uuid) -> Self {
        Self::User { id }
    }

    pub fn client(id: Uuid) -> Self {
        Self::Client { id }
    }

    pub fn system() -> Self {
        Self::System
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    /// A system actor has no identity to carry, so it must not carry a null
    /// one. The absent field is what tells a reader the id is meaningless
    /// here rather than merely unknown.
    #[test]
    fn a_system_actor_serializes_without_an_id_field() {
        let json = serde_json::to_value(Actor::system()).expect("actor serializes");

        assert_eq!(json, json!({ "kind": "system" }));
    }

    #[test]
    fn a_user_actor_serializes_with_its_id() {
        let json = serde_json::to_value(Actor::user(Uuid::from_u128(1))).expect("actor serializes");

        assert_eq!(
            json,
            json!({ "kind": "user", "id": "00000000-0000-0000-0000-000000000001" })
        );
    }
}
