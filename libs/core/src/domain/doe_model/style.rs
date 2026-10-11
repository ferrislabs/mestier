//! How a DOE looks: colors, fonts, cover, running header and footer, the
//! fixed titles. Set per organization, optionally replaced by a model.
//!
//! Every field has a default that reproduces the document as it was printed
//! before styles existed, and a stored style missing a field reads as that
//! default, so adding a field never breaks a saved style.
//!
//! Spec: `docs/chantiers/doe-style.md`.

use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::doe_model::HexColor;

/// Fonts bundled with the renderer. A closed list: a font is a file the
/// renderer embeds, not a name it looks up.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DoeFont {
    #[default]
    Inter,
    Montserrat,
    SourceSerif,
    Barlow,
}

impl DoeFont {
    /// The family name the font files declare.
    pub fn family(self) -> &'static str {
        match self {
            Self::Inter => "Inter",
            Self::Montserrat => "Montserrat",
            Self::SourceSerif => "Source Serif 4",
            Self::Barlow => "Barlow",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LogoPosition {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum LogoSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// A fixed title of the document ("SOMMAIRE"), renamed. Never blank.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
pub struct DoeLabel(String);

impl DoeLabel {
    pub const MAX_CHARS: usize = 120;

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn fixed(value: &str) -> Self {
        Self(value.to_owned())
    }
}

impl TryFrom<String> for DoeLabel {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        let chars = trimmed.chars().count();
        if chars == 0 || chars > Self::MAX_CHARS {
            return Err(CoreError::Conflict(format!(
                "a document title must be 1 to {} characters",
                Self::MAX_CHARS
            )));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl From<DoeLabel> for String {
    fn from(value: DoeLabel) -> Self {
        value.0
    }
}

/// What a header or footer template may refer to.
pub struct TemplateValues<'a> {
    pub title: &'a str,
    pub place: Option<&'a str>,
    pub lot: Option<&'a str>,
    pub revision: &'a str,
    /// Already formatted, `dd/mm/yyyy`.
    pub date: &'a str,
}

/// Header or footer text with `{titre}`, `{lieu}`, `{lot}`, `{indice}` and
/// `{date}`. Checked when saved, so an export never meets an unknown name.
/// Empty prints nothing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
pub struct DoeTemplate(String);

impl DoeTemplate {
    pub const MAX_CHARS: usize = 200;
    pub const VARIABLES: [&'static str; 5] = ["titre", "lieu", "lot", "indice", "date"];

    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn fixed(value: &str) -> Self {
        Self(value.to_owned())
    }

    /// The text with every variable replaced; a missing value prints
    /// nothing rather than the variable's name.
    pub fn fill(&self, values: &TemplateValues<'_>) -> String {
        let mut out = String::with_capacity(self.0.len());
        let mut rest = self.0.as_str();
        while let Some(start) = rest.find('{') {
            out.push_str(&rest[..start]);
            let after = &rest[start + 1..];
            // `try_from` guarantees a known name and a closing brace.
            let end = after.find('}').unwrap_or(after.len());
            out.push_str(match &after[..end] {
                "titre" => values.title,
                "lieu" => values.place.unwrap_or_default(),
                "lot" => values.lot.unwrap_or_default(),
                "indice" => values.revision,
                "date" => values.date,
                _ => "",
            });
            rest = after.get(end + 1..).unwrap_or_default();
        }
        out.push_str(rest);
        out.trim().to_owned()
    }
}

impl TryFrom<String> for DoeTemplate {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.chars().count() > Self::MAX_CHARS {
            return Err(CoreError::Conflict(format!(
                "a header or footer is at most {} characters",
                Self::MAX_CHARS
            )));
        }
        let mut rest = trimmed;
        while let Some(start) = rest.find('{') {
            let after = &rest[start + 1..];
            let Some(end) = after.find('}') else {
                return Err(CoreError::Conflict(
                    "a `{` in a header or footer must open a variable such as {titre}".to_owned(),
                ));
            };
            let name = &after[..end];
            if !Self::VARIABLES.contains(&name) {
                return Err(CoreError::Conflict(format!(
                    "unknown variable {{{name}}}: use {}",
                    Self::VARIABLES.map(|v| format!("{{{v}}}")).join(", ")
                )));
            }
            rest = &after[end + 1..];
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl From<DoeTemplate> for String {
    fn from(value: DoeTemplate) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(default)]
pub struct DoeStyle {
    /// Bands, table headers, the cover banner.
    pub primary: HexColor,
    /// Supply headings and party roles.
    pub secondary: HexColor,
    /// Text written on `primary`.
    pub on_primary: HexColor,
    pub heading_font: DoeFont,
    pub body_font: DoeFont,
    pub logo_key: Option<String>,
    pub cover_logo_position: LogoPosition,
    pub cover_logo_size: LogoSize,
    /// Prints the DOE's cover photo, when it has one.
    pub cover_photo: bool,
    /// Repeats the logo in the running header.
    pub header_logo: bool,
    pub header_text: DoeTemplate,
    pub footer_text: DoeTemplate,
    pub cover_title: DoeLabel,
    pub parties_title: DoeLabel,
    pub contents_title: DoeLabel,
    pub supply_table_title: DoeLabel,
}

impl Default for DoeStyle {
    fn default() -> Self {
        Self {
            primary: HexColor::mestier_default(),
            secondary: HexColor::mestier_default(),
            on_primary: HexColor::white(),
            heading_font: DoeFont::Inter,
            body_font: DoeFont::Inter,
            logo_key: None,
            cover_logo_position: LogoPosition::Left,
            cover_logo_size: LogoSize::Medium,
            cover_photo: true,
            header_logo: false,
            header_text: DoeTemplate::fixed("DOE – {titre}"),
            footer_text: DoeTemplate::fixed("{date}"),
            cover_title: DoeLabel::fixed("DOSSIER DES OUVRAGES EXÉCUTÉS (D.O.E.)"),
            parties_title: DoeLabel::fixed("INTERVENANTS"),
            contents_title: DoeLabel::fixed("SOMMAIRE"),
            supply_table_title: DoeLabel::fixed("Identification de la fourniture"),
        }
    }
}

impl DoeStyle {
    /// Blank logo keys mean no logo.
    pub fn cleaned(mut self) -> Self {
        self.logo_key = self
            .logo_key
            .map(|key| key.trim().to_owned())
            .filter(|key| !key.is_empty());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values() -> TemplateValues<'static> {
        TemplateValues {
            title: "Avenue du 8 mai",
            place: Some("Lapalisse"),
            lot: None,
            revision: "B",
            date: "14/11/2025",
        }
    }

    fn template(raw: &str) -> Result<DoeTemplate, CoreError> {
        DoeTemplate::try_from(raw.to_owned())
    }

    #[test]
    fn default_style_is_the_document_before_styles() {
        let style = DoeStyle::default();
        assert_eq!(style.primary.as_str(), "#0f3d36");
        assert_eq!(style.secondary.as_str(), "#0f3d36");
        assert_eq!(style.on_primary.as_str(), "#ffffff");
        assert_eq!(style.heading_font.family(), "Inter");
        assert_eq!(style.body_font.family(), "Inter");
        assert_eq!(style.header_text.fill(&values()), "DOE – Avenue du 8 mai");
        assert_eq!(style.footer_text.fill(&values()), "14/11/2025");
        assert_eq!(
            style.cover_title.as_str(),
            "DOSSIER DES OUVRAGES EXÉCUTÉS (D.O.E.)"
        );
        assert_eq!(style.parties_title.as_str(), "INTERVENANTS");
        assert_eq!(style.contents_title.as_str(), "SOMMAIRE");
        assert_eq!(
            style.supply_table_title.as_str(),
            "Identification de la fourniture"
        );
    }

    #[test]
    fn missing_fields_read_as_defaults() {
        let style: DoeStyle = serde_json::from_str(r##"{"primary":"#123456"}"##).unwrap();
        assert_eq!(style.primary.as_str(), "#123456");
        assert_eq!(style.contents_title, DoeStyle::default().contents_title);
    }

    #[test]
    fn style_round_trips() {
        let style = DoeStyle {
            heading_font: DoeFont::SourceSerif,
            cover_logo_position: LogoPosition::Center,
            ..DoeStyle::default()
        };
        let json = serde_json::to_string(&style).unwrap();
        assert!(json.contains(r#""heading_font":"source_serif""#));
        assert_eq!(serde_json::from_str::<DoeStyle>(&json).unwrap(), style);
    }

    #[test]
    fn invalid_values_are_refused_when_read() {
        for bad in [
            r##"{"primary":"#12345"}"##,
            r#"{"body_font":"comic_sans"}"#,
            r#"{"contents_title":"  "}"#,
            r#"{"header_text":"{client}"}"#,
        ] {
            assert!(serde_json::from_str::<DoeStyle>(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn template_fills_every_variable() {
        let filled = template("{titre} – {lieu} – {lot} – Indice {indice} – {date}")
            .unwrap()
            .fill(&values());
        assert_eq!(
            filled,
            "Avenue du 8 mai – Lapalisse –  – Indice B – 14/11/2025"
        );
    }

    #[test]
    fn template_with_missing_value_prints_nothing_for_it() {
        assert_eq!(template("{lot}").unwrap().fill(&values()), "");
    }

    #[test]
    fn template_refuses_unknown_or_unclosed_variables() {
        assert!(template("{client}").is_err());
        assert!(template("Lot {lot").is_err());
        assert!(template(&"x".repeat(201)).is_err());
        assert!(template("").is_ok());
        assert!(template("Treyve Paysages } SIRET 123").is_ok());
    }

    #[test]
    fn label_is_trimmed_and_never_blank() {
        assert_eq!(
            DoeLabel::try_from(" SOMMAIRE ".to_owned())
                .unwrap()
                .as_str(),
            "SOMMAIRE"
        );
        assert!(DoeLabel::try_from("   ".to_owned()).is_err());
        assert!(DoeLabel::try_from("x".repeat(121)).is_err());
    }
}
