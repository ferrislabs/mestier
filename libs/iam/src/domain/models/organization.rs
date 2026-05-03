use std::fmt::Display;

use serde::{Deserialize, Serialize};

/// Identifier of an organization as known by the IAM. Adapters map this
/// to whatever the provider uses (group id, sub-realm name, ...).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct IamOrgId(pub String);

impl Display for IamOrgId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IamOrganization {
    pub id: IamOrgId,
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone)]
pub struct IamCreateOrganization {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Default)]
pub struct IamUpdateOrganization {
    pub name: Option<String>,
    pub slug: Option<String>,
}
