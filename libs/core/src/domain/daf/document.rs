use std::collections::HashSet;

use chrono::NaiveDate;

use crate::{
    DocumentStyle, DoeStyle,
    domain::daf::{
        Daf, DafDossier,
        render::{DafDocument, DafSheet, DafSummaryPage, DafSummaryRow},
    },
};

impl DafDocument {
    /// One DAF alone: its sheet and attachments, no summary.
    pub fn single(dossier: &DafDossier, daf: &Daf, style: &DoeStyle) -> Self {
        Self {
            title: dossier.title.clone(),
            issuer: dossier.issuer.clone(),
            summary: None,
            sheets: vec![sheet(dossier, daf)],
            style: DocumentStyle::from(style),
        }
    }

    /// The whole dossier: the summary page, then every DAF by number.
    pub fn dossier(
        dossier: &DafDossier,
        dafs: &[Daf],
        style: &DoeStyle,
        printed_on: NaiveDate,
    ) -> Self {
        let mut sorted: Vec<&Daf> = dafs.iter().collect();
        sorted.sort_by_key(|daf| daf.number);
        Self {
            title: dossier.title.clone(),
            issuer: dossier.issuer.clone(),
            summary: Some(DafSummaryPage {
                printed_on,
                rows: sorted.iter().map(|daf| summary_row(daf)).collect(),
            }),
            sheets: sorted.iter().map(|daf| sheet(dossier, daf)).collect(),
            style: DocumentStyle::from(style),
        }
    }

    /// Storage keys of every file the document places: logo, decorations,
    /// photos and attachments, each once.
    pub fn file_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = Vec::new();
        keys.extend(self.style.logo_key.as_deref());
        keys.extend(self.style.decoration_image_keys());
        for sheet in &self.sheets {
            keys.extend(sheet.photo_key.as_deref());
            keys.extend(sheet.attachments.iter().map(|a| a.key.as_str()));
        }
        let mut seen = HashSet::new();
        keys.retain(|key| seen.insert(*key));
        keys
    }
}

fn sheet(dossier: &DafDossier, daf: &Daf) -> DafSheet {
    DafSheet {
        label: daf.number.label(),
        designation: daf.designation.as_str().to_owned(),
        price_ref: daf.price_ref.clone(),
        supplier: daf.supplier.clone(),
        installer: daf.installer.clone(),
        specs: daf.specs.clone(),
        observations: daf.observations.clone(),
        photo_key: daf.photo_key.clone(),
        attachments: daf.attachments.clone(),
        project_owner: daf
            .project_owner
            .clone()
            .or_else(|| dossier.project_owner.clone()),
        project_manager: daf
            .project_manager
            .clone()
            .or_else(|| dossier.project_manager.clone()),
        issued_on: daf.issued_on,
    }
}

fn summary_row(daf: &Daf) -> DafSummaryRow {
    DafSummaryRow {
        label: daf.number.label(),
        designation: daf.designation.as_str().to_owned(),
        price_ref: daf.price_ref.clone(),
        status: daf.status,
        status_label: daf.status.french().to_owned(),
        sent_on: daf.sent_on,
        issued_on: daf.issued_on,
        returned_on: daf.returned_on,
        review_note: daf.review_note.clone(),
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use uuid::Uuid;

    use super::*;
    use crate::{
        DafAttachment, DafDesignation, DafId, DafNumber, DafStatus, OrganizationId, ProjectId,
    };

    fn dossier() -> DafDossier {
        DafDossier {
            organization_id: OrganizationId(Uuid::nil()),
            project_id: ProjectId(Uuid::nil()),
            title: "Parc Montgroux".to_owned(),
            issuer: "Treyve Paysages".to_owned(),
            project_owner: Some("Commune de Cébazat".to_owned()),
            project_manager: Some("Atelier Casa".to_owned()),
            updated_at: None,
        }
    }

    fn daf(number: u16, designation: &str) -> Daf {
        Daf {
            id: DafId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::nil()),
            project_id: ProjectId(Uuid::nil()),
            number: DafNumber::try_from(number).unwrap(),
            designation: DafDesignation::try_from(designation.to_owned()).unwrap(),
            price_ref: None,
            source_product_id: None,
            supplier: None,
            installer: None,
            specs: None,
            observations: None,
            photo_key: None,
            attachments: Vec::new(),
            project_owner: None,
            project_manager: None,
            status: DafStatus::Pending,
            sent_on: None,
            issued_on: NaiveDate::from_ymd_opt(2026, 11, 18).unwrap(),
            returned_on: None,
            review_note: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    /// @spec-daf-5
    #[test]
    fn a_daf_without_its_own_parties_prints_the_dossiers() {
        let mut own = daf(2, "Ganivelle");
        own.project_manager = Some("BET Paysage".to_owned());

        let document = DafDocument::dossier(
            &dossier(),
            &[own, daf(1, "Geotextile")],
            &DoeStyle::default(),
            NaiveDate::from_ymd_opt(2026, 11, 18).unwrap(),
        );

        assert_eq!(
            document.sheets[0].project_manager.as_deref(),
            Some("Atelier Casa")
        );
        assert_eq!(
            document.sheets[0].project_owner.as_deref(),
            Some("Commune de Cébazat")
        );
        assert_eq!(
            document.sheets[1].project_manager.as_deref(),
            Some("BET Paysage")
        );
    }

    #[test]
    fn the_dossier_is_ordered_by_number_and_labelled() {
        let mut accepted = daf(1, "Geotextile");
        accepted.status = DafStatus::Accepted;
        let document = DafDocument::dossier(
            &dossier(),
            &[daf(3, "Ganivelle"), accepted],
            &DoeStyle::default(),
            NaiveDate::from_ymd_opt(2026, 11, 18).unwrap(),
        );

        let rows = &document.summary.as_ref().unwrap().rows;
        assert_eq!(rows[0].label, "D.A.F n°01");
        assert_eq!(rows[0].status_label, "Acceptée");
        assert_eq!(rows[1].label, "D.A.F n°03");
        assert_eq!(document.sheets[1].designation, "Ganivelle");
    }

    #[test]
    fn a_single_daf_has_no_summary() {
        let document =
            DafDocument::single(&dossier(), &daf(9, "Jeux - Table"), &DoeStyle::default());
        assert!(document.summary.is_none());
        assert_eq!(document.sheets[0].label, "D.A.F n°09");
    }

    #[test]
    fn file_keys_are_listed_once() {
        let mut first = daf(1, "Geotextile");
        first.photo_key = Some("photo".to_owned());
        first.attachments = vec![DafAttachment {
            key: "sheet".to_owned(),
            name: "Fiche".to_owned(),
        }];
        let mut second = daf(2, "Ganivelle");
        second.photo_key = Some("photo".to_owned());

        let document = DafDocument::dossier(
            &dossier(),
            &[first, second],
            &DoeStyle::default(),
            NaiveDate::from_ymd_opt(2026, 11, 18).unwrap(),
        );

        assert_eq!(document.file_keys(), vec!["photo", "sheet"]);
    }
}
