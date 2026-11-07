use serde_json::{Value, json};

use crate::StarterModel;
use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::{Field, SelectOption};

const GROUP_PLANNING: &str = "Planning";
const GROUP_DOE: &str = "DOE";
const GROUP_MAINTENANCE: &str = "Entretien";
const NIL: &str = "00000000-0000-0000-0000-000000000000";

const REPORT_RESOLUTIONS: &[SelectOption] = &[
    SelectOption::new("PENDING", "En attente"),
    SelectOption::new("APPLIED", "Appliquée"),
    SelectOption::new("DISMISSED", "Écartée"),
];

const REPORT_DECISIONS: &[SelectOption] = &[
    SelectOption::new("APPLIED", "Appliquée"),
    SelectOption::new("DISMISSED", "Écartée"),
];

const STARTERS: &[SelectOption] = &[SelectOption::new("landscaping", "Paysagiste")];

const CONTRACT_STATUSES: &[SelectOption] = &[
    SelectOption::new("ACTIVE", "Actif"),
    SelectOption::new("DRAFT", "Brouillon"),
];

const INTERVENTION_STATUSES: &[SelectOption] = &[
    SelectOption::new("PROPOSED", "Proposée"),
    SelectOption::new("VALIDATED", "Validée"),
    SelectOption::new("REJECTED", "Refusée"),
];

const LIST_REPORTS_FIELDS: &[Field] = &[
    Field::select(
        "resolution",
        "Décision (en attente par défaut)",
        false,
        REPORT_RESOLUTIONS,
    ),
    Field::number("limit", "Nombre maximum de signalements", false),
    Field::number("offset", "Décalage", false),
];

const RESOLVE_REPORT_FIELDS: &[Field] = &[
    Field::text("assignment_report_id", "Signalement", true),
    Field::select("resolution", "Décision", true, REPORT_DECISIONS),
    Field::text("resolution_note", "Note de décision", false),
    Field::text("resolved_by_member_id", "Membre qui tranche", true),
];

const LIST_MODELS_FIELDS: &[Field] = &[
    Field::boolean("include_archived", "Inclure les modèles archivés", false),
    Field::number("limit", "Nombre maximum de modèles", false),
    Field::number("offset", "Décalage", false),
];

const CREATE_MODEL_FIELDS: &[Field] = &[
    Field::text("name", "Nom", true),
    Field::text("description", "Description", false),
    Field::json("sections", "Sections", true),
    Field::json("style", "Style (celui de l’organisation par défaut)", false),
];

const FROM_STARTER_FIELDS: &[Field] =
    &[Field::select("starter", "Modèle de départ", true, STARTERS)];

const MODEL_FIELDS: &[Field] = &[Field::text("doe_model_id", "Modèle de DOE", true)];

const UPDATE_MODEL_FIELDS: &[Field] = &[
    Field::text("doe_model_id", "Modèle de DOE", true),
    Field::text("name", "Nom", true),
    Field::text("description", "Description", false),
    Field::json("sections", "Sections", true),
    Field::json("style", "Style (celui de l’organisation par défaut)", false),
];

const LIST_DOES_FIELDS: &[Field] = &[
    Field::number("limit", "Nombre maximum de DOE", false),
    Field::number("offset", "Décalage", false),
];

const DOE_FIELDS: &[Field] = &[Field::text("doe_id", "DOE", true)];

const UPDATE_DOE_FIELDS: &[Field] = &[
    Field::text("doe_id", "DOE", true),
    Field::text("title", "Titre", true),
    Field::text("place", "Lieu", false),
    Field::text("lot", "Lot", false),
    Field::text("issued_on", "Date d’émission (AAAA-MM-JJ)", true),
    Field::text("author", "Auteur", false),
    Field::text("revision", "Indice de révision", true),
    Field::text("cover_photo_key", "Clé de la photo de couverture", false),
    Field::json("parties", "Intervenants", true),
    Field::json("sections", "Sections", true),
    Field::text(
        "expected_updated_at",
        "Dernière modification lue (refus si dépassée)",
        false,
    ),
];

const DRAFT_SUPPLY_FIELDS: &[Field] = &[
    Field::text("doe_id", "DOE", true),
    Field::text("product_id", "Produit", true),
];

const LIST_CONTRACTS_FIELDS: &[Field] = &[
    Field::number("season", "Année de la saison", true),
    Field::boolean("include_archived", "Inclure les contrats archivés", false),
];

const CREATE_CONTRACT_FIELDS: &[Field] = &[
    Field::text("customer_id", "Client", true),
    Field::text("customer_context_id", "Site du client", true),
    Field::text("reference", "Code affaire", false),
    Field::json(
        "season",
        "Saison (année, première et dernière semaine)",
        true,
    ),
    Field::text("manager_member_id", "Responsable", false),
    Field::number("amount_cents", "Montant en centimes", false),
    Field::text("quote_id", "Devis", false),
    Field::text("notes", "Notes", false),
    Field::select(
        "status",
        "Statut (actif par défaut)",
        false,
        CONTRACT_STATUSES,
    ),
    Field::json("lines", "Lignes d’entretien", true),
];

const CONTRACT_FIELDS: &[Field] = &[Field::text("contract_id", "Contrat d’entretien", true)];

const UPDATE_CONTRACT_FIELDS: &[Field] = &[
    Field::text("contract_id", "Contrat d’entretien", true),
    Field::text("reference", "Code affaire", false),
    Field::json(
        "season",
        "Saison (année, première et dernière semaine)",
        true,
    ),
    Field::text("manager_member_id", "Responsable", false),
    Field::number("amount_cents", "Montant en centimes", false),
    Field::text("quote_id", "Devis", false),
    Field::text("notes", "Notes", false),
    Field::select(
        "status",
        "Statut (actif par défaut)",
        false,
        CONTRACT_STATUSES,
    ),
    Field::json("lines", "Lignes d’entretien", true),
];

const GRID_FIELDS: &[Field] = &[Field::number("season", "Année de la saison", true)];

const RECAP_FIELDS: &[Field] = &[Field::text("week", "Semaine ISO (2026-W43)", true)];

const PLAN_FIELDS: &[Field] = &[
    Field::json("line_ids", "Lignes à planifier", true),
    Field::text("week", "Semaine ISO du récapitulatif (2026-W43)", true),
];

const LIST_INTERVENTIONS_FIELDS: &[Field] = &[
    Field::select(
        "status",
        "Statut (proposées par défaut)",
        false,
        INTERVENTION_STATUSES,
    ),
    Field::number("limit", "Nombre maximum d’interventions", false),
    Field::number("offset", "Décalage", false),
];

const RECORD_INTERVENTION_FIELDS: &[Field] = &[
    Field::text("line_id", "Ligne d’entretien", true),
    Field::text("done_on", "Date de la visite (AAAA-MM-JJ)", true),
    Field::text("member_id", "Membre qui l’enregistre", true),
];

const VALIDATE_INTERVENTION_FIELDS: &[Field] = &[
    Field::text("intervention_id", "Intervention", true),
    Field::text("member_id", "Membre qui valide", true),
];

const REJECT_INTERVENTION_FIELDS: &[Field] = &[
    Field::text("intervention_id", "Intervention", true),
    Field::text("member_id", "Membre qui refuse", true),
    Field::text("note", "Motif du refus", false),
];

const GET_WORK_TIME_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre", true),
    Field::text("from", "Du (AAAA-MM-JJ)", true),
    Field::text("to", "Au (AAAA-MM-JJ)", true),
];

const PUT_RHYTHM_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre", true),
    Field::text(
        "effective_from",
        "En vigueur à partir du (AAAA-MM-JJ)",
        true,
    ),
    Field::text("effective_to", "En vigueur jusqu’au (AAAA-MM-JJ)", false),
    Field::json("slots", "Créneaux de la semaine", true),
];

const PUT_WORK_SLOTS_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre", true),
    Field::text("from", "Du (AAAA-MM-JJ)", true),
    Field::text("to", "Au (AAAA-MM-JJ)", true),
    Field::json("slots", "Créneaux de la période", true),
];

fn report_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "task_assignment_id": NIL,
        "reported_minutes": 150,
        "comment": "Deux heures de plus que prévu.",
        "reported_by": NIL,
        "resolution": "PENDING",
        "resolved_by": null,
        "resolved_at": null,
        "resolution_note": null,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn reports_sample() -> Value {
    json!([report_sample()])
}

fn resolved_report_sample() -> Value {
    let mut sample = report_sample();
    sample["resolution"] = json!("APPLIED");
    sample["resolved_by"] = json!(NIL);
    sample["resolved_at"] = json!("2026-01-02T00:00:00Z");
    sample["resolution_note"] = json!("Heures validées.");
    sample
}

fn model_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "name": "Dossier paysagiste",
        "description": "Végétaux, matériaux, mobilier, jeux, photos et plans",
        "sections": StarterModel::Landscaping.sections(),
        "style": null,
        "archived_at": null,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn models_sample() -> Value {
    json!([model_sample()])
}

fn doe_sample() -> Value {
    json!({
        "id": NIL,
        "project_id": NIL,
        "model_id": NIL,
        "title": "DOE Résidence des Lilas",
        "place": "Lyon",
        "lot": "Lot 3 – Espaces verts",
        "issued_on": "2026-01-01",
        "author": "Treyve Paysages",
        "revision": "A",
        "cover_photo_key": null,
        "parties": [{
            "role": "project_owner",
            "name": "Mairie de Lyon",
            "address": null,
            "phone": null,
            "email": null,
            "logo_key": null,
        }],
        "sections": [{
            "id": NIL,
            "title": "Identification des végétaux",
            "enabled": true,
            "content": { "kind": "supplies", "maker_label": "nursery", "supplies": [] },
        }],
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn does_sample() -> Value {
    json!([{
        "id": NIL,
        "project_id": NIL,
        "project_name": "Résidence des Lilas",
        "title": "DOE Résidence des Lilas",
        "revision": "A",
        "issued_on": "2026-01-01",
        "updated_at": "2026-01-01T00:00:00Z",
    }])
}

fn deleted_doe_sample() -> Value {
    json!({ "doe_id": NIL, "deleted": true })
}

fn supply_sample() -> Value {
    json!({
        "id": NIL,
        "source_product_id": NIL,
        "title": null,
        "designation": "Érable champêtre",
        "dpgf_ref": null,
        "destination": null,
        "supplier": {
            "name": "Pépinières Martin",
            "address": null,
            "phone": null,
            "email": null,
        },
        "maker": null,
        "specs": "Cépée, 200/250",
        "photos": [],
        "datasheet_keys": [],
    })
}

fn season_sample() -> Value {
    json!({ "year": 2026, "first_week": 10, "last_week": 44 })
}

fn contract_sample() -> Value {
    json!({
        "id": NIL,
        "customer_id": NIL,
        "customer_context_id": NIL,
        "project_id": NIL,
        "reference": "C26001",
        "season": season_sample(),
        "manager_member_id": NIL,
        "amount_cents": 480000,
        "quote_id": null,
        "notes": null,
        "status": "ACTIVE",
        "lines": [{
            "id": NIL,
            "label": "Tonte",
            "cadence": { "kind": "ROLLING", "every_days": 14 },
            "minutes_per_intervention": 90,
        }],
        "archived_at": null,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn contracts_sample() -> Value {
    json!([contract_sample()])
}

fn site_sample() -> Value {
    json!({
        "customer_name": "Mairie de Lyon",
        "site_label": "Parc des Lilas",
        "address_line": null,
        "postal_code": "69003",
        "city": "Lyon",
        "contact": { "name": "Anne Martin", "phone": null, "email": null },
    })
}

fn due_sample() -> Value {
    json!({ "on": "2026-04-06", "status": "OVERDUE" })
}

fn grid_sample() -> Value {
    json!({
        "year": 2026,
        "contracts": [{
            "id": NIL,
            "reference": "C26001",
            "customer_id": NIL,
            "customer_context_id": NIL,
            "site": site_sample(),
            "season": season_sample(),
            "manager_member_id": NIL,
            "status": "ACTIVE",
            "lines": [{
                "id": NIL,
                "label": "Tonte",
                "cadence": { "kind": "ROLLING", "every_days": 14 },
                "minutes_per_intervention": 90,
                "visits": [{
                    "id": NIL,
                    "done_on": "2026-03-23",
                    "week": 13,
                    "status": "VALIDATED",
                }],
                "due": due_sample(),
            }],
        }],
    })
}

fn recap_sample() -> Value {
    json!({
        "year": 2026,
        "week": 43,
        "groups": [{
            "town": "Lyon",
            "lines": [{
                "line_id": NIL,
                "contract_id": NIL,
                "reference": "C26001",
                "site": site_sample(),
                "label": "Tonte",
                "minutes_per_intervention": 90,
                "last_done_on": "2026-10-12",
                "due": due_sample(),
            }],
        }],
    })
}

fn planned_tasks_sample() -> Value {
    json!([{
        "id": NIL,
        "organization_id": NIL,
        "parent_task_id": null,
        "title": "Tonte – Parc des Lilas",
        "description": null,
        "starts_at": null,
        "ends_at": null,
        "all_day": false,
        "status": "BACKLOG",
        "blocks_availability": false,
        "customer_id": NIL,
        "customer_context_id": NIL,
        "quote_id": null,
        "project_id": NIL,
        "recurrence_id": null,
        "maintenance_line_id": NIL,
        "expenses_cents": 0,
        "expenses_label": null,
        "member_ids": [],
        "assignments": [],
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    }])
}

fn intervention_sample() -> Value {
    json!({
        "id": NIL,
        "line_id": NIL,
        "done_on": "2026-10-12",
        "status": "VALIDATED",
        "task_id": null,
        "proposed_by": NIL,
        "resolved_by": NIL,
        "resolved_at": "2026-10-13T00:00:00Z",
        "note": null,
        "created_at": "2026-10-12T00:00:00Z",
    })
}

fn interventions_sample() -> Value {
    json!([intervention_sample()])
}

fn work_time_sample() -> Value {
    json!({
        "rhythms": [rhythm_sample()],
        "work_slots": work_slots_sample(),
    })
}

fn rhythm_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "employee_id": NIL,
        "effective_from": "2026-01-01",
        "effective_to": null,
        "slots": [{ "weekday": 1, "starts_minute": 480, "ends_minute": 720 }],
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn work_slots_sample() -> Value {
    json!([{
        "id": NIL,
        "organization_id": NIL,
        "member_id": NIL,
        "work_date": "2026-01-05",
        "starts_minute": 480,
        "ends_minute": 720,
    }])
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.assignment_report.list",
        label: "Lister les signalements d’heures",
        description: "Renvoie les corrections d’heures signalées par le terrain, en attente par défaut.",
        group: GROUP_PLANNING,
        fields: LIST_REPORTS_FIELDS,
        read_only: true,
        sample_output: reports_sample,
    },
    ActionSpec {
        kind: "mestier.assignment_report.resolve",
        label: "Trancher un signalement d’heures",
        description: "Applique ou écarte un signalement en attente. Ne modifie pas la tâche : la décision est seulement enregistrée.",
        group: GROUP_PLANNING,
        fields: RESOLVE_REPORT_FIELDS,
        read_only: false,
        sample_output: resolved_report_sample,
    },
    ActionSpec {
        kind: "mestier.doe_model.list",
        label: "Lister les modèles de DOE",
        description: "Renvoie les modèles de DOE de l’organisation, archivés en option.",
        group: GROUP_DOE,
        fields: LIST_MODELS_FIELDS,
        read_only: true,
        sample_output: models_sample,
    },
    ActionSpec {
        kind: "mestier.doe_model.get",
        label: "Lire un modèle de DOE",
        description: "Renvoie un modèle de DOE de l’organisation.",
        group: GROUP_DOE,
        fields: MODEL_FIELDS,
        read_only: true,
        sample_output: model_sample,
    },
    ActionSpec {
        kind: "mestier.doe_model.create",
        label: "Créer un modèle de DOE",
        description: "Crée un modèle de DOE avec ses sections et, en option, son style.",
        group: GROUP_DOE,
        fields: CREATE_MODEL_FIELDS,
        read_only: false,
        sample_output: model_sample,
    },
    ActionSpec {
        kind: "mestier.doe_model.create_from_starter",
        label: "Créer un modèle de DOE depuis un modèle de départ",
        description: "Copie un modèle livré avec Mestier dans l’organisation.",
        group: GROUP_DOE,
        fields: FROM_STARTER_FIELDS,
        read_only: false,
        sample_output: model_sample,
    },
    ActionSpec {
        kind: "mestier.doe_model.update",
        label: "Modifier un modèle de DOE",
        description: "Remplace le nom, la description, les sections et le style d’un modèle non archivé.",
        group: GROUP_DOE,
        fields: UPDATE_MODEL_FIELDS,
        read_only: false,
        sample_output: model_sample,
    },
    ActionSpec {
        kind: "mestier.doe_model.archive",
        label: "Archiver un modèle de DOE",
        description: "Archive un modèle de DOE. Les DOE déjà créés ne changent pas.",
        group: GROUP_DOE,
        fields: MODEL_FIELDS,
        read_only: false,
        sample_output: model_sample,
    },
    ActionSpec {
        kind: "mestier.doe_model.restore",
        label: "Restaurer un modèle de DOE",
        description: "Sort un modèle de DOE des archives.",
        group: GROUP_DOE,
        fields: MODEL_FIELDS,
        read_only: false,
        sample_output: model_sample,
    },
    ActionSpec {
        kind: "mestier.doe.list",
        label: "Lister les DOE",
        description: "Renvoie les DOE de l’organisation, sans leur contenu.",
        group: GROUP_DOE,
        fields: LIST_DOES_FIELDS,
        read_only: true,
        sample_output: does_sample,
    },
    ActionSpec {
        kind: "mestier.doe.get",
        label: "Lire un DOE",
        description: "Renvoie un DOE de l’organisation avec ses intervenants et ses sections.",
        group: GROUP_DOE,
        fields: DOE_FIELDS,
        read_only: true,
        sample_output: doe_sample,
    },
    ActionSpec {
        kind: "mestier.doe.update",
        label: "Modifier un DOE",
        description: "Remplace tout le document. Avec la dernière date de modification lue, refuse d’écraser un changement plus récent.",
        group: GROUP_DOE,
        fields: UPDATE_DOE_FIELDS,
        read_only: false,
        sample_output: doe_sample,
    },
    ActionSpec {
        kind: "mestier.doe.delete",
        label: "Supprimer un DOE",
        description: "Supprime définitivement un DOE, ce qui libère son projet pour un nouveau. Les fichiers restent en stockage.",
        group: GROUP_DOE,
        fields: DOE_FIELDS,
        read_only: false,
        sample_output: deleted_doe_sample,
    },
    ActionSpec {
        kind: "mestier.doe.draft_supply",
        label: "Préparer une fourniture de DOE",
        description: "Copie la fiche technique d’un produit en fourniture à insérer dans un DOE. Rien n’est enregistré.",
        group: GROUP_DOE,
        fields: DRAFT_SUPPLY_FIELDS,
        read_only: true,
        sample_output: supply_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance_contract.list",
        label: "Lister les contrats d’entretien",
        description: "Renvoie les contrats d’entretien d’une saison, archivés en option.",
        group: GROUP_MAINTENANCE,
        fields: LIST_CONTRACTS_FIELDS,
        read_only: true,
        sample_output: contracts_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance_contract.get",
        label: "Lire un contrat d’entretien",
        description: "Renvoie un contrat d’entretien avec ses lignes.",
        group: GROUP_MAINTENANCE,
        fields: CONTRACT_FIELDS,
        read_only: true,
        sample_output: contract_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance_contract.create",
        label: "Créer un contrat d’entretien",
        description: "Crée le contrat, ses lignes et son projet, nommé d’après le client, le site et la saison.",
        group: GROUP_MAINTENANCE,
        fields: CREATE_CONTRACT_FIELDS,
        read_only: false,
        sample_output: contract_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance_contract.update",
        label: "Modifier un contrat d’entretien",
        description: "Remplace le contrat et ses lignes. Le client, le site et l’année restent ceux de la création.",
        group: GROUP_MAINTENANCE,
        fields: UPDATE_CONTRACT_FIELDS,
        read_only: false,
        sample_output: contract_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance_contract.archive",
        label: "Archiver un contrat d’entretien",
        description: "Archive le contrat en gardant ses lignes et ses interventions consultables.",
        group: GROUP_MAINTENANCE,
        fields: CONTRACT_FIELDS,
        read_only: false,
        sample_output: contract_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance_contract.duplicate",
        label: "Dupliquer un contrat d’entretien",
        description: "Copie le contrat et ses lignes sur la saison suivante, avec un nouveau projet et sans intervention.",
        group: GROUP_MAINTENANCE,
        fields: CONTRACT_FIELDS,
        read_only: false,
        sample_output: contract_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance.get_grid",
        label: "Lire la grille d’entretien",
        description: "Renvoie, pour une saison, chaque contrat actif avec ses lignes, les visites de chaque semaine et l’échéance de chaque ligne.",
        group: GROUP_MAINTENANCE,
        fields: GRID_FIELDS,
        read_only: true,
        sample_output: grid_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance.get_recap",
        label: "Lire le récapitulatif d’entretien",
        description: "Renvoie les lignes à planifier pour une semaine, groupées par ville, les retards d’abord.",
        group: GROUP_MAINTENANCE,
        fields: RECAP_FIELDS,
        read_only: true,
        sample_output: recap_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance.plan_lines",
        label: "Planifier des lignes d’entretien",
        description: "Crée une tâche sans date par ligne, dans le projet du contrat. Tout ou rien.",
        group: GROUP_MAINTENANCE,
        fields: PLAN_FIELDS,
        read_only: false,
        sample_output: planned_tasks_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance_intervention.list",
        label: "Lister les interventions d’entretien",
        description: "Renvoie les interventions d’un statut, les plus anciennes d’abord. Proposées par défaut.",
        group: GROUP_MAINTENANCE,
        fields: LIST_INTERVENTIONS_FIELDS,
        read_only: true,
        sample_output: interventions_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance_intervention.record",
        label: "Enregistrer une visite d’entretien",
        description: "Saisit une visite validée d’emblée, qui déplace l’échéance de la ligne.",
        group: GROUP_MAINTENANCE,
        fields: RECORD_INTERVENTION_FIELDS,
        read_only: false,
        sample_output: intervention_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance_intervention.validate",
        label: "Valider une intervention d’entretien",
        description: "Valide une visite proposée par le terrain. Elle déplace désormais l’échéance de la ligne.",
        group: GROUP_MAINTENANCE,
        fields: VALIDATE_INTERVENTION_FIELDS,
        read_only: false,
        sample_output: intervention_sample,
    },
    ActionSpec {
        kind: "mestier.maintenance_intervention.reject",
        label: "Refuser une intervention d’entretien",
        description: "Refuse une visite proposée. L’échéance de la ligne ne bouge pas.",
        group: GROUP_MAINTENANCE,
        fields: REJECT_INTERVENTION_FIELDS,
        read_only: false,
        sample_output: intervention_sample,
    },
    ActionSpec {
        kind: "mestier.work_time.get",
        label: "Lire le temps de travail d’un membre",
        description: "Renvoie les rythmes hebdomadaires et les créneaux datés d’un membre sur une période de 92 jours au plus.",
        group: GROUP_PLANNING,
        fields: GET_WORK_TIME_FIELDS,
        read_only: true,
        sample_output: work_time_sample,
    },
    ActionSpec {
        kind: "mestier.work_time.put_rhythm",
        label: "Définir le rythme d’un membre",
        description: "Remplace le rythme hebdomadaire en vigueur, ou en ouvre un nouveau si la date de début avance.",
        group: GROUP_PLANNING,
        fields: PUT_RHYTHM_FIELDS,
        read_only: false,
        sample_output: rhythm_sample,
    },
    ActionSpec {
        kind: "mestier.work_time.put_work_slots",
        label: "Définir les créneaux datés d’un membre",
        description: "Remplace tous les créneaux du membre sur la période, jamais seulement une différence.",
        group: GROUP_PLANNING,
        fields: PUT_WORK_SLOTS_FIELDS,
        read_only: false,
        sample_output: work_slots_sample,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_operations_without_automation_sense_are_not_described() {
        for spec in SPECS {
            assert!(
                !spec.kind.ends_with("pdf")
                    && !spec.kind.ends_with("preview")
                    && !spec.kind.ends_with("list_starters"),
                "{}",
                spec.kind
            );
        }
    }

    #[test]
    fn reads_are_flagged_read_only_and_writes_are_not() {
        let read_only: Vec<&str> = SPECS
            .iter()
            .filter(|spec| spec.read_only)
            .map(|spec| spec.kind)
            .collect();

        assert_eq!(
            read_only,
            [
                "mestier.assignment_report.list",
                "mestier.doe_model.list",
                "mestier.doe_model.get",
                "mestier.doe.list",
                "mestier.doe.get",
                "mestier.doe.draft_supply",
                "mestier.maintenance_contract.list",
                "mestier.maintenance_contract.get",
                "mestier.maintenance.get_grid",
                "mestier.maintenance.get_recap",
                "mestier.maintenance_intervention.list",
                "mestier.work_time.get",
            ]
        );
    }

    #[test]
    fn the_select_options_are_the_ones_the_services_parse() {
        use std::str::FromStr;

        use crate::{AssignmentReportResolution, ContractStatus, InterventionStatus};

        for option in REPORT_RESOLUTIONS {
            assert!(AssignmentReportResolution::from_str(option.value).is_ok());
        }
        for option in CONTRACT_STATUSES {
            assert!(ContractStatus::from_str(option.value).is_ok());
        }
        for option in INTERVENTION_STATUSES {
            assert!(InterventionStatus::from_str(option.value).is_ok());
        }
        for option in STARTERS {
            assert!(serde_json::from_value::<StarterModel>(json!(option.value)).is_ok());
        }
    }
}
