use mestier_core::{DafDocument, DafRenderer, DoeRenderError, FileBundle};
use serde_json::json;

use crate::{
    bundle::{Collector, Prepared, Use},
    world::DoeWorld,
};

const TEMPLATE: &str = include_str!("daf.typ");

/// Lays DAF sheets and the dossier summary out with Typst. See
/// docs/chantiers/daf.md.
#[derive(Debug, Default, Clone, Copy)]
pub struct TypstDafRenderer;

impl DafRenderer for TypstDafRenderer {
    fn render(
        &self,
        document: &DafDocument,
        files: &FileBundle,
    ) -> Result<Vec<u8>, DoeRenderError> {
        let prepared = prepare(document, files)?;
        DoeWorld::with_template(TEMPLATE, prepared).compile_pdf()
    }
}

fn prepare(document: &DafDocument, bundle: &FileBundle) -> Result<Prepared, DoeRenderError> {
    let mut collector = Collector::new(bundle);
    if let Some(key) = &document.style.logo_key {
        collector.add(key, Use::Picture)?;
    }
    for sheet in &document.sheets {
        if let Some(key) = &sheet.photo_key {
            collector.add(key, Use::Picture)?;
        }
        for attachment in &sheet.attachments {
            collector.add(&attachment.key, Use::Page)?;
        }
    }
    let data = json!({
        "doc": document,
        "files": collector.entries,
    });
    let data = serde_json::to_vec(&data)
        .map_err(|e| DoeRenderError::Typeset(format!("document is not serializable: {e}")))?;
    Ok(Prepared {
        data,
        files: collector.blobs,
    })
}
