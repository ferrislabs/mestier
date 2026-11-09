# Discussions — conformité au rework UI

## Goal

Aligner le module Discussions de Mestier sur le rework UI (`../mestier-rework/src/modules/discussions`, source de vérité visuelle) et câbler chaque action au backend.

## Context

Le rework est une maquette 100 % mock. Mestier a déjà le backend chat (canaux, catégories, messages, threads, réactions, read state, présence, notifications) et une UI proche. Écarts constatés :

| Écart | Backend |
|---|---|
| Route `/discussions` (Mestier : `/chat`) | n/a |
| Noms et avatars réels des auteurs (Mestier affiche « Vous » / « Membre ») | catalogue des membres existant |
| Surlignage des messages qui mentionnent l'utilisateur | `mention_user_ids` existe |
| Présence par personne (pastille en ligne / absent / hors ligne) | `list_presence` existe côté use case, pas de route |
| Séparateur « N nouveaux messages » et badge de non-lus chiffré | `read_state`, `/unread` existent |
| Messages directs (section sidebar, header DM) | absent : `ChannelType` = Text, Thread |
| Épinglés / enregistrés | mock local `message-flags`, pas de backend |

Décisions utilisateur : messages directs en backend complet ; route `/discussions` ; on garde les extras Mestier (catégories, menu admin du canal, panneau de fil, indicateur de frappe).

## Glossary

- **Message direct (DM)** : conversation privée entre deux membres d'une organisation. Canal de type `DIRECT`. Rejeté : « conversation privée », « MP ».
- **Canal** : canal de groupe (`TEXT`).

## Invariants

- Un DM a exactement deux participants distincts de la même organisation [type : table `channel_participants` + contrainte]
- Un seul DM par paire de membres et par organisation [test + index unique]
- Un DM n'est jamais visible, lisible ni écrivable par un non-participant [test]
- Un DM n'a ni catégorie, ni parent, ni projet [contrainte SQL]
- Toute requête est scopée par `org_id` [test]
- Le statut de présence du rework « absent » correspond à `DND` côté backend ; le libellé UI reste « Absent » [type : mapping unique dans `lib/presence.ts`]

## Out of scope

- Épinglés / enregistrés persistés (reste mock, hors périmètre, à ouvrir en issue)
- Groupes de plus de deux personnes
- Recherche plein texte côté serveur (filtre client conservé)
- Renommer le dossier `pages/chat` (seules les routes et le registre changent)

## Design

- Frontend : le module `registry.ts` garde l'id `chat` (clé interne, permissions), `basePath` devient `/discussions`. Les routes `/chat/*` redirigent vers `/discussions/*`.
- Auteurs : `GET /chat/organizations/{id}/directory` renvoie `{user_id, display_name}` des membres occupés (l'API membre n'expose volontairement jamais `users.id`) ; `MessageList` reçoit `authors` ; `MessageRow` affiche nom, avatar, surlignage de mention.
- Présence : route `GET /api/v1/chat/organizations/{organization_id}/presence` (snapshot), consommée par `PresenceProvider`.
- Non-lus : `UnreadSeparator` placé avant le premier message non lu, compteur numérique dans la sidebar.
- DM : `ChannelType::Direct`, table `chat.channel_participants`, `POST /chat/organizations/{id}/direct-channels` (idempotent par paire), listé avec les canaux filtré par participant.

## Acceptance

- `@spec-disc-1` `/discussions` affiche la sidebar et redirige vers le premier canal ; `/chat/x` redirige vers `/discussions/x`.
- `@spec-disc-2` Un message d'un autre membre affiche son nom d'affichage et ses initiales, pas « Membre ».
- `@spec-disc-3` Un message qui mentionne l'utilisateur courant est surligné, le chip de mention est renforcé.
- `@spec-disc-4` La liste des membres et l'en-tête montrent la présence réelle, y compris après rechargement (snapshot).
- `@spec-disc-5` Un canal avec des messages non lus affiche « N nouveaux messages » avant le premier non lu, et un badge chiffré dans la sidebar.
- `@spec-disc-6` Créer un DM avec un membre renvoie le DM existant s'il y en a déjà un.
- `@spec-disc-7` Un non-participant reçoit 404 sur un DM (lecture, écriture, liste).
- `@spec-disc-8` Les DM apparaissent dans la section « Messages directs » de la sidebar avec présence et non-lus.

## Simulation

Non : pas de concurrence distribuée au-delà de l'unicité par paire, couverte par un index unique et un test.

## Workstreams (une PR chacune, empilées)

1. `feat/discussions-route` : spec, route `/discussions`, auteurs réels, surlignage de mention. Front seul.
2. `feat/discussions-presence-unread` : route snapshot de présence, pastilles, séparateur et badges de non-lus.
3. `feat/discussions-direct-backend` : migration, domaine, repo, service, handlers, OpenAPI.
4. `feat/discussions-direct-ui` : section DM, header DM, création de DM, câblage.

Chaque PR cible la précédente. Fichiers de convergence (registre, `Cargo.toml`, `api.client.ts`, `route-tree.gen.ts`, `.sqlx`) : régénérés ou édités à l'intégration.

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd apps/webapp && pnpm check && pnpm test && pnpm build
```

Amended: 2026-11-08 le catalogue des membres ne porte pas `user_id` (invariant de `MemberResponse`), d'où la route annuaire `directory`. Le module garde l'id `chat`.
Amended: 2026-11-08 `/unread` ne renvoie que des ids de canaux : le compte « N nouveaux messages » demande un compteur côté backend (PR 2).
