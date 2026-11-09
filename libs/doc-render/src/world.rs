use std::{collections::HashMap, sync::LazyLock};

use mestier_core::DoeRenderError;
use typst::{
    Library, LibraryExt, World,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime, Duration},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::{Font, FontBook},
    utils::{LazyHash, Scalar},
};

use typst_layout::PagedDocument;

use crate::bundle::Prepared;

const TEMPLATE: &str = include_str!("doe.typ");

static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(|| LazyHash::new(Library::default()));

/// Every family a `DoeFont` names, regular and bold. Licenses sit next to
/// the files.
const FONT_FILES: [&[u8]; 8] = [
    include_bytes!("../fonts/Inter-Regular.ttf"),
    include_bytes!("../fonts/Inter-Bold.ttf"),
    include_bytes!("../fonts/Montserrat-Regular.ttf"),
    include_bytes!("../fonts/Montserrat-Bold.ttf"),
    include_bytes!("../fonts/SourceSerif4-Regular.ttf"),
    include_bytes!("../fonts/SourceSerif4-Bold.ttf"),
    include_bytes!("../fonts/Barlow-Regular.ttf"),
    include_bytes!("../fonts/Barlow-Bold.ttf"),
];

static FONTS: LazyLock<(LazyHash<FontBook>, Vec<Font>)> = LazyLock::new(|| {
    let fonts: Vec<Font> = FONT_FILES
        .iter()
        .flat_map(|data| Font::iter(Bytes::new(*data)))
        .collect();
    (LazyHash::new(FontBook::from_fonts(&fonts)), fonts)
});

fn file_id(path: &str) -> FileId {
    let vpath = VirtualPath::new(path).expect("static virtual path");
    RootedPath::new(VirtualRoot::Project, vpath).intern()
}
pub struct DoeWorld {
    main: Source,
    files: HashMap<FileId, Bytes>,
}

impl DoeWorld {
    pub fn new(prepared: Prepared) -> Self {
        Self::with_template(TEMPLATE, prepared)
    }

    pub fn with_template(template: &str, prepared: Prepared) -> Self {
        let mut files = HashMap::new();
        files.insert(file_id("/data.json"), Bytes::new(prepared.data));
        for (path, bytes) in prepared.files {
            files.insert(file_id(&path), bytes);
        }
        Self {
            main: Source::new(file_id("/main.typ"), template.to_owned()),
            files,
        }
    }

    pub fn compile_pdf(&self) -> Result<Vec<u8>, DoeRenderError> {
        let compiled = typst::compile::<PagedDocument>(self).output;
        let document = compiled.map_err(diagnostics);
        let pdf = document.and_then(|document| {
            let options = typst_pdf::PdfOptions {
                tagged: false,
                ..Default::default()
            };
            typst_pdf::pdf(&document, &options).map_err(diagnostics)
        });
        comemo::evict(0);
        pdf
    }

    pub fn compile_svg(&self, pages: &[usize]) -> Result<Vec<String>, DoeRenderError> {
        let compiled = typst::compile::<PagedDocument>(self).output;
        let svgs = compiled.map_err(diagnostics).map(|document| {
            let options = typst_svg::SvgOptions::default();
            pages
                .iter()
                .filter_map(|&index| document.pages().get(index))
                .map(|page| typst_svg::svg(page, &options))
                .collect()
        });
        comemo::evict(0);
        svgs
    }
}

/// Pixels per point at 150 dpi, enough to read a datasheet.
const RASTER_PIXELS_PER_POINT: f32 = 150.0 / 72.0;
/// Longest side of a rasterized page, so a large plan does not weigh tens of megabytes.
const RASTER_MAX_SIDE_PX: f32 = 1600.0;

/// Every page of a PDF as a PNG, in order. The PDF is laid out by Typst, which
/// embeds PDF pages, then drawn on a bitmap.
pub fn rasterize_pdf(bytes: &[u8], pages: usize) -> Result<Vec<Vec<u8>>, DoeRenderError> {
    let template = format!(
        "#set page(width: auto, height: auto, margin: 0pt)\n\
         #for number in range(1, {} + 1) {{\n\
         \x20 page(width: auto, height: auto, margin: 0pt, image(\"/files/0\", page: number))\n\
         }}\n",
        pages
    );
    let prepared = Prepared {
        data: b"{}".to_vec(),
        files: vec![("/files/0".to_owned(), Bytes::new(bytes.to_vec()))],
    };
    let world = DoeWorld::with_template(&template, prepared);
    let compiled = typst::compile::<PagedDocument>(&world).output;
    let rendered = compiled.map_err(diagnostics).and_then(|document| {
        document
            .pages()
            .iter()
            .map(|page| {
                let size = page.frame.size();
                let longest = size.x.to_pt().max(size.y.to_pt()) as f32;
                let scale = RASTER_PIXELS_PER_POINT.min(RASTER_MAX_SIDE_PX / longest.max(1.0));
                typst_render::render(
                    page,
                    &typst_render::RenderOptions {
                        pixel_per_pt: Scalar::new(f64::from(scale)),
                        ..Default::default()
                    },
                )
                .encode_png()
                .map_err(|error| DoeRenderError::Typeset(format!("PNG encoding failed: {error}")))
            })
            .collect()
    });
    comemo::evict(0);
    rendered
}

fn diagnostics(errors: impl IntoIterator<Item = typst::diag::SourceDiagnostic>) -> DoeRenderError {
    let message = errors
        .into_iter()
        .map(|e| e.message.to_string())
        .collect::<Vec<_>>()
        .join("; ");
    DoeRenderError::Typeset(message)
}

impl World for DoeWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &FONTS.0
    }

    fn main(&self) -> FileId {
        self.main.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main.id() {
            Ok(self.main.clone())
        } else {
            Err(FileError::NotFound(id.vpath().get_without_slash().into()))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files.get(&id).cloned().ok_or(FileError::AccessDenied)
    }

    fn font(&self, index: usize) -> Option<Font> {
        FONTS.1.get(index).cloned()
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}
