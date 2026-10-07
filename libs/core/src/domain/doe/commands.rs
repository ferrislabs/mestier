use chrono::NaiveDate;

use crate::{
    DoeId, DoeModelId, OrganizationId, ProductId, ProjectId,
    domain::doe::{DoeParty, DoeSection, Revision},
};

#[derive(Debug, Clone)]
pub struct CreateDoeCommand {
    /// Authenticated actor. Built by the handler from the request `Identity`.
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
    pub model_id: DoeModelId,
}

/// The editor saves the whole document at once.
#[derive(Debug, Clone)]
pub struct UpdateDoeCommand {
    pub actor: authz::Subject,
    pub id: DoeId,
    pub title: String,
    pub place: Option<String>,
    pub lot: Option<String>,
    pub issued_on: NaiveDate,
    pub author: Option<String>,
    pub revision: Revision,
    pub parties: Vec<DoeParty>,
    pub sections: Vec<DoeSection>,
}

/// Copies a product's technical sheet into a supply the editor can insert.
/// Nothing is persisted.
#[derive(Debug, Clone)]
pub struct DraftSupplyFromProductCommand {
    pub actor: authz::Subject,
    pub doe_id: DoeId,
    pub product_id: ProductId,
}
