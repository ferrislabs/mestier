use chrono::{DateTime, NaiveDate, Utc};

use crate::{
    DafId, LibraryPhotoId, OrganizationId, ProductId, ProjectId, SupplierSnapshot,
    domain::daf::{DafAttachment, DafDesignation, DafNumber, DafStatus},
};

/// Saves the dossier of a project, creating it on first save.
#[derive(Debug, Clone)]
pub struct UpdateDafDossierCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
    pub title: String,
    pub issuer: String,
    pub project_owner: Option<String>,
    pub project_manager: Option<String>,
}

/// Creates the next DAF of a project. With a product, its technical sheet is
/// copied in: supplier, specs, first photo, datasheets as attachments.
#[derive(Debug, Clone)]
pub struct CreateDafCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
    /// `None` takes the designation from the product. One of the two is
    /// required.
    pub designation: Option<DafDesignation>,
    pub product_id: Option<ProductId>,
}

/// The editor saves the whole DAF at once.
#[derive(Debug, Clone)]
pub struct UpdateDafCommand {
    pub actor: authz::Subject,
    pub id: DafId,
    pub number: DafNumber,
    pub designation: DafDesignation,
    pub price_ref: Option<String>,
    pub supplier: Option<SupplierSnapshot>,
    pub installer: Option<String>,
    pub specs: Option<String>,
    pub observations: Option<String>,
    pub photo_key: Option<String>,
    pub attachments: Vec<DafAttachment>,
    pub project_owner: Option<String>,
    pub project_manager: Option<String>,
    pub status: DafStatus,
    pub sent_on: Option<NaiveDate>,
    pub issued_on: NaiveDate,
    pub returned_on: Option<NaiveDate>,
    pub review_note: Option<String>,
    /// The `updated_at` the caller last read; refused with a conflict when
    /// someone else saved in between.
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct DeleteDafCommand {
    pub actor: authz::Subject,
    pub id: DafId,
}

/// Copies a photo of the organization's library and makes it the DAF's
/// image. The copy outlives the library photo.
#[derive(Debug, Clone)]
pub struct PlaceDafPhotoCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: DafId,
    pub photo_id: LibraryPhotoId,
}
