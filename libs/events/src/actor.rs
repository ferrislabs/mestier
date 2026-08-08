use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Who caused an event.
///
/// `User` and `Client` mirror the two variants of `auth::Identity`: a FerrisKey
/// client is an integration acting on its own behalf. Telling the two apart
/// matters the day an automation has to ignore what another automation caused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorKind {
    User,
    Client,
    System,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Actor {
    pub kind: ActorKind,
    /// `None` only for [`ActorKind::System`]: every human or integration action
    /// carries the identity that performed it.
    pub id: Option<Uuid>,
}

impl Actor {
    pub fn user(id: Uuid) -> Self {
        Self {
            kind: ActorKind::User,
            id: Some(id),
        }
    }

    pub fn client(id: Uuid) -> Self {
        Self {
            kind: ActorKind::Client,
            id: Some(id),
        }
    }

    /// Scheduled jobs, migrations, and anything the product does on its own.
    pub fn system() -> Self {
        Self {
            kind: ActorKind::System,
            id: None,
        }
    }
}
