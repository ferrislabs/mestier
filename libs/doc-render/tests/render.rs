use std::time::Instant;

use chrono::NaiveDate;
use doc_render::TypstDoeRenderer;
use image::{ImageEncoder, codecs::jpeg::JpegEncoder, codecs::png::PngEncoder};
use lopdf::{Document, Object, Stream, dictionary};
use mestier_core::{
    BundledFile, DoeDocument, DoeFile, DoeParty, DoeRenderError, DoeRenderer, DoeSupply,
    FileBundle, HexColor, NumberedContent, NumberedSection, NumberedSupply, PartyRole,
    PhotosPerPage, SupplierSnapshot,
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
        accent: HexColor::try_from("#0f3d36".to_owned()).unwrap(),
        logo_key: Some("logo".to_owned()),
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
