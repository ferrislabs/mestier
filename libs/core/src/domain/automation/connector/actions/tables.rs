use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::Field;

const GROUP: &str = "Tables";
const NIL: &str = "00000000-0000-0000-0000-000000000000";

const CREATE_ROW_FIELDS: &[Field] = &[
    Field::text("table_id", "Table (identifiant)", true),
    Field::json("values", "Valeurs par nom de colonne", true),
];

const UPDATE_ROW_FIELDS: &[Field] = &[
    Field::text("table_id", "Table (identifiant)", true),
    Field::text("row_id", "Ligne (identifiant)", true),
    Field::json("values", "Valeurs par nom de colonne", true),
];

const FIND_ROWS_FIELDS: &[Field] = &[
    Field::text("table_id", "Table (identifiant)", true),
    Field::text("column", "Colonne", true),
    Field::text("value", "Valeur", false),
];

fn row_sample() -> Value {
    json!({
        "id": NIL,
        "table_id": NIL,
        "title": "Villa Dupont",
        "values": { "00000000-0000-0000-0000-000000000000": "Villa Dupont" },
        "fields": { "Nom": "Villa Dupont", "Statut": "En cours", "Note": null },
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn rows_sample() -> Value {
    json!([row_sample()])
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.table_row.create",
        label: "Créer une ligne de table",
        description: "Ajoute une ligne à une table ; les valeurs sont données par nom de colonne.",
        group: GROUP,
        fields: CREATE_ROW_FIELDS,
        read_only: false,
        sample_output: row_sample,
    },
    ActionSpec {
        kind: "mestier.table_row.update",
        label: "Modifier une ligne de table",
        description: "Modifie les cellules nommées d’une ligne ; les autres restent inchangées.",
        group: GROUP,
        fields: UPDATE_ROW_FIELDS,
        read_only: false,
        sample_output: row_sample,
    },
    ActionSpec {
        kind: "mestier.table_row.find",
        label: "Chercher des lignes de table",
        description: "Renvoie au plus 50 lignes dont la colonne nommée vaut la valeur donnée.",
        group: GROUP,
        fields: FIND_ROWS_FIELDS,
        read_only: true,
        sample_output: rows_sample,
    },
];

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::automation::connector::field::FieldKind;

    #[test]
    fn only_find_is_read_only() {
        for spec in SPECS {
            assert_eq!(
                spec.read_only,
                spec.kind == "mestier.table_row.find",
                "{}",
                spec.kind
            );
            assert_eq!(spec.group, "Tables");
        }
    }

    #[test]
    fn every_action_names_its_table_and_the_writes_take_values_as_json() {
        for spec in SPECS {
            assert_eq!(spec.fields[0].name, "table_id");
            assert_eq!(spec.fields[0].kind, FieldKind::Text);
            assert!(spec.fields[0].required);
        }
        for spec in &SPECS[..2] {
            let values = spec.fields.last().unwrap();
            assert_eq!(values.name, "values");
            assert_eq!(values.kind, FieldKind::Json);
        }
        assert_eq!(SPECS[1].fields[1].name, "row_id");
        let names: Vec<&str> = SPECS[2].fields.iter().map(|field| field.name).collect();
        assert_eq!(names, ["table_id", "column", "value"]);
    }

    #[test]
    fn a_find_samples_a_list_and_a_write_samples_one_row() {
        assert!(rows_sample().is_array());
        assert_eq!(rows_sample()[0], row_sample());
        assert!(row_sample()["fields"].is_object());
    }
}
