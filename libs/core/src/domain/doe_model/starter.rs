use crate::{
    DoeModelSection, DoeModelSectionKind, StarterModel,
    domain::doe::{BlankPageCount, MakerLabel, PhotosPerPage},
};

impl StarterModel {
    pub fn all() -> &'static [StarterModel] {
        &[StarterModel::Landscaping]
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Landscaping => "Paysagiste",
        }
    }

    pub fn description(self) -> Option<&'static str> {
        match self {
            Self::Landscaping => Some("Végétaux, matériaux, mobilier, jeux, photos et plans"),
        }
    }

    pub fn sections(self) -> Vec<DoeModelSection> {
        match self {
            Self::Landscaping => vec![
                section(
                    "Identification des végétaux",
                    true,
                    DoeModelSectionKind::Supplies {
                        maker_label: MakerLabel::Nursery,
                    },
                ),
                section(
                    "Identification des matériaux",
                    true,
                    DoeModelSectionKind::Supplies {
                        maker_label: MakerLabel::Manufacturer,
                    },
                ),
                section(
                    "Identification du mobilier",
                    true,
                    DoeModelSectionKind::Supplies {
                        maker_label: MakerLabel::Manufacturer,
                    },
                ),
                section(
                    "Identification des jeux",
                    false,
                    DoeModelSectionKind::Supplies {
                        maker_label: MakerLabel::Manufacturer,
                    },
                ),
                section(
                    "Photos générales du chantier",
                    true,
                    DoeModelSectionKind::Photos {
                        per_page: PhotosPerPage::Two,
                    },
                ),
                section("Plans divers", true, DoeModelSectionKind::Plans),
                section(
                    "Notes diverses",
                    true,
                    DoeModelSectionKind::BlankPages {
                        count: BlankPageCount::try_from(1).expect("1 is a valid page count"),
                    },
                ),
            ],
        }
    }
}

fn section(title: &str, enabled: bool, kind: DoeModelSectionKind) -> DoeModelSection {
    DoeModelSection {
        title: title.to_owned(),
        enabled,
        kind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn landscaping_has_seven_sections_with_games_disabled() {
        let sections = StarterModel::Landscaping.sections();
        assert_eq!(sections.len(), 7);
        assert_eq!(sections[0].title, "Identification des végétaux");
        assert_eq!(
            sections[0].kind,
            DoeModelSectionKind::Supplies {
                maker_label: MakerLabel::Nursery
            }
        );
        let games = sections
            .iter()
            .find(|s| s.title == "Identification des jeux")
            .unwrap();
        assert!(!games.enabled);
        assert_eq!(sections.iter().filter(|s| s.enabled).count(), 6);
    }

    #[test]
    fn landscaping_is_named_paysagiste() {
        assert_eq!(StarterModel::Landscaping.name(), "Paysagiste");
        assert_eq!(StarterModel::all(), &[StarterModel::Landscaping]);
    }
}
