use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::{Field, SelectOption};

#[cfg(test)]
#[path = "tests_factures.rs"]
mod tests_factures;

const GROUP: &str = "Factures";
const NIL: &str = "00000000-0000-0000-0000-000000000000";

const KINDS: &[SelectOption] = &[
    SelectOption::new("STANDARD", "Standard"),
    SelectOption::new("DEPOSIT", "Acompte"),
    SelectOption::new("FINAL", "Solde"),
    SelectOption::new("CREDIT_NOTE", "Avoir"),
];

const NATURES: &[SelectOption] = &[
    SelectOption::new("GOODS", "Biens"),
    SelectOption::new("SERVICES", "Services"),
    SelectOption::new("BOTH", "Biens et services"),
];

const LINES_LABEL: &str = "Lignes (label, quantity, unit_price_cents, vat_rate_basis_points)";
const ADDRESS_LABEL: &str = "Adresse de livraison (line1, line2, postal_code, city, country)";

const INVOICE_FIELDS: &[Field] = &[Field::text("invoice_id", "Facture", true)];

const LIST_FIELDS: &[Field] = &[
    Field::number("limit", "Nombre maximum de résultats", false),
    Field::number("offset", "Décalage", false),
];

const NO_FIELDS: &[Field] = &[];

const PROJECT_FIELDS: &[Field] = &[Field::text("project_id", "Projet", true)];

const CREATE_FIELDS: &[Field] = &[
    Field::select("kind", "Type", true, KINDS),
    Field::text("project_id", "Projet", false),
    Field::text("customer_id", "Client", true),
    Field::text("customer_context_id", "Lieu du client", true),
    Field::text("due_at", "Échéance (date et heure RFC 3339)", false),
    Field::text("notes", "Notes", false),
    Field::select("operation_nature", "Nature de l’opération", false, NATURES),
    Field::json("delivery_address", ADDRESS_LABEL, false),
    Field::json("lines", LINES_LABEL, true),
];

const UPDATE_FIELDS: &[Field] = &[
    Field::text("invoice_id", "Facture", true),
    Field::text("project_id", "Projet", false),
    Field::text("customer_id", "Client", true),
    Field::text("customer_context_id", "Lieu du client", true),
    Field::text("due_at", "Échéance (date et heure RFC 3339)", false),
    Field::text("notes", "Notes", false),
    Field::select("operation_nature", "Nature de l’opération", false, NATURES),
    Field::json("delivery_address", ADDRESS_LABEL, false),
    Field::json("lines", LINES_LABEL, true),
];

const ISSUE_FIELDS: &[Field] = &[
    Field::text("invoice_id", "Facture", true),
    Field::boolean(
        "allow_exceeding_total",
        "Autoriser un dépassement du total du devis",
        false,
    ),
];

const CREDIT_NOTE_FIELDS: &[Field] = &[
    Field::text("invoice_id", "Facture d’origine", true),
    Field::json("lines", LINES_LABEL, true),
    Field::text("notes", "Notes", false),
    Field::boolean(
        "allow_exceeding_invoice_total",
        "Autoriser un dépassement du total de la facture",
        false,
    ),
];

const DEPOSIT_FIELDS: &[Field] = &[
    Field::text("project_id", "Projet", true),
    Field::number(
        "percentage_bp",
        "Pourcentage du devis en points de base (3000 = 30 %)",
        true,
    ),
    Field::text("due_at", "Échéance (date et heure RFC 3339)", false),
    Field::text("notes", "Notes", false),
    Field::boolean(
        "allow_exceeding_total",
        "Autoriser un dépassement du total du devis",
        false,
    ),
];

const FINAL_FIELDS: &[Field] = &[
    Field::text("project_id", "Projet", true),
    Field::text("due_at", "Échéance (date et heure RFC 3339)", false),
    Field::text("notes", "Notes", false),
    Field::boolean(
        "allow_exceeding_total",
        "Autoriser un dépassement du total du devis",
        false,
    ),
];

fn invoice_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "number": "FAC-2026-0001",
        "kind": "STANDARD",
        "project_id": NIL,
        "customer_id": NIL,
        "customer_context_id": NIL,
        "status": "DRAFT",
        "issued_at": "2026-01-01T00:00:00Z",
        "due_at": "2026-02-01T00:00:00Z",
        "notes": "Merci de votre confiance.",
        "operation_nature": "SERVICES",
        "delivery_address": {
            "line1": "12 rue des Lilas",
            "line2": null,
            "postal_code": "69001",
            "city": "Lyon",
            "country": "FR",
        },
        "net_cents": 100000,
        "vat_breakdown": [{ "rate_bp": 2000, "vat_cents": 20000 }],
        "gross_cents": 120000,
        "lines": [{
            "id": NIL,
            "label": "Pose de lames",
            "quantity": "10",
            "unit_price_cents": 10000,
            "vat_rate_basis_points": 2000,
            "position": 0,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        }],
        "source_invoice_id": NIL,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

const RECORD_PAYMENT_FIELDS: &[Field] = &[
    Field::text("invoice_id", "Facture", true),
    Field::number("amount_cents", "Montant en centimes", true),
    Field::text("paid_on", "Date du paiement (AAAA-MM-JJ)", true),
    Field::text("method", "Moyen de paiement", true),
    Field::text("reference", "Référence", false),
    Field::text("note", "Note", false),
    Field::boolean(
        "allow_exceeding_total",
        "Autoriser un dépassement du total",
        false,
    ),
];

const DELETE_PAYMENT_FIELDS: &[Field] = &[Field::text("payment_id", "Paiement", true)];

fn payment_sample() -> Value {
    json!({
        "id": NIL,
        "invoice_id": NIL,
        "amount_cents": 40000,
        "paid_on": "2026-01-15",
        "method": "virement",
        "reference": null,
        "note": null,
        "recorded_by": NIL,
        "created_at": "2026-01-15T00:00:00Z",
        "updated_at": "2026-01-15T00:00:00Z",
    })
}

fn deleted_payment_sample() -> Value {
    json!({ "payment_id": NIL, "deleted": true })
}

fn invoices_sample() -> Value {
    json!([invoice_sample()])
}

fn balance_sample() -> Value {
    json!({
        "invoice_id": NIL,
        "gross_cents": 120000,
        "credited_cents": 0,
        "paid_cents": 40000,
        "remaining_cents": 80000,
    })
}

fn payments_sample() -> Value {
    json!([{
        "id": NIL,
        "invoice_id": NIL,
        "amount_cents": 40000,
        "paid_on": "2026-01-15",
        "method": "virement",
        "reference": null,
        "note": null,
        "recorded_by": NIL,
        "created_at": "2026-01-15T00:00:00Z",
        "updated_at": "2026-01-15T00:00:00Z",
    }])
}

fn outstanding_sample() -> Value {
    json!([{
        "customer_id": NIL,
        "outstanding_cents": 80000,
        "oldest_due_at": "2026-02-01T00:00:00Z",
    }])
}

fn billing_summary_sample() -> Value {
    json!({
        "project_id": NIL,
        "quoted_cents": 120000,
        "billed_cents": 40000,
        "remaining_cents": 80000,
    })
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.invoice.get",
        label: "Lire une facture",
        description: "Renvoie une facture de l’organisation avec ses lignes et ses totaux.",
        group: GROUP,
        fields: INVOICE_FIELDS,
        read_only: true,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.list",
        label: "Lister les factures",
        description: "Renvoie les factures de l’organisation, page par page.",
        group: GROUP,
        fields: LIST_FIELDS,
        read_only: true,
        sample_output: invoices_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.list_by_project",
        label: "Lister les factures d’un projet",
        description: "Renvoie toutes les factures d’un projet de l’organisation.",
        group: GROUP,
        fields: PROJECT_FIELDS,
        read_only: true,
        sample_output: invoices_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.create",
        label: "Créer une facture",
        description: "Crée une facture brouillon pour un client et un lieu de l’organisation.",
        group: GROUP,
        fields: CREATE_FIELDS,
        read_only: false,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.update",
        label: "Modifier une facture",
        description: "Remplace le contenu d’une facture brouillon.",
        group: GROUP,
        fields: UPDATE_FIELDS,
        read_only: false,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.issue",
        label: "Émettre une facture",
        description: "Émet une facture brouillon et lui attribue son numéro.",
        group: GROUP,
        fields: ISSUE_FIELDS,
        read_only: false,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.cancel",
        label: "Annuler une facture",
        description: "Annule une facture brouillon ou émise.",
        group: GROUP,
        fields: INVOICE_FIELDS,
        read_only: false,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.issue_credit_note",
        label: "Émettre un avoir",
        description: "Émet un avoir qui corrige une facture émise.",
        group: GROUP,
        fields: CREDIT_NOTE_FIELDS,
        read_only: false,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.list_credit_notes",
        label: "Lister les avoirs d’une facture",
        description: "Renvoie tous les avoirs émis contre une facture.",
        group: GROUP,
        fields: INVOICE_FIELDS,
        read_only: true,
        sample_output: invoices_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.balance",
        label: "Lire le solde d’une facture",
        description: "Renvoie le montant, les avoirs, les paiements et le reste à payer d’une facture.",
        group: GROUP,
        fields: INVOICE_FIELDS,
        read_only: true,
        sample_output: balance_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.list_payments",
        label: "Lister les paiements d’une facture",
        description: "Renvoie les paiements enregistrés sur une facture.",
        group: GROUP,
        fields: INVOICE_FIELDS,
        read_only: true,
        sample_output: payments_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.outstanding_by_customer",
        label: "Lister les impayés par client",
        description: "Renvoie, pour chaque client de l’organisation, le montant restant à payer.",
        group: GROUP,
        fields: NO_FIELDS,
        read_only: true,
        sample_output: outstanding_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.issue_deposit",
        label: "Émettre une facture d’acompte",
        description: "Crée et émet un acompte, en pourcentage du devis d’un projet.",
        group: GROUP,
        fields: DEPOSIT_FIELDS,
        read_only: false,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.issue_final",
        label: "Émettre la facture de solde",
        description: "Crée et émet la facture du reste à facturer d’un projet.",
        group: GROUP,
        fields: FINAL_FIELDS,
        read_only: false,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.project_billing_summary",
        label: "Lire la facturation d’un projet",
        description: "Renvoie le montant du devis, le facturé et le reste à facturer d’un projet.",
        group: GROUP,
        fields: PROJECT_FIELDS,
        read_only: true,
        sample_output: billing_summary_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.record_payment",
        label: "Enregistrer un paiement",
        description: "Enregistre un paiement reçu sur une facture émise, au nom de l’identité Automatisation.",
        group: GROUP,
        fields: RECORD_PAYMENT_FIELDS,
        read_only: false,
        sample_output: payment_sample,
    },
    ActionSpec {
        kind: "mestier.invoice.delete_payment",
        label: "Supprimer un paiement",
        description: "Supprime un paiement enregistré sur une facture de l’organisation.",
        group: GROUP,
        fields: DELETE_PAYMENT_FIELDS,
        read_only: false,
        sample_output: deleted_payment_sample,
    },
];
