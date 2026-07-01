use crate::{
	render_document_pdf, CustomerInfo, DocumentSnapshot, LineSnapshot, SellerInfo,
	VatBucketSnapshot,
};

#[test]
fn test_render_invoice_returns_pdf_bytes() {
	let snapshot = DocumentSnapshot {
		kind: "INVOICE".to_string(),
		reference: Some("FAC-2026-0001".to_string()),
		title: "Facture".to_string(),
		issued_date: Some("01/07/2026".to_string()),
		due_date: Some("31/07/2026".to_string()),
		seller: SellerInfo {
			name: "Atelier Dupont".to_string(),
			siret: Some("123 456 789 00012".to_string()),
			rcs: Some("RCS Paris B 123 456 789".to_string()),
			ape: Some("4339Z".to_string()),
			vat_intracom: Some("FR12123456789".to_string()),
			iban: Some("FR76 1234 5678 9012 3456 7890 123".to_string()),
			bic: Some("BNPAFRPPXXX".to_string()),
		},
		customer: CustomerInfo {
			name: "SARL Martin & Fils".to_string(),
			address: Some("12 rue de la Paix\n75001 Paris".to_string()),
		},
		lines: vec![
			LineSnapshot {
				label: "Pose de carrelage 20m²".to_string(),
				quantity: "20".to_string(),
				unit: "m²".to_string(),
				unit_price_cents: 4500,
				vat_rate: "10".to_string(),
				line_ht_cents: 90000,
			},
			LineSnapshot {
				label: "Fourniture carrelage grès cérame".to_string(),
				quantity: "22".to_string(),
				unit: "m²".to_string(),
				unit_price_cents: 2800,
				vat_rate: "20".to_string(),
				line_ht_cents: 61600,
			},
		],
		vat_breakdown: vec![
			VatBucketSnapshot {
				rate: "10".to_string(),
				base_ht_cents: 90000,
				vat_cents: 9000,
			},
			VatBucketSnapshot {
				rate: "20".to_string(),
				base_ht_cents: 61600,
				vat_cents: 12320,
			},
		],
		total_ht_cents: 151600,
		total_vat_cents: 21320,
		total_ttc_cents: 172920,
		deposit_label: Some("Acompte 30 %".to_string()),
		deposit_amount_cents: Some(51876),
		remaining_cents: Some(121044),
		legal_mentions: vec![
			"Entrepreneur individuel soumis à la TVA. En cas de retard de paiement, une pénalité de 3\u{00d7} le taux d'intérêt légal sera appliquée.".to_string(),
			"Garantie décennale n° 987654321 — Assureur XYZ, valable jusqu'au 31/12/2030.".to_string(),
		],
		footer: Some(
			"Atelier Dupont — 12 rue des Artisans, 75011 Paris — contact@atelier-dupont.fr"
				.to_string(),
		),
		payment_terms: Some("Paiement à 30 jours fin de mois".to_string()),
	};

	let result = render_document_pdf(&snapshot);
	assert!(
		result.is_ok(),
		"render_document_pdf failed: {:?}",
		result.err()
	);

	let bytes = result.unwrap();
	assert!(
		bytes.starts_with(b"%PDF"),
		"Output does not start with %PDF header (first 8 bytes: {:?})",
		&bytes[..8.min(bytes.len())]
	);
	assert!(
		bytes.len() > 1000,
		"PDF suspiciously small: {} bytes",
		bytes.len()
	);
}
