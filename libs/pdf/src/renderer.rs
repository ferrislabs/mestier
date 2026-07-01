use typst::foundations::{Bytes, Dict, IntoValue};
use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};
use typst_layout::PagedDocument;
use typst_pdf::PdfOptions;

use crate::types::{DocumentSnapshot, PdfError};

static TEMPLATE: &str = include_str!("../templates/document.typ");

pub fn render_document_pdf(snapshot: &DocumentSnapshot) -> Result<Vec<u8>, PdfError> {
	// Serialize snapshot to JSON bytes — Typst reads it via json(inputs.at("data"))
	let json_bytes = serde_json::to_vec(snapshot)?;

	// Build engine with embedded fonts from typst-assets (no system font scan)
	let engine = TypstEngine::builder()
		.main_file(TEMPLATE)
		.search_fonts_with(
			TypstKitFontOptions::new()
				.include_system_fonts(false)
				.include_embedded_fonts(true),
		)
		.build();

	// Build input dict: { "data": <raw JSON bytes> }
	// json() in Typst accepts Bytes directly
	let mut inputs = Dict::new();
	inputs.insert("data".into(), Bytes::new(json_bytes).into_value());

	// Compile to a paged document
	let doc: PagedDocument = engine
		.compile_with_input(inputs)
		.output
		.map_err(|err| PdfError::Compile(format!("{err}")))?;

	// Export to PDF bytes
	let pdf_bytes = typst_pdf::pdf(&doc, &PdfOptions::default())
		.map_err(|diags| {
			let messages: Vec<String> = diags
				.iter()
				.map(|d| d.message.to_string())
				.collect();
			PdfError::Export(messages.join("; "))
		})?;

	Ok(pdf_bytes)
}
