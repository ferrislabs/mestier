//! What a DAF renderer is handed. Numbers are labelled, statuses translated
//! and the dossier's parties resolved before the renderer is called, so it
//! only lays out.

use chrono::NaiveDate;
use serde::Serialize;

use crate::{
    DocumentStyle, SupplierSnapshot,
    domain::daf::{DafAttachment, DafStatus},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DafDocument {
    /// The dossier's title, printed in every page's banner.
    pub title: String,
    pub issuer: String,
    /// `Some` prints the summary page first.
    pub summary: Option<DafSummaryPage>,
    pub sheets: Vec<DafSheet>,
    pub style: DocumentStyle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DafSummaryPage {
    /// The banner's "Date de diffusion": the day the dossier is printed.
    pub printed_on: NaiveDate,
    pub rows: Vec<DafSummaryRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DafSummaryRow {
    /// "D.A.F n°01".
    pub label: String,
    pub designation: String,
    pub price_ref: Option<String>,
    pub status: DafStatus,
    /// "En attente", "Acceptée"…
    pub status_label: String,
    pub sent_on: Option<NaiveDate>,
    pub issued_on: NaiveDate,
    pub returned_on: Option<NaiveDate>,
    pub review_note: Option<String>,
}

/// One sheet, followed by its attachments page by page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DafSheet {
    /// "D.A.F n°09".
    pub label: String,
    pub designation: String,
    pub price_ref: Option<String>,
    pub supplier: Option<SupplierSnapshot>,
    pub installer: Option<String>,
    pub specs: Option<String>,
    pub observations: Option<String>,
    pub photo_key: Option<String>,
    pub attachments: Vec<DafAttachment>,
    /// The DAF's own, else the dossier's.
    pub project_owner: Option<String>,
    /// The DAF's own, else the dossier's.
    pub project_manager: Option<String>,
    pub issued_on: NaiveDate,
}
