use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::{Field, SelectOption};

const GROUP: &str = "Planning";
const NIL: &str = "00000000-0000-0000-0000-000000000000";
const MOMENT: &str = "2026-01-01T00:00:00Z";

const TASK_STATUSES: &[SelectOption] = &[
    SelectOption::new("BACKLOG", "À planifier"),
    SelectOption::new("PLANNED", "Planifiée"),
    SelectOption::new("IN_PROGRESS", "En cours"),
    SelectOption::new("DONE", "Terminée"),
    SelectOption::new("CANCELLED", "Annulée"),
];

const DELETE_SCOPES: &[SelectOption] = &[
    SelectOption::new("this_occurrence", "Cette occurrence"),
    SelectOption::new("this_and_following", "Cette occurrence et les suivantes"),
];

const FREQUENCIES: &[SelectOption] = &[
    SelectOption::new("DAILY", "Tous les jours"),
    SelectOption::new("WEEKLY", "Chaque semaine"),
    SelectOption::new("MONTHLY", "Chaque mois"),
];

const GET_PLANNING_FIELDS: &[Field] = &[
    Field::text("from", "Début de la fenêtre (AAAA-MM-JJ)", true),
    Field::text("to", "Fin de la fenêtre (AAAA-MM-JJ)", true),
];

const GET_AVAILABILITY_FIELDS: &[Field] = &[
    Field::text(
        "starts_at",
        "Début du créneau (date et heure ISO 8601)",
        true,
    ),
    Field::text("ends_at", "Fin du créneau (date et heure ISO 8601)", true),
];

const LIST_TEMPLATES_FIELDS: &[Field] = &[
    Field::boolean("include_archived", "Inclure les modèles archivés", false),
    Field::number("limit", "Nombre maximum de modèles", false),
    Field::number("offset", "Décalage", false),
];

const CREATE_TEMPLATE_FIELDS: &[Field] = &[
    Field::text("name", "Nom", true),
    Field::text("description", "Description", false),
    Field::json("tasks", "Tâches du modèle", false),
];

const TEMPLATE_FIELDS: &[Field] = &[Field::text("template_id", "Modèle de projet", true)];

const UPDATE_TEMPLATE_FIELDS: &[Field] = &[
    Field::text("template_id", "Modèle de projet", true),
    Field::text("name", "Nom", true),
    Field::text("description", "Description", false),
];

const INSTANTIATE_TEMPLATE_FIELDS: &[Field] = &[
    Field::text("template_id", "Modèle de projet", true),
    Field::text("name", "Nom du projet", true),
    Field::text("start_date", "Date de début (AAAA-MM-JJ)", true),
    Field::text("customer_id", "Client", false),
    Field::text("customer_context_id", "Contexte du client", false),
    Field::text("quote_id", "Devis", false),
];

const REPLACE_TEMPLATE_TASKS_FIELDS: &[Field] = &[
    Field::text("template_id", "Modèle de projet", true),
    Field::json("tasks", "Tâches du modèle", true),
];

const LIST_PROJECTS_FIELDS: &[Field] = &[
    Field::text("customer_id", "Client", false),
    Field::text("quote_id", "Devis", false),
    Field::boolean("include_archived", "Inclure les projets archivés", false),
    Field::number("limit", "Nombre maximum de projets", false),
    Field::number("offset", "Décalage", false),
];

const CREATE_PROJECT_FIELDS: &[Field] = &[
    Field::text("name", "Nom", true),
    Field::text("customer_id", "Client", false),
    Field::text("customer_context_id", "Contexte du client", false),
    Field::text("quote_id", "Devis", false),
];

const PROJECT_FIELDS: &[Field] = &[Field::text("project_id", "Projet", true)];

const UPDATE_PROJECT_FIELDS: &[Field] = &[
    Field::text("project_id", "Projet", true),
    Field::text("name", "Nom", true),
    Field::text("customer_id", "Client", false),
    Field::text("customer_context_id", "Contexte du client", false),
    Field::text("quote_id", "Devis", false),
];

const CREATE_CHANNEL_FIELDS: &[Field] = &[
    Field::text("project_id", "Projet", true),
    Field::text("name", "Nom du canal", false),
];

const CREATE_DOE_FIELDS: &[Field] = &[
    Field::text("project_id", "Projet", true),
    Field::text("model_id", "Modèle de DOE", true),
];

const LIST_LABELS_FIELDS: &[Field] =
    &[Field::number("limit", "Nombre maximum d’étiquettes", false)];

const CREATE_LABEL_FIELDS: &[Field] = &[
    Field::text("name", "Nom", true),
    Field::text("color", "Couleur (#RRGGBB)", true),
];

const LABEL_FIELDS: &[Field] = &[Field::text("label_id", "Étiquette", true)];

const UPDATE_LABEL_FIELDS: &[Field] = &[
    Field::text("label_id", "Étiquette", true),
    Field::text("name", "Nom", true),
    Field::text("color", "Couleur (#RRGGBB)", true),
];

const LIST_RECURRENCES_FIELDS: &[Field] = &[Field::number(
    "limit",
    "Nombre maximum de récurrences",
    false,
)];

const CREATE_RECURRENCE_FIELDS: &[Field] = &[
    Field::select("frequency", "Fréquence", true, FREQUENCIES),
    Field::json("weekdays", "Jours de la semaine (1 à 7)", false),
    Field::number("day_of_month", "Jour du mois (1 à 31)", false),
    Field::text("starts_on", "Premier jour (AAAA-MM-JJ)", true),
    Field::text("ends_on", "Dernier jour (AAAA-MM-JJ)", false),
    Field::text("timezone", "Fuseau horaire (IANA)", true),
    Field::text("start_time", "Heure de début (HH:MM)", true),
    Field::number("duration_minutes", "Durée en minutes", true),
    Field::boolean("all_day", "Toute la journée", false),
    Field::text("title", "Titre", true),
    Field::text("description", "Description", false),
    Field::boolean("blocks_availability", "Bloque la disponibilité", true),
    Field::text("customer_id", "Client", false),
    Field::text("customer_context_id", "Contexte du client", false),
    Field::text("project_id", "Projet", false),
    Field::json("assignee_member_ids", "Membres assignés", false),
];

const RECURRENCE_FIELDS: &[Field] = &[Field::text("recurrence_id", "Récurrence", true)];

const UPDATE_RECURRENCE_FIELDS: &[Field] = &[
    Field::text("recurrence_id", "Récurrence", true),
    Field::select("frequency", "Fréquence", false, FREQUENCIES),
    Field::json("weekdays", "Jours de la semaine (1 à 7)", false),
    Field::number("day_of_month", "Jour du mois (1 à 31)", false),
    Field::text(
        "ends_on",
        "Dernier jour (AAAA-MM-JJ, nul pour effacer)",
        false,
    ),
    Field::text("start_time", "Heure de début (HH:MM)", false),
    Field::number("duration_minutes", "Durée en minutes", false),
    Field::boolean("all_day", "Toute la journée", false),
    Field::text("title", "Titre", false),
    Field::text("description", "Description (nul pour effacer)", false),
    Field::boolean("blocks_availability", "Bloque la disponibilité", false),
    Field::text("project_id", "Projet (nul pour détacher)", false),
    Field::json("assignee_member_ids", "Membres assignés", false),
];

const LIST_TASKS_FIELDS: &[Field] = &[
    Field::text("parent_task_id", "Tâche parente", false),
    Field::text("project_id", "Projet", false),
    Field::json("statuses", "Statuts", false),
    Field::text("assignee_member_id", "Membre assigné", false),
    Field::text("label_id", "Étiquette", false),
    Field::text("customer_id", "Client", false),
    Field::text("q", "Texte du titre", false),
    Field::boolean("unscheduled", "Seulement les tâches non planifiées", false),
    Field::number("limit", "Nombre maximum de tâches", false),
    Field::number("offset", "Décalage", false),
];

const CREATE_TASK_FIELDS: &[Field] = &[
    Field::text("title", "Titre", true),
    Field::text("description", "Description", false),
    Field::text("parent_task_id", "Tâche parente", false),
    Field::text("starts_at", "Début (date et heure ISO 8601)", false),
    Field::text("ends_at", "Fin (date et heure ISO 8601)", false),
    Field::boolean("all_day", "Toute la journée", false),
    Field::select("status", "Statut", false, TASK_STATUSES),
    Field::boolean("blocks_availability", "Bloque la disponibilité", true),
    Field::text("customer_id", "Client", false),
    Field::text("customer_context_id", "Contexte du client", false),
    Field::text("quote_id", "Devis", false),
    Field::text("project_id", "Projet", false),
    Field::number("expenses_cents", "Frais en centimes", false),
    Field::text("expenses_label", "Libellé des frais", false),
];

const TASK_FIELDS: &[Field] = &[Field::text("task_id", "Tâche", true)];

const UPDATE_TASK_FIELDS: &[Field] = &[
    Field::text("task_id", "Tâche", true),
    Field::text("title", "Titre", false),
    Field::text("description", "Description (nul pour effacer)", false),
    Field::text("parent_task_id", "Tâche parente (nul pour détacher)", false),
    Field::text("starts_at", "Début (nul pour effacer)", false),
    Field::text("ends_at", "Fin (nul pour effacer)", false),
    Field::boolean("all_day", "Toute la journée", false),
    Field::select("status", "Statut", false, TASK_STATUSES),
    Field::boolean("blocks_availability", "Bloque la disponibilité", false),
    Field::json("assignee_member_ids", "Membres assignés", false),
    Field::json("label_ids", "Étiquettes", false),
    Field::json("equipment_ids", "Équipements", false),
    Field::text("project_id", "Projet (nul pour détacher)", false),
    Field::number("expenses_cents", "Frais en centimes", false),
    Field::text(
        "expenses_label",
        "Libellé des frais (nul pour effacer)",
        false,
    ),
    Field::text("preceding_task_id", "Tâche placée juste au-dessus", false),
    Field::text("following_task_id", "Tâche placée juste en dessous", false),
];

const DELETE_TASK_FIELDS: &[Field] = &[
    Field::text("task_id", "Tâche", true),
    Field::select("scope", "Portée", false, DELETE_SCOPES),
];

const BULK_ASSIGN_FIELDS: &[Field] = &[
    Field::json("task_ids", "Tâches", true),
    Field::json("assignee_member_ids", "Membres assignés", true),
];

const LIST_COMMENTS_FIELDS: &[Field] = &[
    Field::text("task_id", "Tâche", true),
    Field::number("limit", "Nombre maximum de commentaires", false),
    Field::number("offset", "Décalage", false),
];

const CREATE_COMMENT_FIELDS: &[Field] = &[
    Field::text("task_id", "Tâche", true),
    Field::text("body", "Texte du commentaire", true),
];

const UPDATE_COMMENT_FIELDS: &[Field] = &[
    Field::text("comment_id", "Commentaire", true),
    Field::text("body", "Nouveau texte", true),
];

const DELETE_COMMENT_FIELDS: &[Field] = &[Field::text("comment_id", "Commentaire", true)];

fn comment_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "task_id": NIL,
        "author": { "id": NIL, "display_name": "Automatisation" },
        "body": "Matériel livré sur place.",
        "created_at": MOMENT,
        "updated_at": MOMENT,
        "author_is_self": true,
    })
}

fn deleted_comment_sample() -> Value {
    json!({ "comment_id": NIL, "deleted": true })
}

fn label_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "name": "Urgent",
        "color": "#E5484D",
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn labels_sample() -> Value {
    json!([label_sample()])
}

fn deleted_label_sample() -> Value {
    json!({ "label_id": NIL, "deleted": true })
}

fn task_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "parent_task_id": null,
        "title": "Tailler la haie",
        "description": null,
        "starts_at": MOMENT,
        "ends_at": "2026-01-01T02:00:00Z",
        "all_day": false,
        "status": "PLANNED",
        "blocks_availability": true,
        "customer_id": null,
        "customer_context_id": null,
        "quote_id": null,
        "project_id": null,
        "recurrence_id": null,
        "maintenance_line_id": null,
        "expenses_cents": 0,
        "expenses_label": null,
        "member_ids": [NIL],
        "assignments": [{ "id": NIL, "member_id": NIL }],
        "child_count": 0,
        "labels": [label_sample()],
        "equipment": [{
            "id": NIL,
            "organization_id": NIL,
            "name": "Taille-haie",
            "hourly_rate_cents": 1500,
            "created_at": MOMENT,
            "updated_at": MOMENT,
        }],
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn tasks_sample() -> Value {
    json!([task_sample()])
}

fn patched_task_sample() -> Value {
    json!({ "task": task_sample(), "detached": false })
}

fn bulk_assigned_sample() -> Value {
    json!({ "tasks": [task_sample()] })
}

fn deleted_task_sample() -> Value {
    json!({ "task_id": NIL, "scope": "this_occurrence", "deleted": true })
}

fn comments_sample() -> Value {
    json!([{
        "id": NIL,
        "organization_id": NIL,
        "task_id": NIL,
        "author": { "id": NIL, "display_name": "Claire Fontaine" },
        "body": "Matériel livré sur place.",
        "created_at": MOMENT,
        "updated_at": MOMENT,
        "author_is_self": false,
    }])
}

fn project_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "name": "Jardin Fontaine",
        "customer_id": NIL,
        "customer_context_id": NIL,
        "quote_id": null,
        "archived_at": null,
        "is_internal": false,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn projects_sample() -> Value {
    json!([project_sample()])
}

fn archived_project_sample() -> Value {
    json!({ "project_id": NIL, "archived": true })
}

fn channel_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "project_id": NIL,
        "name": "jardin-fontaine",
        "topic": null,
        "archived": false,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn doe_sample() -> Value {
    json!({
        "id": NIL,
        "project_id": NIL,
        "model_id": NIL,
        "title": "DOE Jardin Fontaine",
        "place": null,
        "lot": null,
        "issued_on": "2026-01-01",
        "author": null,
        "revision": 1,
        "cover_photo_key": null,
        "parties": [],
        "sections": [],
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn template_task_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "template_id": NIL,
        "title": "Préparer le terrain",
        "description": null,
        "day_offset": 0,
        "starts_minute": 480,
        "ends_minute": 600,
        "all_day": false,
        "blocks_availability": true,
        "expenses_cents": 0,
        "expenses_label": null,
        "parent_index": null,
        "position": 0,
    })
}

fn template_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "name": "Création de jardin",
        "description": null,
        "archived_at": null,
        "tasks": [template_task_sample()],
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn templates_sample() -> Value {
    json!([template_sample()])
}

fn archived_template_sample() -> Value {
    json!({ "template_id": NIL, "archived": true })
}

fn template_tasks_sample() -> Value {
    json!([template_task_sample()])
}

fn instantiated_sample() -> Value {
    json!({ "project": project_sample(), "tasks": [task_sample()] })
}

fn recurrence_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "frequency": "WEEKLY",
        "weekdays": [2],
        "starts_on": "2026-01-06",
        "ends_on": null,
        "horizon_filled_to": "2026-03-06",
        "timezone": "Europe/Paris",
        "start_time": "09:00:00",
        "duration_minutes": 60,
        "all_day": false,
        "title": "Réunion hebdomadaire",
        "description": null,
        "blocks_availability": true,
        "customer_id": null,
        "customer_context_id": null,
        "project_id": null,
        "assignee_member_ids": [NIL],
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn recurrences_sample() -> Value {
    json!([recurrence_sample()])
}

fn deleted_recurrence_sample() -> Value {
    json!({ "recurrence_id": NIL, "deleted": true })
}

fn planning_sample() -> Value {
    json!({
        "timezone": "Europe/Paris",
        "resources": [{
            "resource_id": format!("member:{NIL}"),
            "member_id": NIL,
            "display_name": "Claire Fontaine",
            "employee_id": null,
            "hourly_rate_cents": 2500,
            "weekly_contract_minutes": 2100,
        }],
        "entries": [{
            "kind": "task",
            "id": NIL,
            "parent_task_id": null,
            "recurrence_id": null,
            "starts_at": MOMENT,
            "ends_at": "2026-01-01T02:00:00Z",
            "all_day": false,
            "status": "PLANNED",
            "blocks_availability": true,
            "child_count": 0,
            "title": "Tailler la haie",
            "customer_name": null,
            "context_label": null,
            "description": null,
            "member_ids": [NIL],
            "labels": [],
        }],
        "work_time": [{
            "member_id": NIL,
            "days": [{
                "date": "2026-01-01",
                "intervals": [{ "starts_minute": 480, "ends_minute": 1020 }],
            }],
        }],
    })
}

fn availability_sample() -> Value {
    json!({
        "resources": [{
            "resource_id": format!("member:{NIL}"),
            "available": false,
            "conflicts": [{
                "kind": "overlapping_task",
                "task_id": NIL,
                "starts_at": MOMENT,
                "ends_at": "2026-01-01T02:00:00Z",
            }],
        }],
    })
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.planning.get_planning",
        label: "Lire le planning",
        description: "Renvoie les ressources, les entrées et le temps de travail sur une fenêtre de 92 jours au plus.",
        group: GROUP,
        fields: GET_PLANNING_FIELDS,
        read_only: true,
        sample_output: planning_sample,
    },
    ActionSpec {
        kind: "mestier.planning.get_availability",
        label: "Lire les disponibilités",
        description: "Indique pour chaque ressource si elle est disponible sur un créneau, et pourquoi elle ne l’est pas.",
        group: GROUP,
        fields: GET_AVAILABILITY_FIELDS,
        read_only: true,
        sample_output: availability_sample,
    },
    ActionSpec {
        kind: "mestier.project_template.list",
        label: "Lister les modèles de projet",
        description: "Renvoie les modèles de projet de l’organisation.",
        group: GROUP,
        fields: LIST_TEMPLATES_FIELDS,
        read_only: true,
        sample_output: templates_sample,
    },
    ActionSpec {
        kind: "mestier.project_template.create",
        label: "Créer un modèle de projet",
        description: "Crée un modèle de projet avec ses tâches types.",
        group: GROUP,
        fields: CREATE_TEMPLATE_FIELDS,
        read_only: false,
        sample_output: template_sample,
    },
    ActionSpec {
        kind: "mestier.project_template.get",
        label: "Lire un modèle de projet",
        description: "Renvoie un modèle de projet de l’organisation avec ses tâches.",
        group: GROUP,
        fields: TEMPLATE_FIELDS,
        read_only: true,
        sample_output: template_sample,
    },
    ActionSpec {
        kind: "mestier.project_template.update",
        label: "Modifier un modèle de projet",
        description: "Remplace le nom et la description d’un modèle de projet.",
        group: GROUP,
        fields: UPDATE_TEMPLATE_FIELDS,
        read_only: false,
        sample_output: template_sample,
    },
    ActionSpec {
        kind: "mestier.project_template.archive",
        label: "Archiver un modèle de projet",
        description: "Archive un modèle de projet.",
        group: GROUP,
        fields: TEMPLATE_FIELDS,
        read_only: false,
        sample_output: archived_template_sample,
    },
    ActionSpec {
        kind: "mestier.project_template.restore",
        label: "Restaurer un modèle de projet",
        description: "Sort un modèle de projet de l’archive.",
        group: GROUP,
        fields: TEMPLATE_FIELDS,
        read_only: false,
        sample_output: template_sample,
    },
    ActionSpec {
        kind: "mestier.project_template.replace_tasks",
        label: "Remplacer les tâches d’un modèle",
        description: "Remplace toutes les tâches types d’un modèle de projet.",
        group: GROUP,
        fields: REPLACE_TEMPLATE_TASKS_FIELDS,
        read_only: false,
        sample_output: template_tasks_sample,
    },
    ActionSpec {
        kind: "mestier.project_template.instantiate",
        label: "Créer un projet depuis un modèle",
        description: "Crée un projet et ses tâches à partir d’un modèle, à partir d’une date de début.",
        group: GROUP,
        fields: INSTANTIATE_TEMPLATE_FIELDS,
        read_only: false,
        sample_output: instantiated_sample,
    },
    ActionSpec {
        kind: "mestier.project.list",
        label: "Lister les projets",
        description: "Renvoie les projets de l’organisation, filtrés par client ou par devis.",
        group: GROUP,
        fields: LIST_PROJECTS_FIELDS,
        read_only: true,
        sample_output: projects_sample,
    },
    ActionSpec {
        kind: "mestier.project.create_project",
        label: "Créer un projet",
        description: "Crée un projet, interne ou rattaché à un client et à un devis.",
        group: GROUP,
        fields: CREATE_PROJECT_FIELDS,
        read_only: false,
        sample_output: project_sample,
    },
    ActionSpec {
        kind: "mestier.project.get",
        label: "Lire un projet",
        description: "Renvoie un projet de l’organisation.",
        group: GROUP,
        fields: PROJECT_FIELDS,
        read_only: true,
        sample_output: project_sample,
    },
    ActionSpec {
        kind: "mestier.project.update",
        label: "Modifier un projet",
        description: "Remplace le nom et les rattachements d’un projet. Un rattachement non fourni est retiré.",
        group: GROUP,
        fields: UPDATE_PROJECT_FIELDS,
        read_only: false,
        sample_output: project_sample,
    },
    ActionSpec {
        kind: "mestier.project.archive",
        label: "Archiver un projet",
        description: "Archive un projet et son canal de discussion.",
        group: GROUP,
        fields: PROJECT_FIELDS,
        read_only: false,
        sample_output: archived_project_sample,
    },
    ActionSpec {
        kind: "mestier.project.restore",
        label: "Restaurer un projet",
        description: "Sort un projet de l’archive.",
        group: GROUP,
        fields: PROJECT_FIELDS,
        read_only: false,
        sample_output: project_sample,
    },
    ActionSpec {
        kind: "mestier.project.get_channel",
        label: "Lire le canal d’un projet",
        description: "Renvoie le canal de discussion d’un projet.",
        group: GROUP,
        fields: PROJECT_FIELDS,
        read_only: true,
        sample_output: channel_sample,
    },
    ActionSpec {
        kind: "mestier.project.create_channel",
        label: "Créer le canal d’un projet",
        description: "Crée le canal de discussion d’un projet, s’il n’en a pas déjà un.",
        group: GROUP,
        fields: CREATE_CHANNEL_FIELDS,
        read_only: false,
        sample_output: channel_sample,
    },
    ActionSpec {
        kind: "mestier.project.get_doe",
        label: "Lire le DOE d’un projet",
        description: "Renvoie le dossier des ouvrages exécutés d’un projet.",
        group: GROUP,
        fields: PROJECT_FIELDS,
        read_only: true,
        sample_output: doe_sample,
    },
    ActionSpec {
        kind: "mestier.project.create_doe",
        label: "Créer le DOE d’un projet",
        description: "Crée le dossier des ouvrages exécutés d’un projet à partir d’un modèle.",
        group: GROUP,
        fields: CREATE_DOE_FIELDS,
        read_only: false,
        sample_output: doe_sample,
    },
    ActionSpec {
        kind: "mestier.task_label.list",
        label: "Lister les étiquettes de tâche",
        description: "Renvoie les étiquettes de tâche de l’organisation.",
        group: GROUP,
        fields: LIST_LABELS_FIELDS,
        read_only: true,
        sample_output: labels_sample,
    },
    ActionSpec {
        kind: "mestier.task_label.create",
        label: "Créer une étiquette de tâche",
        description: "Crée une étiquette de tâche avec sa couleur.",
        group: GROUP,
        fields: CREATE_LABEL_FIELDS,
        read_only: false,
        sample_output: label_sample,
    },
    ActionSpec {
        kind: "mestier.task_label.update",
        label: "Modifier une étiquette de tâche",
        description: "Remplace le nom et la couleur d’une étiquette de tâche.",
        group: GROUP,
        fields: UPDATE_LABEL_FIELDS,
        read_only: false,
        sample_output: label_sample,
    },
    ActionSpec {
        kind: "mestier.task_label.delete",
        label: "Supprimer une étiquette de tâche",
        description: "Supprime une étiquette de tâche.",
        group: GROUP,
        fields: LABEL_FIELDS,
        read_only: false,
        sample_output: deleted_label_sample,
    },
    ActionSpec {
        kind: "mestier.task_recurrence.list",
        label: "Lister les récurrences de tâche",
        description: "Renvoie les récurrences de tâche de l’organisation.",
        group: GROUP,
        fields: LIST_RECURRENCES_FIELDS,
        read_only: true,
        sample_output: recurrences_sample,
    },
    ActionSpec {
        kind: "mestier.task_recurrence.create",
        label: "Créer une récurrence de tâche",
        description: "Crée une tâche qui se répète chaque jour, chaque semaine ou chaque mois.",
        group: GROUP,
        fields: CREATE_RECURRENCE_FIELDS,
        read_only: false,
        sample_output: recurrence_sample,
    },
    ActionSpec {
        kind: "mestier.task_recurrence.update",
        label: "Modifier une récurrence de tâche",
        description: "Modifie les champs fournis d’une récurrence. Une valeur nulle efface les champs qui le permettent.",
        group: GROUP,
        fields: UPDATE_RECURRENCE_FIELDS,
        read_only: false,
        sample_output: recurrence_sample,
    },
    ActionSpec {
        kind: "mestier.task_recurrence.delete",
        label: "Supprimer une récurrence de tâche",
        description: "Supprime une récurrence de tâche.",
        group: GROUP,
        fields: RECURRENCE_FIELDS,
        read_only: false,
        sample_output: deleted_recurrence_sample,
    },
    ActionSpec {
        kind: "mestier.task.list",
        label: "Lister les tâches",
        description: "Renvoie les tâches de l’organisation, filtrées par projet, statut, membre, étiquette, client ou texte.",
        group: GROUP,
        fields: LIST_TASKS_FIELDS,
        read_only: true,
        sample_output: tasks_sample,
    },
    ActionSpec {
        kind: "mestier.task.create",
        label: "Créer une tâche",
        description: "Crée une tâche, planifiée ou non, éventuellement rattachée à un projet, un client ou un devis.",
        group: GROUP,
        fields: CREATE_TASK_FIELDS,
        read_only: false,
        sample_output: task_sample,
    },
    ActionSpec {
        kind: "mestier.task.get",
        label: "Lire une tâche",
        description: "Renvoie une tâche de l’organisation avec ses membres, ses étiquettes et ses équipements.",
        group: GROUP,
        fields: TASK_FIELDS,
        read_only: true,
        sample_output: task_sample,
    },
    ActionSpec {
        kind: "mestier.task.update",
        label: "Modifier une tâche",
        description: "Modifie les champs fournis d’une tâche, y compris ses membres, étiquettes, équipements et sa place dans le tableau. Une valeur nulle efface les champs qui le permettent.",
        group: GROUP,
        fields: UPDATE_TASK_FIELDS,
        read_only: false,
        sample_output: patched_task_sample,
    },
    ActionSpec {
        kind: "mestier.task.delete",
        label: "Supprimer une tâche",
        description: "Supprime une tâche, et au choix les occurrences suivantes de sa série.",
        group: GROUP,
        fields: DELETE_TASK_FIELDS,
        read_only: false,
        sample_output: deleted_task_sample,
    },
    ActionSpec {
        kind: "mestier.task.bulk_assign",
        label: "Assigner plusieurs tâches",
        description: "Remplace les membres assignés de toutes les tâches données, en une seule opération.",
        group: GROUP,
        fields: BULK_ASSIGN_FIELDS,
        read_only: false,
        sample_output: bulk_assigned_sample,
    },
    ActionSpec {
        kind: "mestier.task_comment.list",
        label: "Lister les commentaires d’une tâche",
        description: "Renvoie les commentaires d’une tâche, du plus ancien au plus récent.",
        group: GROUP,
        fields: LIST_COMMENTS_FIELDS,
        read_only: true,
        sample_output: comments_sample,
    },
    ActionSpec {
        kind: "mestier.task_comment.create",
        label: "Commenter une tâche",
        description: "Ajoute un commentaire à une tâche, signé par l’identité Automatisation.",
        group: GROUP,
        fields: CREATE_COMMENT_FIELDS,
        read_only: false,
        sample_output: comment_sample,
    },
    ActionSpec {
        kind: "mestier.task_comment.update",
        label: "Modifier un commentaire d’automatisation",
        description: "Remplace le texte d’un commentaire écrit par l’identité Automatisation. Les commentaires des autres auteurs sont refusés.",
        group: GROUP,
        fields: UPDATE_COMMENT_FIELDS,
        read_only: false,
        sample_output: comment_sample,
    },
    ActionSpec {
        kind: "mestier.task_comment.delete",
        label: "Supprimer un commentaire d’automatisation",
        description: "Supprime un commentaire écrit par l’identité Automatisation. Les commentaires des autres auteurs sont refusés.",
        group: GROUP,
        fields: DELETE_COMMENT_FIELDS,
        read_only: false,
        sample_output: deleted_comment_sample,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hand_written_connectors_are_not_generated_again() {
        for spec in SPECS {
            assert!(
                !matches!(
                    spec.kind,
                    "mestier.project.create" | "mestier.task_recurrence.extend_horizon"
                ),
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
                "mestier.planning.get_planning",
                "mestier.planning.get_availability",
                "mestier.project_template.list",
                "mestier.project_template.get",
                "mestier.project.list",
                "mestier.project.get",
                "mestier.project.get_channel",
                "mestier.project.get_doe",
                "mestier.task_label.list",
                "mestier.task_recurrence.list",
                "mestier.task.list",
                "mestier.task.get",
                "mestier.task_comment.list",
            ]
        );
    }

    #[test]
    fn the_task_status_options_are_the_ones_the_service_parses() {
        use std::str::FromStr;

        use crate::TaskStatus;

        for option in TASK_STATUSES {
            assert!(
                TaskStatus::from_str(option.value).is_ok(),
                "{}",
                option.value
            );
        }
    }

    #[test]
    fn a_required_field_is_never_offered_twice() {
        for spec in SPECS {
            let mut names: Vec<&str> = spec.fields.iter().map(|field| field.name).collect();
            names.sort_unstable();
            names.dedup();
            assert_eq!(names.len(), spec.fields.len(), "{}", spec.kind);
        }
    }
}
