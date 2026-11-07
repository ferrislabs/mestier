use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::{Field, SelectOption};

#[cfg(test)]
#[path = "tests_clients.rs"]
mod tests_clients;

const GROUP: &str = "Clients";
const NIL: &str = "00000000-0000-0000-0000-000000000000";

pub(super) const STATUSES: &[SelectOption] = &[
    SelectOption::new("PROSPECT", "Prospect"),
    SelectOption::new("CLIENT", "Client"),
    SelectOption::new("ARCHIVED", "Archivé"),
];

pub(super) const STAGES: &[SelectOption] = &[
    SelectOption::new("NEW", "Nouveau"),
    SelectOption::new("CONTACTED", "Contacté"),
    SelectOption::new("QUALIFIED", "Qualifié"),
    SelectOption::new("QUOTE_SENT", "Devis envoyé"),
    SelectOption::new("WON", "Gagné"),
    SelectOption::new("LOST", "Perdu"),
];

const PAGE_FIELDS: &[Field] = &[
    Field::number("limit", "Nombre maximum de résultats", false),
    Field::number("offset", "Décalage", false),
];

const CUSTOMER_FIELDS: &[Field] = &[Field::text("customer_id", "Client", true)];

const CUSTOMER_PAGE_FIELDS: &[Field] = &[
    Field::text("customer_id", "Client", true),
    Field::number("limit", "Nombre maximum de résultats", false),
    Field::number("offset", "Décalage", false),
];

const UPDATE_CUSTOMER_FIELDS: &[Field] = &[
    Field::text("customer_id", "Client", true),
    Field::select("status", "Statut", true, STATUSES),
    Field::select("pipeline_stage", "Étape du parcours", true, STAGES),
    Field::text("name", "Nom", true),
    Field::text("registration_number", "Numéro d’immatriculation", false),
    Field::text("phone", "Téléphone", false),
    Field::text("email", "E-mail", false),
];

const CONTACT_FIELDS: &[Field] = &[Field::text("contact_id", "Contact", true)];

const CREATE_CONTACT_FIELDS: &[Field] = &[
    Field::text("customer_id", "Client", true),
    Field::text("first_name", "Prénom", true),
    Field::text("last_name", "Nom", true),
    Field::text("role", "Fonction", false),
    Field::text("phone", "Téléphone", false),
    Field::text("email", "E-mail", false),
    Field::boolean("is_primary", "Contact principal", false),
];

const UPDATE_CONTACT_FIELDS: &[Field] = &[
    Field::text("contact_id", "Contact", true),
    Field::text("first_name", "Prénom", true),
    Field::text("last_name", "Nom", true),
    Field::text("role", "Fonction", false),
    Field::text("phone", "Téléphone", false),
    Field::text("email", "E-mail", false),
    Field::boolean("is_primary", "Contact principal", false),
];

const CONTEXT_FIELDS: &[Field] = &[Field::text("context_id", "Lieu", true)];

const CREATE_CONTEXT_FIELDS: &[Field] = &[
    Field::text("customer_id", "Client", true),
    Field::text("label", "Libellé", true),
    Field::text("address_line", "Adresse", false),
    Field::text("postal_code", "Code postal", false),
    Field::text("city", "Ville", false),
    Field::text("photo_key", "Clé de la photo", false),
];

const UPDATE_CONTEXT_FIELDS: &[Field] = &[
    Field::text("context_id", "Lieu", true),
    Field::text("label", "Libellé", true),
    Field::text("address_line", "Adresse", false),
    Field::text("postal_code", "Code postal", false),
    Field::text("city", "Ville", false),
    Field::text("photo_key", "Clé de la photo", false),
];

fn customer_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "status": "PROSPECT",
        "pipeline_stage": "NEW",
        "name": "Claire Fontaine",
        "registration_number": null,
        "phone": null,
        "email": null,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn customers_sample() -> Value {
    json!([customer_sample()])
}

fn deleted_customer_sample() -> Value {
    json!({ "customer_id": NIL, "deleted": true })
}

fn contact_sample() -> Value {
    json!({
        "id": NIL,
        "customer_id": NIL,
        "first_name": "Claire",
        "last_name": "Fontaine",
        "role": null,
        "phone": null,
        "email": null,
        "is_primary": false,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn contacts_sample() -> Value {
    json!([contact_sample()])
}

fn deleted_contact_sample() -> Value {
    json!({ "contact_id": NIL, "deleted": true })
}

fn context_sample() -> Value {
    json!({
        "id": NIL,
        "customer_id": NIL,
        "label": "Chantier principal",
        "address_line": null,
        "postal_code": null,
        "city": null,
        "photo_key": null,
        "created_at": "2026-01-01T00:00:00Z",
        "updated_at": "2026-01-01T00:00:00Z",
    })
}

fn contexts_sample() -> Value {
    json!([context_sample()])
}

fn deleted_context_sample() -> Value {
    json!({ "context_id": NIL, "deleted": true })
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.customer.get",
        label: "Lire un client",
        description: "Renvoie un client de l’organisation.",
        group: GROUP,
        fields: CUSTOMER_FIELDS,
        read_only: true,
        sample_output: customer_sample,
    },
    ActionSpec {
        kind: "mestier.customer.list",
        label: "Lister les clients",
        description: "Renvoie les clients de l’organisation, page par page.",
        group: GROUP,
        fields: PAGE_FIELDS,
        read_only: true,
        sample_output: customers_sample,
    },
    ActionSpec {
        kind: "mestier.customer.update",
        label: "Modifier un client",
        description: "Remplace le statut, l’étape du parcours et les coordonnées d’un client.",
        group: GROUP,
        fields: UPDATE_CUSTOMER_FIELDS,
        read_only: false,
        sample_output: customer_sample,
    },
    ActionSpec {
        kind: "mestier.customer.delete",
        label: "Supprimer un client",
        description: "Supprime un client de l’organisation.",
        group: GROUP,
        fields: CUSTOMER_FIELDS,
        read_only: false,
        sample_output: deleted_customer_sample,
    },
    ActionSpec {
        kind: "mestier.customer_contact.list",
        label: "Lister les contacts d’un client",
        description: "Renvoie les contacts d’un client, page par page.",
        group: GROUP,
        fields: CUSTOMER_PAGE_FIELDS,
        read_only: true,
        sample_output: contacts_sample,
    },
    ActionSpec {
        kind: "mestier.customer_contact.get",
        label: "Lire un contact",
        description: "Renvoie un contact d’un client de l’organisation.",
        group: GROUP,
        fields: CONTACT_FIELDS,
        read_only: true,
        sample_output: contact_sample,
    },
    ActionSpec {
        kind: "mestier.customer_contact.create",
        label: "Créer un contact",
        description: "Ajoute un contact à un client de l’organisation.",
        group: GROUP,
        fields: CREATE_CONTACT_FIELDS,
        read_only: false,
        sample_output: contact_sample,
    },
    ActionSpec {
        kind: "mestier.customer_contact.update",
        label: "Modifier un contact",
        description: "Remplace les informations d’un contact.",
        group: GROUP,
        fields: UPDATE_CONTACT_FIELDS,
        read_only: false,
        sample_output: contact_sample,
    },
    ActionSpec {
        kind: "mestier.customer_contact.delete",
        label: "Supprimer un contact",
        description: "Supprime un contact d’un client de l’organisation.",
        group: GROUP,
        fields: CONTACT_FIELDS,
        read_only: false,
        sample_output: deleted_contact_sample,
    },
    ActionSpec {
        kind: "mestier.customer_context.list",
        label: "Lister les lieux d’un client",
        description: "Renvoie les lieux (chantiers, adresses) d’un client, page par page.",
        group: GROUP,
        fields: CUSTOMER_PAGE_FIELDS,
        read_only: true,
        sample_output: contexts_sample,
    },
    ActionSpec {
        kind: "mestier.customer_context.get",
        label: "Lire un lieu",
        description: "Renvoie un lieu d’un client de l’organisation.",
        group: GROUP,
        fields: CONTEXT_FIELDS,
        read_only: true,
        sample_output: context_sample,
    },
    ActionSpec {
        kind: "mestier.customer_context.create",
        label: "Créer un lieu",
        description: "Ajoute un lieu (chantier, adresse) à un client de l’organisation.",
        group: GROUP,
        fields: CREATE_CONTEXT_FIELDS,
        read_only: false,
        sample_output: context_sample,
    },
    ActionSpec {
        kind: "mestier.customer_context.update",
        label: "Modifier un lieu",
        description: "Remplace les informations d’un lieu.",
        group: GROUP,
        fields: UPDATE_CONTEXT_FIELDS,
        read_only: false,
        sample_output: context_sample,
    },
    ActionSpec {
        kind: "mestier.customer_context.delete",
        label: "Supprimer un lieu",
        description: "Supprime un lieu d’un client de l’organisation.",
        group: GROUP,
        fields: CONTEXT_FIELDS,
        read_only: false,
        sample_output: deleted_context_sample,
    },
];
