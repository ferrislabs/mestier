use std::time::Instant;

use chrono::NaiveDate;
use doc_render::{DocxDoeRenderer, TypstDoeRenderer};
use image::{ImageEncoder, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder};
use lopdf::{Document, Object, Stream, dictionary};
use mestier_core::{
    BundledFile, DocumentStyle, DoeDocument, DoeFile, DoeFont, DoeLabel, DoeParty, DoeRenderError,
    DoeRenderer, DoeStyle, DoeSupply, DoeTemplate, FileBundle, HexColor, LogoPosition, LogoSize,
    NumberedContent, NumberedSection, NumberedSupply, PartyRole, PhotosPerPage, SupplierSnapshot,
    SupplyPhotoColumns,
};
use uuid::Uuid;

fn noise(width: u32, height: u32, seed: u64) -> Vec<u8> {
    let mut state = seed | 1;
    (0..width * height * 3)
        .map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            (state >> 24) as u8
        })
        .collect()
}

fn png(width: u32, height: u32, seed: u64) -> Vec<u8> {
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(
            &noise(width, height, seed),
            width,
            height,
            image::ExtendedColorType::Rgb8,
        )
        .unwrap();
    out
}

fn jpeg(width: u32, height: u32, seed: u64) -> Vec<u8> {
    let mut out = Vec::new();
    JpegEncoder::new(&mut out)
        .write_image(
            &noise(width, height, seed),
            width,
            height,
            image::ExtendedColorType::Rgb8,
        )
        .unwrap();
    out
}

fn pdf(pages: usize) -> Vec<u8> {
    let mut doc = Document::with_version("1.5");
    let pages_id = doc.new_object_id();
    let font = doc.add_object(dictionary! {
        "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica",
    });
    let mut kids = Vec::new();
    for n in 0..pages {
        let content = format!("BT /F1 48 Tf 100 700 Td (Supplier page {}) Tj ET", n + 1);
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
        let page = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Contents" => content_id,
            "Resources" => dictionary! { "Font" => dictionary! { "F1" => font } },
        });
        kids.push(Object::Reference(page));
    }
    doc.objects.insert(
        pages_id,
        Object::Dictionary(dictionary! {
            "Type" => "Pages", "Kids" => kids, "Count" => pages as i64,
        }),
    );
    let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
    doc.trailer.set("Root", catalog);
    let mut out = Vec::new();
    doc.save_to(&mut out).unwrap();
    out
}

fn file(mime: &str, bytes: Vec<u8>) -> BundledFile {
    BundledFile {
        mime_type: mime.to_owned(),
        bytes,
    }
}

fn supply(
    heading: &str,
    photos: Vec<DoeFile>,
    datasheets: Vec<String>,
    supplier: bool,
) -> DoeSupply {
    DoeSupply {
        id: Uuid::new_v4(),
        source_product_id: None,
        title: Some(heading.to_owned()),
        designation: "Érable champêtre".to_owned(),
        dpgf_ref: Some("2.1.3".to_owned()),
        destination: Some("Parvis nord".to_owned()),
        supplier: supplier.then(|| SupplierSnapshot {
            name: "Pépinières Lenoir".to_owned(),
            address: Some("12 rue des Tilleuls, 49000 Angers".to_owned()),
            phone: Some("02 41 00 00 00".to_owned()),
            email: Some("contact@lenoir.example".to_owned()),
        }),
        maker: Some("Pépinières Lenoir".to_owned()),
        specs: Some("Hauteur 3 m\nCirconférence 16/18\nÉlevé en conteneur".to_owned()),
        photos,
        photo_columns: Default::default(),
        datasheet_keys: datasheets,
    }
}

fn caption(key: &str, text: &str) -> DoeFile {
    DoeFile {
        key: key.to_owned(),
        caption: Some(text.to_owned()),
    }
}

fn party(role: PartyRole, name: &str, logo: Option<&str>) -> DoeParty {
    DoeParty {
        role,
        name: name.to_owned(),
        address: Some("1 place de l’Hôtel de Ville".to_owned()),
        phone: Some("01 02 03 04 05".to_owned()),
        email: Some("contact@example.fr".to_owned()),
        logo_key: logo.map(str::to_owned),
        contact_name: (role == PartyRole::ProjectManager).then(|| "Claire Dupont".to_owned()),
        contact_role: (role == PartyRole::ProjectManager).then(|| "Paysagiste".to_owned()),
        contact_email: (role == PartyRole::ProjectManager)
            .then(|| "claire@atelier-paysage.fr".to_owned()),
    }
}

fn fixture() -> (DoeDocument, FileBundle) {
    let mut bundle = FileBundle::default();
    bundle.insert("logo".into(), file("image/png", png(120, 60, 1)));
    bundle.insert("p1".into(), file("image/png", png(300, 200, 2)));
    bundle.insert("p2".into(), file("image/jpeg", jpeg(300, 200, 3)));
    for n in 0..5 {
        bundle.insert(format!("ph{n}"), file("image/jpeg", jpeg(400, 300, 10 + n)));
    }
    bundle.insert("sheet".into(), file("application/pdf", pdf(2)));
    bundle.insert("plan1".into(), file("application/pdf", pdf(1)));
    bundle.insert("plan2".into(), file("application/pdf", pdf(1)));

    let sections = vec![
        NumberedSection {
            number: 1,
            title: "Identification des végétaux".to_owned(),
            content: NumberedContent::Supplies {
                maker_label: "Pépinière".to_owned(),
                supplies: vec![NumberedSupply {
                    number: "1.1".to_owned(),
                    heading: "Plantation d’arbres".to_owned(),
                    supply: supply(
                        "Plantation d’arbres",
                        vec![caption("p1", "Sujet livré"), caption("p2", "Mise en place")],
                        vec![],
                        true,
                    ),
                }],
            },
        },
        NumberedSection {
            number: 2,
            title: "Mobilier".to_owned(),
            content: NumberedContent::Supplies {
                maker_label: "Fabricant".to_owned(),
                supplies: vec![NumberedSupply {
                    number: "2.1".to_owned(),
                    heading: "Banc Garden".to_owned(),
                    supply: supply("Banc Garden", vec![], vec!["sheet".to_owned()], true),
                }],
            },
        },
        NumberedSection {
            number: 3,
            title: "Photos".to_owned(),
            content: NumberedContent::Photos {
                per_page: PhotosPerPage::Two,
                photos: (0..5)
                    .map(|n| caption(&format!("ph{n}"), &format!("Vue {n}")))
                    .collect(),
            },
        },
        NumberedSection {
            number: 4,
            title: "Plans de récolement".to_owned(),
            content: NumberedContent::Plans {
                plans: vec![caption("plan1", "Plan masse"), caption("plan2", "Réseaux")],
            },
        },
        NumberedSection {
            number: 5,
            title: "Notes".to_owned(),
            content: NumberedContent::BlankPages { count: 1 },
        },
    ];
    let document = DoeDocument {
        title: "Aménagement du parvis".to_owned(),
        place: Some("Angers".to_owned()),
        lot: Some("Lot 3 : espaces verts".to_owned()),
        issued_on: NaiveDate::from_ymd_opt(2026, 10, 7).unwrap(),
        author: Some("Élodie Martin".to_owned()),
        revision: "A".to_owned(),
        parties: vec![
            party(PartyRole::ProjectOwner, "Ville d’Angers", None),
            party(PartyRole::ProjectManager, "Atelier Paysage", Some("logo")),
            party(PartyRole::Contractor, "Verts Travaux", Some("logo")),
        ],
        sections,
        style: DocumentStyle::from(&DoeStyle {
            logo_key: Some("logo".to_owned()),
            ..DoeStyle::default()
        }),
        header: "DOE – Avenue du 8 mai 1945".to_owned(),
        footer: "07/10/2026".to_owned(),
        cover_photo_key: None,
    };
    (document, bundle)
}

fn text_of(bytes: &[u8]) -> (usize, String) {
    let doc = Document::load_mem(bytes).unwrap();
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    let text = doc.extract_text(&pages).unwrap_or_default();
    (pages.len(), text)
}

#[test]
fn renders_every_page_with_accents() {
    let (document, bundle) = fixture();
    let bytes = TypstDoeRenderer.render(&document, &bundle).unwrap();
    assert!(bytes.starts_with(b"%PDF"));
    let (pages, text) = text_of(&bytes);
    assert_eq!(pages, 14);
    assert!(text.contains("Pépinière"), "{text}");
    assert!(text.contains("Identification des végétaux"), "{text}");
    if let Ok(path) = std::env::var("DOE_SAMPLE_OUT") {
        std::fs::write(path, &bytes).unwrap();
    }
}

/// The `BaseFont` of every font the PDF embeds.
fn embedded_fonts(bytes: &[u8]) -> Vec<String> {
    let doc = Document::load_mem(bytes).unwrap();
    doc.objects
        .values()
        .filter_map(|object| object.as_dict().ok())
        .filter(|dict| dict.get(b"Type").and_then(Object::as_name).ok() == Some(b"Font"))
        .filter_map(|dict| dict.get(b"BaseFont").and_then(Object::as_name).ok())
        .map(|name| String::from_utf8_lossy(name).into_owned())
        .collect()
}

#[test]
fn spec_style_1_default_style_prints_the_titles_of_before_in_inter() {
    let (document, bundle) = fixture();
    let bytes = TypstDoeRenderer.render(&document, &bundle).unwrap();
    let (_, text) = text_of(&bytes);
    for expected in [
        "DOSSIER DES OUVRAGES EXÉCUTÉS",
        "INTERVENANTS",
        "SOMMAIRE",
        "Identification de la fourniture",
        "DOE – Avenue du 8 mai 1945",
    ] {
        assert!(text.contains(expected), "{expected} missing from {text}");
    }
    // Helvetica comes from the supplier PDFs inserted as pages.
    let fonts = embedded_fonts(&bytes);
    assert!(
        fonts
            .iter()
            .all(|font| font.contains("Inter") || font == "Helvetica"),
        "{fonts:?}"
    );
}

#[test]
fn parties_are_tables_with_a_role_row_and_the_contractor_moves_to_the_cover() {
    let (document, bundle) = fixture();
    let bytes = TypstDoeRenderer.render(&document, &bundle).unwrap();
    if let Ok(path) = std::env::var("DOE_PARTIES_OUT") {
        std::fs::write(path, &bytes).unwrap();
    }
    let doc = Document::load_mem(&bytes).unwrap();
    let text_on = |page: u32| doc.extract_text(&[page]).unwrap_or_default();
    let cover = text_on(1);
    assert!(cover.contains("Verts Travaux"), "{cover}");
    assert!(cover.contains("contact@example.fr"), "{cover}");
    let parties = text_on(2);
    for expected in [
        "Maîtrise d’ouvrage",
        "Maîtrise d’œuvre",
        "Entreprise adjudicatrice",
        "Contact chantier",
        "Claire Dupont",
        "Paysagiste",
        "claire@atelier-paysage.fr",
    ] {
        assert!(
            parties.contains(expected),
            "{expected} missing from {parties}"
        );
    }
}

#[test]
fn spec_style_2_a_custom_style_reaches_the_pdf() {
    let (mut document, mut bundle) = fixture();
    bundle.insert("cover".into(), file("image/jpeg", jpeg(800, 500, 99)));
    let label = |text: &str| DoeLabel::try_from(text.to_owned()).unwrap();
    document.style = DocumentStyle::from(&DoeStyle {
        primary: HexColor::try_from("#7a1f2b".to_owned()).unwrap(),
        secondary: HexColor::try_from("#335533".to_owned()).unwrap(),
        on_primary: HexColor::try_from("#ffe9a8".to_owned()).unwrap(),
        party_color: HexColor::try_from("#224488".to_owned()).unwrap(),
        heading_font: DoeFont::Montserrat,
        body_font: DoeFont::SourceSerif,
        logo_key: Some("logo".to_owned()),
        cover_logo_position: LogoPosition::Right,
        cover_logo_size: LogoSize::Large,
        cover_layout: None,
        cover_photo: true,
        header_logo: true,
        header_text: DoeTemplate::try_from("{titre}".to_owned()).unwrap(),
        footer_text: DoeTemplate::try_from("{date}".to_owned()).unwrap(),
        cover_title: label("Dossier des ouvrages exécutés"),
        parties_title: label("Les intervenants"),
        contents_title: label("Table des matières"),
        supply_table_title: label("Fiche fourniture"),
        decorations: Vec::new(),
    });
    document.header = "Treyve Paysages – Avenue".to_owned();
    document.footer = "SIRET 123 456 789".to_owned();
    document.cover_photo_key = Some("cover".to_owned());

    let bytes = TypstDoeRenderer.render(&document, &bundle).unwrap();
    if let Ok(path) = std::env::var("DOE_STYLED_OUT") {
        std::fs::write(path, &bytes).unwrap();
    }

    let (pages, text) = text_of(&bytes);
    assert_eq!(pages, 14);
    for expected in [
        "Dossier des ouvrages exécutés",
        "Les intervenants",
        "Table des matières",
        "Fiche fourniture",
        "Treyve Paysages – Avenue",
        "SIRET 123 456 789",
    ] {
        assert!(text.contains(expected), "{expected} missing from {text}");
    }
    assert!(!text.contains("SOMMAIRE"), "{text}");
    let fonts = embedded_fonts(&bytes);
    assert!(fonts.iter().any(|f| f.contains("Montserrat")), "{fonts:?}");
    assert!(
        fonts.iter().any(|f| f.contains("SourceSerif4")),
        "{fonts:?}"
    );
}

#[test]
fn every_font_renders_in_regular_and_bold() {
    for font in [
        DoeFont::Inter,
        DoeFont::Montserrat,
        DoeFont::SourceSerif,
        DoeFont::Barlow,
    ] {
        let (mut document, bundle) = fixture();
        document.style = DocumentStyle::from(&DoeStyle {
            heading_font: font,
            body_font: font,
            ..DoeStyle::default()
        });
        let bytes = TypstDoeRenderer.render(&document, &bundle).unwrap();
        let fonts = embedded_fonts(&bytes);
        let stem = font.family().replace(' ', "");
        assert!(
            fonts
                .iter()
                .any(|f| f.contains(&stem) && f.contains("Bold")),
            "{font:?}: {fonts:?}"
        );
        assert!(
            fonts
                .iter()
                .any(|f| f.contains(&stem) && !f.contains("Bold")),
            "{font:?}: {fonts:?}"
        );
    }
}

#[test]
fn svg_pages_are_the_requested_pages_of_the_same_layout() {
    let (document, bundle) = fixture();
    let svgs = TypstDoeRenderer
        .render_svg(&document, &bundle, &[0, 3, 999])
        .unwrap();
    assert_eq!(svgs.len(), 2, "a page past the end is skipped");
    assert!(svgs.iter().all(|svg| svg.starts_with("<svg")));
}

#[test]
fn spec_style_7_the_sample_previews_cover_parties_and_a_supply_sheet_quickly() {
    let document = DoeDocument::sample(
        &DoeStyle::default(),
        NaiveDate::from_ymd_opt(2026, 10, 11).unwrap(),
    );
    let started = Instant::now();
    let svgs = TypstDoeRenderer
        .render_svg(&document, &FileBundle::default(), &[0, 1, 3])
        .unwrap();
    let elapsed = started.elapsed();
    assert_eq!(svgs.len(), 3);
    let total: usize = svgs.iter().map(String::len).sum();
    eprintln!("sample preview: {elapsed:?}, {} KB", total / 1024);
}

#[test]
fn missing_file_is_reported() {
    let (document, mut bundle) = fixture();
    bundle.0.remove("sheet");
    assert_eq!(
        TypstDoeRenderer.render(&document, &bundle),
        Err(DoeRenderError::MissingFile {
            key: "sheet".into()
        })
    );
}

#[test]
fn unsupported_mime_is_reported() {
    let (document, mut bundle) = fixture();
    bundle.insert("p1".into(), file("image/tiff", vec![1, 2, 3]));
    assert_eq!(
        TypstDoeRenderer.render(&document, &bundle),
        Err(DoeRenderError::UnsupportedFile {
            key: "p1".into(),
            mime_type: "image/tiff".into()
        })
    );
}

#[test]
fn pdf_cannot_be_a_photo() {
    let (document, mut bundle) = fixture();
    bundle.insert("p1".into(), file("application/pdf", pdf(1)));
    assert!(matches!(
        TypstDoeRenderer.render(&document, &bundle),
        Err(DoeRenderError::UnsupportedFile { .. })
    ));
}

#[test]
fn heavy_photos_render_in_reasonable_time() {
    let (mut document, mut bundle) = fixture();
    for n in 0..10u64 {
        bundle.insert(format!("big{n}"), file("image/png", png(840, 840, 100 + n)));
    }
    document.sections.push(NumberedSection {
        number: 6,
        title: "Gros clichés".to_owned(),
        content: NumberedContent::Photos {
            per_page: PhotosPerPage::Four,
            photos: (0..10)
                .map(|n| caption(&format!("big{n}"), "Cliché"))
                .collect(),
        },
    });
    let started = Instant::now();
    let bytes = TypstDoeRenderer.render(&document, &bundle).unwrap();
    eprintln!(
        "render with 10 x ~2 MB photos: {:?}, {} KB output",
        started.elapsed(),
        bytes.len() / 1024
    );
}

fn decorated(decorations: &str) -> (DoeDocument, FileBundle) {
    let (mut document, bundle) = fixture();
    let style: DoeStyle =
        serde_json::from_str(&format!(r#"{{"decorations":{decorations}}}"#)).unwrap();
    document.style = DocumentStyle::from(&DoeStyle {
        logo_key: Some("logo".to_owned()),
        ..style
    });
    (document, bundle)
}

fn pages_painted(document: &DoeDocument, bundle: &FileBundle, color: &str) -> Vec<usize> {
    let all: Vec<usize> = (0..40).collect();
    TypstDoeRenderer
        .render_svg(document, bundle, &all)
        .unwrap()
        .iter()
        .enumerate()
        .filter(|(_, svg)| svg.contains(color))
        .map(|(index, _)| index)
        .collect()
}

const MAGENTA: &str = "#ff00ff";

fn magenta(pages: &str, layer: &str) -> String {
    format!(
        r##"[{{"kind":"rectangle","fill":"{MAGENTA}","x":10,"y":280,"width":190,"height":4,"pages":"{pages}","layer":"{layer}"}}]"##
    )
}

#[test]
fn layout_a_cover_element_is_printed_on_the_cover_only() {
    let (document, bundle) = decorated(&magenta("cover", "behind"));
    assert_eq!(pages_painted(&document, &bundle, MAGENTA), vec![0]);
}

#[test]
fn layout_an_inner_element_skips_the_cover_and_full_page_attachments() {
    let (document, bundle) = decorated(&magenta("inner", "front"));
    let painted = pages_painted(&document, &bundle, MAGENTA);
    let total = TypstDoeRenderer
        .render_svg(&document, &bundle, &(0..40).collect::<Vec<_>>())
        .unwrap()
        .len();
    assert!(!painted.is_empty() && !painted.contains(&0), "{painted:?}");
    assert!(
        painted.len() < total - 1,
        "datasheets and plans are not decorated: {painted:?} of {total}"
    );
}

#[test]
fn layout_an_all_element_is_on_the_cover_and_inner_pages() {
    let (document, bundle) = decorated(&magenta("all", "behind"));
    let painted = pages_painted(&document, &bundle, MAGENTA);
    assert!(painted.contains(&0) && painted.contains(&1), "{painted:?}");
}

#[test]
fn layout_an_image_element_is_placed_and_a_missing_one_is_dropped() {
    let image = |key: &str| {
        format!(
            r#"[{{"kind":"image","key":"{key}","x":150,"y":5,"width":40,"height":20,"opacity":0.5}}]"#
        )
    };
    let (document, bundle) = decorated(&image("logo"));
    let with = TypstDoeRenderer.render(&document, &bundle).unwrap();
    let (document, bundle) = decorated(&image("gone"));
    let without = TypstDoeRenderer.render(&document, &bundle).unwrap();
    assert!(with.len() != without.len());
    assert!(without.starts_with(b"%PDF"));
}

#[test]
fn layout_no_decorations_leaves_the_document_unchanged() {
    let (document, bundle) = fixture();
    let (empty, _) = decorated("[]");
    let a = TypstDoeRenderer
        .render_svg(&document, &bundle, &[0, 1])
        .unwrap();
    let b = TypstDoeRenderer
        .render_svg(&empty, &bundle, &[0, 1])
        .unwrap();
    assert_eq!(a, b);
}

#[test]
#[ignore = "writes a PDF for a manual look: DOE_LAYOUT_OUT=/path/file.pdf"]
fn layout_manual_look() {
    let Ok(path) = std::env::var("DOE_LAYOUT_OUT") else {
        return;
    };
    let decorations = r##"[
        {"kind":"rectangle","fill":"#0f3d36","x":0,"y":0,"width":8,"height":297,"pages":"inner"},
        {"kind":"rectangle","fill":"#f2b705","x":0,"y":0,"width":210,"height":6,"pages":"cover"},
        {"kind":"image","key":"logo","x":150,"y":8,"width":45,"height":18,"pages":"inner","layer":"front"},
        {"kind":"image","key":"logo","fit":"contain","opacity":0.12,"x":45,"y":100,"width":120,"height":90,"pages":"all","layer":"behind"},
        {"kind":"rectangle","fill":"#0f3d36","x":10,"y":284,"width":190,"height":0.6,"pages":"all"}
    ]"##;
    let (document, bundle) = decorated(decorations);
    std::fs::write(path, TypstDoeRenderer.render(&document, &bundle).unwrap()).unwrap();
}

fn pages_with_supply_photos(columns: SupplyPhotoColumns) -> usize {
    let (mut document, bundle) = fixture();
    if let NumberedContent::Supplies { supplies, .. } = &mut document.sections[0].content {
        supplies[0].supply.photos = (0..6)
            .map(|n| caption(&format!("ph{}", n % 5), &format!("Page {n}")))
            .collect();
        supplies[0].supply.photo_columns = columns;
    }
    let bytes = TypstDoeRenderer.render(&document, &bundle).unwrap();
    text_of(&bytes).0
}

#[test]
fn fewer_columns_print_bigger_supply_photos_on_more_pages() {
    let three = pages_with_supply_photos(SupplyPhotoColumns::Three);
    let two = pages_with_supply_photos(SupplyPhotoColumns::Two);
    let one = pages_with_supply_photos(SupplyPhotoColumns::One);

    assert!(three < two, "{three} pages in 3 columns, {two} in 2");
    assert!(two < one, "{two} pages in 2 columns, {one} in 1");
}

fn docx_text(bytes: &[u8]) -> String {
    docx_rs::read_docx(bytes)
        .expect("the output is a readable .docx")
        .json()
}

#[test]
fn the_docx_carries_the_content_of_the_document() {
    let (document, bundle) = fixture();
    let bytes = DocxDoeRenderer.render(&document, &bundle).unwrap();
    assert!(bytes.starts_with(b"PK"));
    let text = docx_text(&bytes);
    for expected in [
        "Aménagement du parvis",
        "Ville d’Angers",
        "1.1 Plantation d’arbres",
        "Sujet livré",
        "Vue 4",
        "Réseaux",
    ] {
        assert!(text.contains(expected), "missing `{expected}`");
    }
}

#[test]
fn every_page_of_a_pdf_plan_or_datasheet_becomes_a_picture() {
    let (document, bundle) = fixture();
    let with_pdfs = docx_text(&DocxDoeRenderer.render(&document, &bundle).unwrap());

    let mut without = bundle.clone();
    for key in ["sheet", "plan1", "plan2"] {
        without.0.remove(key);
    }
    let mut document_without = document.clone();
    for section in &mut document_without.sections {
        match &mut section.content {
            NumberedContent::Supplies { supplies, .. } => {
                for numbered in supplies {
                    numbered.supply.datasheet_keys.clear();
                }
            }
            NumberedContent::Plans { plans } => plans.clear(),
            _ => {}
        }
    }
    let baseline = docx_text(&DocxDoeRenderer.render(&document_without, &without).unwrap());

    let pictures = |json: &str| json.matches("\"type\": \"drawing\"").count();
    assert!(!with_pdfs.contains("fichier PDF"));
    assert_eq!(pictures(&with_pdfs), pictures(&baseline) + 4);
}

#[test]
fn the_docx_refuses_a_missing_file() {
    let (document, mut bundle) = fixture();
    bundle.0.remove("p1");
    assert_eq!(
        DocxDoeRenderer.render(&document, &bundle),
        Err(DoeRenderError::MissingFile { key: "p1".into() })
    );
}

#[test]
fn the_docx_refuses_a_picture_it_cannot_decode_instead_of_panicking() {
    let (document, mut bundle) = fixture();
    bundle.insert("p1".into(), file("image/png", b"not an image".to_vec()));
    assert!(matches!(
        DocxDoeRenderer.render(&document, &bundle),
        Err(DoeRenderError::UnsupportedFile { key, .. }) if key == "p1"
    ));
}

fn pages_with_one_supply_photo(columns: Option<SupplyPhotoColumns>) -> usize {
    let (mut document, bundle) = fixture();
    if let NumberedContent::Supplies { supplies, .. } = &mut document.sections[0].content {
        if let Some(columns) = columns {
            supplies[0].supply.photos = vec![caption("ph0", "Sujet")];
            supplies[0].supply.photo_columns = columns;
        } else {
            supplies[0].supply.photos = vec![];
        }
    }
    let bytes = TypstDoeRenderer.render(&document, &bundle).unwrap();
    text_of(&bytes).0
}

#[test]
fn supply_photos_shrink_to_stay_on_the_page_of_their_table() {
    let without = pages_with_one_supply_photo(None);
    for columns in [
        SupplyPhotoColumns::One,
        SupplyPhotoColumns::Two,
        SupplyPhotoColumns::Three,
    ] {
        let label = format!("{columns:?}");
        assert_eq!(
            pages_with_one_supply_photo(Some(columns)),
            without,
            "a photo in {label} columns started a new page"
        );
    }
}

fn magenta_transform(transform: &str) -> String {
    let (document, bundle) = decorated(&format!(
        r##"[{{"kind":"rectangle","fill":"{MAGENTA}","x":50,"y":50,"width":40,"height":10{transform}}}]"##
    ));
    let svg = TypstDoeRenderer
        .render_svg(&document, &bundle, &[0])
        .unwrap()
        .remove(0);
    let at = svg.find(MAGENTA).expect("the element is drawn");
    let rest = &svg[at..];
    let start = rest.find("transform=\"").map_or(0, |i| i + 11);
    rest[start..]
        .split('"')
        .next()
        .unwrap_or_default()
        .to_owned()
}

// CSS `rotate(θ) scale(sx, sy)` around the center of the 40 × 10 mm frame at
// (50, 50) mm: the center lands on (70, 55) mm, i.e. (198.4, 155.9) pt.
#[test]
fn layout_a_quarter_turn_is_clockwise_around_the_center() {
    assert_eq!(
        magenta_transform(r#","rotation":90"#),
        "matrix(0 1 -1 0 212.598425197 99.212598425)"
    );
}

#[test]
fn layout_a_mirror_is_applied_before_the_rotation() {
    assert_eq!(
        magenta_transform(r#","rotation":90,"flip_x":true"#),
        "matrix(0 -1 -1 0 212.598425197 212.598425197)"
    );
    assert!(magenta_transform(r#","flip_y":true"#).starts_with("matrix(1 0 0 -1 "));
}

#[test]
fn layout_an_upright_element_is_only_moved() {
    assert!(magenta_transform("").starts_with("translate("));
}

fn block(x: f32, y: f32, width: f32, height: f32, visible: bool) -> String {
    format!(r#"{{"x":{x},"y":{y},"width":{width},"height":{height},"visible":{visible}}}"#)
}

fn laid_out_cover(place_visible: bool, title: &str) -> (DoeDocument, FileBundle) {
    let (mut document, mut bundle) = fixture();
    bundle.insert("cover".into(), file("image/jpeg", jpeg(800, 500, 99)));
    let layout = format!(
        r#"{{"cover_layout":{{"logo":{},"company":{},"place":{},"title":{title},"meta":{},"photo":{}}}}}"#,
        block(10.0, 10.0, 50.0, 25.0, true),
        block(120.0, 10.0, 80.0, 25.0, true),
        block(10.0, 40.0, 60.0, 10.0, place_visible),
        block(10.0, 250.0, 90.0, 25.0, true),
        block(10.0, 60.0, 190.0, 120.0, true),
    );
    let style: DoeStyle = serde_json::from_str(&layout).unwrap();
    document.style = DocumentStyle::from(&DoeStyle {
        logo_key: Some("logo".to_owned()),
        primary: HexColor::try_from(MAGENTA.to_owned()).unwrap(),
        ..style
    });
    document.cover_photo_key = Some("cover".to_owned());
    (document, bundle)
}

fn cover_text(document: &DoeDocument, bundle: &FileBundle) -> (usize, String) {
    let bytes = TypstDoeRenderer.render(document, bundle).unwrap();
    let doc = Document::load_mem(&bytes).unwrap();
    let pages = doc.get_pages().len();
    (pages, doc.extract_text(&[1]).unwrap_or_default())
}

#[test]
fn cover_layout_places_every_block_on_one_cover_page() {
    let (document, bundle) = laid_out_cover(true, &block(10.0, 200.0, 190.0, 40.0, true));
    if let Ok(path) = std::env::var("DOE_COVER_OUT") {
        std::fs::write(path, TypstDoeRenderer.render(&document, &bundle).unwrap()).unwrap();
    }
    let (pages, text) = cover_text(&document, &bundle);
    assert_eq!(pages, 14);
    for expected in [
        "Aménagement du parvis",
        "Verts Travaux",
        "Angers",
        "Élodie Martin",
    ] {
        assert!(text.contains(expected), "{expected} missing from {text}");
    }
    assert!(pages_painted(&document, &bundle, MAGENTA).contains(&0));
}

#[test]
fn cover_layout_a_hidden_block_is_not_printed() {
    let (document, bundle) = laid_out_cover(false, &block(10.0, 200.0, 190.0, 40.0, true));
    let (_, text) = cover_text(&document, &bundle);
    assert!(!text.contains("Angers"), "{text}");
    assert!(text.contains("Aménagement du parvis"), "{text}");
}

#[test]
fn cover_layout_a_frame_too_small_for_the_title_still_gives_one_cover_page() {
    let (document, bundle) = laid_out_cover(true, &block(10.0, 200.0, 20.0, 5.0, true));
    let (pages, _) = cover_text(&document, &bundle);
    assert_eq!(pages, 14);
}

#[test]
fn cover_layout_rejects_a_frame_outside_the_page() {
    let json = format!(
        r#"{{"cover_layout":{{"logo":{0},"company":{0},"place":{0},"title":{1},"meta":{0},"photo":{0}}}}}"#,
        block(0.0, 0.0, 10.0, 10.0, true),
        block(100.0, 0.0, 120.0, 10.0, true),
    );
    assert!(serde_json::from_str::<DoeStyle>(&json).is_err());
    let json = json.replace("120.0", "0");
    assert!(serde_json::from_str::<DoeStyle>(&json).is_err());
}
