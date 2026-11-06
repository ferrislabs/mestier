use std::collections::HashMap;

use serde::Serialize;
use serde_json::json;
use thiserror::Error;

use super::descriptor::{AuthRequirement, ConnectorDescriptor, ExecutionMode};
use super::field::{Field, FieldKind, SelectOption, VisibleWhen};
use crate::domain::automation::credential_kind::HTTP_CREDENTIAL_KINDS;
use crate::domain::automation::workflow::Branch;

/// Every connector kind the product can pose in a workflow graph, keyed by
/// kind and version.
///
/// Mirrors `events::EventCatalogue` on purpose: the trigger picker and the
/// action picker are the same UI pattern, fed by two catalogues built the
/// same way.
#[derive(Debug, Default, Serialize)]
pub struct ConnectorCatalogue {
    /// Nested rather than keyed by `(kind, version)`: lookups come from the
    /// database with a runtime `&str`, which cannot be turned into the
    /// `&'static str` half of a tuple key without scanning the whole map.
    descriptors: HashMap<&'static str, HashMap<u16, ConnectorDescriptor>>,
}

impl ConnectorCatalogue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &mut self,
        descriptor: ConnectorDescriptor,
    ) -> Result<(), ConnectorCatalogueError> {
        let versions = self.descriptors.entry(descriptor.kind).or_default();

        if versions.contains_key(&descriptor.version) {
            return Err(ConnectorCatalogueError::Duplicate {
                kind: descriptor.kind,
                version: descriptor.version,
            });
        }

        versions.insert(descriptor.version, descriptor);
        Ok(())
    }

    pub fn get(&self, kind: &str, version: u16) -> Option<&ConnectorDescriptor> {
        self.descriptors.get(kind)?.get(&version)
    }

    /// Every descriptor, in no particular order. The action picker and the
    /// public connector documentation are both built from this.
    pub fn descriptors(&self) -> impl Iterator<Item = &ConnectorDescriptor> {
        self.descriptors.values().flat_map(HashMap::values)
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum ConnectorCatalogueError {
    #[error("connector `{kind}` version {version} is already registered")]
    Duplicate { kind: &'static str, version: u16 },
}

const LOOP_FIELDS: &[Field] = &[Field {
    name: "items",
    label: "Éléments",
    required: true,
    kind: FieldKind::Json,
    expression: true,
    secret: false,
    visible_when: None,
}];

const CONDITION_FIELDS: &[Field] = &[Field {
    name: "predicate",
    label: "Condition",
    required: true,
    kind: FieldKind::Text,
    expression: true,
    secret: false,
    visible_when: None,
}];

const CONFIG_FIELDS: &[Field] = &[Field {
    name: "variables",
    label: "Variables",
    required: true,
    kind: FieldKind::Json,
    expression: true,
    secret: false,
    visible_when: None,
}];

const WAIT_FIELDS: &[Field] = &[
    Field {
        name: "duration",
        label: "Durée",
        required: true,
        kind: FieldKind::Number,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "unit",
        label: "Unité",
        required: true,
        kind: FieldKind::Select {
            options: &[
                SelectOption {
                    value: "minutes",
                    label: "Minutes",
                },
                SelectOption {
                    value: "hours",
                    label: "Heures",
                },
                SelectOption {
                    value: "days",
                    label: "Jours",
                },
            ],
        },
        expression: false,
        secret: false,
        visible_when: None,
    },
];

const MERGE_FIELDS: &[Field] = &[
    Field {
        name: "mode",
        label: "Mode",
        required: true,
        kind: FieldKind::Select {
            options: &[
                SelectOption {
                    value: "append",
                    label: "Ajouter les éléments à la suite",
                },
                SelectOption {
                    value: "combine_by_position",
                    label: "Combiner par position",
                },
                SelectOption {
                    value: "combine_by_field",
                    label: "Combiner par champ",
                },
            ],
        },
        expression: false,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "field",
        label: "Champ de jointure",
        required: false,
        kind: FieldKind::Text,
        expression: false,
        secret: false,
        visible_when: Some(VisibleWhen {
            field: "mode",
            any_of: &["combine_by_field"],
        }),
    },
];

const CUSTOMER_CREATE_FIELDS: &[Field] = &[
    Field {
        name: "name",
        label: "Nom",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "email",
        label: "E-mail",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "phone",
        label: "Téléphone",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const SUPPORT_ASSIGN_TICKET_FIELDS: &[Field] = &[
    Field {
        name: "ticket_id",
        label: "Ticket",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "members",
        label: "Membres",
        required: true,
        kind: FieldKind::Json,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const SUPPORT_ADD_BADGE_FIELDS: &[Field] = &[
    Field {
        name: "ticket_id",
        label: "Ticket",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "label",
        label: "Libellé",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "tone",
        label: "Ton",
        required: false,
        kind: FieldKind::Select {
            options: &[
                SelectOption {
                    value: "neutral",
                    label: "Neutre",
                },
                SelectOption {
                    value: "brand",
                    label: "Marque",
                },
                SelectOption {
                    value: "success",
                    label: "Succès",
                },
                SelectOption {
                    value: "warning",
                    label: "Avertissement",
                },
                SelectOption {
                    value: "error",
                    label: "Erreur",
                },
            ],
        },
        expression: false,
        secret: false,
        visible_when: None,
    },
];

const NOTE_ADD_FIELDS: &[Field] = &[
    Field {
        name: "ticket_id",
        label: "Ticket",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "body",
        label: "Note",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const QUOTE_SEARCH_FIELDS: &[Field] = &[
    Field {
        name: "status",
        label: "Statut",
        required: false,
        kind: FieldKind::Select {
            options: &[
                SelectOption {
                    value: "DRAFT",
                    label: "Brouillon",
                },
                SelectOption {
                    value: "SENT",
                    label: "Envoyé",
                },
                SelectOption {
                    value: "ACCEPTED",
                    label: "Accepté",
                },
                SelectOption {
                    value: "DECLINED",
                    label: "Refusé",
                },
                SelectOption {
                    value: "CANCELLED",
                    label: "Annulé",
                },
            ],
        },
        expression: false,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "since_days",
        label: "Envoyés depuis (jours)",
        required: false,
        kind: FieldKind::Number,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "limit",
        label: "Nombre maximum",
        required: false,
        kind: FieldKind::Number,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const PROJECT_CREATE_FIELDS: &[Field] = &[
    Field {
        name: "quote_id",
        label: "Devis",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "name",
        label: "Nom du projet",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const CHAT_POST_MESSAGE_FIELDS: &[Field] = &[
    Field {
        name: "channel",
        label: "Canal",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "message",
        label: "Message",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const SMTP_SEND_FIELDS: &[Field] = &[
    Field {
        name: "to",
        label: "Destinataire",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "cc",
        label: "Copie",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "subject",
        label: "Objet",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "body",
        label: "Corps",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "format",
        label: "Format",
        required: false,
        kind: FieldKind::Select {
            options: &[
                SelectOption {
                    value: "text",
                    label: "Texte",
                },
                SelectOption {
                    value: "html",
                    label: "HTML",
                },
            ],
        },
        expression: false,
        secret: false,
        visible_when: None,
    },
];

const COLLECTION_OPERATION_FIELDS: &[Field] = &[
    Field {
        name: "path",
        label: "Paramètres de chemin",
        required: false,
        kind: FieldKind::Json,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "query",
        label: "Paramètres de requête",
        required: false,
        kind: FieldKind::Json,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "headers",
        label: "En-têtes",
        required: false,
        kind: FieldKind::Json,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "body",
        label: "Corps",
        required: false,
        kind: FieldKind::Json,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const HTTP_REQUEST_FIELDS: &[Field] = &[
    Field {
        name: "method",
        label: "Méthode",
        required: true,
        kind: FieldKind::Select {
            options: &[
                SelectOption {
                    value: "GET",
                    label: "GET",
                },
                SelectOption {
                    value: "POST",
                    label: "POST",
                },
                SelectOption {
                    value: "PUT",
                    label: "PUT",
                },
                SelectOption {
                    value: "PATCH",
                    label: "PATCH",
                },
                SelectOption {
                    value: "DELETE",
                    label: "DELETE",
                },
            ],
        },
        expression: false,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "url",
        label: "URL",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "headers",
        label: "En-têtes",
        required: false,
        kind: FieldKind::Json,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "body",
        label: "Corps",
        required: false,
        kind: FieldKind::Json,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "timeout_seconds",
        label: "Délai d'expiration (secondes)",
        required: false,
        kind: FieldKind::Number,
        expression: false,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "signing_credential_id",
        label: "Clé de signature sortante",
        required: false,
        kind: FieldKind::Text,
        expression: false,
        secret: false,
        visible_when: None,
    },
];

const ODOO_CREATE_PARTNER_FIELDS: &[Field] = &[
    Field {
        name: "name",
        label: "Nom",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "email",
        label: "E-mail",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "phone",
        label: "Téléphone",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const ODOO_UPDATE_PARTNER_FIELDS: &[Field] = &[
    Field {
        name: "partner_id",
        label: "Identifiant du partenaire",
        required: true,
        kind: FieldKind::Number,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "name",
        label: "Nom",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "email",
        label: "E-mail",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "phone",
        label: "Téléphone",
        required: false,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

const ODOO_CREATE_INVOICE_FIELDS: &[Field] = &[
    Field {
        name: "partner_id",
        label: "Identifiant du partenaire",
        required: true,
        kind: FieldKind::Number,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "description",
        label: "Libellé de la ligne",
        required: true,
        kind: FieldKind::Text,
        expression: true,
        secret: false,
        visible_when: None,
    },
    Field {
        name: "amount",
        label: "Montant",
        required: true,
        kind: FieldKind::Number,
        expression: true,
        secret: false,
        visible_when: None,
    },
];

/// Assembles the catalogue. Single point of extension: a module contributes
/// a line here, never a change to the backbone.
pub fn connector_catalogue() -> ConnectorCatalogue {
    let mut catalogue = ConnectorCatalogue::new();

    for descriptor in descriptors() {
        catalogue
            .register(descriptor)
            .expect("the catalogue is built from static descriptors, so a clash is a bug");
    }

    catalogue
}

fn descriptors() -> Vec<ConnectorDescriptor> {
    let mut all = flow_descriptors();
    all.extend(customer_descriptors());
    all.extend(support_descriptors());
    all.extend(workspace_descriptors());
    all.extend(smtp_descriptors());
    all.extend(http_descriptors());
    all.extend(collection_descriptors());
    all.extend(odoo_descriptors());
    all.extend(task_recurrence_descriptors());
    all
}

/// The flow-control connectors: they act on the graph itself rather than an
/// external system, so they need no authentication and are declared here
/// instead of being owned by a bounded context.
fn flow_descriptors() -> Vec<ConnectorDescriptor> {
    vec![
        ConnectorDescriptor {
            kind: "flow.loop",
            version: 1,
            family: "flow",
            label: "Pour chaque",
            auth: AuthRequirement::None,
            fields: LOOP_FIELDS,
            branches: &[Branch::Each, Branch::After],
            execution: ExecutionMode::Once,
            output_example: json!({ "item": "…", "index": 0 }),
            output_mirrors_field: None,
            sample_output: json!({ "items": ["…"] }),
        },
        ConnectorDescriptor {
            kind: "flow.condition",
            version: 1,
            family: "flow",
            label: "Condition",
            auth: AuthRequirement::None,
            fields: CONDITION_FIELDS,
            branches: &[Branch::Then, Branch::Else],
            execution: ExecutionMode::PerItem,
            output_example: json!({ "matched": true }),
            output_mirrors_field: None,
            sample_output: json!({ "matched": true }),
        },
        ConnectorDescriptor {
            kind: "flow.config",
            version: 1,
            family: "flow",
            label: "Configuration",
            auth: AuthRequirement::None,
            fields: CONFIG_FIELDS,
            branches: &[],
            execution: ExecutionMode::Once,
            output_example: json!({ "customer_id": "…", "retry_limit": 3 }),
            output_mirrors_field: Some("variables"),
            sample_output: json!({ "customer_id": "…", "retry_limit": 3 }),
        },
        ConnectorDescriptor {
            kind: "flow.wait",
            version: 1,
            family: "flow",
            label: "Attendre",
            auth: AuthRequirement::None,
            fields: WAIT_FIELDS,
            branches: &[],
            execution: ExecutionMode::Once,
            output_example: json!({
                "duration": 2,
                "unit": "days",
                "resume_at": "2026-01-01T00:00:00Z",
            }),
            output_mirrors_field: None,
            sample_output: json!({
                "duration": 2,
                "unit": "days",
                "resume_at": "2026-01-01T00:00:00Z",
            }),
        },
        ConnectorDescriptor {
            kind: "flow.merge",
            version: 1,
            family: "flow",
            label: "Fusionner",
            auth: AuthRequirement::None,
            fields: MERGE_FIELDS,
            branches: &[],
            execution: ExecutionMode::Once,
            output_example: json!({ "id": "…" }),
            output_mirrors_field: None,
            sample_output: json!([{ "id": "…" }]),
        },
    ]
}

/// The one internal connector proved by the run engine (#200): it calls
/// Mestier's own `create_customer` use case, so it needs no credential of
/// its own — the run already acts inside this instance.
fn customer_descriptors() -> Vec<ConnectorDescriptor> {
    vec![ConnectorDescriptor {
        kind: "mestier.customer.create",
        version: 1,
        family: "mestier",
        label: "Créer un client",
        auth: AuthRequirement::None,
        fields: CUSTOMER_CREATE_FIELDS,
        branches: &[],
        execution: ExecutionMode::PerItem,
        output_example: json!({ "id": "…", "name": "…" }),
        output_mirrors_field: None,
        sample_output: json!({ "id": "00000000-0000-0000-0000-000000000000", "name": "…" }),
    }]
}

fn support_descriptors() -> Vec<ConnectorDescriptor> {
    vec![
        ConnectorDescriptor {
            kind: "mestier.support.assign_ticket",
            version: 1,
            family: "mestier",
            label: "Assigner un ticket",
            auth: AuthRequirement::None,
            fields: SUPPORT_ASSIGN_TICKET_FIELDS,
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: json!({ "ticket_id": "…", "assignee_member_id": "…" }),
            output_mirrors_field: None,
            sample_output: json!({
                "ticket_id": "00000000-0000-0000-0000-000000000000",
                "assignee_member_id": "00000000-0000-0000-0000-000000000000",
            }),
        },
        ConnectorDescriptor {
            kind: "mestier.support.add_badge",
            version: 1,
            family: "mestier",
            label: "Ajouter un badge",
            auth: AuthRequirement::None,
            fields: SUPPORT_ADD_BADGE_FIELDS,
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: json!({ "ticket_id": "…", "label": "High", "tone": "neutral" }),
            output_mirrors_field: None,
            sample_output: json!({
                "ticket_id": "00000000-0000-0000-0000-000000000000",
                "label": "High",
                "tone": "neutral",
            }),
        },
    ]
}

fn workspace_descriptors() -> Vec<ConnectorDescriptor> {
    vec![
        ConnectorDescriptor {
            kind: "mestier.note.add",
            version: 1,
            family: "mestier",
            label: "Ajouter une note",
            auth: AuthRequirement::None,
            fields: NOTE_ADD_FIELDS,
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: json!({ "ticket_id": "…", "note_id": "…" }),
            output_mirrors_field: None,
            sample_output: json!({
                "ticket_id": "00000000-0000-0000-0000-000000000000",
                "note_id": "00000000-0000-0000-0000-000000000000",
            }),
        },
        ConnectorDescriptor {
            kind: "mestier.quote.search",
            version: 1,
            family: "mestier",
            label: "Rechercher des devis",
            auth: AuthRequirement::None,
            fields: QUOTE_SEARCH_FIELDS,
            branches: &[],
            execution: ExecutionMode::Once,
            output_example: json!({
                "id": "…",
                "reference": "DV-2026-0001",
                "title": "…",
                "status": "SENT",
                "customer_id": "…",
                "net_cents": 0,
                "gross_cents": 0,
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
            }),
            output_mirrors_field: None,
            sample_output: json!([{
                "id": "00000000-0000-0000-0000-000000000000",
                "reference": "DV-2026-0001",
                "title": "Rénovation de la salle de bain",
                "status": "SENT",
                "customer_id": "00000000-0000-0000-0000-000000000000",
                "net_cents": 100000,
                "gross_cents": 120000,
                "created_at": "2026-01-01T00:00:00Z",
                "updated_at": "2026-01-01T00:00:00Z",
            }]),
        },
        ConnectorDescriptor {
            kind: "mestier.project.create",
            version: 1,
            family: "mestier",
            label: "Créer un projet",
            auth: AuthRequirement::None,
            fields: PROJECT_CREATE_FIELDS,
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: json!({ "project_id": "…", "name": "…", "quote_id": "…" }),
            output_mirrors_field: None,
            sample_output: json!({
                "project_id": "00000000-0000-0000-0000-000000000000",
                "name": "Rénovation de la salle de bain",
                "quote_id": "00000000-0000-0000-0000-000000000000",
            }),
        },
        ConnectorDescriptor {
            kind: "mestier.chat.post_message",
            version: 1,
            family: "mestier",
            label: "Publier dans un canal Discussions",
            auth: AuthRequirement::None,
            fields: CHAT_POST_MESSAGE_FIELDS,
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: json!({ "message_id": "…", "channel_id": "…" }),
            output_mirrors_field: None,
            sample_output: json!({
                "message_id": "00000000-0000-0000-0000-000000000000",
                "channel_id": "00000000-0000-0000-0000-000000000000",
            }),
        },
    ]
}

fn smtp_descriptors() -> Vec<ConnectorDescriptor> {
    vec![ConnectorDescriptor {
        kind: "smtp.send",
        version: 1,
        family: "smtp",
        label: "E-mail SMTP",
        auth: AuthRequirement::Exactly("smtp"),
        fields: SMTP_SEND_FIELDS,
        branches: &[],
        execution: ExecutionMode::PerItem,
        output_example: json!({
            "message_id": "<8f2c1d@duval.fr>",
            "accepted": ["claire.fontaine@fontaine-immo.fr"],
        }),
        output_mirrors_field: None,
        sample_output: json!({
            "message_id": "<8f2c1d@duval.fr>",
            "accepted": ["claire.fontaine@fontaine-immo.fr"],
        }),
    }]
}

/// The one network connector proved by #202: an outbound call to wherever
/// the resolved `url` points, authenticated with any of three schemes and
/// optionally signed with a second, distinct credential
/// (`signing_credential_id`). `credential_id` stays `Option` — an
/// unauthenticated call is a real, supported case.
fn http_descriptors() -> Vec<ConnectorDescriptor> {
    vec![ConnectorDescriptor {
        kind: "http.request",
        version: 1,
        family: "http",
        label: "Requête HTTP",
        auth: AuthRequirement::Optional(HTTP_CREDENTIAL_KINDS),
        fields: HTTP_REQUEST_FIELDS,
        branches: &[],
        execution: ExecutionMode::PerItem,
        output_example: json!({
            "status": 200,
            "headers": { "content-type": "application/json" },
            "body": {},
        }),
        output_mirrors_field: None,
        sample_output: json!({ "status": 200, "headers": {}, "body": {} }),
    }]
}

fn collection_descriptors() -> Vec<ConnectorDescriptor> {
    vec![ConnectorDescriptor {
        kind: "collection.operation",
        version: 1,
        family: "http",
        label: "Opération d'une collection",
        auth: AuthRequirement::Optional(HTTP_CREDENTIAL_KINDS),
        fields: COLLECTION_OPERATION_FIELDS,
        branches: &[],
        execution: ExecutionMode::PerItem,
        output_example: json!({
            "status": 200,
            "headers": { "content-type": "application/json" },
            "body": {},
        }),
        output_mirrors_field: None,
        sample_output: json!({ "status": 200, "headers": {}, "body": {} }),
    }]
}

/// The three Odoo actions #202 ships: each a typed envelope over the same
/// authenticate-then-`execute_kw` HTTP call `http.request` could make by
/// hand — named fields and an imposed `odoo_api` credential in exchange for
/// giving up the generality.
fn odoo_descriptors() -> Vec<ConnectorDescriptor> {
    vec![
        ConnectorDescriptor {
            kind: "odoo.create_partner",
            version: 1,
            family: "odoo",
            label: "Créer un partenaire Odoo",
            auth: AuthRequirement::Exactly("odoo_api"),
            fields: ODOO_CREATE_PARTNER_FIELDS,
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: json!({ "id": 42 }),
            output_mirrors_field: None,
            sample_output: json!({ "id": 42 }),
        },
        ConnectorDescriptor {
            kind: "odoo.update_partner",
            version: 1,
            family: "odoo",
            label: "Mettre à jour un partenaire Odoo",
            auth: AuthRequirement::Exactly("odoo_api"),
            fields: ODOO_UPDATE_PARTNER_FIELDS,
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: json!({ "id": 42, "updated": true }),
            output_mirrors_field: None,
            sample_output: json!({ "id": 42, "updated": true }),
        },
        ConnectorDescriptor {
            kind: "odoo.create_invoice",
            version: 1,
            family: "odoo",
            label: "Créer une facture Odoo",
            auth: AuthRequirement::Exactly("odoo_api"),
            fields: ODOO_CREATE_INVOICE_FIELDS,
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: json!({ "id": 99 }),
            output_mirrors_field: None,
            sample_output: json!({ "id": 99 }),
        },
    ]
}

/// The one connector the horizon-extension pass (#293) runs: no fields, no
/// credential — it reads `org_id` off the run itself and extends whatever
/// that organization's recurrences need, the same "calls an existing use
/// case, so every business rule already enforced there applies here too"
/// shape as `mestier.customer.create`. Placed by
/// `MestierUseCase::find_or_create_recurrence_horizon_workflow`, never by a
/// human dragging it into a graph — but described here anyway, like every
/// other kind, so the anti-drift test in `infrastructure::automation::connectors`
/// still catches an implementation with no matching descriptor or the
/// reverse.
fn task_recurrence_descriptors() -> Vec<ConnectorDescriptor> {
    vec![ConnectorDescriptor {
        kind: "mestier.task_recurrence.extend_horizon",
        version: 1,
        family: "mestier",
        label: "Étendre l'horizon de récurrence",
        auth: AuthRequirement::None,
        fields: &[],
        branches: &[],
        execution: ExecutionMode::PerItem,
        output_example: json!({ "materialized": 0 }),
        output_mirrors_field: None,
        sample_output: json!({ "materialized": 0 }),
    }]
}

#[cfg(test)]
mod tests {
    use serde_json::to_value;

    use super::*;
    use crate::domain::automation::credential_kind::credential_kind;

    fn descriptor(kind: &'static str, version: u16) -> ConnectorDescriptor {
        ConnectorDescriptor {
            kind,
            version,
            family: "flow",
            label: "Loop",
            auth: AuthRequirement::None,
            fields: &[],
            branches: &[],
            execution: ExecutionMode::PerItem,
            output_example: json!({ "index": 0 }),
            output_mirrors_field: None,
            sample_output: json!({ "index": 0 }),
        }
    }

    #[test]
    fn a_registered_descriptor_can_be_read_back() {
        let mut catalogue = ConnectorCatalogue::new();

        catalogue
            .register(descriptor("flow.loop", 1))
            .expect("first registration succeeds");

        assert_eq!(
            catalogue.get("flow.loop", 1),
            Some(&descriptor("flow.loop", 1))
        );
    }

    #[test]
    fn registering_the_same_kind_and_version_twice_fails() {
        let mut catalogue = ConnectorCatalogue::new();
        catalogue
            .register(descriptor("flow.loop", 1))
            .expect("first registration succeeds");

        let result = catalogue.register(descriptor("flow.loop", 1));

        assert_eq!(
            result,
            Err(ConnectorCatalogueError::Duplicate {
                kind: "flow.loop",
                version: 1
            })
        );
    }

    #[test]
    fn two_versions_of_the_same_kind_coexist() {
        let mut catalogue = ConnectorCatalogue::new();

        catalogue
            .register(descriptor("flow.loop", 1))
            .expect("version 1 registers");
        catalogue
            .register(descriptor("flow.loop", 2))
            .expect("version 2 registers alongside version 1");

        assert!(catalogue.get("flow.loop", 1).is_some());
        assert!(catalogue.get("flow.loop", 2).is_some());
    }

    #[test]
    fn an_unknown_kind_reads_back_as_none() {
        let catalogue = ConnectorCatalogue::new();

        assert_eq!(catalogue.get("flow.loop", 1), None);
    }

    #[test]
    fn the_catalogue_contains_the_flow_connectors() {
        let catalogue = connector_catalogue();

        assert!(catalogue.get("flow.loop", 1).is_some());
        assert!(catalogue.get("flow.condition", 1).is_some());
        assert!(catalogue.get("flow.config", 1).is_some());
    }

    #[test]
    fn the_catalogue_contains_the_flow_config_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("flow.config", 1)
            .expect("flow.config is described");
        assert_eq!(descriptor.auth, AuthRequirement::None);
        assert!(descriptor.branches.is_empty());
        assert!(
            descriptor
                .fields
                .iter()
                .any(|f| f.name == "variables" && f.required && f.expression)
        );
    }

    /// The one internal connector the run engine (#200) proves itself
    /// against, described here so the anti-drift test in
    /// `infrastructure::automation::connectors` has something to match its
    /// implementation against.
    #[test]
    fn the_catalogue_contains_the_customer_create_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("mestier.customer.create", 1)
            .expect("mestier.customer.create is described");
        assert_eq!(descriptor.auth, AuthRequirement::None);
        assert!(
            descriptor
                .fields
                .iter()
                .any(|f| f.name == "name" && f.required)
        );
    }

    #[test]
    fn the_catalogue_describes_the_support_assign_ticket_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("mestier.support.assign_ticket", 1)
            .expect("mestier.support.assign_ticket is described");
        assert_eq!(descriptor.family, "mestier");
        assert_eq!(descriptor.label, "Assigner un ticket");
        assert_eq!(descriptor.auth, AuthRequirement::None);
        assert!(descriptor.branches.is_empty());
        assert_eq!(
            descriptor.output_example,
            json!({ "ticket_id": "…", "assignee_member_id": "…" })
        );

        let json = to_value(descriptor).expect("descriptor serializes");
        assert_eq!(
            json["fields"],
            json!([
                {
                    "name": "ticket_id",
                    "label": "Ticket",
                    "required": true,
                    "kind": "Text",
                    "expression": true,
                    "secret": false,
                    "visible_when": null
                },
                {
                    "name": "members",
                    "label": "Membres",
                    "required": true,
                    "kind": "Json",
                    "expression": true,
                    "secret": false,
                    "visible_when": null
                }
            ])
        );
    }

    #[test]
    fn the_catalogue_describes_the_support_add_badge_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("mestier.support.add_badge", 1)
            .expect("mestier.support.add_badge is described");
        assert_eq!(descriptor.family, "mestier");
        assert_eq!(descriptor.label, "Ajouter un badge");
        assert_eq!(descriptor.auth, AuthRequirement::None);
        assert!(descriptor.branches.is_empty());
        assert_eq!(
            descriptor.output_example,
            json!({ "ticket_id": "…", "label": "High", "tone": "neutral" })
        );

        let json = to_value(descriptor).expect("descriptor serializes");
        assert_eq!(
            json["fields"],
            json!([
                {
                    "name": "ticket_id",
                    "label": "Ticket",
                    "required": true,
                    "kind": "Text",
                    "expression": true,
                    "secret": false,
                    "visible_when": null
                },
                {
                    "name": "label",
                    "label": "Libellé",
                    "required": true,
                    "kind": "Text",
                    "expression": true,
                    "secret": false,
                    "visible_when": null
                },
                {
                    "name": "tone",
                    "label": "Ton",
                    "required": false,
                    "kind": {
                        "Select": {
                            "options": [
                                { "value": "neutral", "label": "Neutre" },
                                { "value": "brand", "label": "Marque" },
                                { "value": "success", "label": "Succès" },
                                { "value": "warning", "label": "Avertissement" },
                                { "value": "error", "label": "Erreur" }
                            ]
                        }
                    },
                    "expression": false,
                    "secret": false,
                    "visible_when": null
                }
            ])
        );
    }

    #[test]
    fn the_support_tone_options_are_exactly_the_badge_tones() {
        let catalogue = connector_catalogue();
        let descriptor = catalogue
            .get("mestier.support.add_badge", 1)
            .expect("mestier.support.add_badge is described");
        let tone = descriptor
            .fields
            .iter()
            .find(|field| field.name == "tone")
            .expect("add_badge has a tone field");
        let FieldKind::Select { options } = tone.kind else {
            panic!("tone must be a select");
        };

        let offered: Vec<&str> = options.iter().map(|option| option.value).collect();
        let tones: Vec<&str> = crate::BadgeTone::ALL
            .iter()
            .map(|tone| tone.as_str())
            .collect();

        assert_eq!(offered, tones);
    }

    /// The connector the horizon-extension pass (#293) drives — no fields,
    /// no credential, since it acts on the run's own `org_id`.
    #[test]
    fn the_catalogue_contains_the_recurrence_horizon_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("mestier.task_recurrence.extend_horizon", 1)
            .expect("mestier.task_recurrence.extend_horizon is described");
        assert_eq!(descriptor.auth, AuthRequirement::None);
        assert!(descriptor.fields.is_empty());
    }

    /// `http.request`'s auth stays optional: `credential_id` is `Option`, so
    /// an unauthenticated call is a supported case, not an oversight.
    #[test]
    fn the_catalogue_contains_the_http_request_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("http.request", 1)
            .expect("http.request is described");
        assert_eq!(
            descriptor.auth,
            AuthRequirement::Optional(HTTP_CREDENTIAL_KINDS)
        );
        assert!(descriptor.auth.is_satisfied_without_a_credential());
        assert!(
            descriptor
                .fields
                .iter()
                .any(|f| f.name == "url" && f.required && f.expression)
        );
        assert!(
            descriptor
                .fields
                .iter()
                .any(|f| f.name == "method" && f.required)
        );
        assert!(
            descriptor
                .fields
                .iter()
                .any(|f| f.name == "signing_credential_id" && !f.required),
            "signing is a second, distinct credential slot from `credential_id`"
        );
    }

    #[test]
    fn the_catalogue_contains_the_collection_operation_connector() {
        let catalogue = connector_catalogue();

        let descriptor = catalogue
            .get("collection.operation", 1)
            .expect("collection.operation is described");

        assert_eq!(descriptor.family, "http");
        assert_eq!(
            descriptor.auth,
            AuthRequirement::Optional(HTTP_CREDENTIAL_KINDS)
        );
        assert!(descriptor.auth.is_satisfied_without_a_credential());
        let names: Vec<&str> = descriptor.fields.iter().map(|field| field.name).collect();
        assert_eq!(names, ["path", "query", "headers", "body"]);
        for field in descriptor.fields {
            assert!(
                !field.required,
                "{} is checked against the operation",
                field.name
            );
            assert!(field.expression, "{}", field.name);
            assert_eq!(field.kind, FieldKind::Json, "{}", field.name);
        }
        for copied in ["url", "method", "base_url", "security"] {
            assert!(
                !names.contains(&copied),
                "`{copied}` must come from the stored collection, never from the node"
            );
        }
        assert_eq!(
            descriptor.output_example,
            catalogue.get("http.request", 1).unwrap().output_example
        );
    }

    /// Each Odoo action imposes exactly one auth scheme — no `AnyOf`, unlike
    /// `http.request` — and a credential-bearing model action requires the
    /// fields that identify what it acts on.
    #[test]
    fn the_catalogue_contains_the_three_odoo_connectors() {
        let catalogue = connector_catalogue();

        for kind in [
            "odoo.create_partner",
            "odoo.update_partner",
            "odoo.create_invoice",
        ] {
            let descriptor = catalogue
                .get(kind, 1)
                .unwrap_or_else(|| panic!("{kind} is described"));
            assert_eq!(descriptor.auth, AuthRequirement::Exactly("odoo_api"));
            assert_eq!(descriptor.family, "odoo");
        }

        let create_partner = catalogue.get("odoo.create_partner", 1).unwrap();
        assert!(
            create_partner
                .fields
                .iter()
                .any(|f| f.name == "name" && f.required)
        );

        let update_partner = catalogue.get("odoo.update_partner", 1).unwrap();
        assert!(
            update_partner
                .fields
                .iter()
                .any(|f| f.name == "partner_id" && f.required)
        );

        let create_invoice = catalogue.get("odoo.create_invoice", 1).unwrap();
        assert!(
            create_invoice
                .fields
                .iter()
                .any(|f| f.name == "amount" && f.required)
        );
    }

    /// The check that keeps a typo in an `AuthRequirement` from reaching
    /// production: a scheme named here but absent from `auth_schemes()`
    /// would otherwise only surface once someone tries to attach a
    /// credential to the connector.
    #[test]
    fn every_scheme_named_by_the_catalogue_is_a_known_auth_scheme() {
        let catalogue = connector_catalogue();

        for descriptor in catalogue.descriptors() {
            for scheme_kind in descriptor.auth.scheme_kinds() {
                assert!(
                    credential_kind(scheme_kind).is_some(),
                    "`{}` requires unknown auth scheme `{scheme_kind}`",
                    descriptor.kind
                );
            }
        }
    }

    #[test]
    fn the_http_connectors_accept_every_http_kind_and_nothing_else() {
        let catalogue = connector_catalogue();

        for connector in ["http.request", "collection.operation"] {
            let descriptor = catalogue.get(connector, 1).unwrap();
            for kind in [
                "bearer_token",
                "http_basic",
                "http_header",
                "api_key_query",
                "digest",
                "oauth2_authorization_code",
                "oauth2_client_credentials",
                "oauth1",
                "jwt",
                "mtls",
                "custom_auth",
            ] {
                assert!(descriptor.auth.accepts(kind), "{connector} {kind}");
            }
            for kind in ["hmac", "smtp", "imap", "ssh", "ftp", "sql", "odoo_api"] {
                assert!(!descriptor.auth.accepts(kind), "{connector} {kind}");
            }
        }
    }

    #[test]
    fn every_descriptor_carries_an_output_example() {
        for descriptor in connector_catalogue().descriptors() {
            assert!(
                !descriptor.output_example.is_null(),
                "`{}` has no output example, so nothing documents its shape",
                descriptor.kind
            );
        }
    }

    #[test]
    fn only_the_connectors_that_work_on_the_whole_list_run_once() {
        let catalogue = connector_catalogue();
        let mut once: Vec<&str> = catalogue
            .descriptors()
            .filter(|descriptor| descriptor.execution == ExecutionMode::Once)
            .map(|descriptor| descriptor.kind)
            .collect();
        once.sort_unstable();

        assert_eq!(
            once,
            [
                "flow.config",
                "flow.loop",
                "flow.merge",
                "flow.wait",
                "mestier.quote.search"
            ]
        );
    }

    #[test]
    fn the_list_sources_sample_a_list_of_items_and_the_others_one_item() {
        let catalogue = connector_catalogue();

        for descriptor in catalogue.descriptors() {
            let is_list = matches!(descriptor.kind, "flow.merge" | "mestier.quote.search");
            assert_eq!(
                descriptor.sample_output.is_array(),
                is_list,
                "{}",
                descriptor.kind
            );
            let first = if is_list {
                &descriptor.sample_output[0]
            } else {
                &descriptor.sample_output
            };
            assert!(first.is_object(), "{}", descriptor.kind);
        }
    }

    #[test]
    fn every_descriptor_carries_a_sample_output() {
        for descriptor in connector_catalogue().descriptors() {
            assert!(
                !descriptor.sample_output.is_null(),
                "`{}` has no sample output, so a dry run has nothing to answer with",
                descriptor.kind
            );
        }
    }

    #[test]
    fn http_request_and_collection_operation_sample_a_bare_ok_response() {
        let catalogue = connector_catalogue();

        for kind in ["http.request", "collection.operation"] {
            assert_eq!(
                catalogue.get(kind, 1).unwrap().sample_output,
                json!({ "status": 200, "headers": {}, "body": {} }),
                "{kind}"
            );
        }
    }

    #[test]
    fn the_flow_samples_have_the_shape_the_engine_stores_for_them() {
        let catalogue = connector_catalogue();

        let looped = &catalogue.get("flow.loop", 1).unwrap().sample_output;
        assert!(looped["items"].is_array(), "{looped}");
        assert_eq!(
            catalogue.get("flow.condition", 1).unwrap().sample_output,
            json!({ "matched": true })
        );
        assert!(
            catalogue
                .get("flow.config", 1)
                .unwrap()
                .sample_output
                .is_object()
        );
    }

    #[test]
    fn the_samples_of_the_internal_and_odoo_connectors_name_what_a_run_returns() {
        let catalogue = connector_catalogue();
        let keys = |kind: &str| -> Vec<String> {
            let sample = &catalogue.get(kind, 1).unwrap().sample_output;
            let mut keys: Vec<String> = sample.as_object().unwrap().keys().cloned().collect();
            keys.sort();
            keys
        };

        assert_eq!(keys("mestier.customer.create"), ["id", "name"]);
        assert_eq!(
            keys("mestier.support.assign_ticket"),
            ["assignee_member_id", "ticket_id"]
        );
        assert_eq!(
            keys("mestier.support.add_badge"),
            ["label", "ticket_id", "tone"]
        );
        assert_eq!(
            keys("mestier.task_recurrence.extend_horizon"),
            ["materialized"]
        );
        assert_eq!(keys("odoo.create_partner"), ["id"]);
        assert_eq!(keys("odoo.update_partner"), ["id", "updated"]);
        assert_eq!(keys("odoo.create_invoice"), ["id"]);
    }

    /// The catalogue leaves the crate as JSON (#203): both registered
    /// versions of a kind must survive the trip.
    #[test]
    fn the_catalogue_serializes_every_registered_descriptor() {
        let mut catalogue = ConnectorCatalogue::new();
        catalogue
            .register(descriptor("flow.loop", 1))
            .expect("version 1 registers");
        catalogue
            .register(descriptor("flow.loop", 2))
            .expect("version 2 registers");

        let json = to_value(&catalogue).expect("catalogue serializes");

        assert_eq!(json["descriptors"]["flow.loop"]["1"]["version"], 1);
        assert_eq!(json["descriptors"]["flow.loop"]["2"]["version"], 2);
    }
}
