use std::collections::HashMap;

use mestier_core::{DoeDocument, DoeRenderError, FileBundle, NumberedContent};
use serde_json::{Value, json};
use typst::foundations::Bytes;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Use {
    Picture,
    Page,
}

pub struct Prepared {
    pub data: Vec<u8>,
    pub files: Vec<(String, Bytes)>,
}

/// Pages inserted from supplier datasheets and plans across one document.
/// Bounds the layout work of a single export; a real DOE inserts a few dozen.
const MAX_INSERTED_PAGES: usize = 600;

struct Collector<'a> {
    bundle: &'a FileBundle,
    inserted_pages: usize,
    order: Vec<String>,
    entries: HashMap<String, Value>,
    blobs: Vec<(String, Bytes)>,
}

impl Collector<'_> {
    fn add(&mut self, key: &str, usage: Use) -> Result<(), DoeRenderError> {
        if self.entries.contains_key(key) {
            return Ok(());
        }
        let file = self
            .bundle
            .get(key)
            .ok_or_else(|| DoeRenderError::MissingFile {
                key: key.to_owned(),
            })?;
        let unsupported = || DoeRenderError::UnsupportedFile {
            key: key.to_owned(),
            mime_type: file.mime_type.clone(),
        };
        let path = format!("/files/{}", self.order.len());
        let (kind, pages) = match file.mime_type.as_str() {
            "image/jpeg" | "image/png" | "image/webp" => ("image", 1),
            "application/pdf" if usage == Use::Page => {
                let pages = lopdf::Document::load_mem(&file.bytes)
                    .map_err(|e| DoeRenderError::Typeset(format!("unreadable PDF `{key}`: {e}")))?
                    .get_pages()
                    .len();
                if pages == 0 {
                    return Err(DoeRenderError::Typeset(format!("PDF `{key}` has no page")));
                }
                self.inserted_pages += pages;
                if self.inserted_pages > MAX_INSERTED_PAGES {
                    return Err(DoeRenderError::TooLarge(format!(
                        "more than {MAX_INSERTED_PAGES} inserted PDF pages"
                    )));
                }
                ("pdf", pages)
            }
            _ => return Err(unsupported()),
        };
        self.blobs
            .push((path.clone(), Bytes::new(file.bytes.clone())));
        self.order.push(key.to_owned());
        self.entries.insert(
            key.to_owned(),
            json!({ "path": path, "kind": kind, "pages": pages }),
        );
        Ok(())
    }
}
pub fn prepare(document: &DoeDocument, bundle: &FileBundle) -> Result<Prepared, DoeRenderError> {
    let mut collector = Collector {
        bundle,
        inserted_pages: 0,
        order: Vec::new(),
        entries: HashMap::new(),
        blobs: Vec::new(),
    };
    if let Some(key) = &document.style.logo_key {
        collector.add(key, Use::Picture)?;
    }
    if let Some(key) = &document.cover_photo_key {
        collector.add(key, Use::Picture)?;
    }
    for party in &document.parties {
        if let Some(key) = &party.logo_key {
            collector.add(key, Use::Picture)?;
        }
    }
    for section in &document.sections {
        match &section.content {
            NumberedContent::Supplies { supplies, .. } => {
                for numbered in supplies {
                    for photo in &numbered.supply.photos {
                        collector.add(&photo.key, Use::Picture)?;
                    }
                    for key in &numbered.supply.datasheet_keys {
                        collector.add(key, Use::Page)?;
                    }
                }
            }
            NumberedContent::Photos { photos, .. } => {
                for photo in photos {
                    collector.add(&photo.key, Use::Picture)?;
                }
            }
            NumberedContent::Plans { plans } => {
                for plan in plans {
                    collector.add(&plan.key, Use::Page)?;
                }
            }
            NumberedContent::BlankPages { .. } => {}
        }
    }

    let mut doc = serde_json::to_value(document)
        .map_err(|e| DoeRenderError::Typeset(format!("document is not serializable: {e}")))?;
    if let Some(parties) = doc.get_mut("parties").and_then(Value::as_array_mut) {
        for (value, party) in parties.iter_mut().zip(&document.parties) {
            value["role_label"] = Value::from(party.role.french());
        }
    }
    let data = json!({
        "doc": doc,
        "files": collector.entries,
        "issued_fr": document.issued_on.format("%d/%m/%Y").to_string(),
    });
    let data = serde_json::to_vec(&data)
        .map_err(|e| DoeRenderError::Typeset(format!("document is not serializable: {e}")))?;
    Ok(Prepared {
        data,
        files: collector.blobs,
    })
}
