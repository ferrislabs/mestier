use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::Field;

#[cfg(test)]
#[path = "tests_reporting.rs"]
mod tests_reporting;

const GROUP: &str = "Rapports";
const NIL: &str = "00000000-0000-0000-0000-000000000000";

const PERIOD_FIELDS: &[Field] = &[
    Field::text("from", "Début de la période (AAAA-MM-JJ)", true),
    Field::text("to", "Fin de la période incluse (AAAA-MM-JJ)", true),
];

fn project_sample() -> Value {
    json!({
        "project_id": NIL,
        "name": "Terrasse en bois",
        "customer_id": NIL,
        "quoted_cents": 120000,
        "labour_cost_cents": 40000,
        "equipment_cost_cents": 5000,
        "expenses_cents": 2000,
        "supplier_cost_cents": 30000,
        "planned_minutes": 960,
        "occupied_minutes": 960,
        "overlapping_minutes": 0,
        "margin_cents": 43000,
        "members_without_rate": [],
    })
}

fn profitability_sample() -> Value {
    json!({
        "projects": [project_sample()],
        "members": [{
            "member_id": NIL,
            "planned_minutes": 960,
            "labour_cost_cents": 40000,
            "missing_cost": "HOURLY_RATE",
        }],
    })
}

fn worked_hours_sample() -> Value {
    json!({
        "members": [{
            "member_id": NIL,
            "planned_minutes": 960,
            "missing_cost": "HOURLY_RATE",
        }],
        "total_planned_minutes": 960,
    })
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.reporting.profitability",
        label: "Calculer la rentabilité",
        description: "Renvoie, par projet et par membre, le coût prévu face au montant des devis sur une période.",
        group: GROUP,
        fields: PERIOD_FIELDS,
        read_only: true,
        sample_output: profitability_sample,
    },
    ActionSpec {
        kind: "mestier.reporting.worked_hours",
        label: "Calculer les heures planifiées",
        description: "Renvoie les minutes planifiées par membre et leur total sur une période.",
        group: GROUP,
        fields: PERIOD_FIELDS,
        read_only: true,
        sample_output: worked_hours_sample,
    },
];
