use chrono::NaiveDate;
use doc_render::{MaintenanceRecap, RecapContact, RecapRow, TypstRecapRenderer};
use lopdf::Document;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

fn text_of(bytes: &[u8]) -> (usize, String) {
    let doc = Document::load_mem(bytes).unwrap();
    let pages: Vec<u32> = doc.get_pages().keys().copied().collect();
    let text = doc.extract_text(&pages).unwrap_or_default();
    (pages.len(), text)
}

fn row(site: &str, town: &str, overdue: bool) -> RecapRow {
    RecapRow {
        reference: None,
        site: site.to_owned(),
        town: town.to_owned(),
        line: "Tonte avec ramassage".to_owned(),
        last_visit: Some(date(2026, 10, 5)),
        due_on: if overdue {
            date(2026, 10, 12)
        } else {
            date(2026, 10, 21)
        },
        overdue,
        planned_minutes: Some(90),
        contact: None,
    }
}

#[test]
fn spec_ent_12_the_pdf_text_holds_every_site_and_the_accented_town() {
    let recap = MaintenanceRecap::sample(date(2026, 10, 19));

    let bytes = TypstRecapRenderer.render(&recap).unwrap();

    assert!(bytes.starts_with(b"%PDF"));
    let (_, text) = text_of(&bytes);
    for row in &recap.rows {
        assert!(text.contains(&row.site), "{} missing from {text}", row.site);
    }
    assert!(text.contains("Saint-Pourçain-sur-Sioule"), "{text}");
    if let Ok(path) = std::env::var("RECAP_SAMPLE_OUT") {
        std::fs::write(path, &bytes).unwrap();
    }
}

#[test]
fn the_title_names_the_week_and_the_contact_is_printed() {
    let mut recap = MaintenanceRecap::sample(date(2026, 10, 19));
    recap.rows[0].contact = Some(RecapContact {
        name: "Élodie Gaume".to_owned(),
        phone: Some("04 70 00 00 00".to_owned()),
        email: None,
    });

    let (_, text) = text_of(&TypstRecapRenderer.render(&recap).unwrap());

    assert!(text.contains("S43 2026"), "{text}");
    assert!(text.contains("Élodie Gaume"), "{text}");
    assert!(text.contains("04 70 00 00 00"), "{text}");
}

#[test]
fn an_overdue_line_is_marked_and_listed_before_the_others_of_its_town() {
    let recap = MaintenanceRecap {
        week_start: date(2026, 10, 19),
        generated_on: date(2026, 10, 15),
        rows: vec![
            row("Site à l'heure", "Vichy", false),
            row("Site en retard", "Vichy", true),
        ],
    };

    let (_, text) = text_of(&TypstRecapRenderer.render(&recap).unwrap());

    let late = text.find("Site en retard").expect("late site");
    let on_time = text.find("Site à l'heure").expect("on-time site");
    assert!(late < on_time, "{text}");
    assert!(text.contains("En retard"), "{text}");
}

#[test]
fn a_town_with_an_overdue_line_comes_first() {
    let recap = MaintenanceRecap {
        week_start: date(2026, 10, 19),
        generated_on: date(2026, 10, 15),
        rows: vec![row("Site A", "Abrest", false), row("Site Z", "Vichy", true)],
    };

    let (_, text) = text_of(&TypstRecapRenderer.render(&recap).unwrap());

    assert!(text.find("Vichy").unwrap() < text.find("Abrest").unwrap());
}

#[test]
fn an_empty_recap_says_so() {
    let recap = MaintenanceRecap {
        week_start: date(2026, 10, 19),
        generated_on: date(2026, 10, 15),
        rows: Vec::new(),
    };

    let (pages, text) = text_of(&TypstRecapRenderer.render(&recap).unwrap());

    assert_eq!(pages, 1);
    assert!(text.contains("Aucune prestation"), "{text}");
}

#[test]
fn a_long_recap_spills_over_several_pages() {
    let recap = MaintenanceRecap {
        week_start: date(2026, 10, 19),
        generated_on: date(2026, 10, 15),
        rows: (0..120)
            .map(|n| row(&format!("Site numéro {n}"), "Vichy", n % 7 == 0))
            .collect(),
    };

    let (pages, text) = text_of(&TypstRecapRenderer.render(&recap).unwrap());

    assert!(pages > 1, "{pages} page");
    assert!(text.contains("Site numéro 119"), "{text}");
}
