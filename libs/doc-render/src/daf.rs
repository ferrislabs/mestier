use mestier_core::{DafDocument, DafRenderer, DoeRenderError, FileBundle};

/// Lays DAF sheets and the dossier summary out with Typst. See
/// docs/chantiers/daf.md.
#[derive(Debug, Default, Clone, Copy)]
pub struct TypstDafRenderer;

impl DafRenderer for TypstDafRenderer {
    fn render(
        &self,
        _document: &DafDocument,
        _files: &FileBundle,
    ) -> Result<Vec<u8>, DoeRenderError> {
        Err(DoeRenderError::Typeset(
            "DAF rendering is not written yet".to_owned(),
        ))
    }
}
