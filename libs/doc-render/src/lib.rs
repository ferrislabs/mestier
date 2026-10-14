mod bundle;
mod recap;
mod world;

pub use recap::{MaintenanceRecap, RecapContact, RecapRow, TypstRecapRenderer};

use mestier_core::{DoeDocument, DoeRenderError, DoeRenderer, FileBundle};
#[derive(Debug, Default, Clone, Copy)]
pub struct TypstDoeRenderer;

impl DoeRenderer for TypstDoeRenderer {
    fn render(
        &self,
        document: &DoeDocument,
        files: &FileBundle,
    ) -> Result<Vec<u8>, DoeRenderError> {
        let prepared = bundle::prepare(document, files)?;
        let world = world::DoeWorld::new(prepared);
        world.compile_pdf()
    }

    fn render_svg(
        &self,
        document: &DoeDocument,
        files: &FileBundle,
        pages: &[usize],
    ) -> Result<Vec<String>, DoeRenderError> {
        let prepared = bundle::prepare(document, files)?;
        let world = world::DoeWorld::new(prepared);
        world.compile_svg(pages)
    }
}
