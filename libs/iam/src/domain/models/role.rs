use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::domain::models::organization::IamOrgId;

/// Identifier of a role as known by the IAM.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IamRoleId(pub String);

impl Display for IamRoleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IamRole {
    pub id: IamRoleId,
    pub organization_id: IamOrgId,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub struct IamCreateRole {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct IamUpdateRole {
    pub name: Option<String>,
    pub description: Option<String>,
}
