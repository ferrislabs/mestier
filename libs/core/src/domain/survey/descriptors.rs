use events::EventDescriptor;
use serde_json::{Value, json};

#[cfg(test)]
pub fn emitted_events() -> Vec<(&'static str, u16)> {
    vec![
        ("survey.opened", 1),
        ("survey.response_submitted", 1),
        ("survey.response_completed", 1),
        ("survey_campaign.started", 1),
        ("survey_campaign.ended", 1),
    ]
}

fn campaign_example() -> Value {
    json!({
        "campaign_id": "018f3b2a-0000-7000-8000-000000000040",
        "campaign": {
            "id": "018f3b2a-0000-7000-8000-000000000040",
            "organization_id": "018f3b2a-0000-7000-8000-000000000002",
            "name": "Satisfaction printemps",
            "starts_at": "2026-03-01T08:00:00Z",
            "ends_at": "2026-03-31T18:00:00Z",
            "metadata": { "saison": "printemps" },
        },
    })
}

pub fn descriptors() -> Vec<EventDescriptor> {
    vec![
        EventDescriptor {
            name: "survey.opened",
            version: 1,
            label: "Questionnaire ouvert",
            subject_kind: "survey",
            payload_example: json!({
                "survey_id": "018f3b2a-0000-7000-8000-000000000030",
                "campaign_id": "018f3b2a-0000-7000-8000-000000000040",
                "metadata": { "saison": "printemps", "chantier": "Rénovation Dupont" },
                "survey": {
                    "id": "018f3b2a-0000-7000-8000-000000000030",
                    "organization_id": "018f3b2a-0000-7000-8000-000000000002",
                    "title": "Satisfaction après chantier",
                    "description": "Quelques questions sur votre chantier",
                    "visibility": "MEMBERS",
                    "allow_multiple_responses": false,
                    "response_cap": 100,
                    "opened_at": "2026-03-01T08:00:00Z",
                },
            }),
        },
        EventDescriptor {
            name: "survey.response_submitted",
            version: 1,
            label: "Réponse enregistrée",
            subject_kind: "survey_response",
            payload_example: json!({
                "survey_id": "018f3b2a-0000-7000-8000-000000000030",
                "campaign_id": "018f3b2a-0000-7000-8000-000000000040",
                "metadata": { "saison": "printemps", "chantier": "Rénovation Dupont" },
                "response": {
                    "id": "018f3b2a-0000-7000-8000-000000000050",
                    "member_id": "018f3b2a-0000-7000-8000-000000000004",
                    "status": "IN_PROGRESS",
                },
                "answered_questions": 2,
            }),
        },
        EventDescriptor {
            name: "survey.response_completed",
            version: 1,
            label: "Réponse complète",
            subject_kind: "survey_response",
            payload_example: json!({
                "survey_id": "018f3b2a-0000-7000-8000-000000000030",
                "campaign_id": "018f3b2a-0000-7000-8000-000000000040",
                "survey": {
                    "id": "018f3b2a-0000-7000-8000-000000000030",
                    "title": "Satisfaction après chantier",
                },
                "response": {
                    "id": "018f3b2a-0000-7000-8000-000000000050",
                    "member_id": "018f3b2a-0000-7000-8000-000000000004",
                    "completed_at": "2026-03-02T10:30:00Z",
                    "identifier_value": null,
                },
                "metadata": { "saison": "printemps", "chantier": "Rénovation Dupont" },
                "answers": [
                    {
                        "question_id": "018f3b2a-0000-7000-8000-000000000061",
                        "label": "Êtes-vous satisfait ?",
                        "kind": "SINGLE_CHOICE",
                        "value": "Oui",
                    },
                    {
                        "question_id": "018f3b2a-0000-7000-8000-000000000062",
                        "label": "Un commentaire ?",
                        "kind": "TEXT",
                        "value": null,
                    },
                ],
            }),
        },
        EventDescriptor {
            name: "survey_campaign.started",
            version: 1,
            label: "Campagne démarrée",
            subject_kind: "survey_campaign",
            payload_example: campaign_example(),
        },
        EventDescriptor {
            name: "survey_campaign.ended",
            version: 1,
            label: "Campagne terminée",
            subject_kind: "survey_campaign",
            payload_example: campaign_example(),
        },
    ]
}
