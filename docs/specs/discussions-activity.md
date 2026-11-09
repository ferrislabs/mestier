# Discussions — page « Activité » et préférences de notification

## Goal

Une page `/discussions/activite`, comme celle de Slack : y lister ce que je dois lire, c'est-à-dire les messages qui me mentionnent et les messages non lus des canaux que j'ai choisi de suivre. Un réglage de notification par canal décide de ce qui y apparaît.

## Context

Le serveur sait déjà créer des notifications `MENTION` (le `REPLY` existe en type mais rien ne le produit), tenir un état de lecture par canal et chiffrer les non-lus. Il n'existe aucun réglage de notification. Le bouton « Nouveau message » en haut de la sidebar est remplacé par « Activité » ; le « + » des messages directs reste le moyen d'écrire à quelqu'un.

## Glossary

- **Activité** : la liste des éléments à lire d'une personne dans une organisation. Rejeté : « fil de notifications ».
- **Niveau de notification** : `ALL` (tous les messages), `MENTIONS` (mentions seulement), `NOTHING` (rien), un par canal et par personne.
- **Élément d'activité** : une mention (`MENTION`) ou un canal suivi qui a des messages non lus (`UNREAD`).

## Invariants

- Le niveau par défaut est `ALL` pour un message direct, `MENTIONS` pour un canal [test]
- Une mention n'est jamais créée pour quelqu'un dont le niveau sur ce canal est `NOTHING` [test]
- Un canal au niveau `NOTHING` ne produit aucun élément d'activité, même pour des mentions déjà reçues [test]
- Un élément d'activité n'expose jamais un canal que la personne ne peut pas voir, ni un message direct dont elle n'est pas l'un des deux participants [test]
- Un non-lu n'apparaît que pour un canal au niveau `ALL` [test]
- Un niveau est personnel : lire ou écrire celui d'un autre est impossible [type : l'id vient de l'identité]
- Toute requête est scopée par `org_id` [test]

## Out of scope

- Notifications `REPLY` (rien ne les produit), notifications push et e-mail
- Niveau par défaut d'organisation ou par catégorie, mise en sourdine temporaire
- Marquer un non-lu depuis la page sans ouvrir le canal

## Design

- Table `chat.notification_preference(user_id, channel_id, org_id, level)`, une ligne par choix explicite ; sans ligne, le défaut s'applique.
- `PUT /chat/channels/{channel_id}/notification-level` `{level}` (visible par la personne) ; `GET /chat/organizations/{organization_id}/notification-levels` renvoie les choix explicites.
- `GET /chat/organizations/{organization_id}/activity?limit=` renvoie `[{kind, channel_id, message_id, author_user_id, author_type, excerpt, created_at, read, unread_count, notification_id}]`, du plus récent au plus ancien. Les extraits ne contiennent que du texte déjà lisible par la personne.
- `create_message` ignore les destinataires dont le niveau est `NOTHING`.
- Front : route `discussions.activite`, page `ActivityFeature` / `ActivityUI`, filtres Tout / Mentions / Non lus, « Tout marquer comme lu » (notifications de mention). Le niveau se règle dans le menu contextuel d'un canal (« Notifications »). Le bouton de la sidebar devient « Activité » avec le nombre de mentions non lues.

## Acceptance

- `@spec-act-1` Une mention reçue apparaît dans l'activité avec l'auteur, l'extrait et le canal, et passe en lu au clic.
- `@spec-act-2` Un canal au niveau `ALL` avec des messages non lus apparaît comme élément `UNREAD` avec son compte ; au niveau `MENTIONS` il n'apparaît pas.
- `@spec-act-3` Un message direct non lu apparaît par défaut.
- `@spec-act-4` Passer un canal à `NOTHING` retire ses éléments et empêche les nouvelles mentions.
- `@spec-act-5` Une mention dans un canal devenu invisible ou dans un message direct d'autrui n'apparaît pas.
- `@spec-act-6` Le bouton de la sidebar affiche le nombre de mentions non lues.

## Simulation

Non : pas de concurrence au-delà d'un `UPSERT` sur (personne, canal).

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/webapp && pnpm check && pnpm test && pnpm build
```
