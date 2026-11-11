use std::{borrow::Cow, collections::HashSet, ops::RangeInclusive};

use chrono::NaiveDate;
use encoding_rs::{Encoding, WINDOWS_1252};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::domain::knowledge::tables::{
    MAX_COLUMNS, ResolvedRow, SelectOptionId, SelectOptions, TableColumn, TableColumnId,
    TableError, TableItemName,
    column::{ColumnType, Decimals, MAX_DECIMALS, MAX_TEXT_CHARS},
    column_named,
    name::MAX_NAME_CHARS,
    value::{CellError, CellValue, RowValues},
};

pub const MAX_CSV_BYTES: usize = 5 * 1024 * 1024;
pub const MAX_CSV_ROWS: usize = 10_000;
pub const MAX_LISTED_INVALID_CELLS: usize = 200;
pub const MAX_REPORTED_VALUE_CHARS: usize = 100;
pub const MULTI_SELECT_SEPARATOR: char = ',';
pub const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";

pub fn decode(bytes: &[u8]) -> Cow<'_, str> {
    if let Some((encoding, length)) = Encoding::for_bom(bytes) {
        return encoding
            .decode_without_bom_handling(bytes.get(length..).unwrap_or_default())
            .0;
    }
    match std::str::from_utf8(bytes) {
        Ok(text) => Cow::Borrowed(text),
        Err(_) => WINDOWS_1252.decode_without_bom_handling(bytes).0,
    }
}

pub fn delimiter(text: &str) -> u8 {
    const CANDIDATES: [char; 3] = [';', ',', '\t'];
    let mut counts = [0usize; 3];
    let mut quoted = false;
    for character in text.chars() {
        match character {
            '"' => quoted = !quoted,
            '\n' | '\r' if !quoted => break,
            _ if !quoted => {
                if let Some(index) = CANDIDATES.iter().position(|&c| c == character) {
                    counts[index] += 1;
                }
            }
            _ => {}
        }
    }
    let mut best = 0;
    for index in 1..CANDIDATES.len() {
        if counts[index] > counts[best] {
            best = index;
        }
    }
    u8::try_from(CANDIDATES[best]).unwrap_or(b';')
}

fn collapsed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvRow {
    pub line: u64,
    pub cells: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CsvSheet {
    pub headers: Vec<String>,
    pub rows: Vec<CsvRow>,
}

impl CsvSheet {
    pub fn read(bytes: &[u8]) -> Result<Self, TableError> {
        if bytes.len() > MAX_CSV_BYTES {
            return Err(TableError::CsvTooLarge);
        }
        let text = decode(bytes);
        let mut reader = ::csv::ReaderBuilder::new()
            .delimiter(delimiter(&text))
            .has_headers(false)
            .flexible(true)
            .from_reader(text.as_bytes());
        let mut records = reader.records();
        let header = records
            .next()
            .ok_or(TableError::CsvEmpty)?
            .map_err(|_| TableError::CsvUnreadable)?;
        let headers: Vec<String> = header.iter().map(collapsed).collect();
        let width = headers.len();
        let mut rows = Vec::new();
        for (ordinal, record) in (2..).zip(records) {
            let record = record.map_err(|_| TableError::CsvUnreadable)?;
            if record.iter().all(|field| field.trim().is_empty()) {
                continue;
            }
            if rows.len() == MAX_CSV_ROWS {
                return Err(TableError::CsvTooManyRows);
            }
            rows.push(CsvRow {
                line: ordinal,
                cells: (0..width)
                    .map(|index| record.get(index).unwrap_or_default().to_owned())
                    .collect(),
            });
        }
        let kept: Vec<bool> = headers
            .iter()
            .enumerate()
            .map(|(index, header)| {
                !header.is_empty() || rows.iter().any(|row| !row.cells[index].trim().is_empty())
            })
            .collect();
        if !kept.contains(&true) {
            return Err(TableError::CsvEmpty);
        }
        let keep = |cells: Vec<String>| -> Vec<String> {
            cells
                .into_iter()
                .zip(&kept)
                .filter_map(|(cell, kept)| kept.then_some(cell))
                .collect()
        };
        Ok(Self {
            headers: keep(headers),
            rows: rows
                .into_iter()
                .map(|row| CsvRow {
                    line: row.line,
                    cells: keep(row.cells),
                })
                .collect(),
        })
    }

    fn filled(&self, index: usize) -> Vec<&str> {
        self.rows
            .iter()
            .filter_map(|row| row.cells.get(index))
            .map(|cell| cell.trim())
            .filter(|cell| !cell.is_empty())
            .collect()
    }
}

fn shown_header(index: usize, header: &str) -> String {
    if header.is_empty() {
        format!("Colonne {}", index + 1)
    } else {
        header.to_owned()
    }
}

fn column_names(headers: &[String]) -> Vec<String> {
    let mut taken: HashSet<String> = HashSet::new();
    headers
        .iter()
        .enumerate()
        .map(|(index, header)| {
            let base: String = shown_header(index, header)
                .chars()
                .take(MAX_NAME_CHARS)
                .collect::<String>()
                .trim_end()
                .to_owned();
            let mut name = base.clone();
            let mut rank = 2;
            while taken.contains(&name.to_lowercase()) {
                let suffix = format!(" ({rank})");
                let room = MAX_NAME_CHARS.saturating_sub(suffix.chars().count());
                let kept: String = base.chars().take(room).collect();
                name = format!("{}{suffix}", kept.trim_end());
                rank += 1;
            }
            taken.insert(name.to_lowercase());
            name
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnPlan {
    pub name: TableItemName,
    pub column_type: ColumnType,
}

pub fn plan_columns(sheet: &CsvSheet) -> Result<Vec<ColumnPlan>, TableError> {
    if sheet.headers.len() > MAX_COLUMNS {
        return Err(TableError::CsvTooManyColumns);
    }
    column_names(&sheet.headers)
        .iter()
        .enumerate()
        .map(|(index, name)| {
            Ok(ColumnPlan {
                name: TableItemName::try_from(name.as_str())?,
                column_type: if index == 0 {
                    ColumnType::Text
                } else {
                    guess(&sheet.filled(index))
                },
            })
        })
        .collect()
}

fn guess(values: &[&str]) -> ColumnType {
    if values.is_empty() {
        return ColumnType::Text;
    }
    if values
        .iter()
        .any(|value| value.chars().count() > MAX_TEXT_CHARS)
    {
        return ColumnType::LongText;
    }
    if values.iter().all(|value| checkbox_word(value).is_some()) {
        return ColumnType::Checkbox;
    }
    if values.iter().any(|value| value.trim_end().ends_with('€'))
        && values.iter().all(|value| cents(value).is_some())
    {
        return ColumnType::Currency;
    }
    if let Some(figures) = values
        .iter()
        .map(|value| Figure::read(value))
        .collect::<Option<Vec<_>>>()
    {
        let decimals = figures
            .iter()
            .map(|figure| figure.fraction.len())
            .max()
            .unwrap_or_default()
            .min(usize::from(MAX_DECIMALS));
        return u8::try_from(decimals)
            .ok()
            .and_then(|decimals| Decimals::try_from(decimals).ok())
            .map_or(ColumnType::Text, ColumnType::Number);
    }
    if values.iter().all(|value| date(value).is_some()) {
        return ColumnType::Date;
    }
    ColumnType::Text
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Figure {
    negative: bool,
    whole: String,
    fraction: String,
}

fn grouped(integer: &str, separator: char) -> Option<String> {
    let mut groups = integer.split(separator);
    let first = groups.next()?;
    if first.is_empty() || first.len() > 3 {
        return None;
    }
    let mut whole = first.to_owned();
    for group in groups {
        if group.len() != 3 {
            return None;
        }
        whole.push_str(group);
    }
    Some(whole)
}

impl Figure {
    fn read(written: &str) -> Option<Self> {
        let compact: String = written
            .chars()
            .filter(|c| !matches!(c, ' ' | '\u{a0}' | '\u{202f}'))
            .collect();
        let (negative, body) = match compact.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, compact.strip_prefix('+').unwrap_or(&compact)),
        };
        if body.is_empty()
            || !body
                .chars()
                .all(|c| c.is_ascii_digit() || c == ',' || c == '.')
        {
            return None;
        }
        let commas = body.matches(',').count();
        let dots = body.matches('.').count();
        let (grouping, mark) = match (commas, dots) {
            (0, 0) => (None, None),
            (1, 0) => (None, Some(',')),
            (0, 1) => (None, Some('.')),
            (_, 0) => (Some(','), None),
            (0, _) => (Some('.'), None),
            (1, _) if body.rfind(',') > body.rfind('.') => (Some('.'), Some(',')),
            (_, 1) if body.rfind('.') > body.rfind(',') => (Some(','), Some('.')),
            _ => return None,
        };
        let (integer, fraction) = match mark {
            Some(mark) => body.rsplit_once(mark)?,
            None => (body, ""),
        };
        let whole = match grouping {
            Some(separator) => grouped(integer, separator)?,
            None => integer.to_owned(),
        };
        if !whole.chars().all(|c| c.is_ascii_digit())
            || !fraction.chars().all(|c| c.is_ascii_digit())
            || (whole.is_empty() && fraction.is_empty())
        {
            return None;
        }
        Some(Self {
            negative,
            whole: if whole.is_empty() {
                "0".to_owned()
            } else {
                whole
            },
            fraction: fraction.to_owned(),
        })
    }

    fn to_json(&self) -> Option<Value> {
        if self.fraction.chars().all(|c| c == '0')
            && let Ok(whole) = self.whole.parse::<i64>()
        {
            return Some(Value::from(if self.negative { -whole } else { whole }));
        }
        let sign = if self.negative { "-" } else { "" };
        let fraction = if self.fraction.is_empty() {
            "0"
        } else {
            &self.fraction
        };
        let real: f64 = format!("{sign}{}.{fraction}", self.whole).parse().ok()?;
        serde_json::Number::from_f64(real).map(Value::Number)
    }

    fn cents(&self) -> Option<i64> {
        if self.fraction.len() > 2 {
            return None;
        }
        let whole: i64 = self.whole.parse().ok()?;
        let fraction: i64 = format!("{:0<2}", self.fraction).parse().ok()?;
        let cents = whole.checked_mul(100)?.checked_add(fraction)?;
        Some(if self.negative { -cents } else { cents })
    }
}

fn cents(written: &str) -> Option<i64> {
    let without_symbol = written.replace('€', "");
    let trimmed = without_symbol.trim();
    let amount = trimmed
        .len()
        .checked_sub(3)
        .and_then(|start| trimmed.get(start..).map(|tail| (start, tail)))
        .filter(|(_, tail)| tail.eq_ignore_ascii_case("eur"))
        .and_then(|(start, _)| trimmed.get(..start))
        .unwrap_or(trimmed);
    Figure::read(amount.trim())?.cents()
}

fn digits(part: &str, lengths: RangeInclusive<usize>) -> Option<u32> {
    if lengths.contains(&part.len()) && part.bytes().all(|byte| byte.is_ascii_digit()) {
        part.parse().ok()
    } else {
        None
    }
}

fn three(written: &str, separator: char) -> Option<(&str, &str, &str)> {
    let mut parts = written.split(separator);
    let found = (parts.next()?, parts.next()?, parts.next()?);
    parts.next().is_none().then_some(found)
}

fn date(written: &str) -> Option<NaiveDate> {
    let written = written.trim();
    let (year, month, day) = if let Some((day, month, year)) = three(written, '/') {
        (
            digits(year, 4..=4)?,
            digits(month, 1..=2)?,
            digits(day, 1..=2)?,
        )
    } else {
        let (year, month, day) = three(written, '-')?;
        (
            digits(year, 4..=4)?,
            digits(month, 2..=2)?,
            digits(day, 2..=2)?,
        )
    };
    NaiveDate::from_ymd_opt(i32::try_from(year).ok()?, month, day)
}

fn checkbox_word(written: &str) -> Option<bool> {
    match written.trim().to_lowercase().as_str() {
        "oui" | "vrai" | "x" | "true" => Some(true),
        "non" | "faux" | "false" => Some(false),
        _ => None,
    }
}

fn checked(written: &str) -> Option<bool> {
    checkbox_word(written).or(match written.trim() {
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    })
}

fn choices(options: &SelectOptions, written: &str) -> Result<Vec<SelectOptionId>, CellError> {
    if let Some(id) = options.resolve(written) {
        return Ok(vec![id]);
    }
    let mut ids = Vec::new();
    for label in written
        .split(MULTI_SELECT_SEPARATOR)
        .map(str::trim)
        .filter(|label| !label.is_empty())
    {
        let id = options.resolve(label).ok_or(CellError::UnknownOption)?;
        if !ids.contains(&id) {
            ids.push(id);
        }
    }
    Ok(ids)
}

fn id_of(written: &str, error: CellError) -> Result<Uuid, CellError> {
    written.parse().map_err(|_| error)
}

pub fn cell_from_text(
    column_type: &ColumnType,
    written: &str,
) -> Result<Option<CellValue>, CellError> {
    let trimmed = written.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    let candidate = match column_type {
        ColumnType::Text | ColumnType::LongText => Value::String(trimmed.to_owned()),
        ColumnType::Number(_) => Figure::read(trimmed)
            .and_then(|figure| figure.to_json())
            .ok_or(CellError::NotNumber)?,
        ColumnType::Currency => Value::from(cents(trimmed).ok_or(CellError::NotCents)?),
        ColumnType::Date => Value::String(
            date(trimmed)
                .ok_or(CellError::NotDate)?
                .format("%Y-%m-%d")
                .to_string(),
        ),
        ColumnType::Checkbox => Value::Bool(checked(trimmed).ok_or(CellError::NotBoolean)?),
        ColumnType::SingleSelect(options) => Value::String(
            options
                .resolve(trimmed)
                .ok_or(CellError::UnknownOption)?
                .to_string(),
        ),
        ColumnType::MultiSelect(options) => Value::Array(
            choices(options, trimmed)?
                .iter()
                .map(|id| Value::String(id.to_string()))
                .collect(),
        ),
        ColumnType::Link(kind) => {
            json!({ "kind": kind.as_str(), "id": id_of(trimmed, CellError::NotLink)? })
        }
        ColumnType::File | ColumnType::Member => {
            Value::String(id_of(trimmed, CellError::NotId)?.to_string())
        }
    };
    CellValue::parse(column_type, &candidate)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnMapping {
    targets: Vec<Option<TableColumnId>>,
    ignored: Vec<String>,
}

impl ColumnMapping {
    pub fn by_name(sheet: &CsvSheet, columns: &[TableColumn]) -> Result<Self, TableError> {
        let mut used = HashSet::new();
        let mut targets = Vec::with_capacity(sheet.headers.len());
        let mut ignored = Vec::new();
        for (index, header) in sheet.headers.iter().enumerate() {
            match column_named(columns, header).filter(|column| used.insert(column.id)) {
                Some(column) => targets.push(Some(column.id)),
                None => {
                    targets.push(None);
                    ignored.push(shown_header(index, header));
                }
            }
        }
        if targets.iter().all(Option::is_none) {
            return Err(TableError::CsvNoKnownColumn);
        }
        Ok(Self { targets, ignored })
    }

    pub fn by_position(ids: &[TableColumnId]) -> Self {
        Self {
            targets: ids.iter().copied().map(Some).collect(),
            ignored: Vec::new(),
        }
    }

    pub fn ignored(&self) -> &[String] {
        &self.ignored
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InvalidCsvCell {
    pub row: u64,
    pub column: String,
    pub value: String,
    pub reason: CellError,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub rows_created: u64,
    pub ignored_columns: Vec<String>,
    pub invalid_cells: Vec<InvalidCsvCell>,
    pub invalid_cell_count: u64,
}

impl ImportReport {
    pub fn new(
        rows_created: u64,
        ignored_columns: Vec<String>,
        mut invalid_cells: Vec<InvalidCsvCell>,
    ) -> Self {
        invalid_cells.sort_by_key(|cell| cell.row);
        let invalid_cell_count = u64::try_from(invalid_cells.len()).unwrap_or(u64::MAX);
        invalid_cells.truncate(MAX_LISTED_INVALID_CELLS);
        for cell in &mut invalid_cells {
            if cell.value.chars().count() > MAX_REPORTED_VALUE_CHARS {
                cell.value = cell.value.chars().take(MAX_REPORTED_VALUE_CHARS).collect();
            }
        }
        Self {
            rows_created,
            ignored_columns,
            invalid_cells,
            invalid_cell_count,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedRow {
    pub line: u64,
    pub values: RowValues,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportPlan {
    pub rows: Vec<PlannedRow>,
    pub invalid_cells: Vec<InvalidCsvCell>,
}

pub fn convert(sheet: &CsvSheet, columns: &[TableColumn], mapping: &ColumnMapping) -> ImportPlan {
    let targets: Vec<Option<&TableColumn>> = mapping
        .targets
        .iter()
        .map(|target| target.and_then(|id| columns.iter().find(|column| column.id == id)))
        .collect();
    let mut invalid_cells = Vec::new();
    let mut rows = Vec::with_capacity(sheet.rows.len());
    for row in &sheet.rows {
        let mut values = RowValues::default();
        for (cell, target) in row.cells.iter().zip(&targets) {
            let Some(column) = target else {
                continue;
            };
            match cell_from_text(&column.column_type, cell) {
                Ok(Some(value)) => values.insert(column.id, value),
                Ok(None) => {}
                Err(reason) => invalid_cells.push(InvalidCsvCell {
                    row: row.line,
                    column: column.name.as_str().to_owned(),
                    value: cell.trim().to_owned(),
                    reason,
                }),
            }
        }
        rows.push(PlannedRow {
            line: row.line,
            values,
        });
    }
    ImportPlan {
        rows,
        invalid_cells,
    }
}

fn sheet_amount(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let magnitude = cents.unsigned_abs();
    format!("{sign}{},{:02}", magnitude / 100, magnitude % 100)
}

fn defused(text: String) -> String {
    if text.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{text}")
    } else {
        text
    }
}

fn exported(column: &TableColumn, row: &ResolvedRow) -> String {
    match column.column_type {
        ColumnType::Currency => row
            .row
            .values
            .get(&column.key())
            .and_then(Value::as_i64)
            .map(sheet_amount),
        ColumnType::Text | ColumnType::LongText => row.shown(column).map(defused),
        _ => row.shown(column),
    }
    .unwrap_or_default()
}

pub fn write(columns: &[&TableColumn], rows: &[ResolvedRow]) -> Result<Vec<u8>, ::csv::Error> {
    let mut writer = ::csv::WriterBuilder::new()
        .delimiter(b';')
        .terminator(::csv::Terminator::CRLF)
        .from_writer(UTF8_BOM.to_vec());
    writer.write_record(columns.iter().map(|column| column.name.as_str()))?;
    for row in rows {
        writer.write_record(columns.iter().map(|column| exported(column, row)))?;
    }
    writer
        .into_inner()
        .map_err(|error| ::csv::Error::from(error.into_error()))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use chrono::Utc;
    use common::generate_uuid_v7;
    use serde_json::{Map, Value, json};

    use super::*;
    use crate::{
        KnowledgeResourceType, OrganizationId,
        domain::knowledge::tables::{
            CellLabel, KnowledgeTableId, SelectOptionInput, SelectOptions, TableRow, TableRowId,
            column::Decimals,
        },
    };

    fn sheet(text: &str) -> CsvSheet {
        CsvSheet::read(text.as_bytes()).unwrap()
    }

    fn column(name: &str, column_type: ColumnType) -> TableColumn {
        TableColumn {
            id: TableColumnId(generate_uuid_v7()),
            table_id: KnowledgeTableId(generate_uuid_v7()),
            organization_id: OrganizationId(generate_uuid_v7()),
            name: TableItemName::try_from(name).unwrap(),
            column_type,
            position: 0,
            is_primary: false,
            created_at: Utc::now(),
        }
    }

    fn select(labels: &[&str]) -> SelectOptions {
        SelectOptions::parse(
            &labels
                .iter()
                .map(|label| SelectOptionInput {
                    id: None,
                    label: (*label).to_owned(),
                    color: None,
                })
                .collect::<Vec<_>>(),
        )
        .unwrap()
    }

    fn json_of(column_type: &ColumnType, written: &str) -> Result<Option<Value>, CellError> {
        cell_from_text(column_type, written).map(|cell| cell.map(CellValue::into_json))
    }

    fn number(decimals: u8) -> ColumnType {
        ColumnType::Number(Decimals::try_from(decimals).unwrap())
    }

    #[test]
    fn utf8_with_or_without_bom_and_windows_1252_decode_to_the_same_text() {
        assert_eq!(decode("Désignation".as_bytes()), "Désignation");
        assert_eq!(decode(b"\xEF\xBB\xBFD\xC3\xA9signation"), "Désignation");
        assert_eq!(decode(b"D\xE9signation;Qt\xE9;\x80"), "Désignation;Qté;€");
        assert_eq!(
            decode(b"\xFF\xFEN\x00\xE9\x00"),
            "Né",
            "UTF-16 with its byte order mark"
        );
    }

    #[test]
    fn the_delimiter_is_the_most_frequent_separator_of_the_header_outside_quotes() {
        assert_eq!(delimiter("Nom;Prix;Date\r\n1,5;2,5;x"), b';');
        assert_eq!(delimiter("Nom,Prix,Date\n"), b',');
        assert_eq!(delimiter("Nom\tPrix\tDate\n"), b'\t');
        assert_eq!(delimiter("\"Nom; complet\",Prix,Date\n"), b',');
        assert_eq!(delimiter("Nom\n"), b';');
    }

    #[test]
    fn a_french_excel_export_reads_with_its_lines_and_without_blank_records() {
        let read = CsvSheet::read(
            b"\xEF\xBB\xBFD\xC3\xA9signation;Prix;\r\n\"Vis; inox\";12,50;\r\n;;\r\n\r\nClou;0,10;\r\n",
        )
        .unwrap();
        assert_eq!(read.headers, vec!["Désignation", "Prix"]);
        assert_eq!(
            read.rows,
            vec![
                CsvRow {
                    line: 2,
                    cells: vec!["Vis; inox".to_owned(), "12,50".to_owned()],
                },
                CsvRow {
                    line: 4,
                    cells: vec!["Clou".to_owned(), "0,10".to_owned()],
                },
            ]
        );
    }

    #[test]
    fn a_short_record_reads_as_empty_cells() {
        let read = sheet("A;B;C\nx\n");
        assert_eq!(read.rows[0].cells, vec!["x", "", ""]);
    }

    #[test]
    fn an_empty_file_is_refused() {
        assert_eq!(CsvSheet::read(b""), Err(TableError::CsvEmpty));
        assert_eq!(
            CsvSheet::read(b"\xEF\xBB\xBF\r\n"),
            Err(TableError::CsvEmpty)
        );
    }

    #[test]
    fn a_file_over_five_megabytes_or_ten_thousand_rows_is_refused() {
        let large = vec![b'x'; MAX_CSV_BYTES + 1];
        assert_eq!(CsvSheet::read(&large), Err(TableError::CsvTooLarge));

        let rows = |count: usize| {
            let mut text = String::from("Nom\n");
            for n in 0..count {
                text.push_str(&format!("{n}\n"));
            }
            text
        };
        assert_eq!(
            CsvSheet::read(rows(MAX_CSV_ROWS).as_bytes())
                .unwrap()
                .rows
                .len(),
            MAX_CSV_ROWS
        );
        assert_eq!(
            CsvSheet::read(rows(MAX_CSV_ROWS + 1).as_bytes()),
            Err(TableError::CsvTooManyRows)
        );
    }

    fn planned(text: &str) -> Vec<(String, ColumnType)> {
        plan_columns(&sheet(text))
            .unwrap()
            .into_iter()
            .map(|plan| (plan.name.into_inner(), plan.column_type))
            .collect()
    }

    #[test]
    fn a_new_table_guesses_number_date_checkbox_and_text() {
        let guessed = planned(
            "Réf;Quantité;Prix;Livré le;Commandé;Fait;Note\n\
             12;12,50;1 250,5;05/11/2026;2026-11-05;oui;à voir\n\
             13;3;1\u{a0}000;5/1/2026;2026-01-31;x;\n\
             14;;-7.25;;;Non;12\n",
        );
        assert_eq!(
            guessed,
            vec![
                ("Réf".to_owned(), ColumnType::Text),
                ("Quantité".to_owned(), number(2)),
                ("Prix".to_owned(), number(2)),
                ("Livré le".to_owned(), ColumnType::Date),
                ("Commandé".to_owned(), ColumnType::Date),
                ("Fait".to_owned(), ColumnType::Checkbox),
                ("Note".to_owned(), ColumnType::Text),
            ]
        );
    }

    #[test]
    fn amounts_with_a_euro_sign_make_a_currency_column() {
        let guessed = planned(
            "Nom;Montant;Prix;Total;Mixte\n\
             a;1 250,50 €;12 €;3;12,345 €\n\
             b;3\u{a0}€;7,5;4,5;x €\n\
             c;99,9;;;\n",
        );
        assert_eq!(guessed[1].1, ColumnType::Currency);
        assert_eq!(guessed[2].1, ColumnType::Currency);
        assert_eq!(guessed[3].1, number(1));
        assert_eq!(guessed[4].1, ColumnType::Text);
    }

    #[test]
    fn a_long_value_makes_a_long_text_and_an_empty_column_a_text() {
        let long = "x".repeat(1_001);
        let guessed = planned(&format!("Nom;Description;Vide\nA;{long};\nB;court;\n"));
        assert_eq!(guessed[1].1, ColumnType::LongText);
        assert_eq!(guessed[2].1, ColumnType::Text);
    }

    #[test]
    fn decimals_follow_the_longest_fraction_up_to_four() {
        assert_eq!(planned("Nom;N\na;1\nb;2\n")[1].1, number(0));
        assert_eq!(planned("Nom;N\na;1,123456\n")[1].1, number(4));
    }

    #[test]
    fn headers_are_cleaned_numbered_and_deduplicated() {
        let long = "y".repeat(120);
        let names: Vec<String> = planned(&format!(
            "Statut;;statut;\"Prix\n  HT\";{long};{long}\na;b;c;d;e;f\n"
        ))
        .into_iter()
        .map(|(name, _)| name)
        .collect();
        assert_eq!(names[0], "Statut");
        assert_eq!(names[1], "Colonne 2");
        assert_eq!(names[2], "statut (2)");
        assert_eq!(names[3], "Prix HT");
        assert_eq!(names[4], "y".repeat(100));
        assert_eq!(names[5], format!("{} (2)", "y".repeat(96)));
    }

    #[test]
    fn more_than_fifty_columns_cannot_become_a_table() {
        let header = |count: usize| {
            (0..count)
                .map(|n| format!("C{n}"))
                .collect::<Vec<_>>()
                .join(";")
        };
        assert_eq!(plan_columns(&sheet(&header(50))).unwrap().len(), 50);
        assert_eq!(
            plan_columns(&sheet(&header(51))),
            Err(TableError::CsvTooManyColumns)
        );
    }

    #[test]
    fn text_is_kept_trimmed_and_bounded() {
        assert_eq!(
            json_of(&ColumnType::Text, "  Villa "),
            Ok(Some(json!("Villa")))
        );
        assert_eq!(json_of(&ColumnType::Text, "   "), Ok(None));
        assert_eq!(
            json_of(&ColumnType::Text, &"x".repeat(1_001)),
            Err(CellError::TextTooLong)
        );
        assert_eq!(
            json_of(&ColumnType::LongText, "ligne 1\nligne 2"),
            Ok(Some(json!("ligne 1\nligne 2")))
        );
    }

    #[test]
    fn a_number_reads_in_french_or_english() {
        let column = number(2);
        for (written, expected) in [
            ("12,50", json!(12.5)),
            ("12.50", json!(12.5)),
            ("1 250,5", json!(1250.5)),
            ("1\u{a0}250,5", json!(1250.5)),
            ("1\u{202f}250", json!(1250)),
            ("1,250.75", json!(1250.75)),
            ("1.250,75", json!(1250.75)),
            ("1,234,567", json!(1_234_567)),
            ("-3", json!(-3)),
            ("+0,5", json!(0.5)),
            ("12,00", json!(12)),
        ] {
            assert_eq!(json_of(&column, written), Ok(Some(expected)), "{written}");
        }
        for wrong in [
            "douze",
            "12,5,0.1",
            "1e5",
            "1,25,0",
            "-",
            "12 €",
            "1.2.3,4,5",
        ] {
            assert_eq!(
                json_of(&column, wrong),
                Err(CellError::NotNumber),
                "{wrong}"
            );
        }
    }

    #[test]
    fn an_amount_reads_euros_into_cents() {
        for (written, cents) in [
            ("1 250,50 €", 125_050),
            ("1250.5", 125_050),
            ("€12", 1_200),
            ("-0,05", -5),
            ("3 EUR", 300),
        ] {
            assert_eq!(
                json_of(&ColumnType::Currency, written),
                Ok(Some(json!(cents))),
                "{written}"
            );
        }
        assert_eq!(
            json_of(&ColumnType::Currency, "12,345"),
            Err(CellError::NotCents)
        );
        assert_eq!(
            json_of(&ColumnType::Currency, "beaucoup"),
            Err(CellError::NotCents)
        );
    }

    #[test]
    fn a_date_reads_in_both_formats_and_must_exist() {
        for written in ["05/11/2026", "5/11/2026", "2026-11-05"] {
            assert_eq!(
                json_of(&ColumnType::Date, written),
                Ok(Some(json!("2026-11-05"))),
                "{written}"
            );
        }
        for wrong in ["31/02/2026", "05/11/26", "2026-2-5", "11/2026", "demain"] {
            assert_eq!(
                json_of(&ColumnType::Date, wrong),
                Err(CellError::NotDate),
                "{wrong}"
            );
        }
    }

    #[test]
    fn a_checkbox_reads_french_and_english_words() {
        for written in ["oui", "OUI", "vrai", "x", "X", "true", "1"] {
            assert_eq!(
                json_of(&ColumnType::Checkbox, written),
                Ok(Some(json!(true))),
                "{written}"
            );
        }
        for written in ["non", "Faux", "false", "0"] {
            assert_eq!(
                json_of(&ColumnType::Checkbox, written),
                Ok(Some(json!(false))),
                "{written}"
            );
        }
        assert_eq!(
            json_of(&ColumnType::Checkbox, "peut-être"),
            Err(CellError::NotBoolean)
        );
    }

    #[test]
    fn a_choice_reads_by_label_and_a_multiple_choice_splits_on_commas() {
        let options = select(&["En cours", "Terminé", "Prix, délais"]);
        let [ongoing, done, both] = [
            options.as_slice()[0].id.to_string(),
            options.as_slice()[1].id.to_string(),
            options.as_slice()[2].id.to_string(),
        ];
        let single = ColumnType::SingleSelect(options.clone());
        assert_eq!(json_of(&single, "en cours"), Ok(Some(json!(ongoing))));
        assert_eq!(json_of(&single, "Annulé"), Err(CellError::UnknownOption));

        let multi = ColumnType::MultiSelect(options);
        assert_eq!(
            json_of(&multi, "terminé, En cours,terminé"),
            Ok(Some(json!([done, ongoing])))
        );
        assert_eq!(json_of(&multi, "Prix, délais"), Ok(Some(json!([both]))));
        assert_eq!(
            json_of(&multi, "En cours, Annulé"),
            Err(CellError::UnknownOption)
        );
    }

    #[test]
    fn a_link_a_file_and_a_member_read_their_id_only() {
        let id = generate_uuid_v7();
        let link = ColumnType::Link(KnowledgeResourceType::Customer);
        assert_eq!(
            json_of(&link, &format!(" {id} ")),
            Ok(Some(json!({"kind": "customer", "id": id})))
        );
        assert_eq!(json_of(&link, "Dupont SARL"), Err(CellError::NotLink));
        assert_eq!(
            json_of(&ColumnType::File, &id.to_string()),
            Ok(Some(json!(id)))
        );
        assert_eq!(
            json_of(&ColumnType::Member, "Léa Martin"),
            Err(CellError::NotId)
        );
    }

    #[test]
    fn columns_are_matched_by_name_without_case_and_the_rest_is_listed() {
        let name = column("Nom", ColumnType::Text);
        let price = column("Prix HT", number(2));
        let columns = [name.clone(), price.clone()];
        let mapping =
            ColumnMapping::by_name(&sheet("NOM;prix  ht;Remarque;nom\n"), &columns).unwrap();
        assert_eq!(
            mapping.targets,
            vec![Some(name.id), Some(price.id), None, None]
        );
        assert_eq!(mapping.ignored(), ["Remarque", "nom"]);
        assert_eq!(
            ColumnMapping::by_name(&sheet("Autre;Encore\n"), &columns),
            Err(TableError::CsvNoKnownColumn)
        );
    }

    #[test]
    fn an_invalid_cell_is_left_empty_and_reported_with_its_line_and_column() {
        let name = column("Nom", ColumnType::Text);
        let start = column("Début", ColumnType::Date);
        let columns = [name.clone(), start.clone()];
        let read = sheet("Nom;Début;Inconnue\nVilla;31/02/2026;x\nGrange;05/11/2026;y\n");
        let mapping = ColumnMapping::by_name(&read, &columns).unwrap();

        let plan = convert(&read, &columns, &mapping);

        assert_eq!(plan.rows.len(), 2);
        assert_eq!(plan.rows[0].line, 2);
        assert_eq!(
            plan.rows[0].values.to_json(),
            Map::from_iter([(name.key(), json!("Villa"))])
        );
        assert_eq!(
            plan.rows[1].values.to_json(),
            Map::from_iter([
                (name.key(), json!("Grange")),
                (start.key(), json!("2026-11-05"))
            ])
        );
        assert_eq!(
            plan.invalid_cells,
            vec![InvalidCsvCell {
                row: 2,
                column: "Début".to_owned(),
                value: "31/02/2026".to_owned(),
                reason: CellError::NotDate,
            }]
        );
    }

    #[test]
    fn a_new_table_maps_its_columns_by_position() {
        let first = column("Statut", ColumnType::Text);
        let second = column("statut (2)", ColumnType::Text);
        let columns = [first.clone(), second.clone()];
        let read = sheet("Statut;statut\nA;B\n");

        let plan = convert(
            &read,
            &columns,
            &ColumnMapping::by_position(&[first.id, second.id]),
        );

        assert_eq!(
            plan.rows[0].values.to_json(),
            Map::from_iter([(first.key(), json!("A")), (second.key(), json!("B"))])
        );
    }

    #[test]
    fn the_report_sorts_counts_and_caps_the_invalid_cells() {
        let cell = |row: u64| InvalidCsvCell {
            row,
            column: "Début".to_owned(),
            value: "x".repeat(150),
            reason: CellError::NotDate,
        };
        let cells: Vec<_> = (0..MAX_LISTED_INVALID_CELLS as u64 + 5)
            .rev()
            .map(cell)
            .collect();
        let report = ImportReport::new(3, vec!["Remarque".to_owned()], cells);
        assert_eq!(report.rows_created, 3);
        assert_eq!(
            report.invalid_cell_count,
            MAX_LISTED_INVALID_CELLS as u64 + 5
        );
        assert_eq!(report.invalid_cells.len(), MAX_LISTED_INVALID_CELLS);
        assert_eq!(report.invalid_cells[0].row, 0);
        assert_eq!(report.invalid_cells[0].value.chars().count(), 100);
        assert_eq!(report.ignored_columns, ["Remarque"]);
    }

    fn resolved(
        values: Map<String, Value>,
        labels: BTreeMap<TableColumnId, CellLabel>,
    ) -> ResolvedRow {
        ResolvedRow {
            row: TableRow {
                id: TableRowId(generate_uuid_v7()),
                table_id: KnowledgeTableId(generate_uuid_v7()),
                organization_id: OrganizationId(generate_uuid_v7()),
                values,
                created_by: None,
                updated_by: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            labels,
        }
    }

    #[test]
    fn an_export_opens_in_a_french_excel() {
        let options = select(&["Urgent", "Client"]);
        let [urgent, client] = [
            options.as_slice()[0].id.to_string(),
            options.as_slice()[1].id.to_string(),
        ];
        let name = column("Nom", ColumnType::Text);
        let quantity = column("Quantité", number(2));
        let amount = column("Montant", ColumnType::Currency);
        let start = column("Début", ColumnType::Date);
        let done = column("Fait", ColumnType::Checkbox);
        let tags = column("Tags", ColumnType::MultiSelect(options));
        let customer = column("Client", ColumnType::Link(KnowledgeResourceType::Customer));
        let columns = [&name, &quantity, &amount, &start, &done, &tags, &customer];
        let rows = [
            resolved(
                Map::from_iter([
                    (name.key(), json!("Villa; \"Dupont\"")),
                    (quantity.key(), json!(12.5)),
                    (amount.key(), json!(125_050)),
                    (start.key(), json!("2026-11-05")),
                    (done.key(), json!(true)),
                    (tags.key(), json!([urgent, client])),
                    (
                        customer.key(),
                        json!({"kind": "customer", "id": generate_uuid_v7()}),
                    ),
                ]),
                BTreeMap::from([(customer.id, CellLabel::Missing)]),
            ),
            resolved(
                Map::from_iter([
                    (name.key(), json!("=HYPERLINK(\"x\")")),
                    (done.key(), json!(false)),
                    (amount.key(), json!(-5)),
                ]),
                BTreeMap::new(),
            ),
        ];

        let written = write(&columns, &rows).unwrap();

        assert!(written.starts_with(UTF8_BOM));
        let text = std::str::from_utf8(&written[UTF8_BOM.len()..]).unwrap();
        assert_eq!(
            text,
            "Nom;Quantité;Montant;Début;Fait;Tags;Client\r\n\
             \"Villa; \"\"Dupont\"\"\";12,5;1250,50;05/11/2026;Oui;Urgent, Client;Élément supprimé\r\n\
             \"'=HYPERLINK(\"\"x\"\")\";;-0,05;;Non;;\r\n"
        );
    }

    #[test]
    fn an_export_reads_back_as_an_import() {
        let name = column("Nom", ColumnType::Text);
        let amount = column("Montant", ColumnType::Currency);
        let start = column("Début", ColumnType::Date);
        let columns = [name.clone(), amount.clone(), start.clone()];
        let row = resolved(
            Map::from_iter([
                (name.key(), json!("Villa")),
                (amount.key(), json!(125_050)),
                (start.key(), json!("2026-11-05")),
            ]),
            BTreeMap::new(),
        );
        let written = write(
            &columns.iter().collect::<Vec<_>>(),
            std::slice::from_ref(&row),
        )
        .unwrap();
        let read = CsvSheet::read(&written).unwrap();
        let plan = convert(
            &read,
            &columns,
            &ColumnMapping::by_name(&read, &columns).unwrap(),
        );
        assert_eq!(plan.rows[0].values.to_json(), row.row.values);
        assert!(plan.invalid_cells.is_empty());
    }
}
