use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Identifier of a user as known by the IAM. Opaque on purpose: providers
/// use heterogeneous formats (UUID, slug, numeric, ...).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IamUserId(pub String);

impl Display for IamUserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// A user as represented in the IAM. Mirrors only the fields Mestier cares
/// about — provider-specific attributes stay inside the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IamUser {
    pub id: IamUserId,
    pub email: String,
    pub username: String,
    pub name: Option<String>,
    pub email_verified: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct IamCreateUser {
    pub email: String,
    pub username: String,
    pub name: Option<String>,
    /// When `true`, the IAM is asked to send the credential-setup e-mail.
    /// Adapters that don't support this should ignore it.
    pub send_invite_email: bool,
}

#[derive(Debug, Clone, Default)]
pub struct IamUpdateUser {
    pub email: Option<String>,
    pub username: Option<String>,
    pub name: Option<String>,
    pub enabled: Option<bool>,
}
