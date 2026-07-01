mod renderer;
mod types;

pub use renderer::render_document_pdf;
pub use types::{
	CustomerInfo, DocumentSnapshot, LineSnapshot, PdfError, SellerInfo, VatBucketSnapshot,
};

#[cfg(test)]
mod tests;
