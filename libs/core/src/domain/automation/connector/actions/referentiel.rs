use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::{Field, SelectOption};

#[cfg(test)]
#[path = "tests_referentiel.rs"]
mod tests_referentiel;

const GROUP: &str = "Référentiel";
const NIL: &str = "00000000-0000-0000-0000-000000000000";
const STAMP: &str = "2026-01-01T00:00:00Z";

const ABSENCE_KINDS: &[SelectOption] = &[
    SelectOption::new("LEAVE", "Congé"),
    SelectOption::new("SICK", "Arrêt maladie"),
    SelectOption::new("UNAVAILABLE", "Indisponibilité"),
];

const PAGE_FIELDS: &[Field] = &[
    Field::number("limit", "Nombre maximum de résultats", false),
    Field::number("offset", "Décalage", false),
];

const PRODUCT_ID_FIELDS: &[Field] = &[Field::text("product_id", "Produit", true)];

const PRODUCT_CREATE_FIELDS: &[Field] = &[
    Field::text("name", "Nom", true),
    Field::text("unit", "Unité", true),
    Field::number("unit_price_cents", "Prix unitaire (centimes)", true),
    Field::text("sku", "Référence", false),
    Field::number(
        "default_vat_rate_bp",
        "Taux de TVA par défaut (points de base)",
        false,
    ),
    Field::text("description", "Description", false),
    Field::json("photo_keys", "Clés des photos", false),
    Field::text("supplier_id", "Fournisseur", false),
    Field::text("manufacturer", "Fabricant", false),
    Field::text("technical_specs", "Caractéristiques techniques", false),
    Field::json("datasheet_keys", "Clés des fiches techniques", false),
];

const PRODUCT_UPDATE_FIELDS: &[Field] = &[
    Field::text("product_id", "Produit", true),
    Field::text("name", "Nom", true),
    Field::text("unit", "Unité", true),
    Field::number("unit_price_cents", "Prix unitaire (centimes)", true),
    Field::text("sku", "Référence", false),
    Field::number(
        "default_vat_rate_bp",
        "Taux de TVA par défaut (points de base)",
        false,
    ),
    Field::text("description", "Description", false),
    Field::json("photo_keys", "Clés des photos", false),
    Field::text("supplier_id", "Fournisseur", false),
    Field::text("manufacturer", "Fabricant", false),
    Field::text("technical_specs", "Caractéristiques techniques", false),
    Field::json("datasheet_keys", "Clés des fiches techniques", false),
];

const SERVICE_RATE_ID_FIELDS: &[Field] =
    &[Field::text("service_rate_id", "Tarif de prestation", true)];

const SERVICE_RATE_CREATE_FIELDS: &[Field] = &[
    Field::text("label", "Libellé", true),
    Field::text("unit", "Unité", true),
    Field::number("rate_cents", "Tarif (centimes)", true),
    Field::number(
        "default_vat_rate_bp",
        "Taux de TVA par défaut (points de base)",
        false,
    ),
    Field::text("description", "Description", false),
];

const SERVICE_RATE_UPDATE_FIELDS: &[Field] = &[
    Field::text("service_rate_id", "Tarif de prestation", true),
    Field::text("label", "Libellé", true),
    Field::text("unit", "Unité", true),
    Field::number("rate_cents", "Tarif (centimes)", true),
    Field::number(
        "default_vat_rate_bp",
        "Taux de TVA par défaut (points de base)",
        false,
    ),
    Field::text("description", "Description", false),
];

const EQUIPMENT_ID_FIELDS: &[Field] = &[Field::text("equipment_id", "Équipement", true)];

const EQUIPMENT_CREATE_FIELDS: &[Field] = &[
    Field::text("name", "Nom", true),
    Field::number("hourly_rate_cents", "Tarif horaire (centimes)", true),
];

const EQUIPMENT_UPDATE_FIELDS: &[Field] = &[
    Field::text("equipment_id", "Équipement", true),
    Field::text("name", "Nom", true),
    Field::number("hourly_rate_cents", "Tarif horaire (centimes)", true),
];

const CUSTOM_UNIT_CREATE_FIELDS: &[Field] = &[Field::text("code", "Code de l’unité", true)];

const ABSENCE_CREATE_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre", true),
    Field::select("kind", "Type", true, ABSENCE_KINDS),
    Field::text("starts_at", "Début (RFC 3339)", true),
    Field::text("ends_at", "Fin (RFC 3339)", true),
    Field::boolean("all_day", "Journée entière", false),
    Field::text("note", "Note", false),
];

const ABSENCE_UPDATE_FIELDS: &[Field] = &[
    Field::text("absence_id", "Absence", true),
    Field::select("kind", "Type", false, ABSENCE_KINDS),
    Field::text("starts_at", "Début (RFC 3339)", false),
    Field::text("ends_at", "Fin (RFC 3339)", false),
    Field::boolean("all_day", "Journée entière", false),
    Field::text("note", "Note", false),
    Field::boolean("clear_note", "Effacer la note", false),
];

const ABSENCE_ID_FIELDS: &[Field] = &[Field::text("absence_id", "Absence", true)];

const MEMBER_ID_FIELDS: &[Field] = &[Field::text("member_id", "Membre", true)];

const EMPLOYEE_PROFILE_UPSERT_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre", true),
    Field::boolean("is_salaried", "Salarié (coût mensuel)", true),
    Field::number(
        "weekly_contract_minutes",
        "Minutes de contrat par semaine",
        true,
    ),
    Field::number("hourly_rate_cents", "Taux horaire (centimes)", false),
    Field::number(
        "monthly_cost_cents",
        "Coût mensuel chargé (centimes)",
        false,
    ),
];

const EMPLOYEE_ID_FIELDS: &[Field] = &[Field::text("employee_id", "Employé", true)];

const COST_BASIS_SET_FIELDS: &[Field] = &[
    Field::text("employee_id", "Employé", true),
    Field::text("effective_from", "Effectif à partir du (AAAA-MM-JJ)", true),
    Field::boolean("is_salaried", "Salarié (coût mensuel)", true),
    Field::number(
        "weekly_contract_minutes",
        "Minutes de contrat par semaine",
        true,
    ),
    Field::number("hourly_rate_cents", "Taux horaire (centimes)", false),
    Field::number(
        "monthly_cost_cents",
        "Coût mensuel chargé (centimes)",
        false,
    ),
];

const COST_BASIS_CORRECT_FIELDS: &[Field] = &[
    Field::text("cost_basis_id", "Version de coût", true),
    Field::text("effective_from", "Effectif à partir du (AAAA-MM-JJ)", true),
    Field::boolean("is_salaried", "Salarié (coût mensuel)", true),
    Field::number(
        "weekly_contract_minutes",
        "Minutes de contrat par semaine",
        true,
    ),
    Field::text("effective_to", "Effectif jusqu’au (AAAA-MM-JJ)", false),
    Field::number("hourly_rate_cents", "Taux horaire (centimes)", false),
    Field::number(
        "monthly_cost_cents",
        "Coût mensuel chargé (centimes)",
        false,
    ),
];

fn product_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "name": "Radiateur à inertie 1500 W",
        "sku": "RAD-1500",
        "unit": "UNIT",
        "unit_price_cents": 24900,
        "default_vat_rate_bp": 2000,
        "description": "Radiateur connecté",
        "photo_keys": [],
        "supplier_id": NIL,
        "manufacturer": "Thermor",
        "technical_specs": "1500 W, classe II",
        "datasheet_keys": [],
        "created_at": STAMP,
        "updated_at": STAMP,
    })
}

fn products_sample() -> Value {
    json!([product_sample()])
}

fn service_rate_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "label": "Main-d’œuvre plomberie",
        "unit": "HOUR",
        "rate_cents": 6500,
        "default_vat_rate_bp": 2000,
        "description": "Taux horaire standard",
        "created_at": STAMP,
        "updated_at": STAMP,
    })
}

fn service_rates_sample() -> Value {
    json!([service_rate_sample()])
}

fn equipment_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "name": "Nacelle élévatrice",
        "hourly_rate_cents": 4500,
        "created_at": STAMP,
        "updated_at": STAMP,
    })
}

fn equipment_list_sample() -> Value {
    json!([equipment_sample()])
}

fn custom_unit_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "code": "palette",
        "created_at": STAMP,
    })
}

fn custom_units_sample() -> Value {
    json!([custom_unit_sample()])
}

fn absence_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "member_id": NIL,
        "kind": "LEAVE",
        "starts_at": STAMP,
        "ends_at": "2026-01-02T00:00:00Z",
        "all_day": true,
        "note": "Congés payés",
        "created_at": STAMP,
        "updated_at": STAMP,
    })
}

fn absences_sample() -> Value {
    json!([absence_sample()])
}

fn employee_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "member_id": NIL,
        "hourly_rate_cents": 2800,
        "is_salaried": false,
        "monthly_cost_cents": null,
        "effective_hourly_rate_cents": 2800,
        "weekly_contract_minutes": 2100,
        "created_at": STAMP,
        "updated_at": STAMP,
    })
}

fn employees_sample() -> Value {
    json!([employee_sample()])
}

fn cost_basis_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "employee_id": NIL,
        "effective_from": "2026-01-01",
        "effective_to": null,
        "hourly_rate_cents": 2800,
        "is_salaried": false,
        "monthly_cost_cents": null,
        "effective_hourly_rate_cents": 2800,
        "weekly_contract_minutes": 2100,
        "created_at": STAMP,
        "updated_at": STAMP,
    })
}

fn cost_bases_sample() -> Value {
    json!([cost_basis_sample()])
}

fn deleted_product_sample() -> Value {
    json!({ "product_id": NIL, "deleted": true })
}

fn deleted_service_rate_sample() -> Value {
    json!({ "service_rate_id": NIL, "deleted": true })
}

fn deleted_equipment_sample() -> Value {
    json!({ "equipment_id": NIL, "deleted": true })
}

fn deleted_absence_sample() -> Value {
    json!({ "absence_id": NIL, "deleted": true })
}

fn removed_profile_sample() -> Value {
    json!({ "member_id": NIL, "removed": true })
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.product.list",
        label: "Lister les produits",
        description: "Renvoie les produits du catalogue de l’organisation.",
        group: GROUP,
        fields: PAGE_FIELDS,
        read_only: true,
        sample_output: products_sample,
    },
    ActionSpec {
        kind: "mestier.product.get",
        label: "Lire un produit",
        description: "Renvoie un produit du catalogue.",
        group: GROUP,
        fields: PRODUCT_ID_FIELDS,
        read_only: true,
        sample_output: product_sample,
    },
    ActionSpec {
        kind: "mestier.product.create",
        label: "Créer un produit",
        description: "Ajoute un produit au catalogue de l’organisation.",
        group: GROUP,
        fields: PRODUCT_CREATE_FIELDS,
        read_only: false,
        sample_output: product_sample,
    },
    ActionSpec {
        kind: "mestier.product.update",
        label: "Modifier un produit",
        description: "Remplace les informations d’un produit du catalogue.",
        group: GROUP,
        fields: PRODUCT_UPDATE_FIELDS,
        read_only: false,
        sample_output: product_sample,
    },
    ActionSpec {
        kind: "mestier.product.delete",
        label: "Supprimer un produit",
        description: "Retire un produit du catalogue.",
        group: GROUP,
        fields: PRODUCT_ID_FIELDS,
        read_only: false,
        sample_output: deleted_product_sample,
    },
    ActionSpec {
        kind: "mestier.service_rate.list",
        label: "Lister les tarifs de prestation",
        description: "Renvoie les tarifs de prestation de l’organisation.",
        group: GROUP,
        fields: PAGE_FIELDS,
        read_only: true,
        sample_output: service_rates_sample,
    },
    ActionSpec {
        kind: "mestier.service_rate.get",
        label: "Lire un tarif de prestation",
        description: "Renvoie un tarif de prestation.",
        group: GROUP,
        fields: SERVICE_RATE_ID_FIELDS,
        read_only: true,
        sample_output: service_rate_sample,
    },
    ActionSpec {
        kind: "mestier.service_rate.create",
        label: "Créer un tarif de prestation",
        description: "Ajoute un tarif de prestation à l’organisation.",
        group: GROUP,
        fields: SERVICE_RATE_CREATE_FIELDS,
        read_only: false,
        sample_output: service_rate_sample,
    },
    ActionSpec {
        kind: "mestier.service_rate.update",
        label: "Modifier un tarif de prestation",
        description: "Remplace les informations d’un tarif de prestation.",
        group: GROUP,
        fields: SERVICE_RATE_UPDATE_FIELDS,
        read_only: false,
        sample_output: service_rate_sample,
    },
    ActionSpec {
        kind: "mestier.service_rate.delete",
        label: "Supprimer un tarif de prestation",
        description: "Retire un tarif de prestation.",
        group: GROUP,
        fields: SERVICE_RATE_ID_FIELDS,
        read_only: false,
        sample_output: deleted_service_rate_sample,
    },
    ActionSpec {
        kind: "mestier.equipment.list",
        label: "Lister les équipements",
        description: "Renvoie les équipements de l’organisation.",
        group: GROUP,
        fields: PAGE_FIELDS,
        read_only: true,
        sample_output: equipment_list_sample,
    },
    ActionSpec {
        kind: "mestier.equipment.get",
        label: "Lire un équipement",
        description: "Renvoie un équipement.",
        group: GROUP,
        fields: EQUIPMENT_ID_FIELDS,
        read_only: true,
        sample_output: equipment_sample,
    },
    ActionSpec {
        kind: "mestier.equipment.create",
        label: "Créer un équipement",
        description: "Ajoute un équipement à l’organisation.",
        group: GROUP,
        fields: EQUIPMENT_CREATE_FIELDS,
        read_only: false,
        sample_output: equipment_sample,
    },
    ActionSpec {
        kind: "mestier.equipment.update",
        label: "Modifier un équipement",
        description: "Remplace le nom et le tarif horaire d’un équipement.",
        group: GROUP,
        fields: EQUIPMENT_UPDATE_FIELDS,
        read_only: false,
        sample_output: equipment_sample,
    },
    ActionSpec {
        kind: "mestier.equipment.delete",
        label: "Supprimer un équipement",
        description: "Retire un équipement de l’organisation.",
        group: GROUP,
        fields: EQUIPMENT_ID_FIELDS,
        read_only: false,
        sample_output: deleted_equipment_sample,
    },
    ActionSpec {
        kind: "mestier.custom_unit.list",
        label: "Lister les unités personnalisées",
        description: "Renvoie les unités propres à l’organisation.",
        group: GROUP,
        fields: PAGE_FIELDS,
        read_only: true,
        sample_output: custom_units_sample,
    },
    ActionSpec {
        kind: "mestier.custom_unit.create",
        label: "Créer une unité personnalisée",
        description: "Ajoute une unité utilisable sur les produits, tarifs et lignes de devis.",
        group: GROUP,
        fields: CUSTOM_UNIT_CREATE_FIELDS,
        read_only: false,
        sample_output: custom_unit_sample,
    },
    ActionSpec {
        kind: "mestier.absence.list",
        label: "Lister les absences",
        description: "Renvoie les absences des membres de l’organisation.",
        group: GROUP,
        fields: PAGE_FIELDS,
        read_only: true,
        sample_output: absences_sample,
    },
    ActionSpec {
        kind: "mestier.absence.create",
        label: "Créer une absence",
        description: "Déclare un congé, un arrêt maladie ou une indisponibilité d’un membre.",
        group: GROUP,
        fields: ABSENCE_CREATE_FIELDS,
        read_only: false,
        sample_output: absence_sample,
    },
    ActionSpec {
        kind: "mestier.absence.update",
        label: "Modifier une absence",
        description: "Modifie les champs renseignés d’une absence.",
        group: GROUP,
        fields: ABSENCE_UPDATE_FIELDS,
        read_only: false,
        sample_output: absence_sample,
    },
    ActionSpec {
        kind: "mestier.absence.delete",
        label: "Supprimer une absence",
        description: "Retire une absence.",
        group: GROUP,
        fields: ABSENCE_ID_FIELDS,
        read_only: false,
        sample_output: deleted_absence_sample,
    },
    ActionSpec {
        kind: "mestier.employee_profile.list",
        label: "Lister les profils salariés",
        description: "Renvoie les profils contractuels des membres de l’organisation.",
        group: GROUP,
        fields: PAGE_FIELDS,
        read_only: true,
        sample_output: employees_sample,
    },
    ActionSpec {
        kind: "mestier.employee_profile.upsert",
        label: "Définir le profil salarié d’un membre",
        description: "Rattache ou remplace le contrat d’un membre.",
        group: GROUP,
        fields: EMPLOYEE_PROFILE_UPSERT_FIELDS,
        read_only: false,
        sample_output: employee_sample,
    },
    ActionSpec {
        kind: "mestier.employee_profile.remove",
        label: "Retirer le profil salarié d’un membre",
        description: "Détache le contrat d’un membre sans supprimer le membre.",
        group: GROUP,
        fields: MEMBER_ID_FIELDS,
        read_only: false,
        sample_output: removed_profile_sample,
    },
    ActionSpec {
        kind: "mestier.cost_basis.list",
        label: "Lister l’historique de coût d’un employé",
        description: "Renvoie les versions de coût d’un employé, de la plus ancienne à la plus récente.",
        group: GROUP,
        fields: EMPLOYEE_ID_FIELDS,
        read_only: true,
        sample_output: cost_bases_sample,
    },
    ActionSpec {
        kind: "mestier.cost_basis.set",
        label: "Dater une nouvelle base de coût",
        description: "Ouvre une nouvelle version de coût pour un employé à une date donnée.",
        group: GROUP,
        fields: COST_BASIS_SET_FIELDS,
        read_only: false,
        sample_output: cost_basis_sample,
    },
    ActionSpec {
        kind: "mestier.cost_basis.correct",
        label: "Corriger une base de coût",
        description: "Corrige une version de coût saisie par erreur.",
        group: GROUP,
        fields: COST_BASIS_CORRECT_FIELDS,
        read_only: false,
        sample_output: cost_basis_sample,
    },
];
