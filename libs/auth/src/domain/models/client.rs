use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Client {
    pub id: String,
    pub client_id: String,
    pub roles: Vec<String>,
    pub scopes: Vec<String>,
}
