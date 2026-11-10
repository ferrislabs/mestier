use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub roles: Vec<String>,
    #[serde(default)]
    pub delegated: bool,
    /// The FerrisKey realm roles of the token. Kept apart from `roles`, which
    /// feeds the policy subject: there, `mestier:admin` would pass every check
    /// of every organization. Read only by checks that name the role.
    #[serde(default)]
    pub realm_roles: Vec<String>,
}
