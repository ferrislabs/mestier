use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::{Field, SelectOption};

#[cfg(test)]
#[path = "tests_devis.rs"]
mod tests_devis;

const GROUP: &str = "Devis";
const NIL: &str = "00000000-0000-0000-0000-000000000000";

const STATUSES: &[SelectOption] = &[
    SelectOption::new("DRAFT", "Brouillon"),
    SelectOption::new("SENT", "Envoyé"),
    SelectOption::new("ACCEPTED", "Accepté"),
    SelectOption::new("DECLINED", "Refusé"),
    SelectOption::new("CANCELLED", "Annulé"),
];

const QUOTE_FIELDS: &[Field] = &[Field::text("quote_id", "Devis", true)];

const CREATE_FIELDS: &[Field] = &[
    Field::text("title", "Titre", true),
    Field::text("customer_id", "Client", true),
    Field::text("customer_context_id", "Lieu du client", true),
    Field::json(
        "lines",
        "Lignes (service_rate_id, label, quantity, unit, unit_price_cents, vat_rate_bp, notes, photo_keys)",
        true,
    ),
];

const UPDATE_FIELDS: &[Field] = &[
    Field::text("quote_id", "Devis", true),
    Field::text("title", "Titre", true),
    Field::text("customer_id", "Client", true),
    Field::text("customer_context_id", "Lieu du client", true),
    Field::select("status", "Statut", true, STATUSES),
    Field::json(
        "lines",
        "Lignes (service_rate_id, label, quantity, unit, unit_price_cents, vat_rate_bp, notes, photo_keys)",
        true,
    ),
];

const STATUS_FIELDS: &[Field] = &[
    Field::text("quote_id", "Devis", true),
    Field::select("status", "Statut", true, STATUSES),
];

const PLAN_FIELDS: &[Field] = &[
    Field::text("quote_id", "Devis", true),
    Field::text("name", "Nom du projet", true),
    Field::boolean(
        "force_new",
        "Créer un second projet si le devis en a déjà un",
        false,
    ),
    Field::json(
        "tasks",
        "Tâches (parent_index, title, description, starts_at, ends_at, all_day, blocks_availability, expenses_cents, expenses_label, quote_line_ids)",
        false,
    ),
];

fn quote_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "reference": "DEV-2026-0001",
        "title": "Terrasse en bois",
        "customer_id": NIL,
        "customer_context_id": NIL,
        "status": "DRAFT",
        "net_cents": 100000,
        "vat_breakdown": [{ "rate_bp": 2000, "vat_cents": 20000 }],
        "gross_cents": 120000,
        "lines": [{
            "id": NIL,
            "service_rate_id": null,
            "label": "Pose de lames",
            "quantity": "10",
            "unit": "m²",
            "unit_price_cents": 10000,
            "vat_rate_bp": 2000,
            "notes": null,
            "photo_keys": [],
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        }],
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn deleted_quote_sample() -> Value {
    json!({ "quote_id": NIL, "deleted": true })
}

fn plan_proposal_sample() -> Value {
    json!({
        "quote": quote_sample(),
        "tasks": [{
            "quote_line_id": NIL,
            "title": "Pose de lames",
            "suggested_minutes": 600,
        }],
    })
}

fn plan_sample() -> Value {
    json!({
        "project": {
            "id": NIL,
            "name": "Terrasse en bois",
            "customer_id": NIL,
            "customer_context_id": NIL,
            "quote_id": NIL,
            "created_at": "2026-01-01T00:00:00Z",
            "updated_at": "2026-01-01T00:00:00Z",
        },
        "tasks": [{
            "id": NIL,
            "parent_task_id": null,
            "title": "Pose de lames",
            "description": null,
            "starts_at": "2026-01-02T08:00:00Z",
            "ends_at": "2026-01-02T12:00:00Z",
            "all_day": false,
            "project_id": NIL,
            "expenses_cents": 0,
            "expenses_label": null,
        }],
    })
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.quote.get",
        label: "Lire un devis",
        description: "Renvoie un devis de l’organisation avec ses lignes et ses totaux.",
        group: GROUP,
        fields: QUOTE_FIELDS,
        read_only: true,
        sample_output: quote_sample,
    },
    ActionSpec {
        kind: "mestier.quote.create",
        label: "Créer un devis",
        description: "Crée un devis brouillon pour un client et un lieu de l’organisation.",
        group: GROUP,
        fields: CREATE_FIELDS,
        read_only: false,
        sample_output: quote_sample,
    },
    ActionSpec {
        kind: "mestier.quote.update",
        label: "Modifier un devis",
        description: "Remplace le titre, le client, le statut et toutes les lignes d’un devis.",
        group: GROUP,
        fields: UPDATE_FIELDS,
        read_only: false,
        sample_output: quote_sample,
    },
    ActionSpec {
        kind: "mestier.quote.update_status",
        label: "Changer le statut d’un devis",
        description: "Fait passer un devis à un autre statut.",
        group: GROUP,
        fields: STATUS_FIELDS,
        read_only: false,
        sample_output: quote_sample,
    },
    ActionSpec {
        kind: "mestier.quote.delete",
        label: "Supprimer un devis",
        description: "Supprime un devis de l’organisation.",
        group: GROUP,
        fields: QUOTE_FIELDS,
        read_only: false,
        sample_output: deleted_quote_sample,
    },
    ActionSpec {
        kind: "mestier.quote.plan_proposal",
        label: "Proposer un planning pour un devis",
        description: "Renvoie une tâche suggérée par ligne d’un devis accepté, sans rien créer.",
        group: GROUP,
        fields: QUOTE_FIELDS,
        read_only: true,
        sample_output: plan_proposal_sample,
    },
    ActionSpec {
        kind: "mestier.quote.create_plan",
        label: "Planifier un devis",
        description: "Crée le projet et les tâches d’un devis accepté.",
        group: GROUP,
        fields: PLAN_FIELDS,
        read_only: false,
        sample_output: plan_sample,
    },
];
