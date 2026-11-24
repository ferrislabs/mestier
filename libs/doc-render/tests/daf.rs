use chrono::NaiveDate;
use doc_render::TypstDafRenderer;
use image::{ImageEncoder, codecs::png::PngEncoder};
use lopdf::{Document, Object, Stream, dictionary};
use mestier_core::{
    BundledFile, DafAttachment, DafDocument, DafRenderer, DafSheet, DafStatus, DafSummaryPage,
    DafSummaryRow, DocumentStyle, DoeRenderError, DoeStyle, FileBundle, SupplierSnapshot,
};

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn png(width: u32, height: u32) -> Vec<u8> {
    let pixels: Vec<u8> = (0..width * height * 3).map(|n| (n % 251) as u8).collect();
    let mut out = Vec::new();
    PngEncoder::new(&mut out)
        .write_image(&pixels, width, height, image::ExtendedColorType::Rgb8)
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
        let content = format!("BT /F1 48 Tf 100 700 Td (Datasheet page {}) Tj ET", n + 1);
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

fn text_of(bytes: &[u8]) -> (usize, String) {
    let doc = Document::load_mem(bytes).unwrap();
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    let text = doc.extract_text(&pages).unwrap_or_default();
    (pages.len(), text)
}

fn sheet(number: u32, designation: &str) -> DafSheet {
    DafSheet {
        label: format!("D.A.F n°{number:02}"),
        designation: designation.to_owned(),
        price_ref: Some("2.1.3".to_owned()),
        supplier: Some(SupplierSnapshot {
            name: "Pépinières Lenoir".to_owned(),
            address: Some("12 rue des Tilleuls, 49000 Angers".to_owned()),
            phone: Some("02 41 00 00 00".to_owned()),
            email: Some("contact@lenoir.example".to_owned()),
        }),
        installer: Some("Treyve Paysages\nChef de chantier".to_owned()),
        specs: Some("Hauteur 3 m\nCirconférence 16/18".to_owned()),
        observations: Some("À livrer avant le 15.".to_owned()),
        photo_key: None,
        attachments: Vec::new(),
        project_owner: Some("Commune de Cébazat".to_owned()),
        project_manager: Some("Atelier Casa".to_owned()),
        issued_on: date(2026, 11, 18),
    }
}

fn summary_row(number: u32, designation: &str, status: DafStatus) -> DafSummaryRow {
    DafSummaryRow {
        label: format!("D.A.F n°{number:02}"),
        designation: designation.to_owned(),
        price_ref: Some("2.1.3".to_owned()),
        status,
        status_label: "Acceptée".to_owned(),
        sent_on: Some(date(2026, 11, 10)),
        issued_on: date(2026, 11, 18),
        returned_on: None,
        review_note: Some("RAS".to_owned()),
    }
}

fn document(sheets: Vec<DafSheet>, with_summary: bool) -> DafDocument {
    let rows = sheets
        .iter()
        .enumerate()
        .map(|(n, s)| summary_row(n as u32 + 1, &s.designation, DafStatus::Accepted))
        .collect();
    DafDocument {
        title: "Aménagement du parc Montgroux".to_owned(),
        issuer: "Treyve Paysages".to_owned(),
        summary: with_summary.then(|| DafSummaryPage {
            printed_on: date(2026, 11, 18),
            rows,
        }),
        sheets,
        style: DocumentStyle::from(&DoeStyle::default()),
    }
}

#[test]
fn spec_daf_6_dossier_pdf_has_summary_sheets_and_attachment_pages() {
    let mut first = sheet(1, "Érable champêtre");
    first.photo_key = Some("photo".to_owned());
    let mut second = sheet(2, "Banc Garden");
    second.attachments = vec![DafAttachment {
        key: "datasheet".to_owned(),
        name: "Fiche technique banc.pdf".to_owned(),
    }];
    let mut bundle = FileBundle::default();
    bundle.insert("photo".into(), file("image/png", png(300, 200)));
    bundle.insert("datasheet".into(), file("application/pdf", pdf(2)));

    let bytes = TypstDafRenderer
        .render(&document(vec![first, second], true), &bundle)
        .unwrap();

    let (pages, text) = text_of(&bytes);
    assert_eq!(pages, 5, "{text}");
    for expected in [
        "Récapitulatif des D.A.F",
        "Demande d'Agrément de Fourniture",
        "VISAS",
        "SOMMAIRE",
        "Fiche technique banc.pdf",
    ] {
        assert!(text.contains(expected), "{expected} missing from {text}");
    }
    if let Ok(path) = std::env::var("DAF_SAMPLE_OUT") {
        std::fs::write(path, &bytes).unwrap();
    }
}

#[test]
fn a_dossier_without_summary_starts_with_the_first_sheet() {
    let bytes = TypstDafRenderer
        .render(
            &document(vec![sheet(1, "Érable")], false),
            &FileBundle::default(),
        )
        .unwrap();

    let (pages, text) = text_of(&bytes);
    assert_eq!(pages, 1);
    assert!(!text.contains("SOMMAIRE"), "{text}");
}

#[test]
fn a_photo_and_long_specs_render_without_error() {
    let mut long = sheet(1, "Érable champêtre");
    long.photo_key = Some("photo".to_owned());
    long.specs = Some(
        (0..400)
            .map(|n| format!("Caractéristique numéro {n} : valeur détaillée"))
            .collect::<Vec<_>>()
            .join("\n"),
    );
    long.observations = Some("Observation. ".repeat(80));
    let mut bundle = FileBundle::default();
    bundle.insert("photo".into(), file("image/png", png(600, 900)));

    let bytes = TypstDafRenderer
        .render(&document(vec![long], false), &bundle)
        .unwrap();

    let (pages, _) = text_of(&bytes);
    assert!(pages >= 1);
}

#[test]
fn a_missing_file_is_named() {
    let mut broken = sheet(1, "Érable");
    broken.photo_key = Some("absent".to_owned());

    let error = TypstDafRenderer
        .render(&document(vec![broken], false), &FileBundle::default())
        .unwrap_err();

    assert_eq!(
        error,
        DoeRenderError::MissingFile {
            key: "absent".to_owned()
        }
    );
}

#[test]
fn an_image_attachment_takes_one_page() {
    let mut with_image = sheet(1, "Érable");
    with_image.attachments = vec![DafAttachment {
        key: "scan".to_owned(),
        name: "scan.png".to_owned(),
    }];
    let mut bundle = FileBundle::default();
    bundle.insert("scan".into(), file("image/png", png(200, 300)));

    let bytes = TypstDafRenderer
        .render(&document(vec![with_image], false), &bundle)
        .unwrap();

    assert_eq!(text_of(&bytes).0, 2);
}
