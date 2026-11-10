//! What an article keeps of its text: validation of the typed fields and the
//! text pulled out of an uploaded file so it can be searched.

use super::error::{KnowledgeError, KnowledgeOutcome};

pub const MAX_TITLE_CHARS: usize = 200;
pub const MAX_NOTE_CHARS: usize = 100_000;
pub const MAX_TAGS: usize = 20;
pub const MAX_TAG_CHARS: usize = 40;
pub const MAX_FOLDER_NAME_CHARS: usize = 100;
/// A folder at the root is at depth 1.
pub const MAX_FOLDER_DEPTH: usize = 10;
pub const MAX_MOVED_ARTICLES: usize = 100;
/// Bound on the folder tree sent in one answer.
pub const MAX_LISTED_FOLDERS: u64 = 5_000;
pub const MAX_FILE_BYTES: usize = 25 * 1024 * 1024;
pub const MAX_EXTRACTED_CHARS: usize = 500_000;
/// Bound on what a hostile PDF can inflate to while its text is read.
const MAX_PDF_DECOMPRESSED_BYTES: usize = 100 * 1024 * 1024;

const ACCEPTED_FILE_TYPES: [&str; 8] = [
    "application/pdf",
    "text/plain",
    "text/csv",
    "image/jpeg",
    "image/png",
    "image/webp",
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
];

pub fn normalize_title(raw: &str) -> KnowledgeOutcome<String> {
    let title = raw.trim();
    match title.chars().count() {
        0 => Err(KnowledgeError::TitleEmpty),
        count if count > MAX_TITLE_CHARS => Err(KnowledgeError::TitleTooLong),
        _ => Ok(title.to_owned()),
    }
}

pub fn normalize_folder_name(raw: &str) -> KnowledgeOutcome<String> {
    let name = raw.trim();
    match name.chars().count() {
        0 => Err(KnowledgeError::FolderNameEmpty),
        count if count > MAX_FOLDER_NAME_CHARS => Err(KnowledgeError::FolderNameTooLong),
        _ => Ok(name.to_owned()),
    }
}

pub fn normalize_note(raw: &str) -> KnowledgeOutcome<String> {
    let note = raw.trim();
    match note.chars().count() {
        0 => Err(KnowledgeError::NoteEmpty),
        count if count > MAX_NOTE_CHARS => Err(KnowledgeError::NoteTooLong),
        _ => Ok(note.to_owned()),
    }
}

pub fn normalize_tags<S: AsRef<str>>(raw: &[S]) -> KnowledgeOutcome<Vec<String>> {
    let mut tags: Vec<String> = Vec::with_capacity(raw.len());
    for tag in raw {
        let tag = tag.as_ref().trim().to_lowercase();
        match tag.chars().count() {
            0 => return Err(KnowledgeError::TagEmpty),
            count if count > MAX_TAG_CHARS => return Err(KnowledgeError::TagTooLong),
            _ => {}
        }
        if !tags.contains(&tag) {
            tags.push(tag);
        }
    }
    if tags.len() > MAX_TAGS {
        return Err(KnowledgeError::TooManyTags);
    }
    Ok(tags)
}

/// Checks size and type of an upload. The type comes from the client, so it
/// only gates what is accepted: the bytes are never trusted to be what it says.
pub fn check_file(mime_type: &str, bytes: &[u8]) -> KnowledgeOutcome<()> {
    if bytes.is_empty() {
        return Err(KnowledgeError::EmptyFile);
    }
    if bytes.len() > MAX_FILE_BYTES {
        return Err(KnowledgeError::FileTooLarge);
    }
    if !ACCEPTED_FILE_TYPES.contains(&mime_type) {
        return Err(KnowledgeError::UnsupportedFile);
    }
    Ok(())
}

/// The searchable text of a file, or `None` when there is none to read: a
/// scanned PDF, an image, an office document (not read in this version), or a
/// file whose text cannot be decoded. A file that cannot be read stays
/// searchable by its title and tags, it is never refused for that.
pub fn extract_text(mime_type: &str, bytes: &[u8]) -> Option<String> {
    let text = match mime_type {
        "application/pdf" => pdf_text(bytes)?,
        "text/plain" | "text/csv" => String::from_utf8(bytes.to_vec()).ok()?,
        _ => return None,
    };
    let collapsed = collapse_whitespace(&text);
    if collapsed.is_empty() {
        return None;
    }
    Some(collapsed.chars().take(MAX_EXTRACTED_CHARS).collect())
}

fn pdf_text(bytes: &[u8]) -> Option<String> {
    let document = lopdf::Document::load_mem(bytes).ok()?;
    let pages: Vec<u32> = document.get_pages().keys().copied().collect();
    document
        .extract_text_with_limit(&pages, MAX_PDF_DECOMPRESSED_BYTES)
        .ok()
}

fn collapse_whitespace(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for word in text.split_whitespace() {
        if !out.is_empty() {
            out.push(' ');
        }
        out.push_str(word);
    }
    out
}

#[cfg(test)]
mod tests {
    use lopdf::{Document, Object, Stream, dictionary};

    use super::*;

    fn pdf_with(text: &str) -> Vec<u8> {
        let mut doc = Document::with_version("1.5");
        let pages_id = doc.new_object_id();
        let font = doc.add_object(dictionary! {
            "Type" => "Font", "Subtype" => "Type1", "BaseFont" => "Helvetica",
        });
        let content = format!("BT /F1 24 Tf 100 700 Td ({text}) Tj ET");
        let content_id = doc.add_object(Stream::new(dictionary! {}, content.into_bytes()));
        let page = doc.add_object(dictionary! {
            "Type" => "Page",
            "Parent" => pages_id,
            "MediaBox" => vec![0.into(), 0.into(), 595.into(), 842.into()],
            "Contents" => content_id,
            "Resources" => dictionary! { "Font" => dictionary! { "F1" => font } },
        });
        doc.objects.insert(
            pages_id,
            Object::Dictionary(dictionary! {
                "Type" => "Pages", "Kids" => vec![Object::Reference(page)], "Count" => 1,
            }),
        );
        let catalog = doc.add_object(dictionary! { "Type" => "Catalog", "Pages" => pages_id });
        doc.trailer.set("Root", catalog);
        let mut out = Vec::new();
        doc.save_to(&mut out).unwrap();
        out
    }

    #[test]
    fn a_pdf_gives_its_text() {
        let text = extract_text("application/pdf", &pdf_with("Devis arrosage automatique"));
        assert!(text.unwrap().contains("Devis arrosage automatique"));
    }

    #[test]
    fn a_pdf_with_no_text_gives_none() {
        assert_eq!(extract_text("application/pdf", &pdf_with("")), None);
    }

    #[test]
    fn broken_bytes_give_none_instead_of_an_error() {
        assert_eq!(extract_text("application/pdf", b"not a pdf"), None);
        assert_eq!(extract_text("text/plain", &[0xff, 0xfe, 0xfd]), None);
    }

    #[test]
    fn plain_text_is_kept_with_its_whitespace_collapsed() {
        assert_eq!(
            extract_text("text/plain", b"  Prix  HT\n\n 120 \t EUR "),
            Some("Prix HT 120 EUR".to_owned())
        );
    }

    #[test]
    fn an_image_has_no_text() {
        assert_eq!(extract_text("image/png", b"\x89PNG"), None);
    }

    #[test]
    fn extracted_text_is_capped() {
        let long = "mot ".repeat(MAX_EXTRACTED_CHARS);
        let text = extract_text("text/plain", long.as_bytes()).unwrap();
        assert_eq!(text.chars().count(), MAX_EXTRACTED_CHARS);
    }

    #[test]
    fn a_file_is_checked_for_size_and_type() {
        assert_eq!(
            check_file("application/pdf", b""),
            Err(KnowledgeError::EmptyFile)
        );
        assert_eq!(
            check_file("application/pdf", &vec![0; MAX_FILE_BYTES + 1]),
            Err(KnowledgeError::FileTooLarge)
        );
        assert_eq!(
            check_file("application/zip", b"x"),
            Err(KnowledgeError::UnsupportedFile)
        );
        assert_eq!(check_file("application/pdf", b"x"), Ok(()));
    }

    #[test]
    fn title_note_and_tags_are_trimmed_and_bounded() {
        assert_eq!(
            normalize_title("  Devis Dupont "),
            Ok("Devis Dupont".to_owned())
        );
        assert_eq!(normalize_title("   "), Err(KnowledgeError::TitleEmpty));
        assert_eq!(
            normalize_title(&"a".repeat(MAX_TITLE_CHARS + 1)),
            Err(KnowledgeError::TitleTooLong)
        );
        assert_eq!(normalize_note(" \n "), Err(KnowledgeError::NoteEmpty));
        assert_eq!(
            normalize_tags(&[" Devis ", "devis", "TARIF"]),
            Ok(vec!["devis".to_owned(), "tarif".to_owned()])
        );
        assert_eq!(normalize_tags(&["a", " "]), Err(KnowledgeError::TagEmpty));
        let many: Vec<String> = (0..=MAX_TAGS).map(|n| format!("t{n}")).collect();
        assert_eq!(normalize_tags(&many), Err(KnowledgeError::TooManyTags));
    }
}
