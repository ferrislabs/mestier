use std::collections::HashSet;

use crate::domain::{
    doe::{
        Doe, DoeSectionContent,
        render::{DoeDocument, NumberedContent, NumberedSection, NumberedSupply},
    },
    doe_model::DoeSettings,
};

impl DoeDocument {
    pub fn from_doe(doe: &Doe, settings: &DoeSettings) -> Self {
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

        Self {
            title: doe.title.clone(),
            place: doe.place.clone(),
            lot: doe.lot.clone(),
            issued_on: doe.issued_on,
            author: doe.author.clone(),
            revision: doe.revision.as_str().to_owned(),
            parties: doe.parties.clone(),
            sections,
            accent: settings.accent.clone(),
            logo_key: settings.logo_key.clone(),
        }
    }

    pub fn file_keys(&self) -> Vec<&str> {
        let mut keys: Vec<&str> = Vec::new();
        keys.extend(self.parties.iter().filter_map(|p| p.logo_key.as_deref()));
        keys.extend(self.logo_key.as_deref());
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
            title: "Avenue du 8 mai".to_owned(),
            place: None,
            lot: None,
            issued_on: NaiveDate::from_ymd_opt(2026, 10, 7).unwrap(),
            author: None,
            revision: Revision::try_from("A".to_owned()).unwrap(),
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

    fn settings(doe: &Doe) -> DoeSettings {
        DoeSettings::default_for(doe.organization_id)
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

        let document = DoeDocument::from_doe(&doe, &settings(&doe));

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
    fn header_and_settings_are_carried_over() {
        let mut doe = doe(vec![], vec![]);
        doe.place = Some("Lapalisse".to_owned());
        let mut settings = settings(&doe);
        settings.logo_key = Some("logo".to_owned());

        let document = DoeDocument::from_doe(&doe, &settings);

        assert_eq!(document.title, "Avenue du 8 mai");
        assert_eq!(document.place.as_deref(), Some("Lapalisse"));
        assert_eq!(document.revision, "A");
        assert_eq!(document.accent, settings.accent);
        assert_eq!(document.logo_key.as_deref(), Some("logo"));
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
        let document = DoeDocument::from_doe(&doe, &settings(&doe));
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
        let mut settings = settings(&doe);
        settings.logo_key = Some("logo".to_owned());

        let document = DoeDocument::from_doe(&doe, &settings);

        assert_eq!(
            document.file_keys(),
            vec!["party-logo", "logo", "p1", "shared", "d1", "ph1", "plan1"]
        );
    }
}
