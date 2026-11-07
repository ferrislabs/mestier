use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::{Field, SelectOption};

const GROUP: &str = "Support";
const NIL: &str = "00000000-0000-0000-0000-000000000000";

const STATUSES: &[SelectOption] = &[
    SelectOption::new("new", "Nouveau"),
    SelectOption::new("in_progress", "En cours"),
    SelectOption::new("waiting_customer", "En attente du client"),
    SelectOption::new("resolved", "Résolu"),
];

const CREATE_TICKET_FIELDS: &[Field] = &[
    Field::text("subject", "Sujet", true),
    Field::text("body", "Message", true),
    Field::json("custom_fields", "Champs personnalisés", false),
    Field::text("external_id", "Identifiant externe", false),
];

const TICKET_FIELDS: &[Field] = &[Field::text("ticket_id", "Ticket", true)];

const LIST_TICKETS_FIELDS: &[Field] = &[
    Field::json("statuses", "Statuts", false),
    Field::text("assignee_member_id", "Membre assigné", false),
    Field::text("badge", "Badge", false),
    Field::text("custom_field_key", "Clé du champ personnalisé", false),
    Field::text("custom_field_value", "Valeur du champ personnalisé", false),
    Field::number("limit", "Nombre maximum de tickets", false),
    Field::number("offset", "Décalage", false),
];

const TRANSITION_FIELDS: &[Field] = &[
    Field::text("ticket_id", "Ticket", true),
    Field::select("status", "Statut", true, STATUSES),
];

const SET_ASSIGNEE_FIELDS: &[Field] = &[
    Field::text("ticket_id", "Ticket", true),
    Field::text(
        "assignee_member_id",
        "Membre (vide pour désassigner)",
        false,
    ),
];

const REMOVE_BADGE_FIELDS: &[Field] = &[
    Field::text("ticket_id", "Ticket", true),
    Field::text("label", "Libellé du badge", true),
];

const LIST_THREAD_FIELDS: &[Field] = &[
    Field::text("ticket_id", "Ticket", true),
    Field::boolean(
        "client_visible_only",
        "Messages visibles du client seulement",
        false,
    ),
];

const REPLY_FIELDS: &[Field] = &[
    Field::text("ticket_id", "Ticket", true),
    Field::text("body", "Message", true),
];

const DRAFT_FIELDS: &[Field] = &[Field::text("entry_id", "Brouillon", true)];

const EDIT_DRAFT_FIELDS: &[Field] = &[
    Field::text("entry_id", "Brouillon", true),
    Field::text("body", "Message", true),
];

fn entry_sample() -> Value {
    json!({
        "id": NIL,
        "ticket_id": NIL,
        "kind": "reply",
        "author_member_id": null,
        "author_name": "Automatisation",
        "body": "Bonjour, nous passons demain matin.",
        "attachments": [],
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn deleted_draft_sample() -> Value {
    json!({ "entry_id": NIL, "deleted": true })
}

fn ticket_sample() -> Value {
    json!({
        "id": NIL,
        "number": 42,
        "subject": "Chaudière en panne",
        "body": "Plus d'eau chaude depuis ce matin.",
        "status": "new",
        "assignee_member_id": NIL,
        "assigned_at": "2026-01-01T00:00:00Z",
        "custom_fields": {},
        "badges": [{ "label": "High", "tone": "neutral" }],
        "external_id": null,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn created_ticket_sample() -> Value {
    let mut sample = ticket_sample();
    sample["created"] = json!(true);
    sample
}

fn tickets_sample() -> Value {
    json!([ticket_sample()])
}

fn removed_badge_sample() -> Value {
    json!({ "ticket_id": NIL, "label": "High", "removed": true })
}

fn thread_sample() -> Value {
    json!([{
        "id": NIL,
        "ticket_id": NIL,
        "kind": "internal_note",
        "author_member_id": null,
        "author_name": "Automatisation",
        "body": "Rappeler le client.",
        "attachments": [],
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    }])
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.support.create_ticket",
        label: "Créer un ticket",
        description: "Ouvre un ticket de support. Avec un identifiant externe déjà connu, renvoie le ticket existant.",
        group: GROUP,
        fields: CREATE_TICKET_FIELDS,
        read_only: false,
        sample_output: created_ticket_sample,
    },
    ActionSpec {
        kind: "mestier.support.get_ticket",
        label: "Lire un ticket",
        description: "Renvoie un ticket de l'organisation.",
        group: GROUP,
        fields: TICKET_FIELDS,
        read_only: true,
        sample_output: ticket_sample,
    },
    ActionSpec {
        kind: "mestier.support.list_tickets",
        label: "Lister les tickets",
        description: "Renvoie les tickets de l'organisation, filtrés par statut, assigné, badge ou champ personnalisé.",
        group: GROUP,
        fields: LIST_TICKETS_FIELDS,
        read_only: true,
        sample_output: tickets_sample,
    },
    ActionSpec {
        kind: "mestier.support.transition_ticket",
        label: "Changer le statut d'un ticket",
        description: "Fait passer un ticket à un autre statut.",
        group: GROUP,
        fields: TRANSITION_FIELDS,
        read_only: false,
        sample_output: ticket_sample,
    },
    ActionSpec {
        kind: "mestier.support.set_assignee",
        label: "Définir l'assigné d'un ticket",
        description: "Assigne un ticket à un membre, ou le désassigne si aucun membre n'est donné.",
        group: GROUP,
        fields: SET_ASSIGNEE_FIELDS,
        read_only: false,
        sample_output: ticket_sample,
    },
    ActionSpec {
        kind: "mestier.support.remove_badge",
        label: "Retirer un badge",
        description: "Retire un badge d'un ticket.",
        group: GROUP,
        fields: REMOVE_BADGE_FIELDS,
        read_only: false,
        sample_output: removed_badge_sample,
    },
    ActionSpec {
        kind: "mestier.support.list_thread",
        label: "Lire le fil d'un ticket",
        description: "Renvoie les messages, réponses et notes d'un ticket.",
        group: GROUP,
        fields: LIST_THREAD_FIELDS,
        read_only: true,
        sample_output: thread_sample,
    },
    ActionSpec {
        kind: "mestier.support.send_reply",
        label: "Répondre à un ticket",
        description: "Envoie une réponse visible du client, signée Automatisation.",
        group: GROUP,
        fields: REPLY_FIELDS,
        read_only: false,
        sample_output: entry_sample,
    },
    ActionSpec {
        kind: "mestier.support.save_draft",
        label: "Enregistrer un brouillon de réponse",
        description: "Crée un brouillon de réponse, signé Automatisation, non visible du client.",
        group: GROUP,
        fields: REPLY_FIELDS,
        read_only: false,
        sample_output: entry_sample,
    },
    ActionSpec {
        kind: "mestier.support.edit_draft",
        label: "Modifier un brouillon",
        description: "Remplace le texte d'un brouillon.",
        group: GROUP,
        fields: EDIT_DRAFT_FIELDS,
        read_only: false,
        sample_output: entry_sample,
    },
    ActionSpec {
        kind: "mestier.support.delete_draft",
        label: "Supprimer un brouillon",
        description: "Supprime un brouillon.",
        group: GROUP,
        fields: DRAFT_FIELDS,
        read_only: false,
        sample_output: deleted_draft_sample,
    },
    ActionSpec {
        kind: "mestier.support.send_draft",
        label: "Envoyer un brouillon",
        description: "Transforme un brouillon en réponse visible du client, signée Automatisation.",
        group: GROUP,
        fields: DRAFT_FIELDS,
        read_only: false,
        sample_output: entry_sample,
    },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_hand_written_support_connectors_are_not_generated_again() {
        for spec in SPECS {
            assert!(
                !matches!(
                    spec.kind,
                    "mestier.support.assign_ticket"
                        | "mestier.support.add_badge"
                        | "mestier.note.add"
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
                "mestier.support.get_ticket",
                "mestier.support.list_tickets",
                "mestier.support.list_thread"
            ]
        );
    }

    #[test]
    fn the_status_options_are_the_ones_the_service_parses() {
        use std::str::FromStr;

        use crate::TicketStatus;

        for option in STATUSES {
            assert!(
                TicketStatus::from_str(option.value).is_ok(),
                "{}",
                option.value
            );
        }
        assert_eq!(STATUSES.len(), TicketStatus::ALL.len());
    }
}
