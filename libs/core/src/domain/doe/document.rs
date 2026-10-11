use std::collections::HashSet;

use chrono::{NaiveDate, Utc};
use uuid::Uuid;

use crate::domain::{
    doe::{
        Doe, DoeSectionContent,
        render::{DocumentStyle, DoeDocument, NumberedContent, NumberedSection, NumberedSupply},
    },
    doe_model::{DoeStyle, TemplateValues},
};
use crate::{
    OrganizationId, ProjectId,
    domain::doe::{
        BlankPageCount, DoeId, DoeParty, DoeSection, DoeSupply, MakerLabel, PartyRole, Revision,
        SupplierSnapshot,
    },
};

impl DoeDocument {
    pub fn from_doe(doe: &Doe, style: &DoeStyle) -> Self {
        let sections = doe
            .sections
            .iter()
            .filter(|section| section.enabled)
            .enumerate()
            .map(|(index, section)| {
                let number = index as u16 + 1;
                let content = match &section.content {
                    DoeSectionContent::Supplies {
                        maker_label,
                        supplies,
                    } => NumberedContent::Supplies {
                        maker_label: maker_label.french().to_owned(),
                        supplies: supplies
                            .iter()
                            .enumerate()
                            .map(|(position, supply)| NumberedSupply {
                                number: format!("{number}.{}", position + 1),
                                heading: supply
                                    .title
                                    .clone()
                                    .unwrap_or_else(|| supply.designation.clone()),
                                supply: supply.clone(),
                            })
                            .collect(),
                    },
                    DoeSectionContent::Photos { per_page, photos } => NumberedContent::Photos {
                        per_page: *per_page,
                        photos: photos.clone(),
                    },
                    DoeSectionContent::Plans { plans } => NumberedContent::Plans {
                        plans: plans.clone(),
                    },
                    DoeSectionContent::BlankPages { count } => {
                        NumberedContent::BlankPages { count: count.get() }
                    }
                };
                NumberedSection {
                    number,
                    title: section.title.clone(),
                    content,
                }
            })
            .collect();

        let date = doe.issued_on.format("%d/%m/%Y").to_string();
        let values = TemplateValues {
            title: &doe.title,
            place: doe.place.as_deref(),
            lot: doe.lot.as_deref(),
            revision: doe.revision.as_str(),
            date: &date,
        };

        Self {
            title: doe.title.clone(),
            place: doe.place.clone(),
            lot: doe.lot.clone(),
            issued_on: doe.issued_on,
            author: doe.author.clone(),
            revision: doe.revision.as_str().to_owned(),
            parties: doe.parties.clone(),
            sections,
            style: DocumentStyle::from(style),
            header: style.header_text.fill(&values),
            footer: style.footer_text.fill(&values),
            cover_photo_key: doe.cover_photo_key.clone().filter(|_| style.cover_photo),
        }
    }

    /// A made-up DOE laid out with `style`, to preview a style before saving
    /// it. Reads no job data; the only file it can reference is the style's
    /// logo.
    pub fn sample(style: &DoeStyle, issued_on: NaiveDate) -> Self {
        let party = |role: PartyRole, name: &str, address: &str| DoeParty {
            role,
            name: name.to_owned(),
            address: Some(address.to_owned()),
            phone: Some("01 23 45 67 89".to_owned()),
            email: None,
            logo_key: None,
        };
        let supply = |title: &str, designation: &str, specs: &str| DoeSupply {
            id: Uuid::nil(),
            source_product_id: None,
            title: Some(title.to_owned()),
            designation: designation.to_owned(),
            dpgf_ref: Some("EV 3".to_owned()),
            destination: Some("Voir plan de plantation".to_owned()),
            supplier: Some(SupplierSnapshot {
                name: "Pépinières d'exemple".to_owned(),
                address: Some("1 rue des Lilas, 03120 Lapalisse".to_owned()),
                phone: Some("04 70 00 00 00".to_owned()),
                email: None,
            }),
            maker: Some("Pépinières d'exemple".to_owned()),
            specs: Some(specs.to_owned()),
            photos: Vec::new(),
            datasheet_keys: Vec::new(),
        };
        let section = |title: &str, content: DoeSectionContent| DoeSection {
            id: Uuid::nil(),
            title: title.to_owned(),
            enabled: true,
            content,
        };
        let now = Utc::now();
        let doe = Doe {
            id: DoeId(Uuid::nil()),
            organization_id: OrganizationId(Uuid::nil()),
            project_id: ProjectId(Uuid::nil()),
            model_id: None,
            title: "Aménagement d'exemple".to_owned(),
            place: Some("Lapalisse (03120)".to_owned()),
            lot: Some("Lot 2 – Espaces verts".to_owned()),
            issued_on,
            author: Some("Conducteur de travaux".to_owned()),
            revision: Revision(String::from("A")),
            cover_photo_key: None,
            parties: vec![
                party(
                    PartyRole::ProjectOwner,
                    "Commune d'exemple",
                    "Place de la Mairie, 03120 Lapalisse",
                ),
                party(
                    PartyRole::Contractor,
                    "Votre entreprise",
                    "2 route des Jardins, 03120 Lapalisse",
                ),
            ],
            sections: vec![
                section(
                    "Identification des végétaux",
                    DoeSectionContent::Supplies {
                        maker_label: MakerLabel::Nursery,
                        supplies: vec![
                            supply(
                                "Plantation d'arbres",
                                "Arbres",
                                "Acer platanoides T14/16\nAmelanchier lamarckii CP 150/200",
                            ),
                            supply(
                                "Plantation d'arbustes",
                                "Arbustes",
                                "Gaura lindheimeri godet\nPerovskia Blue Spire godet",
                            ),
                        ],
                    },
                ),
                section(
                    "Notes",
                    DoeSectionContent::BlankPages {
                        count: BlankPageCount(1),
                    },
                ),
            ],
            created_at: now,
            updated_at: now,
        };
        Self::from_doe(&doe, style)
    }

    pub fn file_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = Vec::new();
        keys.extend(self.parties.iter().filter_map(|p| p.logo_key.as_deref()));
        keys.extend(self.style.logo_key.as_deref());
        keys.extend(self.cover_photo_key.as_deref());
        for section in &self.sections {
            match &section.content {
                NumberedContent::Supplies { supplies, .. } => {
                    for numbered in supplies {
                        keys.extend(numbered.supply.photos.iter().map(|f| f.key.as_str()));
                        keys.extend(numbered.supply.datasheet_keys.iter().map(String::as_str));
                    }
                }
                NumberedContent::Photos { photos, .. } => {
                    keys.extend(photos.iter().map(|f| f.key.as_str()));
                }
                NumberedContent::Plans { plans } => {
                    keys.extend(plans.iter().map(|f| f.key.as_str()));
                }
                NumberedContent::BlankPages { .. } => {}
            }
        }
        let mut seen = HashSet::new();
        keys.retain(|key| seen.insert(*key));
        keys
    }
}

#[cfg(test)]
mod tests {
    use chrono::{NaiveDate, Utc};
    use uuid::Uuid;

    use super::*;
    use crate::{
        OrganizationId, ProjectId,
        domain::doe::{
            BlankPageCount, DoeFile, DoeId, DoeParty, DoeSection, DoeSupply, MakerLabel, PartyRole,
            PhotosPerPage, Revision,
        },
        domain::doe_model::{DoeFont, DoeLabel, DoeTemplate},
    };

    fn supply(designation: &str, title: Option<&str>) -> DoeSupply {
        DoeSupply {
            id: Uuid::new_v4(),
            source_product_id: None,
            title: title.map(str::to_owned),
            designation: designation.to_owned(),
            dpgf_ref: None,
            destination: None,
            supplier: None,
            maker: None,
            specs: None,
            photos: Vec::new(),
            datasheet_keys: Vec::new(),
        }
    }

    fn file(key: &str) -> DoeFile {
        DoeFile {
            key: key.to_owned(),
            caption: None,
        }
    }

    fn section(title: &str, enabled: bool, content: DoeSectionContent) -> DoeSection {
        DoeSection {
            id: Uuid::new_v4(),
            title: title.to_owned(),
            enabled,
            content,
        }
    }

    fn doe(sections: Vec<DoeSection>, parties: Vec<DoeParty>) -> Doe {
        let now = Utc::now();
        Doe {
            id: DoeId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            project_id: ProjectId(Uuid::new_v4()),
            model_id: None,
            title: "Avenue du 8 mai".to_owned(),
            place: None,
            lot: None,
            issued_on: NaiveDate::from_ymd_opt(2026, 10, 7).unwrap(),
            author: None,
            revision: Revision::try_from("A".to_owned()).unwrap(),
            cover_photo_key: None,
            parties,
            sections,
            created_at: now,
            updated_at: now,
        }
    }

    fn supplies(items: Vec<DoeSupply>) -> DoeSectionContent {
        DoeSectionContent::Supplies {
            maker_label: MakerLabel::Nursery,
            supplies: items,
        }
    }

    fn style() -> DoeStyle {
        DoeStyle::default()
    }

    #[test]
    fn spec_doe_4_disabled_sections_are_dropped_and_numbering_is_contiguous() {
        let doe = doe(
            vec![
                section(
                    "Végétaux",
                    true,
                    supplies(vec![
                        supply("Arbres", Some("Plantation d'arbres")),
                        supply("Vivaces", None),
                    ]),
                ),
                section(
                    "Matériaux",
                    true,
                    supplies(vec![supply("Pouzzolane", None)]),
                ),
                section("Mobilier", true, supplies(vec![])),
                section("Jeux", false, supplies(vec![supply("Toboggan", None)])),
                section(
                    "Photos",
                    true,
                    DoeSectionContent::Photos {
                        per_page: PhotosPerPage::Two,
                        photos: vec![],
                    },
                ),
            ],
            vec![],
        );

        let document = DoeDocument::from_doe(&doe, &style());

        let numbers: Vec<(u16, &str)> = document
            .sections
            .iter()
            .map(|s| (s.number, s.title.as_str()))
            .collect();
        assert_eq!(
            numbers,
            vec![
                (1, "Végétaux"),
                (2, "Matériaux"),
                (3, "Mobilier"),
                (4, "Photos")
            ]
        );

        let NumberedContent::Supplies {
            maker_label,
            supplies,
        } = &document.sections[0].content
        else {
            panic!("expected supplies");
        };
        assert_eq!(maker_label, "Pépinière");
        let rendered: Vec<(&str, &str)> = supplies
            .iter()
            .map(|s| (s.number.as_str(), s.heading.as_str()))
            .collect();
        assert_eq!(
            rendered,
            vec![("1.1", "Plantation d'arbres"), ("1.2", "Vivaces")]
        );
    }

    #[test]
    fn header_and_style_are_carried_over() {
        let mut doe = doe(vec![], vec![]);
        doe.place = Some("Lapalisse".to_owned());
        let style = DoeStyle {
            logo_key: Some("logo".to_owned()),
            heading_font: DoeFont::SourceSerif,
            contents_title: DoeLabel::try_from("Table des matières".to_owned()).unwrap(),
            ..DoeStyle::default()
        };

        let document = DoeDocument::from_doe(&doe, &style);

        assert_eq!(document.title, "Avenue du 8 mai");
        assert_eq!(document.place.as_deref(), Some("Lapalisse"));
        assert_eq!(document.revision, "A");
        assert_eq!(document.style.primary, style.primary);
        assert_eq!(document.style.logo_key.as_deref(), Some("logo"));
        assert_eq!(document.style.heading_font, "Source Serif 4");
        assert_eq!(document.style.body_font, "Inter");
        assert_eq!(document.style.contents_title, "Table des matières");
    }

    #[test]
    fn spec_style_1_default_style_prints_the_header_and_footer_of_before() {
        let doe = doe(vec![], vec![]);

        let document = DoeDocument::from_doe(&doe, &style());

        assert_eq!(document.header, "DOE – Avenue du 8 mai");
        assert_eq!(document.footer, "07/10/2026");
        assert_eq!(document.style.primary.as_str(), "#0f3d36");
        assert_eq!(document.style.on_primary.as_str(), "#ffffff");
        assert_eq!(document.style.heading_font, "Inter");
    }

    #[test]
    fn spec_style_5_templates_are_filled_and_missing_values_print_nothing() {
        let mut doe = doe(vec![], vec![]);
        doe.lot = Some("Lot 2".to_owned());
        let style = DoeStyle {
            header_text: DoeTemplate::try_from("{titre} – {lot}".to_owned()).unwrap(),
            footer_text: DoeTemplate::try_from("Treyve Paysages {lieu}".to_owned()).unwrap(),
            ..DoeStyle::default()
        };

        let document = DoeDocument::from_doe(&doe, &style);

        assert_eq!(document.header, "Avenue du 8 mai – Lot 2");
        assert_eq!(document.footer, "Treyve Paysages");
    }

    #[test]
    fn spec_style_6_cover_photo_is_printed_only_when_the_style_asks() {
        let mut doe = doe(vec![], vec![]);
        doe.cover_photo_key = Some("cover".to_owned());

        let shown = DoeDocument::from_doe(&doe, &style());
        let hidden = DoeDocument::from_doe(
            &doe,
            &DoeStyle {
                cover_photo: false,
                ..DoeStyle::default()
            },
        );

        assert_eq!(shown.cover_photo_key.as_deref(), Some("cover"));
        assert!(shown.file_keys().contains(&"cover"));
        assert_eq!(hidden.cover_photo_key, None);
        assert!(!hidden.file_keys().contains(&"cover"));
    }

    #[test]
    fn spec_style_7_sample_references_no_file_but_the_logo() {
        let style = DoeStyle {
            logo_key: Some("logos/treyve".to_owned()),
            ..DoeStyle::default()
        };

        let document = DoeDocument::sample(&style, NaiveDate::from_ymd_opt(2026, 10, 11).unwrap());

        assert_eq!(document.file_keys(), vec!["logos/treyve"]);
        assert_eq!(document.sections.len(), 2);
        assert_eq!(document.header, "DOE – Aménagement d'exemple");
    }

    #[test]
    fn blank_pages_keep_their_count() {
        let doe = doe(
            vec![section(
                "Notes",
                true,
                DoeSectionContent::BlankPages {
                    count: BlankPageCount::try_from(3).unwrap(),
                },
            )],
            vec![],
        );
        let document = DoeDocument::from_doe(&doe, &style());
        assert_eq!(
            document.sections[0].content,
            NumberedContent::BlankPages { count: 3 }
        );
    }

    #[test]
    fn file_keys_are_deduplicated_in_stable_order_and_skip_disabled_sections() {
        let mut with_files = supply("Arbres", None);
        with_files.photos = vec![file("p1"), file("shared")];
        with_files.datasheet_keys = vec!["d1".to_owned(), "shared".to_owned()];
        let mut hidden = supply("Toboggan", None);
        hidden.photos = vec![file("hidden")];
        let party = DoeParty {
            role: PartyRole::Contractor,
            name: "Treyve".to_owned(),
            address: None,
            phone: None,
            email: None,
            logo_key: Some("party-logo".to_owned()),
        };
        let doe = doe(
            vec![
                section("Végétaux", true, supplies(vec![with_files])),
                section("Jeux", false, supplies(vec![hidden])),
                section(
                    "Photos",
                    true,
                    DoeSectionContent::Photos {
                        per_page: PhotosPerPage::One,
                        photos: vec![file("ph1"), file("p1")],
                    },
                ),
                section(
                    "Plans",
                    true,
                    DoeSectionContent::Plans {
                        plans: vec![file("plan1")],
                    },
                ),
            ],
            vec![party],
        );
        let style = DoeStyle {
            logo_key: Some("logo".to_owned()),
            ..DoeStyle::default()
        };

        let document = DoeDocument::from_doe(&doe, &style);

        assert_eq!(
            document.file_keys(),
            vec!["party-logo", "logo", "p1", "shared", "d1", "ph1", "plan1"]
        );
    }
}
