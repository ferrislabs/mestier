//! The DOE as an editable Word document.
//!
//! Same content and numbering as the PDF, laid out with plain Word
//! structures (headings, tables, inline pictures) so the reader can edit it.
//! Pixel fidelity with the Typst layout is not a goal. A .docx cannot hold a
//! PDF page, so each page of a plan or supplier datasheet is drawn on a bitmap
//! and placed as a picture.

use std::io::Cursor;

use crate::world::rasterize_pdf;
use docx_rs::{
    AlignmentType, BreakType, Docx, Footer, Header, Paragraph, Pic, Run, RunFonts, Style,
    StyleType, Table, TableCell, TableRow,
};
use image::{GenericImageView, ImageFormat, imageops::FilterType};
use mestier_core::{
    DoeDocument, DoeFile, DoeParty, DoeRenderError, DoeSupply, FileBundle, NumberedContent,
    NumberedSupply,
};

/// Longest side of an embedded picture. Word only supports PNG through the
/// writer, so a full-resolution photo would weigh several megabytes.
const MAX_PICTURE_SIDE: u32 = 1200;
const PICTURE_WIDTH_PX: u32 = 520;
const HALF_PICTURE_WIDTH_PX: u32 = 250;
const PAGE_PICTURE_WIDTH_PX: u32 = 600;
const MAX_PICTURES: usize = 400;

#[derive(Debug, Default, Clone, Copy)]
pub struct DocxDoeRenderer;

impl DocxDoeRenderer {
    pub fn render(
        &self,
        document: &DoeDocument,
        files: &FileBundle,
    ) -> Result<Vec<u8>, DoeRenderError> {
        let mut layout = Layout {
            document,
            files,
            pictures: 0,
        };
        let docx = layout.build()?;
        let mut out = Cursor::new(Vec::new());
        docx.build()
            .pack(&mut out)
            .map_err(|error| DoeRenderError::Typeset(format!("docx packing failed: {error}")))?;
        Ok(out.into_inner())
    }
}

struct Layout<'a> {
    document: &'a DoeDocument,
    files: &'a FileBundle,
    pictures: usize,
}

impl Layout<'_> {
    fn build(&mut self) -> Result<Docx, DoeRenderError> {
        let style = &self.document.style;
        let body = fonts(&style.body_font);
        let heading = fonts(&style.heading_font);
        let primary = style.primary.as_str().trim_start_matches('#').to_owned();

        let mut docx = Docx::new()
            .default_fonts(body)
            .default_size(22)
            .add_style(heading_style(
                "Heading1",
                "Heading 1",
                0,
                32,
                &primary,
                &heading,
            ))
            .add_style(heading_style(
                "Heading2",
                "Heading 2",
                1,
                26,
                &primary,
                &heading,
            ))
            .header(Header::new().add_paragraph(text_line(&self.document.header, 18)))
            .footer(Footer::new().add_paragraph(text_line(&self.document.footer, 18)));

        docx = self.cover(docx, &primary, &heading)?;
        docx = self.parties(docx);
        docx = self.contents(docx);
        for section in &self.document.sections {
            docx = docx.add_paragraph(
                Paragraph::new()
                    .style("Heading1")
                    .page_break_before(true)
                    .add_run(Run::new().add_text(format!("{}. {}", section.number, section.title))),
            );
            docx = match &section.content {
                NumberedContent::Supplies {
                    maker_label,
                    supplies,
                } => self.supplies(docx, maker_label, supplies)?,
                NumberedContent::Photos { per_page, photos } => {
                    let columns = if per_page.get() == 1 { 1 } else { 2 };
                    self.picture_grid(docx, photos, columns)?
                }
                NumberedContent::Plans { plans } => self.plans(docx, plans)?,
                NumberedContent::BlankPages { count } => (0..*count).fold(docx, |docx, _| {
                    docx.add_paragraph(
                        Paragraph::new().add_run(Run::new().add_break(BreakType::Page)),
                    )
                }),
            };
        }
        Ok(docx)
    }

    fn cover(
        &mut self,
        mut docx: Docx,
        primary: &str,
        heading: &RunFonts,
    ) -> Result<Docx, DoeRenderError> {
        let document = self.document;
        if let Some(key) = &document.style.logo_key
            && let Some(picture) = self.optional_picture(key, 180)?
        {
            docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_image(picture)));
        }
        docx = docx
            .add_paragraph(
                Paragraph::new().align(AlignmentType::Center).add_run(
                    Run::new()
                        .add_text(&document.style.cover_title)
                        .size(24)
                        .color(primary)
                        .fonts(heading.clone()),
                ),
            )
            .add_paragraph(
                Paragraph::new().align(AlignmentType::Center).add_run(
                    Run::new()
                        .add_text(&document.title)
                        .bold()
                        .size(52)
                        .color(primary)
                        .fonts(heading.clone()),
                ),
            );
        if let Some(key) = &document.cover_photo_key
            && let Some(picture) = self.optional_picture(key, PICTURE_WIDTH_PX)?
        {
            docx = docx.add_paragraph(
                Paragraph::new()
                    .align(AlignmentType::Center)
                    .add_run(Run::new().add_image(picture)),
            );
        }
        let mut lines: Vec<String> = Vec::new();
        lines.extend(document.place.clone());
        lines.extend(document.lot.clone());
        lines.push(format!(
            "Indice {} du {}",
            document.revision,
            document.issued_on.format("%d/%m/%Y")
        ));
        lines.extend(
            document
                .author
                .as_ref()
                .map(|author| format!("Rédigé par {author}")),
        );
        for line in lines {
            docx = docx.add_paragraph(
                Paragraph::new()
                    .align(AlignmentType::Center)
                    .add_run(Run::new().add_text(line)),
            );
        }
        Ok(docx)
    }

    fn parties(&self, mut docx: Docx) -> Docx {
        if self.document.parties.is_empty() {
            return docx;
        }
        docx = docx.add_paragraph(
            Paragraph::new()
                .style("Heading1")
                .page_break_before(true)
                .add_run(Run::new().add_text(&self.document.style.parties_title)),
        );
        let rows = self.document.parties.iter().map(party_row).collect();
        docx.add_table(Table::new(rows))
    }

    fn contents(&self, mut docx: Docx) -> Docx {
        docx = docx.add_paragraph(
            Paragraph::new()
                .style("Heading1")
                .page_break_before(true)
                .add_run(Run::new().add_text(&self.document.style.contents_title)),
        );
        for section in &self.document.sections {
            docx = docx
                .add_paragraph(Paragraph::new().add_run(
                    Run::new().add_text(format!("{}. {}", section.number, section.title)),
                ));
            if let NumberedContent::Supplies { supplies, .. } = &section.content {
                for numbered in supplies {
                    docx = docx.add_paragraph(
                        Paragraph::new()
                            .indent(Some(360), None, None, None)
                            .add_run(
                                Run::new()
                                    .add_text(format!("{} {}", numbered.number, numbered.heading)),
                            ),
                    );
                }
            }
        }
        docx
    }

    fn supplies(
        &mut self,
        mut docx: Docx,
        maker_label: &str,
        supplies: &[NumberedSupply],
    ) -> Result<Docx, DoeRenderError> {
        for numbered in supplies {
            docx =
                docx.add_paragraph(Paragraph::new().style("Heading2").add_run(
                    Run::new().add_text(format!("{} {}", numbered.number, numbered.heading)),
                ));
            docx = docx.add_table(Table::new(supply_rows(&numbered.supply, maker_label)));
            docx = self.picture_grid(docx, &numbered.supply.photos, 2)?;
            for key in &numbered.supply.datasheet_keys {
                docx = self.pdf_pages(docx, key, None)?;
            }
        }
        Ok(docx)
    }

    fn plans(&mut self, mut docx: Docx, plans: &[DoeFile]) -> Result<Docx, DoeRenderError> {
        for plan in plans {
            let file = self.file(&plan.key)?;
            if file.mime_type == "application/pdf" {
                docx = self.pdf_pages(docx, &plan.key, plan.caption.as_deref())?;
                continue;
            }
            let picture = self.picture(&plan.key, PICTURE_WIDTH_PX)?;
            docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_image(picture)));
            if let Some(caption) = &plan.caption {
                docx = docx.add_paragraph(caption_line(caption));
            }
        }
        Ok(docx)
    }

    fn pdf_pages(
        &mut self,
        mut docx: Docx,
        key: &str,
        caption: Option<&str>,
    ) -> Result<Docx, DoeRenderError> {
        let file = self.file(key)?;
        let pages = lopdf::Document::load_mem(&file.bytes)
            .map_err(|error| DoeRenderError::Typeset(format!("unreadable PDF `{key}`: {error}")))?
            .get_pages()
            .len();
        if pages == 0 {
            return Err(DoeRenderError::Typeset(format!("PDF `{key}` has no page")));
        }
        if self.pictures + pages > MAX_PICTURES {
            return Err(DoeRenderError::TooLarge(format!(
                "more than {MAX_PICTURES} pictures"
            )));
        }
        for png in rasterize_pdf(&file.bytes, pages)? {
            self.pictures += 1;
            let (width, height) = image::load_from_memory(&png)
                .map_err(|error| {
                    DoeRenderError::Typeset(format!("rasterized page of `{key}`: {error}"))
                })?
                .dimensions();
            let shown_width = PAGE_PICTURE_WIDTH_PX.min(width).max(1);
            let shown_height = ((u64::from(height) * u64::from(shown_width))
                / u64::from(width.max(1)))
            .max(1) as u32;
            let picture = Pic::new_with_dimensions(png, width, height)
                .size(shown_width * 9525, shown_height * 9525);
            docx = docx.add_paragraph(Paragraph::new().add_run(Run::new().add_image(picture)));
        }
        if let Some(caption) = caption {
            docx = docx.add_paragraph(caption_line(caption));
        }
        Ok(docx)
    }

    fn picture_grid(
        &mut self,
        docx: Docx,
        photos: &[DoeFile],
        columns: usize,
    ) -> Result<Docx, DoeRenderError> {
        if photos.is_empty() {
            return Ok(docx);
        }
        let width = if columns == 1 {
            PICTURE_WIDTH_PX
        } else {
            HALF_PICTURE_WIDTH_PX
        };
        let mut rows = Vec::new();
        for chunk in photos.chunks(columns) {
            let mut cells = Vec::new();
            for photo in chunk {
                let mut cell = TableCell::new().add_paragraph(
                    Paragraph::new()
                        .add_run(Run::new().add_image(self.picture(&photo.key, width)?)),
                );
                if let Some(caption) = &photo.caption {
                    cell = cell.add_paragraph(caption_line(caption));
                }
                cells.push(cell);
            }
            while cells.len() < columns {
                cells.push(TableCell::new().add_paragraph(Paragraph::new()));
            }
            rows.push(TableRow::new(cells));
        }
        Ok(docx.add_table(Table::new(rows)))
    }

    fn file(&self, key: &str) -> Result<&mestier_core::BundledFile, DoeRenderError> {
        self.files
            .get(key)
            .ok_or_else(|| DoeRenderError::MissingFile {
                key: key.to_owned(),
            })
    }

    fn optional_picture(
        &mut self,
        key: &str,
        width_px: u32,
    ) -> Result<Option<Pic>, DoeRenderError> {
        match self.files.get(key) {
            Some(_) => self.picture(key, width_px).map(Some),
            None => Ok(None),
        }
    }

    fn picture(&mut self, key: &str, width_px: u32) -> Result<Pic, DoeRenderError> {
        self.pictures += 1;
        if self.pictures > MAX_PICTURES {
            return Err(DoeRenderError::TooLarge(format!(
                "more than {MAX_PICTURES} pictures"
            )));
        }
        let file = self.file(key)?;
        let unsupported = || DoeRenderError::UnsupportedFile {
            key: key.to_owned(),
            mime_type: file.mime_type.clone(),
        };
        if !matches!(
            file.mime_type.as_str(),
            "image/jpeg" | "image/png" | "image/webp"
        ) {
            return Err(unsupported());
        }
        let decoded = image::load_from_memory(&file.bytes).map_err(|_| unsupported())?;
        let decoded = if decoded.width().max(decoded.height()) > MAX_PICTURE_SIDE {
            decoded.resize(MAX_PICTURE_SIDE, MAX_PICTURE_SIDE, FilterType::Triangle)
        } else {
            decoded
        };
        let (width, height) = decoded.dimensions();
        let mut png = Cursor::new(Vec::new());
        decoded
            .write_to(&mut png, ImageFormat::Png)
            .map_err(|_| unsupported())?;
        let shown_width = width.min(width_px).max(1);
        let shown_height =
            ((u64::from(height) * u64::from(shown_width)) / u64::from(width.max(1))).max(1) as u32;
        Ok(Pic::new_with_dimensions(png.into_inner(), width, height)
            .size(shown_width * 9525, shown_height * 9525))
    }
}

fn fonts(family: &str) -> RunFonts {
    RunFonts::new()
        .ascii(family)
        .hi_ansi(family)
        .east_asia(family)
        .cs(family)
}

fn heading_style(
    id: &str,
    name: &str,
    level: usize,
    size: usize,
    color: &str,
    fonts: &RunFonts,
) -> Style {
    Style::new(id, StyleType::Paragraph)
        .name(name)
        .bold()
        .size(size)
        .color(color)
        .fonts(fonts.clone())
        .outline_lvl(level)
}

fn text_line(text: &str, size: usize) -> Paragraph {
    Paragraph::new().add_run(Run::new().add_text(text).size(size))
}

fn caption_line(text: &str) -> Paragraph {
    Paragraph::new().add_run(Run::new().add_text(text).italic().size(18))
}

fn multiline(text: &str) -> Paragraph {
    let mut run = Run::new();
    for (index, line) in text.lines().enumerate() {
        if index > 0 {
            run = run.add_break(BreakType::TextWrapping);
        }
        run = run.add_text(line);
    }
    Paragraph::new().add_run(run)
}

fn labelled_row(label: &str, value: &str) -> TableRow {
    TableRow::new(vec![
        TableCell::new().add_paragraph(Paragraph::new().add_run(Run::new().add_text(label).bold())),
        TableCell::new().add_paragraph(multiline(value)),
    ])
}

fn party_row(party: &DoeParty) -> TableRow {
    let mut lines = vec![party.name.clone()];
    lines.extend(party.address.clone());
    lines.extend(party.phone.clone());
    lines.extend(party.email.clone());
    lines.extend(party.contact_name.clone());
    lines.extend(party.contact_role.clone());
    lines.extend(party.contact_email.clone());
    lines.extend(party.contact_phone.clone());
    labelled_row(party.role.french(), &lines.join("\n"))
}

fn supply_rows(supply: &DoeSupply, maker_label: &str) -> Vec<TableRow> {
    let mut rows = vec![labelled_row("Désignation", &supply.designation)];
    let mut push = |label: &str, value: &Option<String>| {
        if let Some(value) = value {
            rows.push(labelled_row(label, value));
        }
    };
    push("Ligne du DPGF", &supply.dpgf_ref);
    push("Destination", &supply.destination);
    push(maker_label, &supply.maker);
    push("Caractéristiques", &supply.specs);
    if let Some(supplier) = &supply.supplier {
        let mut lines = vec![supplier.name.clone()];
        lines.extend(supplier.address.clone());
        lines.extend(supplier.phone.clone());
        lines.extend(supplier.email.clone());
        rows.push(labelled_row("Fournisseur", &lines.join("\n")));
    }
    rows
}
