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

/// A length in millimeters on the A4 page, never negative or beyond the page.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "f32", into = "f32")]
pub struct Millimeters(u32);

impl Millimeters {
    pub const PAGE_WIDTH: f32 = 210.0;
    pub const PAGE_HEIGHT: f32 = 297.0;

    pub fn get(self) -> f32 {
        self.0 as f32 / 100.0
    }
}

impl TryFrom<f32> for Millimeters {
    type Error = CoreError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !value.is_finite() || !(0.0..=Millimeters::PAGE_HEIGHT).contains(&value) {
            return Err(CoreError::Conflict(
                "a position or size must be between 0 and 297 mm".to_owned(),
            ));
        }
        Ok(Self((value * 100.0).round() as u32))
    }
}

impl From<Millimeters> for f32 {
    fn from(value: Millimeters) -> Self {
        value.get()
    }
}

/// How opaque an element is, from 0 (invisible) to 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "f32", into = "f32")]
pub struct Opacity(u16);

impl Opacity {
    pub fn get(self) -> f32 {
        f32::from(self.0) / 1000.0
    }
}

impl Default for Opacity {
    fn default() -> Self {
        Self(1000)
    }
}

impl TryFrom<f32> for Opacity {
    type Error = CoreError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !value.is_finite() || !(0.0..=1.0).contains(&value) {
            return Err(CoreError::Conflict(
                "an opacity must be between 0 and 1".to_owned(),
            ));
        }
        Ok(Self((value * 1000.0).round() as u16))
    }
}

impl From<Opacity> for f32 {
    fn from(value: Opacity) -> Self {
        value.get()
    }
}

/// A clockwise turn around the element's center, in whole degrees from 0 to 359.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "f32", into = "u16")]
pub struct Rotation(u16);

impl Rotation {
    pub fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<f32> for Rotation {
    type Error = CoreError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if !value.is_finite() {
            return Err(CoreError::Conflict(
                "a rotation must be a number of degrees".to_owned(),
            ));
        }
        Ok(Self(value.round().rem_euclid(360.0) as u16 % 360))
    }
}

impl From<Rotation> for u16 {
    fn from(value: Rotation) -> Self {
        value.0
    }
}

/// Which pages an element is printed on. Full-page attachments (supplier
/// datasheets, plans) never carry one.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DecorationPages {
    Cover,
    Inner,
    #[default]
    All,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DecorationLayer {
    #[default]
    Behind,
    Front,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum ImageFit {
    #[default]
    Contain,
    Cover,
}

/// What an element draws. A line is a thin rectangle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DecorationShape {
    Image {
        key: String,
        #[serde(default)]
        fit: ImageFit,
        #[serde(default)]
        opacity: Opacity,
    },
    Rectangle {
        fill: HexColor,
        #[serde(default)]
        opacity: Opacity,
        #[serde(default)]
        radius: Millimeters,
    },
}

/// One element placed on the page, repeated on every page it applies to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "RawDecoration")]
pub struct DoeDecoration {
    #[serde(flatten)]
    pub shape: DecorationShape,
    pub x: Millimeters,
    pub y: Millimeters,
    pub width: Millimeters,
    pub height: Millimeters,
    #[serde(default)]
    pub pages: DecorationPages,
    #[serde(default)]
    pub layer: DecorationLayer,
    #[serde(default)]
    pub rotation: Rotation,
    /// Mirrors the element left to right, before the rotation.
    #[serde(default)]
    pub flip_x: bool,
    /// Mirrors the element top to bottom, before the rotation.
    #[serde(default)]
    pub flip_y: bool,
}

#[derive(Deserialize)]
struct RawDecoration {
    #[serde(flatten)]
    shape: DecorationShape,
    x: Millimeters,
    y: Millimeters,
    width: Millimeters,
    height: Millimeters,
    #[serde(default)]
    pages: DecorationPages,
    #[serde(default)]
    layer: DecorationLayer,
    #[serde(default)]
    rotation: Rotation,
    #[serde(default)]
    flip_x: bool,
    #[serde(default)]
    flip_y: bool,
}

impl TryFrom<RawDecoration> for DoeDecoration {
    type Error = CoreError;

    fn try_from(raw: RawDecoration) -> Result<Self, Self::Error> {
        let fits = raw.x.get() + raw.width.get() <= Millimeters::PAGE_WIDTH
            && raw.y.get() + raw.height.get() <= Millimeters::PAGE_HEIGHT;
        if !fits {
            return Err(CoreError::Conflict(
                "an element must stay inside the A4 page".to_owned(),
            ));
        }
        if raw.width.get() <= 0.0 || raw.height.get() <= 0.0 {
            return Err(CoreError::Conflict(
                "an element needs a width and a height above 0".to_owned(),
            ));
        }
        Ok(Self {
            shape: raw.shape,
            x: raw.x,
            y: raw.y,
            width: raw.width,
            height: raw.height,
            pages: raw.pages,
            layer: raw.layer,
            rotation: raw.rotation,
            flip_x: raw.flip_x,
            flip_y: raw.flip_y,
        })
    }
}

/// One block of the cover, placed in millimeters from the page's top-left
/// corner, whatever the margins. What it prints stays the same; a text that
/// overflows its frame is cut.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "RawCoverBlock")]
pub struct CoverBlock {
    pub x: Millimeters,
    pub y: Millimeters,
    pub width: Millimeters,
    pub height: Millimeters,
    pub visible: bool,
}

#[derive(Deserialize)]
struct RawCoverBlock {
    x: Millimeters,
    y: Millimeters,
    width: Millimeters,
    height: Millimeters,
    #[serde(default = "visible_by_default")]
    visible: bool,
}

fn visible_by_default() -> bool {
    true
}

impl TryFrom<RawCoverBlock> for CoverBlock {
    type Error = CoreError;

    fn try_from(raw: RawCoverBlock) -> Result<Self, Self::Error> {
        let fits = raw.x.get() + raw.width.get() <= Millimeters::PAGE_WIDTH
            && raw.y.get() + raw.height.get() <= Millimeters::PAGE_HEIGHT;
        if !fits {
            return Err(CoreError::Conflict(
                "a cover block must stay inside the A4 page".to_owned(),
            ));
        }
        if raw.width.get() <= 0.0 || raw.height.get() <= 0.0 {
            return Err(CoreError::Conflict(
                "a cover block needs a width and a height above 0".to_owned(),
            ));
        }
        Ok(Self {
            x: raw.x,
            y: raw.y,
            width: raw.width,
            height: raw.height,
            visible: raw.visible,
        })
    }
}

/// Where each block of the cover goes. All six are always present: a block
/// is hidden, never missing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct CoverLayout {
    pub logo: CoverBlock,
    /// The contractor's name and details.
    pub company: CoverBlock,
    pub place: CoverBlock,
    /// The band: document name, DOE title, lot.
    pub title: CoverBlock,
    /// Date, author, revision.
    pub meta: CoverBlock,
    pub photo: CoverBlock,
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
    /// The role rows of the parties table: "Maîtrise d'ouvrage" and the like.
    pub party_color: HexColor,
    pub heading_font: DoeFont,
    pub body_font: DoeFont,
    pub logo_key: Option<String>,
    pub cover_logo_position: LogoPosition,
    pub cover_logo_size: LogoSize,
    /// Free placement of the cover's blocks; none keeps the cover flowing
    /// top to bottom.
    pub cover_layout: Option<CoverLayout>,
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
    /// Graphic elements repeated on the pages; the last one is on top.
    #[serde(deserialize_with = "bounded_decorations")]
    pub decorations: Vec<DoeDecoration>,
}

impl DoeStyle {
    pub const MAX_DECORATIONS: usize = 20;
}

fn bounded_decorations<'de, D>(deserializer: D) -> Result<Vec<DoeDecoration>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let decorations = Vec::<DoeDecoration>::deserialize(deserializer)?;
    if decorations.len() > DoeStyle::MAX_DECORATIONS {
        return Err(serde::de::Error::custom(format!(
            "a style holds at most {} elements",
            DoeStyle::MAX_DECORATIONS
        )));
    }
    Ok(decorations)
}

impl Default for DoeStyle {
    fn default() -> Self {
        Self {
            primary: HexColor::mestier_default(),
            secondary: HexColor::mestier_default(),
            on_primary: HexColor::white(),
            party_color: HexColor::mestier_default(),
            heading_font: DoeFont::Inter,
            body_font: DoeFont::Inter,
            logo_key: None,
            cover_logo_position: LogoPosition::Left,
            cover_logo_size: LogoSize::Medium,
            cover_layout: None,
            cover_photo: true,
            header_logo: false,
            header_text: DoeTemplate::fixed("DOE – {titre}"),
            footer_text: DoeTemplate::fixed("{date}"),
            cover_title: DoeLabel::fixed("DOSSIER DES OUVRAGES EXÉCUTÉS (D.O.E.)"),
            parties_title: DoeLabel::fixed("INTERVENANTS"),
            contents_title: DoeLabel::fixed("SOMMAIRE"),
            supply_table_title: DoeLabel::fixed("Identification de la fourniture"),
            decorations: Vec::new(),
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
        assert_eq!(style.party_color.as_str(), "#0f3d36");
        assert!(style.cover_layout.is_none());
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

    fn style_with(decoration: &str) -> Result<DoeStyle, serde_json::Error> {
        serde_json::from_str(&format!(r#"{{"decorations":[{decoration}]}}"#))
    }

    const RECT: &str = r##""kind":"rectangle","fill":"#ff0000""##;

    #[test]
    fn a_style_without_decorations_has_none() {
        assert!(DoeStyle::default().decorations.is_empty());
        let style: DoeStyle = serde_json::from_str("{}").unwrap();
        assert!(style.decorations.is_empty());
    }

    #[test]
    fn decoration_round_trips_with_defaults() {
        let style = style_with(&format!(
            r#"{{{RECT},"x":10,"y":280.5,"width":190,"height":2}}"#
        ))
        .unwrap();
        let decoration = &style.decorations[0];
        assert_eq!(decoration.pages, DecorationPages::All);
        assert_eq!(decoration.layer, DecorationLayer::Behind);
        assert_eq!(decoration.y.get(), 280.5);
        let json = serde_json::to_string(&style).unwrap();
        assert_eq!(serde_json::from_str::<DoeStyle>(&json).unwrap(), style);
    }

    #[test]
    fn decoration_outside_the_page_or_empty_is_refused() {
        for bad in [
            r#""x":200,"y":0,"width":20,"height":5"#,
            r#""x":0,"y":290,"width":5,"height":10"#,
            r#""x":0,"y":0,"width":0,"height":5"#,
            r#""x":-1,"y":0,"width":5,"height":5"#,
        ] {
            assert!(style_with(&format!("{{{RECT},{bad}}}")).is_err(), "{bad}");
        }
    }

    #[test]
    fn opacity_outside_zero_to_one_is_refused() {
        let image = |opacity: &str| {
            style_with(&format!(
                r#"{{"kind":"image","key":"k","opacity":{opacity},"x":0,"y":0,"width":10,"height":10}}"#
            ))
        };
        assert!(image("0.5").is_ok());
        assert!(image("1.2").is_err());
        assert!(image("-0.1").is_err());
    }

    #[test]
    fn more_than_twenty_decorations_are_refused() {
        let one = format!(r#"{{{RECT},"x":0,"y":0,"width":1,"height":1}}"#);
        let many = |n: usize| vec![one.as_str(); n].join(",");
        assert!(style_with(&many(20)).is_ok());
        assert!(style_with(&many(21)).is_err());
    }

    #[test]
    fn a_decoration_without_transform_is_upright_and_unflipped() {
        let style =
            style_with(&format!(r#"{{{RECT},"x":0,"y":0,"width":10,"height":10}}"#)).unwrap();
        let decoration = &style.decorations[0];
        assert_eq!(decoration.rotation.get(), 0);
        assert!(!decoration.flip_x);
        assert!(!decoration.flip_y);
    }

    #[test]
    fn rotation_is_kept_between_0_and_359_degrees() {
        let turned = |rotation: &str| {
            style_with(&format!(
                r#"{{{RECT},"x":0,"y":0,"width":10,"height":10,"rotation":{rotation}}}"#
            ))
            .map(|style| style.decorations[0].rotation.get())
        };
        assert_eq!(turned("-90").unwrap(), 270);
        assert_eq!(turned("360").unwrap(), 0);
        assert_eq!(turned("12.4").unwrap(), 12);
        assert_eq!(turned("359.6").unwrap(), 0);
        assert_eq!(turned("725").unwrap(), 5);
        assert!(turned(r#""90""#).is_err());
        assert!(Rotation::try_from(f32::NAN).is_err());
        assert!(Rotation::try_from(f32::INFINITY).is_err());
    }

    #[test]
    fn a_rotated_element_may_overflow_the_page() {
        let style = style_with(&format!(
            r#"{{{RECT},"x":0,"y":0,"width":210,"height":20,"rotation":45,"flip_x":true}}"#
        ))
        .unwrap();
        let json = serde_json::to_value(&style.decorations[0]).unwrap();
        assert_eq!(json["rotation"], 45);
        assert_eq!(json["flip_x"], true);
        assert_eq!(json["flip_y"], false);
    }
}
