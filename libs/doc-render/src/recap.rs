use std::cmp::Ordering;

use chrono::{Datelike, Days, NaiveDate};
use mestier_core::DoeRenderError;
use serde::Serialize;
use serde_json::json;

use crate::{bundle::Prepared, world::DoeWorld};

const TEMPLATE: &str = include_str!("maintenance_recap.typ");

const NO_TOWN: &str = "Commune non renseignée";

const MONTHS: [&str; 12] = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
];

/// The weekly maintenance recap as the PDF prints it: one row per line due
/// that week or overdue, in no particular order. Grouping by town and
/// ordering are the renderer's job, so the page and the PDF cannot drift
/// apart on how a row is shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaintenanceRecap {
    /// The Monday of the week the recap covers.
    pub week_start: NaiveDate,
    pub generated_on: NaiveDate,
    pub rows: Vec<RecapRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecapRow {
    /// The "code affaire" of the contract.
    pub reference: Option<String>,
    pub site: String,
    pub town: String,
    pub line: String,
    pub last_visit: Option<NaiveDate>,
    pub due_on: NaiveDate,
    pub overdue: bool,
    pub planned_minutes: Option<u32>,
    /// The customer's primary contact.
    pub contact: Option<RecapContact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecapContact {
    pub name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
}

impl MaintenanceRecap {
    /// A made-up recap for the week starting `week_start`, spread over three
    /// towns, one of them with an accent and hyphens.
    pub fn sample(week_start: NaiveDate) -> Self {
        let before = |days: u64| {
            week_start
                .checked_sub_days(Days::new(days))
                .unwrap_or(week_start)
        };
        let after = |days: u64| {
            week_start
                .checked_add_days(Days::new(days))
                .unwrap_or(week_start)
        };
        let contact = |name: &str, phone: &str| {
            Some(RecapContact {
                name: name.to_owned(),
                phone: Some(phone.to_owned()),
                email: None,
            })
        };
        let row = |reference: &str,
                   site: &str,
                   town: &str,
                   line: &str,
                   last_visit: NaiveDate,
                   due_on: NaiveDate,
                   minutes: u32,
                   contact: Option<RecapContact>| RecapRow {
            reference: Some(reference.to_owned()),
            site: site.to_owned(),
            town: town.to_owned(),
            line: line.to_owned(),
            last_visit: Some(last_visit),
            due_on,
            overdue: due_on < week_start,
            planned_minutes: Some(minutes),
            contact,
        };
        Self {
            week_start,
            generated_on: before(4),
            rows: vec![
                row(
                    "C25029",
                    "Domaine Gaume",
                    "Saint-Pourçain-sur-Sioule",
                    "Tonte avec ramassage",
                    before(18),
                    after(1),
                    120,
                    contact("Claire Gaume", "04 70 45 12 30"),
                ),
                row(
                    "C25029",
                    "Domaine Gaume",
                    "Saint-Pourçain-sur-Sioule",
                    "Taille des haies",
                    before(50),
                    before(6),
                    240,
                    contact("Claire Gaume", "04 70 45 12 30"),
                ),
                row(
                    "C25114",
                    "Résidence Les Acacias",
                    "Vichy",
                    "Tonte avec ramassage",
                    before(13),
                    after(1),
                    90,
                    contact("Syndic Bonnet", "04 70 98 00 11"),
                ),
                row(
                    "C25131",
                    "Clinique du Parc",
                    "Cusset",
                    "Désherbage des massifs",
                    before(27),
                    after(3),
                    150,
                    None,
                ),
            ],
        }
    }

    fn iso_label(&self) -> String {
        let week = self.week_start.iso_week();
        format!("S{} {}", week.week(), week.year())
    }

    fn span_label(&self) -> String {
        let end = self
            .week_start
            .checked_add_days(Days::new(6))
            .unwrap_or(self.week_start);
        let month = |date: NaiveDate| MONTHS[date.month0() as usize];
        if self.week_start.year() != end.year() {
            format!(
                "du {} {} {} au {} {} {}",
                self.week_start.day(),
                month(self.week_start),
                self.week_start.year(),
                end.day(),
                month(end),
                end.year()
            )
        } else if self.week_start.month() != end.month() {
            format!(
                "du {} {} au {} {} {}",
                self.week_start.day(),
                month(self.week_start),
                end.day(),
                month(end),
                end.year()
            )
        } else {
            format!(
                "du {} au {} {} {}",
                self.week_start.day(),
                end.day(),
                month(end),
                end.year()
            )
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TypstRecapRenderer;

impl TypstRecapRenderer {
    pub fn render(&self, recap: &MaintenanceRecap) -> Result<Vec<u8>, DoeRenderError> {
        let prepared = prepare(recap)?;
        DoeWorld::with_template(TEMPLATE, prepared).compile_pdf()
    }
}

#[derive(Serialize)]
struct TownGroup<'a> {
    town: &'a str,
    rows: Vec<RowView<'a>>,
}

#[derive(Serialize)]
struct RowView<'a> {
    reference: Option<&'a str>,
    site: &'a str,
    line: &'a str,
    last_visit: Option<String>,
    due_on: String,
    overdue: bool,
    planned: Option<String>,
    contact: Option<ContactView<'a>>,
}

#[derive(Serialize)]
struct ContactView<'a> {
    name: &'a str,
    phone: Option<&'a str>,
    email: Option<&'a str>,
}

fn prepare(recap: &MaintenanceRecap) -> Result<Prepared, DoeRenderError> {
    let groups: Vec<TownGroup<'_>> = group_by_town(&recap.rows)
        .into_iter()
        .map(|(town, rows)| TownGroup {
            town,
            rows: rows.into_iter().map(row_view).collect(),
        })
        .collect();
    let overdue = recap.rows.iter().filter(|row| row.overdue).count();
    let data = json!({
        "week": recap.iso_label(),
        "span": recap.span_label(),
        "generated": french_date(recap.generated_on),
        "total": recap.rows.len(),
        "overdue": overdue,
        "groups": groups,
    });
    let data = serde_json::to_vec(&data)
        .map_err(|e| DoeRenderError::Typeset(format!("recap is not serializable: {e}")))?;
    Ok(Prepared {
        data,
        files: Vec::new(),
    })
}

/// Towns holding an overdue line first, then alphabetical; inside a town,
/// overdue lines first, then by due date.
fn group_by_town(rows: &[RecapRow]) -> Vec<(&str, Vec<&RecapRow>)> {
    let mut groups: Vec<(&str, Vec<&RecapRow>)> = Vec::new();
    for row in rows {
        let town = town_of(row);
        match groups.iter_mut().find(|(name, _)| *name == town) {
            Some((_, members)) => members.push(row),
            None => groups.push((town, vec![row])),
        }
    }
    for (_, members) in &mut groups {
        members.sort_by(|a, b| {
            b.overdue
                .cmp(&a.overdue)
                .then(a.due_on.cmp(&b.due_on))
                .then_with(|| a.site.cmp(&b.site))
                .then_with(|| a.line.cmp(&b.line))
        });
    }
    groups.sort_by(|(town_a, rows_a), (town_b, rows_b)| {
        let overdue = |rows: &[&RecapRow]| rows.iter().any(|row| row.overdue);
        overdue(rows_b)
            .cmp(&overdue(rows_a))
            .then_with(|| compare_towns(town_a, town_b))
    });
    groups
}

fn town_of(row: &RecapRow) -> &str {
    let town = row.town.trim();
    if town.is_empty() { NO_TOWN } else { town }
}

fn compare_towns(a: &str, b: &str) -> Ordering {
    a.to_lowercase().cmp(&b.to_lowercase())
}

fn row_view(row: &RecapRow) -> RowView<'_> {
    RowView {
        reference: row.reference.as_deref().filter(|r| !r.is_empty()),
        site: &row.site,
        line: &row.line,
        last_visit: row.last_visit.map(french_date),
        due_on: french_date(row.due_on),
        overdue: row.overdue,
        planned: row.planned_minutes.map(duration_label),
        contact: row.contact.as_ref().map(|contact| ContactView {
            name: &contact.name,
            phone: contact.phone.as_deref().filter(|p| !p.is_empty()),
            email: contact.email.as_deref().filter(|e| !e.is_empty()),
        }),
    }
}

fn french_date(date: NaiveDate) -> String {
    date.format("%d/%m/%Y").to_string()
}

fn duration_label(minutes: u32) -> String {
    match (minutes / 60, minutes % 60) {
        (0, minutes) => format!("{minutes} min"),
        (hours, 0) => format!("{hours} h"),
        (hours, minutes) => format!("{hours} h {minutes:02}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn row(site: &str, town: &str, overdue: bool, due_on: NaiveDate) -> RecapRow {
        RecapRow {
            reference: None,
            site: site.to_owned(),
            town: town.to_owned(),
            line: "Tonte".to_owned(),
            last_visit: None,
            due_on,
            overdue,
            planned_minutes: None,
            contact: None,
        }
    }

    #[test]
    fn towns_with_an_overdue_line_come_first_then_alphabetical() {
        let rows = [
            row("a", "Vichy", false, date(2026, 10, 20)),
            row("b", "Cusset", false, date(2026, 10, 20)),
            row("c", "Saint-Yorre", true, date(2026, 10, 12)),
            row("d", "abrest", false, date(2026, 10, 20)),
        ];

        let towns: Vec<&str> = group_by_town(&rows).iter().map(|(t, _)| *t).collect();

        assert_eq!(towns, ["Saint-Yorre", "abrest", "Cusset", "Vichy"]);
    }

    #[test]
    fn inside_a_town_overdue_lines_come_first_then_by_due_date() {
        let rows = [
            row("late in week", "Vichy", false, date(2026, 10, 23)),
            row("early in week", "Vichy", false, date(2026, 10, 20)),
            row("overdue", "Vichy", true, date(2026, 10, 12)),
        ];

        let groups = group_by_town(&rows);

        let sites: Vec<&str> = groups[0].1.iter().map(|r| r.site.as_str()).collect();
        assert_eq!(sites, ["overdue", "early in week", "late in week"]);
    }

    #[test]
    fn a_blank_town_is_grouped_under_a_placeholder() {
        let rows = [
            row("a", " ", false, date(2026, 10, 20)),
            row("b", "", false, date(2026, 10, 20)),
        ];

        let groups = group_by_town(&rows);

        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, NO_TOWN);
    }

    #[test]
    fn durations_read_in_hours_and_minutes() {
        assert_eq!(duration_label(45), "45 min");
        assert_eq!(duration_label(120), "2 h");
        assert_eq!(duration_label(90), "1 h 30");
        assert_eq!(duration_label(65), "1 h 05");
    }

    #[test]
    fn the_week_label_follows_the_iso_year() {
        let recap = MaintenanceRecap {
            week_start: date(2026, 12, 28),
            generated_on: date(2026, 12, 24),
            rows: Vec::new(),
        };

        assert_eq!(recap.iso_label(), "S53 2026");
        assert_eq!(recap.span_label(), "du 28 décembre 2026 au 3 janvier 2027");
    }

    #[test]
    fn the_span_names_one_month_when_the_week_stays_in_it() {
        let recap = MaintenanceRecap {
            week_start: date(2026, 10, 19),
            generated_on: date(2026, 10, 15),
            rows: Vec::new(),
        };

        assert_eq!(recap.span_label(), "du 19 au 25 octobre 2026");
    }
}
