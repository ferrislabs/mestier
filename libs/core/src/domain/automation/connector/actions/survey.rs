use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::Field;

#[cfg(test)]
#[path = "tests_survey.rs"]
mod tests_survey;

const GROUP: &str = "Questionnaires";
const NIL: &str = "00000000-0000-0000-0000-000000000000";
const SLUG: &str = "q7Xk2mP9vR4tL8wZ3nB6cA";

const SURVEY_FIELDS: &[Field] = &[Field::text("survey_id", "Questionnaire", true)];

const OPEN_FIELDS: &[Field] = &[
    Field::text("survey_id", "Questionnaire", true),
    Field::text("campaign_id", "Campagne de rattachement", false),
];

const RESPONSES_FIELDS: &[Field] = &[
    Field::text("survey_id", "Questionnaire", true),
    Field::number(
        "limit",
        "Nombre de réponses, les plus récentes d’abord (50 par défaut, 100 au plus)",
        false,
    ),
    Field::number(
        "offset",
        "Réponses à sauter avant de lister (0 par défaut)",
        false,
    ),
];

fn survey_sample() -> Value {
    json!({
        "id": NIL,
        "title": "Bilan de fin de session",
        "status": "OPENED",
        "visibility": "PUBLIC",
        "campaign_ids": [],
        "public_slug": SLUG,
        "opened_at": "2026-01-01T00:00:00Z",
        "closed_at": null,
    })
}

fn closed_survey_sample() -> Value {
    json!({
        "id": NIL,
        "title": "Bilan de fin de session",
        "status": "CLOSED",
        "visibility": "PUBLIC",
        "campaign_ids": [],
        "public_slug": SLUG,
        "opened_at": "2026-01-01T00:00:00Z",
        "closed_at": "2026-01-31T00:00:00Z",
    })
}

pub(super) fn link_sample() -> Value {
    json!({
        "survey_id": NIL,
        "status": "OPENED",
        "public_slug": SLUG,
        "path": format!("/s/{SLUG}"),
    })
}

pub(super) fn responses_sample() -> Value {
    json!({
        "total": 1,
        "responses": [{
            "id": NIL,
            "member_id": null,
            "completed_at": "2026-01-02T00:00:00Z",
            "identifier_value": null,
            "graph_version": 1,
            "answers": [{
                "question_id": NIL,
                "label": "Nom",
                "kind": "TEXT",
                "value": "Claire Fontaine",
            }],
        }],
    })
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.survey.open",
        label: "Ouvrir un questionnaire",
        description: "Ouvre un questionnaire en brouillon à son audience. Ajoute la campagne aux campagnes du questionnaire si elle est renseignée, dans le même temps : si l’ouverture échoue, le brouillon reste inchangé.",
        group: GROUP,
        fields: OPEN_FIELDS,
        read_only: false,
        sample_output: survey_sample,
    },
    ActionSpec {
        kind: "mestier.survey.close",
        label: "Fermer un questionnaire",
        description: "Ferme un questionnaire ouvert : il ne reçoit plus de réponse.",
        group: GROUP,
        fields: SURVEY_FIELDS,
        read_only: false,
        sample_output: closed_survey_sample,
    },
    ActionSpec {
        kind: "mestier.survey.get_link",
        label: "Lire le lien d’un questionnaire",
        description: "Renvoie l’identifiant public et le chemin de la page de réponse d’un questionnaire public ouvert.",
        group: GROUP,
        fields: SURVEY_FIELDS,
        read_only: true,
        sample_output: link_sample,
    },
    ActionSpec {
        kind: "mestier.survey.list_responses",
        label: "Lister les réponses d’un questionnaire",
        description: "Renvoie les réponses complétées d’un questionnaire, les plus récentes d’abord, 50 par défaut et 100 au plus par appel, avec leur nombre total et les réponses du chemin parcouru.",
        group: GROUP,
        fields: RESPONSES_FIELDS,
        read_only: true,
        sample_output: responses_sample,
    },
];
