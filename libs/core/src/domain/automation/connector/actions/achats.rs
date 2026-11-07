use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::Field;

const GROUP: &str = "Achats";
const NIL: &str = "00000000-0000-0000-0000-000000000000";
const MOMENT: &str = "2026-01-01T00:00:00Z";

const PAGE_FIELDS: &[Field] = &[
    Field::number("limit", "Nombre maximum d’éléments", false),
    Field::number("offset", "Décalage", false),
];

const SUPPLIER_FIELDS: &[Field] = &[Field::text("supplier_id", "Fournisseur", true)];

const CREATE_SUPPLIER_FIELDS: &[Field] = &[
    Field::text("name", "Nom", true),
    Field::text("registration_number", "SIRET", false),
    Field::text("vat_number", "Numéro de TVA", false),
    Field::text("address", "Adresse", false),
    Field::text("phone", "Téléphone", false),
    Field::text("email", "Adresse e-mail", false),
];

const UPDATE_SUPPLIER_FIELDS: &[Field] = &[
    Field::text("supplier_id", "Fournisseur", true),
    Field::text("name", "Nom", true),
    Field::text("registration_number", "SIRET", false),
    Field::text("vat_number", "Numéro de TVA", false),
    Field::text("address", "Adresse", false),
    Field::text("phone", "Téléphone", false),
    Field::text("email", "Adresse e-mail", false),
];

const INVOICE_FIELDS: &[Field] = &[Field::text(
    "supplier_invoice_id",
    "Facture fournisseur",
    true,
)];

const NOTES_FIELDS: &[Field] = &[
    Field::text("supplier_invoice_id", "Facture fournisseur", true),
    Field::text("notes", "Notes", false),
];

const ALLOCATIONS_FIELDS: &[Field] = &[
    Field::text(
        "supplier_invoice_line_id",
        "Ligne de facture fournisseur",
        true,
    ),
    Field::json(
        "allocations",
        "Répartition : liste de { project_id, amount_cents }",
        true,
    ),
];

const PROJECT_FIELDS: &[Field] = &[Field::text("project_id", "Projet", true)];

fn supplier_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "name": "Point P",
        "registration_number": null,
        "vat_number": null,
        "address": null,
        "phone": null,
        "email": null,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn supplier_list_sample() -> Value {
    json!([supplier_sample()])
}

fn allocation_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "supplier_invoice_line_id": NIL,
        "project_id": NIL,
        "amount_cents": 1000,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn allocation_list_sample() -> Value {
    json!([allocation_sample()])
}

fn invoice_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "supplier_id": null,
        "supplier_name": "Point P",
        "supplier_registration_number": null,
        "supplier_vat_number": null,
        "number": "F-2026-4471",
        "issued_on": "2026-01-01",
        "due_on": null,
        "received_at": MOMENT,
        "source": "FACTUR_X",
        "status": "RECEIVED",
        "currency": "EUR",
        "source_file_key": null,
        "source_file_mime_type": null,
        "notes": null,
        "net_cents": 45000,
        "vat_breakdown": [{ "rate_bp": 2000, "vat_cents": 9000 }],
        "gross_cents": 54000,
        "lines": [{
            "id": NIL,
            "supplier_invoice_id": NIL,
            "label": "Sable 0/4",
            "quantity": "2",
            "unit": null,
            "unit_price_cents": 22500,
            "line_total_cents": 45000,
            "vat_rate_basis_points": 2000,
            "position": 0,
            "allocations": [],
            "created_at": MOMENT,
            "updated_at": MOMENT,
        }],
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn invoice_list_sample() -> Value {
    json!([invoice_sample()])
}

fn project_costs_sample() -> Value {
    json!({
        "project_id": NIL,
        "allocated_cents": 1000,
        "lines": [{
            "allocation_id": NIL,
            "supplier_invoice_id": NIL,
            "supplier_invoice_number": "F-2026-4471",
            "supplier_name": "Point P",
            "supplier_invoice_line_id": NIL,
            "line_label": "Sable 0/4",
            "amount_cents": 1000,
            "created_at": MOMENT,
        }],
    })
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.achats.list_suppliers",
        label: "Lister les fournisseurs",
        description: "Retourne les fournisseurs de l’organisation, par pages.",
        group: GROUP,
        fields: PAGE_FIELDS,
        read_only: true,
        sample_output: supplier_list_sample,
    },
    ActionSpec {
        kind: "mestier.achats.get_supplier",
        label: "Lire un fournisseur",
        description: "Retourne la fiche d’un fournisseur de l’organisation.",
        group: GROUP,
        fields: SUPPLIER_FIELDS,
        read_only: true,
        sample_output: supplier_sample,
    },
    ActionSpec {
        kind: "mestier.achats.create_supplier",
        label: "Créer un fournisseur",
        description: "Ajoute un fournisseur à l’organisation.",
        group: GROUP,
        fields: CREATE_SUPPLIER_FIELDS,
        read_only: false,
        sample_output: supplier_sample,
    },
    ActionSpec {
        kind: "mestier.achats.update_supplier",
        label: "Modifier un fournisseur",
        description: "Remplace les informations d’un fournisseur ; les champs laissés vides sont effacés.",
        group: GROUP,
        fields: UPDATE_SUPPLIER_FIELDS,
        read_only: false,
        sample_output: supplier_sample,
    },
    ActionSpec {
        kind: "mestier.achats.list_supplier_invoices",
        label: "Lister les factures fournisseur",
        description: "Retourne les factures fournisseur de l’organisation, par pages.",
        group: GROUP,
        fields: PAGE_FIELDS,
        read_only: true,
        sample_output: invoice_list_sample,
    },
    ActionSpec {
        kind: "mestier.achats.get_supplier_invoice",
        label: "Lire une facture fournisseur",
        description: "Retourne une facture fournisseur avec ses lignes et la répartition de chaque ligne entre projets.",
        group: GROUP,
        fields: INVOICE_FIELDS,
        read_only: true,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.achats.update_supplier_invoice_notes",
        label: "Modifier les notes d’une facture fournisseur",
        description: "Remplace les notes d’une facture fournisseur ; des notes vides les effacent.",
        group: GROUP,
        fields: NOTES_FIELDS,
        read_only: false,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.achats.confirm_supplier_invoice",
        label: "Confirmer une facture fournisseur",
        description: "Valide une facture fournisseur reçue, avec des notes facultatives.",
        group: GROUP,
        fields: NOTES_FIELDS,
        read_only: false,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.achats.reject_supplier_invoice",
        label: "Rejeter une facture fournisseur",
        description: "Refuse une facture fournisseur reçue, avec des notes facultatives.",
        group: GROUP,
        fields: NOTES_FIELDS,
        read_only: false,
        sample_output: invoice_sample,
    },
    ActionSpec {
        kind: "mestier.achats.replace_line_allocations",
        label: "Répartir une ligne de facture fournisseur entre projets",
        description: "Remplace la répartition d’une ligne de facture fournisseur entre les projets de l’organisation.",
        group: GROUP,
        fields: ALLOCATIONS_FIELDS,
        read_only: false,
        sample_output: allocation_list_sample,
    },
    ActionSpec {
        kind: "mestier.achats.get_project_supplier_costs",
        label: "Lire les coûts fournisseur d’un projet",
        description: "Retourne le total réparti sur un projet et le détail des lignes de facture fournisseur concernées.",
        group: GROUP,
        fields: PROJECT_FIELDS,
        read_only: true,
        sample_output: project_costs_sample,
    },
];
