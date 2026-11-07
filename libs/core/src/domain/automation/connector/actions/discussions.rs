use serde_json::{Value, json};

use crate::domain::automation::connector::action::ActionSpec;
use crate::domain::automation::connector::field::{Field, SelectOption};

const GROUP: &str = "Discussions";
const NIL: &str = "00000000-0000-0000-0000-000000000000";
const MOMENT: &str = "2026-01-01T00:00:00Z";

const TARGET_TYPES: &[SelectOption] = &[
    SelectOption::new("role", "Rôle"),
    SelectOption::new("member", "Membre"),
];

const WINDOW_FIELDS: &[Field] = &[
    Field::number("limit", "Nombre maximum d’éléments", false),
    Field::number("offset", "Décalage", false),
];

const CREATE_CATEGORY_FIELDS: &[Field] = &[
    Field::text("name", "Nom", true),
    Field::number("position", "Position", false),
];

const UPDATE_CATEGORY_FIELDS: &[Field] = &[
    Field::text("category_id", "Catégorie", true),
    Field::text("name", "Nom", false),
    Field::number("position", "Position", false),
];

const CATEGORY_FIELDS: &[Field] = &[Field::text("category_id", "Catégorie", true)];

const CHANNEL_FIELDS: &[Field] = &[Field::text("channel_id", "Canal", true)];

const CREATE_CHANNEL_FIELDS: &[Field] = &[
    Field::text("name", "Nom", true),
    Field::text("topic", "Sujet", false),
    Field::number("position", "Position", false),
    Field::text("category_id", "Catégorie", false),
];

const UPDATE_CHANNEL_FIELDS: &[Field] = &[
    Field::text("channel_id", "Canal", true),
    Field::text("name", "Nom", false),
    Field::text("topic", "Sujet", false),
    Field::number("position", "Position", false),
    Field::text("category_id", "Catégorie", false),
];

const CREATE_THREAD_FIELDS: &[Field] = &[
    Field::text("channel_id", "Canal parent", true),
    Field::text("name", "Nom du fil", true),
    Field::text("origin_message_id", "Message d’origine", false),
];

const UPDATE_THREAD_FIELDS: &[Field] = &[
    Field::text("thread_id", "Fil", true),
    Field::text("name", "Nom du fil", false),
    Field::boolean("archived", "Archivé", false),
];

const THREAD_FIELDS: &[Field] = &[Field::text("thread_id", "Fil", true)];

const LIST_MESSAGES_FIELDS: &[Field] = &[
    Field::text("channel_id", "Canal", true),
    Field::text("before", "Avant le message", false),
    Field::text("after", "Après le message", false),
    Field::number("limit", "Nombre maximum de messages", false),
];

const EDIT_MESSAGE_FIELDS: &[Field] = &[
    Field::text("message_id", "Message", true),
    Field::text("content", "Contenu", true),
];

const MESSAGE_FIELDS: &[Field] = &[Field::text("message_id", "Message", true)];

const REACTORS_FIELDS: &[Field] = &[
    Field::text("message_id", "Message", true),
    Field::text("emoji", "Émoji", true),
];

const UPDATE_WEBHOOK_FIELDS: &[Field] = &[
    Field::text("webhook_id", "Webhook", true),
    Field::text("name", "Nom", false),
    Field::text("avatar_url", "URL de l’avatar", false),
];

const CREATE_WEBHOOK_FIELDS: &[Field] = &[
    Field::text("channel_id", "Canal", true),
    Field::text("name", "Nom", true),
    Field::text("avatar_url", "URL de l’avatar", false),
];

const REACTION_FIELDS: &[Field] = &[
    Field::text("message_id", "Message", true),
    Field::text("emoji", "Émoji", true),
];

const WEBHOOK_FIELDS: &[Field] = &[Field::text("webhook_id", "Webhook", true)];

const SET_EVERYONE_FIELDS: &[Field] = &[
    Field::text("channel_id", "Canal", true),
    Field::number("allow", "Droits accordés (masque de bits)", true),
    Field::number("deny", "Droits refusés (masque de bits)", true),
];

const SET_TARGET_FIELDS: &[Field] = &[
    Field::text("channel_id", "Canal", true),
    Field::select("target_type", "Type de cible", true, TARGET_TYPES),
    Field::text("target_id", "Cible", true),
    Field::number("allow", "Droits accordés (masque de bits)", true),
    Field::number("deny", "Droits refusés (masque de bits)", true),
];

const REMOVE_TARGET_FIELDS: &[Field] = &[
    Field::text("channel_id", "Canal", true),
    Field::select("target_type", "Type de cible", true, TARGET_TYPES),
    Field::text("target_id", "Cible", true),
];

fn category_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "name": "Chantiers",
        "position": 0,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn category_list_sample() -> Value {
    json!([category_sample()])
}

fn deleted_category_sample() -> Value {
    json!({ "category_id": NIL, "deleted": true })
}

fn channel_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "channel_type": "TEXT",
        "name": "general",
        "topic": null,
        "position": 0,
        "category_id": null,
        "parent_id": null,
        "origin_message_id": null,
        "archived": false,
        "project_id": null,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn thread_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "channel_type": "THREAD",
        "name": "Fil",
        "topic": null,
        "position": 0,
        "category_id": null,
        "parent_id": NIL,
        "origin_message_id": null,
        "archived": false,
        "project_id": null,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn thread_list_sample() -> Value {
    json!([thread_sample()])
}

fn channel_list_sample() -> Value {
    json!([channel_sample()])
}

fn deleted_channel_sample() -> Value {
    json!({ "channel_id": NIL, "deleted": true })
}

fn deleted_thread_sample() -> Value {
    json!({ "thread_id": NIL, "deleted": true })
}

fn message_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "channel_id": NIL,
        "author_type": "SYSTEM",
        "author_user_id": null,
        "author_webhook_id": null,
        "content": "Bonjour",
        "components": null,
        "mention_user_ids": [],
        "mention_role_ids": [],
        "mention_channel_ids": [],
        "mention_everyone": false,
        "reactions": [],
        "attachments": [],
        "edited_at": null,
        "created_at": MOMENT,
    })
}

fn message_list_sample() -> Value {
    json!([message_sample()])
}

fn deleted_message_sample() -> Value {
    json!({ "message_id": NIL, "deleted": true })
}

fn reactors_sample() -> Value {
    json!([{ "user_id": NIL }])
}

fn webhook_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "channel_id": NIL,
        "name": "Alertes",
        "avatar_url": null,
        "created_by": NIL,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn created_webhook_sample() -> Value {
    json!({
        "id": NIL,
        "organization_id": NIL,
        "channel_id": NIL,
        "name": "Alertes",
        "avatar_url": null,
        "created_by": NIL,
        "created_at": MOMENT,
        "updated_at": MOMENT,
        "note": "The webhook token is not exposed in automation output.",
    })
}

fn reaction_sample() -> Value {
    json!({ "message_id": NIL, "emoji": "👍", "reacted": true })
}

fn reaction_removed_sample() -> Value {
    json!({ "message_id": NIL, "emoji": "👍", "reacted": false })
}

fn webhook_list_sample() -> Value {
    json!([webhook_sample()])
}

fn deleted_webhook_sample() -> Value {
    json!({ "webhook_id": NIL, "deleted": true })
}

fn overwrite_sample() -> Value {
    json!({
        "target_type": "role",
        "target_id": NIL,
        "allow": 32,
        "deny": 0,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn overwrite_list_sample() -> Value {
    json!([overwrite_sample()])
}

fn overwrite_everyone_sample() -> Value {
    json!({
        "target_type": "everyone",
        "target_id": null,
        "allow": 32,
        "deny": 0,
        "created_at": MOMENT,
        "updated_at": MOMENT,
    })
}

fn removed_everyone_sample() -> Value {
    json!({ "channel_id": NIL, "target_type": "everyone", "target_id": null, "removed": true })
}

fn removed_overwrite_sample() -> Value {
    json!({ "channel_id": NIL, "target_type": "role", "target_id": NIL, "removed": true })
}

pub(super) const SPECS: &[ActionSpec] = &[
    ActionSpec {
        kind: "mestier.discussions.list_categories",
        label: "Lister les catégories de discussion",
        description: "Retourne les catégories de discussion de l’organisation.",
        group: GROUP,
        fields: WINDOW_FIELDS,
        read_only: true,
        sample_output: category_list_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.create_category",
        label: "Créer une catégorie de discussion",
        description: "Crée une catégorie qui regroupe des canaux de discussion.",
        group: GROUP,
        fields: CREATE_CATEGORY_FIELDS,
        read_only: false,
        sample_output: category_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.update_category",
        label: "Modifier une catégorie de discussion",
        description: "Change le nom ou la position d’une catégorie ; ce qui n’est pas renseigné reste inchangé.",
        group: GROUP,
        fields: UPDATE_CATEGORY_FIELDS,
        read_only: false,
        sample_output: category_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.delete_category",
        label: "Supprimer une catégorie de discussion",
        description: "Supprime une catégorie de discussion de l’organisation.",
        group: GROUP,
        fields: CATEGORY_FIELDS,
        read_only: false,
        sample_output: deleted_category_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.list_channels",
        label: "Lister les canaux de discussion",
        description: "Retourne tous les canaux de l’organisation, sans filtrer par droits de lecture.",
        group: GROUP,
        fields: WINDOW_FIELDS,
        read_only: true,
        sample_output: channel_list_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.get_channel",
        label: "Lire un canal de discussion",
        description: "Retourne un canal de l’organisation.",
        group: GROUP,
        fields: CHANNEL_FIELDS,
        read_only: true,
        sample_output: channel_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.create_channel",
        label: "Créer un canal de discussion",
        description: "Crée un canal textuel, éventuellement rangé dans une catégorie.",
        group: GROUP,
        fields: CREATE_CHANNEL_FIELDS,
        read_only: false,
        sample_output: channel_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.update_channel",
        label: "Modifier un canal de discussion",
        description: "Change le nom, le sujet, la position ou la catégorie d’un canal ; ce qui n’est pas renseigné reste inchangé.",
        group: GROUP,
        fields: UPDATE_CHANNEL_FIELDS,
        read_only: false,
        sample_output: channel_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.delete_channel",
        label: "Supprimer un canal de discussion",
        description: "Supprime un canal ; le canal d’un projet ne peut pas l’être directement.",
        group: GROUP,
        fields: CHANNEL_FIELDS,
        read_only: false,
        sample_output: deleted_channel_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.list_threads",
        label: "Lister les fils d’un canal",
        description: "Retourne les fils de discussion ouverts sous un canal.",
        group: GROUP,
        fields: CHANNEL_FIELDS,
        read_only: true,
        sample_output: thread_list_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.create_thread",
        label: "Créer un fil de discussion",
        description: "Ouvre un fil sous un canal textuel, éventuellement à partir d’un message.",
        group: GROUP,
        fields: CREATE_THREAD_FIELDS,
        read_only: false,
        sample_output: thread_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.update_thread",
        label: "Modifier un fil de discussion",
        description: "Renomme ou archive un fil ; ce qui n’est pas renseigné reste inchangé.",
        group: GROUP,
        fields: UPDATE_THREAD_FIELDS,
        read_only: false,
        sample_output: thread_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.delete_thread",
        label: "Supprimer un fil de discussion",
        description: "Supprime un fil de l’organisation.",
        group: GROUP,
        fields: THREAD_FIELDS,
        read_only: false,
        sample_output: deleted_thread_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.list_messages",
        label: "Lister les messages d’un canal",
        description: "Retourne les messages d’un canal, du plus récent au plus ancien, avec un curseur avant ou après.",
        group: GROUP,
        fields: LIST_MESSAGES_FIELDS,
        read_only: true,
        sample_output: message_list_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.edit_message",
        label: "Modifier un message de l’automatisation",
        description: "Réécrit le contenu d’un message publié par une automatisation ; les messages des utilisateurs ne peuvent pas être modifiés.",
        group: GROUP,
        fields: EDIT_MESSAGE_FIELDS,
        read_only: false,
        sample_output: message_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.delete_message",
        label: "Supprimer un message",
        description: "Supprime un message de l’organisation, quel que soit son auteur.",
        group: GROUP,
        fields: MESSAGE_FIELDS,
        read_only: false,
        sample_output: deleted_message_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.list_reactors",
        label: "Lister les auteurs d’une réaction",
        description: "Retourne les identifiants des utilisateurs qui ont réagi à un message avec un émoji donné.",
        group: GROUP,
        fields: REACTORS_FIELDS,
        read_only: true,
        sample_output: reactors_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.list_webhooks",
        label: "Lister les webhooks d’un canal",
        description: "Retourne les webhooks d’un canal, sans leur jeton.",
        group: GROUP,
        fields: CHANNEL_FIELDS,
        read_only: true,
        sample_output: webhook_list_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.update_webhook",
        label: "Modifier un webhook de canal",
        description: "Change le nom ou l’avatar d’un webhook ; ce qui n’est pas renseigné reste inchangé.",
        group: GROUP,
        fields: UPDATE_WEBHOOK_FIELDS,
        read_only: false,
        sample_output: webhook_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.delete_webhook",
        label: "Supprimer un webhook de canal",
        description: "Supprime un webhook ; son jeton cesse aussitôt de fonctionner.",
        group: GROUP,
        fields: WEBHOOK_FIELDS,
        read_only: false,
        sample_output: deleted_webhook_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.list_channel_permissions",
        label: "Lister les droits spécifiques d’un canal",
        description: "Retourne les exceptions de droits d’un canal : tout le monde, rôles et membres.",
        group: GROUP,
        fields: CHANNEL_FIELDS,
        read_only: true,
        sample_output: overwrite_list_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.set_everyone_permissions",
        label: "Définir les droits de tout le monde sur un canal",
        description: "Crée ou remplace l’exception de droits qui s’applique à tous les membres d’un canal.",
        group: GROUP,
        fields: SET_EVERYONE_FIELDS,
        read_only: false,
        sample_output: overwrite_everyone_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.remove_everyone_permissions",
        label: "Retirer les droits de tout le monde sur un canal",
        description: "Supprime l’exception de droits qui s’applique à tous les membres d’un canal.",
        group: GROUP,
        fields: CHANNEL_FIELDS,
        read_only: false,
        sample_output: removed_everyone_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.set_target_permissions",
        label: "Définir les droits d’un rôle ou d’un membre sur un canal",
        description: "Crée ou remplace l’exception de droits d’un rôle ou d’un utilisateur membre de l’organisation.",
        group: GROUP,
        fields: SET_TARGET_FIELDS,
        read_only: false,
        sample_output: overwrite_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.remove_target_permissions",
        label: "Retirer les droits d’un rôle ou d’un membre sur un canal",
        description: "Supprime l’exception de droits d’un rôle ou d’un utilisateur sur un canal.",
        group: GROUP,
        fields: REMOVE_TARGET_FIELDS,
        read_only: false,
        sample_output: removed_overwrite_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.create_webhook",
        label: "Créer un webhook de canal",
        description: "Crée un webhook sur un canal, au nom de l’identité Automatisation. Le jeton n’est jamais exposé dans le résultat.",
        group: GROUP,
        fields: CREATE_WEBHOOK_FIELDS,
        read_only: false,
        sample_output: created_webhook_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.add_reaction",
        label: "Ajouter une réaction",
        description: "Ajoute une réaction à un message, au nom de l’identité Automatisation.",
        group: GROUP,
        fields: REACTION_FIELDS,
        read_only: false,
        sample_output: reaction_sample,
    },
    ActionSpec {
        kind: "mestier.discussions.remove_reaction",
        label: "Retirer une réaction",
        description: "Retire la réaction de l’identité Automatisation sur un message.",
        group: GROUP,
        fields: REACTION_FIELDS,
        read_only: false,
        sample_output: reaction_removed_sample,
    },
];
