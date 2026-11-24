use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{DafAttachment, DafStatus, LibraryPhotoId, ProductId, SupplierSnapshot};
use serde::Deserialize;
use utoipa::ToSchema;

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDafDossierRequest {
    pub title: String,
    pub issuer: String,
    #[serde(default)]
    pub project_owner: Option<String>,
    #[serde(default)]
    pub project_manager: Option<String>,
}

/// One of `designation` and `product_id` is required. With a product, its
/// technical sheet is copied and its name is the default designation.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDafRequest {
    #[serde(default)]
    pub designation: Option<String>,
    #[serde(default)]
    pub product_id: Option<ProductId>,
}

/// The whole DAF, as the editor saves it.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateDafRequest {
    /// 1 to 999, unique in the project.
    pub number: u16,
    pub designation: String,
    #[serde(default)]
    pub price_ref: Option<String>,
    #[serde(default)]
    pub supplier: Option<SupplierSnapshot>,
    #[serde(default)]
    pub installer: Option<String>,
    #[serde(default)]
    pub specs: Option<String>,
    #[serde(default)]
    pub observations: Option<String>,
    #[serde(default)]
    pub photo_key: Option<String>,
    #[serde(default)]
    pub attachments: Vec<DafAttachment>,
    /// Absent or null prints the dossier's.
    #[serde(default)]
    pub project_owner: Option<String>,
    /// Absent or null prints the dossier's.
    #[serde(default)]
    pub project_manager: Option<String>,
    pub status: DafStatus,
    #[serde(default)]
    pub sent_on: Option<NaiveDate>,
    pub issued_on: NaiveDate,
    #[serde(default)]
    pub returned_on: Option<NaiveDate>,
    #[serde(default)]
    pub review_note: Option<String>,
    /// The `updated_at` read before editing. When it is no longer current
    /// the save is refused with 409 instead of overwriting another change.
    #[serde(default)]
    pub expected_updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlaceDafPhotoRequest {
    /// A photo of the organization's library. It is copied: deleting it from
    /// the library later keeps the DAF's image.
    pub photo_id: LibraryPhotoId,
}
