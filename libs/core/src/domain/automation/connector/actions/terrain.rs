use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::{Field, SelectOption};

const GROUP: &str = "Terrain";
const NIL: &str = "00000000-0000-0000-0000-000000000000";
const MOMENT: &str = "2026-01-01T00:00:00Z";

const PHASES: &[SelectOption] = &[
    SelectOption::new("BEFORE", "Avant"),
    SelectOption::new("DURING", "Pendant"),
    SelectOption::new("AFTER", "Après"),
];

const RESOLUTIONS: &[SelectOption] = &[
    SelectOption::new("PENDING", "En attente"),
    SelectOption::new("APPLIED", "Appliqué"),
    SelectOption::new("DISMISSED", "Écarté"),
];

const LIST_TASKS_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre de terrain", true),
    Field::text(
        "work_date",
        "Jour (AAAA-MM-JJ, aujourd’hui par défaut)",
        false,
    ),
];

const MEMBER_FIELDS: &[Field] = &[Field::text("member_id", "Membre de terrain", true)];

const START_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre de terrain", true),
    Field::text("task_id", "Tâche", true),
];

const DECLARE_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre de terrain", true),
    Field::text("task_id", "Tâche", true),
    Field::text("started_at", "Début (date et heure RFC 3339)", true),
    Field::text("ended_at", "Fin (date et heure RFC 3339)", true),
];

const STOP_FIELDS: &[Field] = &[Field::text("time_entry_id", "Pointage", true)];

const RECOVER_FIELDS: &[Field] = &[
    Field::text("time_entry_id", "Pointage", true),
    Field::text("ended_at", "Fin (date et heure RFC 3339)", true),
];

const PHOTO_FIELDS: &[Field] = &[
    Field::text("time_entry_id", "Pointage", true),
    Field::select("phase", "Phase", true, PHASES),
    Field::text("storage_key", "Clé de stockage de la photo", true),
];

const END_DAY_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre de terrain", true),
    Field::text(
        "ended_at",
        "Fin de journée (date et heure RFC 3339, maintenant par défaut)",
        false,
    ),
];

const REPORT_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre de terrain", true),
    Field::text("task_assignment_id", "Affectation", true),
    Field::number("reported_minutes", "Minutes déclarées", true),
    Field::text("comment", "Commentaire", false),
];

const AMEND_FIELDS: &[Field] = &[
    Field::text("assignment_report_id", "Déclaration", true),
    Field::number("reported_minutes", "Minutes déclarées", true),
    Field::text("comment", "Commentaire", false),
];

const WITHDRAW_FIELDS: &[Field] = &[Field::text("assignment_report_id", "Déclaration", true)];

const LIST_REPORTS_FIELDS: &[Field] = &[
    Field::text(
        "member_id",
        "Membre déclarant (toute l’organisation si vide)",
        false,
    ),
    Field::select("resolution", "Résolution", false, RESOLUTIONS),
    Field::number("limit", "Nombre maximum de déclarations", false),
    Field::number("offset", "Décalage", false),
];

const INTERVENTION_FIELDS: &[Field] = &[
    Field::text("member_id", "Membre proposant", true),
    Field::text("task_id", "Tâche", true),
    Field::text("done_on", "Date de l’intervention (AAAA-MM-JJ)", true),
];

fn entry_json(finished: bool) -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "task_id": NIL,
        "employee_id": NIL,
        "started_at": MOMENT,
        "ended_at": finished.then_some(MOMENT),
        "worked_minutes": finished.then_some(60),
        "photos": [],
    })
}

fn running_entry_sample() -> Value {
    entry_json(false)
}

fn finished_entry_sample() -> Value {
    entry_json(true)
}

fn field_task_sample() -> Value {
    json!({
        "id": NIL,
        "title": "Tonte",
        "description": null,
        "starts_at": MOMENT,
        "ends_at": MOMENT,
        "all_day": false,
        "status": "PLANNED",
        "customer_id": null,
        "customer_context_id": null,
        "maintenance_line_id": null,
        "task_assignment_id": NIL,
    })
}

fn field_task_list_sample() -> Value {
    json!([field_task_sample()])
}

fn current_sample() -> Value {
    json!({ "running": running_entry_sample(), "day_ended_at": null })
}

fn photo_sample() -> Value {
    json!({
        "id": NIL,
        "time_entry_id": NIL,
        "phase": "BEFORE",
        "storage_key": "photos/chantier.jpg",
        "created_at": MOMENT,
    })
}

fn day_log_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "employee_id": NIL,
        "work_date": "2026-01-01",
        "ended_at": MOMENT,
    })
}

fn report_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "task_assignment_id": NIL,
        "reported_minutes": 90,
        "comment": null,
        "reported_by": NIL,
        "resolution": "PENDING",
        "resolved_by": null,
        "resolved_at": null,
        "resolution_note": null,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn report_list_sample() -> Value {
    json!([report_sample()])
}

fn withdrawn_sample() -> Value {
    json!({ "assignment_report_id": NIL, "withdrawn": true })
}

fn intervention_sample() -> Value {
    json!({
        "id": NIL,
        "line_id": NIL,
        "task_id": NIL,
        "done_on": "2026-01-01",
        "status": "PROPOSED",
    })
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.terrain.list_tasks",
        label: "Lister les tâches de terrain d’un membre",
        description: "Retourne les tâches affectées à un membre pour un jour donné, avec l’identifiant de son affectation.",
        group: GROUP,
        fields: LIST_TASKS_FIELDS,
        read_only: true,
        sample_output: field_task_list_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.get_current",
        label: "Lire l’état de terrain d’un membre",
        description: "Retourne le pointage en cours du membre et l’heure de fin de sa journée si elle est terminée.",
        group: GROUP,
        fields: MEMBER_FIELDS,
        read_only: true,
        sample_output: current_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.start_time_entry",
        label: "Démarrer un pointage",
        description: "Ouvre un pointage sur une tâche pour un membre, à l’instant présent.",
        group: GROUP,
        fields: START_FIELDS,
        read_only: false,
        sample_output: running_entry_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.declare_time_entry",
        label: "Déclarer un pointage passé",
        description: "Enregistre après coup un pointage complet sur une tâche pour un membre.",
        group: GROUP,
        fields: DECLARE_FIELDS,
        read_only: false,
        sample_output: finished_entry_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.stop_time_entry",
        label: "Arrêter un pointage",
        description: "Clôt un pointage en cours à l’instant présent.",
        group: GROUP,
        fields: STOP_FIELDS,
        read_only: false,
        sample_output: finished_entry_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.recover_time_entry",
        label: "Récupérer un pointage oublié",
        description: "Clôt à l’heure indiquée un pointage laissé ouvert par oubli.",
        group: GROUP,
        fields: RECOVER_FIELDS,
        read_only: false,
        sample_output: finished_entry_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.attach_time_entry_photo",
        label: "Rattacher une photo à un pointage",
        description: "Associe à un pointage une photo déjà stockée, pour la phase indiquée.",
        group: GROUP,
        fields: PHOTO_FIELDS,
        read_only: false,
        sample_output: photo_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.end_day",
        label: "Terminer la journée d’un membre",
        description: "Déclare la fin de journée d’un membre et clôt ses pointages ouverts.",
        group: GROUP,
        fields: END_DAY_FIELDS,
        read_only: false,
        sample_output: day_log_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.report_assignment",
        label: "Déclarer le temps passé sur une affectation",
        description: "Enregistre, pour un membre, les minutes réellement passées sur une affectation, en attente de validation.",
        group: GROUP,
        fields: REPORT_FIELDS,
        read_only: false,
        sample_output: report_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.amend_assignment_report",
        label: "Corriger une déclaration de temps",
        description: "Modifie les minutes ou le commentaire d’une déclaration encore en attente.",
        group: GROUP,
        fields: AMEND_FIELDS,
        read_only: false,
        sample_output: report_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.withdraw_assignment_report",
        label: "Retirer une déclaration de temps",
        description: "Supprime une déclaration de temps encore en attente.",
        group: GROUP,
        fields: WITHDRAW_FIELDS,
        read_only: false,
        sample_output: withdrawn_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.list_assignment_reports",
        label: "Lister les déclarations de temps",
        description: "Retourne les déclarations de temps d’un membre, ou de toute l’organisation si aucun membre n’est donné.",
        group: GROUP,
        fields: LIST_REPORTS_FIELDS,
        read_only: true,
        sample_output: report_list_sample,
    },
    ActionSpec {
        kind: "mestier.terrain.propose_maintenance_intervention",
        label: "Proposer une intervention d’entretien",
        description: "Propose, au nom d’un membre, l’intervention d’un contrat d’entretien réalisée sur une tâche à une date donnée.",
        group: GROUP,
        fields: INTERVENTION_FIELDS,
        read_only: false,
        sample_output: intervention_sample,
    },
];
