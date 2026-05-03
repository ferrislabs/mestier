use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: Option<String>,
    pub name: Option<String>,
    pub roles: Vec<String>,
}
