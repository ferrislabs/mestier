# Cursus — sous-projet 1 : le socle

## Goal

Donner à un centre de formation (Ascor) le référentiel de l'alternance dans Mestier : établissements, formations, promotions, trinômes, contrats, entreprises d'accueil, et faire entrer les apprentis et les tuteurs entreprise dans un portail FerrisKey qui ne leur montre que leurs trinômes.

## Context

Cursus remplace Studea (Effet B), l'outil de suivi pédagogique de l'alternance qu'Ascor utilise aujourd'hui. Étude de l'outil : https://claude.ai/artifact/AboKNw6bczbR752mfwSCeH. Une organisation Mestier qui active Cursus tient le rôle d'une instance Studea. Le module est découpé en sous-projets : 1 socle (ce document), 2 livret, 3 suivi (assiduité, compétences, notes), 4 pilotage (statistiques, mails, imports). Décision d'accès : [ADR 0015](../adr/0015-cursus-participants-are-not-members.md). Aucune donnée n'est importée de Studea.

## Glossary

Voir `docs/glossary.md` : Cursus, Participant, Triad (trinôme), Promotion, Host company (entreprise), Closure reason (motif de clôture).

## Invariants

- Toute table porte `org_id` et toute requête filtre dessus [test]
- Un participant n'est jamais un `Member` ; aucune route hors `/cursus/me` et `/cursus/invitations` ne lui répond [test]
- Une requête de portail ne renvoie que les trinômes du `ParticipantScope` ; hors périmètre, la réponse est 404 [test]
- Un lien tuteur archivé (`archived_at` non nul) ne donne que la lecture [type : `TriadAccess::ReadOnly`]
- Un apprenti a au plus un trinôme par promotion [test : `UNIQUE (promotion_id, apprentice_id)`]
- Un participant est lié à au plus un utilisateur par organisation, et un utilisateur à au plus un participant par organisation [test : `UNIQUE (org_id, user_id)`]
- Un contrat a au plus un tuteur principal et un tuteur secondaire [type : `TutorRank` + clé `(contract_id, rank)`]
- Clore un contrat archive ses liens tuteur dans la même transaction que `cursus.contract.closed` [test]
- Clore un trinôme clôt ses contrats ouverts avec la même date, le même motif et la même note [test]
- Clore un contrat ou un trinôme déjà clos est refusé [test, 409]
- `end_date >= start_date` sur un contrat [type : constructeur `ContractPeriod::new`]
- Seul le hash SHA-256 du jeton d'invitation est stocké [test]
- Dans l'organisation où il est membre, un utilisateur passe par la console ; `/portail/$slug` le redirige [test front]

## Out of scope

- Livret, questionnaires, assiduité, compétences, notes, statistiques
- Import depuis Studea, identifiants externes (CRM, OfaLink, ECFA)
- Composantes, tuteur tertiaire, gestionnaire entreprise, activation de modules par promotion
- Mail d'invitation intégré : comme pour les membres, le lien est montré à l'opérateur et l'événement `cursus.participant.invited` permet à un workflow d'automatisation de l'envoyer
- Filtre « seulement mes trinômes » pour les tuteurs pédagogiques (membres)
- `scope` sur les motifs de clôture
- Écriture depuis le portail (le portail est en lecture seule dans ce sous-projet)

## Design

### Permissions

Quatre bits dans `Permissions`, ajoutés à `default_admin_business_permissions()` et au rôle `admin` existant par migration, jamais au rôle `member` :

| Bit | Valeur | Action authz | Couvre |
|---|---|---|---|
| `VIEW_CURSUS` | `1 << 23` | — (contrôle direct du bit, comme `VIEW_SUPPORT`) | Lire tout le module |
| `MANAGE_CURSUS_CATALOG` | `1 << 24` | `cursus.catalog.manage` | Établissements, formations, promotions, motifs de clôture |
| `MANAGE_CURSUS_TRIADS` | `1 << 25` | `cursus.triads.manage` | Trinômes, contrats, entreprises d'accueil |
| `MANAGE_CURSUS_PARTICIPANTS` | `1 << 26` | `cursus.participants.manage` | Participants, invitations, désactivation |

Un `MANAGE_*` n'implique pas `VIEW_CURSUS`. Les routes `/cursus/me/*` ne demandent aucun bit : elles résolvent un participant.

### Contextes bornés

Un répertoire par contexte dans `libs/core/src/{domain,application,infrastructure}` :

| Contexte | Agrégats | Marqueur registry |
|---|---|---|
| `cursus_catalog` | Establishment, Training, Promotion, ClosureReason | `CursusCatalog` |
| `cursus_host_company` | HostCompany | `CursusHostCompany` |
| `cursus_participant` | Participant, Apprentice, CompanyTutor, invitation | `CursusParticipant` |
| `cursus_triad` | Triad, Contract, ContractTutor, `ParticipantScope` | `CursusTriad` |

Handlers : un crate `libs/handlers-cursus`, un sous-module par agrégat, plus `me` (portail et « Mes trinômes ») et `invitation` (acceptation). Le contrôle du bit `VIEW_CURSUS` se fait par `require_view_cursus` dans le crate, sur le modèle de `handlers_support::require_view_support`. Les `MANAGE_*` sont vérifiés dans le use case par `policy::require`.

### Schéma (`cursus`)

```sql
CREATE SCHEMA IF NOT EXISTS cursus;

cursus.establishment  (id, org_id, name, uai NULL, siret NULL, address NULL, is_main_site BOOL, archived_at NULL, created_at, updated_at)
cursus.training       (id, org_id, establishment_id, name, rncp_code NULL, rncp_label NULL, level NULL, degree_type NULL, archived_at NULL, created_at, updated_at)
cursus.promotion      (id, org_id, training_id, label, start_year INT, end_year INT CHECK (end_year >= start_year), archived_at NULL, created_at, updated_at)
cursus.closure_reason (id, org_id, label, archived_at NULL, created_at)

cursus.host_company   (id, org_id, name, siret NULL, address NULL, legal_form NULL, naf_code NULL, employee_count INT NULL, director_name NULL, archived_at NULL, created_at, updated_at)

cursus.participant    (id, org_id, user_id NULL REFERENCES users, last_name, first_name NULL, email, phone NULL,
                       invitation_token_hash NULL, invitation_expires_at NULL, deactivated_at NULL, created_at, updated_at,
                       UNIQUE (org_id, user_id), UNIQUE (org_id, lower(email)), UNIQUE (invitation_token_hash))
cursus.apprentice     (id, org_id, participant_id UNIQUE, birth_date NULL, ine_code NULL, address NULL, created_at, updated_at)
cursus.company_tutor  (id, org_id, participant_id, host_company_id, job_title NULL, created_at, updated_at, UNIQUE (participant_id, host_company_id))

cursus.triad          (id, org_id, promotion_id, apprentice_id, pedagogic_tutor_member_id NULL REFERENCES members,
                       closed_at NULL, closure_reason_id NULL, closure_note NULL, created_at, updated_at,
                       UNIQUE (promotion_id, apprentice_id))
cursus.contract       (id, org_id, triad_id, host_company_id, kind TEXT CHECK (kind IN ('APPRENTICESHIP','PROFESSIONALIZATION')),
                       number NULL, start_date DATE, end_date DATE CHECK (end_date >= start_date),
                       closed_at NULL, closure_reason_id NULL, closure_note NULL,
                       cerfa_file_key NULL, mission_sheet_file_key NULL, created_at, updated_at)
cursus.contract_tutor (contract_id, company_tutor_id, org_id, rank TEXT CHECK (rank IN ('MAIN','SECONDARY')), archived_at NULL,
                       PRIMARY KEY (contract_id, rank))
```

Statut d'un participant, calculé et jamais stocké : `Pending` (pas de `user_id`), `Active`, `Inactive` (`deactivated_at` non nul). `degree_type` et `level` sont du texte libre ; `kind` est l'enum `ContractKind`.

### Accès des participants

```rust
pub enum TriadRole { Apprentice, MainTutor, SecondaryTutor }
pub enum TriadAccess { Active, ReadOnly { since: DateTime<Utc> } }
pub struct TriadLink { pub triad_id: TriadId, pub role: TriadRole, pub access: TriadAccess }
pub struct ParticipantScope { pub participant_id: ParticipantId, pub organization_id: OrganizationId, pub links: Vec<TriadLink> }
impl ParticipantScope { pub fn link(&self, triad_id: TriadId) -> Option<&TriadLink>; }
```

Calculé à chaque requête par le use case `participant_scope(user_id, organization_id)` : le participant non désactivé de cet utilisateur dans l'organisation, son trinôme d'apprenti (`Active`), et chaque `contract_tutor` de ses fiches tuteur (`ReadOnly` si `archived_at`). Un tuteur présent sur plusieurs contrats du même trinôme garde le lien le plus fort (`Active` l'emporte). Pas de participant : `CoreError::Forbidden`. Jamais stocké sur `MestierUseCase`.

### Invitation

`POST .../participants/{id}/invitation` génère un jeton par `invitation::token::generate()`, stocke son hash et une expiration à 14 jours, renvoie le jeton en clair une seule fois, émet `cursus.participant.invited`. Réinviter remplace le jeton. `POST /api/v1/cursus/invitations/{token}/accept` (authentifié, sans adhésion) : jeton inconnu ou expiré → 410 ; participant déjà lié à un autre utilisateur → 409 ; l'utilisateur a déjà un participant dans cette organisation → 409. Sinon `user_id` est rempli, le hash effacé, `cursus.participant.activated` émis, et la réponse renvoie l'organisation (id, slug, nom).

### Routes (préfixe `/api/v1`)

Opérateurs, sous `/organizations/{organization_id}/cursus` ; `GET` demande `VIEW_CURSUS`, les écritures le bit `MANAGE_*` de la ligne :

| Ressource | Routes | Bit d'écriture |
|---|---|---|
| Établissements | `GET/POST /establishments`, `GET/PATCH /establishments/{id}`, `POST /establishments/{id}/archive` | CATALOG |
| Formations | `GET/POST /trainings` (`?establishment_id`), `GET/PATCH /trainings/{id}`, `POST /trainings/{id}/archive` | CATALOG |
| Promotions | `GET/POST /promotions` (`?training_id&archived`), `GET/PATCH /promotions/{id}`, `POST /promotions/{id}/archive` | CATALOG |
| Motifs de clôture | `GET/POST /closure-reasons`, `PATCH /closure-reasons/{id}`, `POST /closure-reasons/{id}/archive` | CATALOG |
| Entreprises | `GET/POST /host-companies`, `GET/PATCH /host-companies/{id}`, `POST /host-companies/{id}/archive` | TRIADS |
| Participants | `GET/POST /participants` (`?kind=apprentice|company_tutor&status`), `GET/PATCH /participants/{id}`, `POST /participants/{id}/invitation`, `POST /participants/{id}/deactivate`, `POST /participants/{id}/reactivate` | PARTICIPANTS |
| Fiches | `PUT /participants/{id}/apprentice`, `POST /participants/{id}/company-tutors`, `DELETE /company-tutors/{id}` | PARTICIPANTS |
| Trinômes | `GET/POST /triads` (`?promotion_id&closed`), `GET/PATCH /triads/{id}`, `POST /triads/{id}/close` | TRIADS |
| Contrats | `GET/POST /triads/{id}/contracts`, `GET/PATCH /contracts/{id}`, `POST /contracts/{id}/close`, `PUT /contracts/{id}/tutors` (`{main, secondary}`), `PUT /contracts/{id}/files/{cerfa|mission_sheet}` (multipart), `GET /contracts/{id}/files/{kind}` (URL présignée) | TRIADS |

Portail et « Mes trinômes », sans bit :

- `GET /cursus/me/spaces` → organisations où l'utilisateur a un participant actif : `[{organization_id, slug, name, is_member}]`
- `GET /organizations/{organization_id}/cursus/me/triads` → mes trinômes avec rôle et accès
- `GET /organizations/{organization_id}/cursus/me/triads/{triad_id}` → trinôme, promotion, formation, apprenti, contrats et tuteurs ; 404 hors périmètre
- `POST /cursus/invitations/{token}/accept`

`POST /participants` accepte un bloc facultatif `apprentice` ou `company_tutor` (`host_company_id`, `job_title`) créé dans la même transaction. Listes paginées avec le crate `pagination` (référentiels courts : liste simple). Clôture : `{closed_at: date, closure_reason_id, closure_note?}`.

### Événements (`libs/events`, descripteurs ajoutés à `automation::catalogue`)

`cursus.participant.invited`, `cursus.participant.activated`, `cursus.triad.created`, `cursus.triad.closed`, `cursus.contract.created`, `cursus.contract.closed`. Sujet : l'entité concernée. Version 1.

### Front

Console (`apps/webapp/src/pages/cursus/`, module `cursus` dans `registry.ts`, icône `GraduationCap`, `requiredPermission` absent car « Mes trinômes » doit rester visible) :

| Entrée | Route | Visible si |
|---|---|---|
| Promotions | `/o/$slug/cursus/promotions`, `/o/$slug/cursus/promotions/$promotionId` (ses trinômes) | `VIEW_CURSUS` |
| Trinômes | `/o/$slug/cursus/trinomes`, `/o/$slug/cursus/trinomes/$triadId` (contrats, clôture) | `VIEW_CURSUS` |
| Participants | `/o/$slug/cursus/participants`, `/o/$slug/cursus/participants/$participantId` | `VIEW_CURSUS` |
| Entreprises | `/o/$slug/cursus/entreprises`, `/o/$slug/cursus/entreprises/$hostCompanyId` | `VIEW_CURSUS` |
| Mes trinômes | `/o/$slug/cursus/mes-trinomes` | l'utilisateur a un participant dans l'organisation |
| Réglages Cursus (établissements, formations, motifs de clôture) | section de `SETTINGS_SECTIONS` | `MANAGE_CURSUS_CATALOG` |

Boutons d'écriture affichés selon le bit `MANAGE_*`. Les quatre bits sont ajoutés à `PermissionName` et `PERMISSION_CATALOG` (zone `cursus`).

Portail : layout `/_portal` (`AuthGate`, sans `OrgGate`, sans shell), routes `/portail` (mes espaces), `/portail/$slug` (mes trinômes, archivés marqués « Historique, lecture seule »), `/portail/$slug/trinomes/$triadId`, `/portail/invitation/$token`. Un espace où `is_member` est vrai redirige vers `/o/$slug/cursus/mes-trinomes`. Le menu utilisateur de la console affiche « Mon portail Cursus » si `GET /cursus/me/spaces` renvoie un espace où `is_member` est faux.

### Erreurs

| Cas | Statut |
|---|---|
| Trinôme hors du périmètre d'un participant | 404 |
| Écriture d'un tuteur archivé (sous-projet 2) | 403 |
| Membre sans le bit | 403 |
| Second trinôme d'un apprenti dans une promotion, double clôture, invitation déjà liée | 409 |
| Jeton d'invitation inconnu ou expiré | 410 |

## Acceptance

- `@spec-cur-1` Un admin crée un établissement, une formation et une promotion ; un membre sans `VIEW_CURSUS` reçoit 403 sur leur liste.
- `@spec-cur-2` Un membre avec `VIEW_CURSUS` mais sans `MANAGE_CURSUS_CATALOG` lit les promotions et reçoit 403 à la création.
- `@spec-cur-3` Le rôle `admin` d'une organisation existante a les quatre bits après migration ; le rôle `member` n'en a aucun.
- `@spec-cur-4` Un motif de clôture archivé n'est plus listé par défaut mais reste lisible sur une clôture passée.
- `@spec-cur-5` Créer un second trinôme pour le même apprenti dans la même promotion renvoie 409.
- `@spec-cur-6` Inviter un participant renvoie un jeton ; l'accepter remplit `user_id` ; le réutiliser renvoie 410.
- `@spec-cur-7` Un apprenti voit son trinôme via `/cursus/me/triads` ; un autre trinôme de la même organisation renvoie 404.
- `@spec-cur-8` Le tuteur de l'entreprise A ne voit jamais un trinôme dont le contrat est chez l'entreprise B.
- `@spec-cur-9` Clore un contrat avec motif et note archive ses liens tuteur : le tuteur voit encore le trinôme, en `ReadOnly`.
- `@spec-cur-10` Clore un trinôme clôt ses contrats ouverts ; une seconde clôture renvoie 409.
- `@spec-cur-11` Un participant reçoit 403 sur une route opérateur de Cursus et sur toute route d'un autre module.
- `@spec-cur-12` Les données d'une organisation ne sont jamais visibles depuis une autre (repositories et `/me`).
- `@spec-cur-13` Front : les boutons de création sont absents sans le bit `MANAGE_*` correspondant.
- `@spec-cur-14` Front : un membre qui ouvre `/portail/$slug` de son organisation est redirigé vers « Mes trinômes ».
- `@spec-cur-15` Les migrations `.down.sql` s'appliquent et laissent la base dans son état précédent.

## Simulation

Non : pas de concurrence au-delà des contraintes d'unicité, que la base arbitre.

## Workstreams (PR empilées)

| # | Branche | Contenu | Dépend de |
|---|---|---|---|
| 1 | `feat/cursus-1-catalog` | Spec, ADR, glossaire ; les 4 bits et leur migration ; crate `handlers-cursus` ; `cursus_catalog` complet (back) | `main` |
| 2 | `feat/cursus-2-participants` | `cursus_host_company`, `cursus_participant`, invitation et acceptation (back) | 1 |
| 3 | `feat/cursus-3-triads` | `cursus_triad` : trinômes, contrats, tuteurs, clôtures, fichiers, `ParticipantScope`, routes `/me`, événements (back) | 2 |
| 4 | `feat/cursus-4-console` | Module console, réglages, catalogue des permissions (front) | 3 |
| 5 | `feat/cursus-5-portal` | Layout `/_portal`, invitation, « Mes trinômes », redirection (front) | 4 |

Fichiers tenus par l'orchestrateur (aucun sous-agent ne les écrit) : `Cargo.toml`, `libs/core/src/infrastructure/registry.rs`, `libs/core/src/{domain,application,infrastructure}/mod.rs`, `libs/core/src/lib.rs`, `libs/core/src/domain/role/mod.rs`, `libs/core/src/application/mod.rs` (authorizer), `libs/core/src/domain/automation/catalogue.rs`, `libs/handlers-cursus/src/lib.rs`, `apps/api/src/{router,openapi}.rs`, `migrations/`, `apps/webapp/src/modules/registry.ts`, `apps/webapp/src/pages/settings/registry.tsx`.

Régénérés à l'intégration : `.sqlx/` (`cargo sqlx prepare --workspace -- --all-targets`, base locale), `apps/webapp/src/api/api.client.ts` et `api.tanstack.ts` (`pnpm gen:api`), `apps/webapp/src/routeTree.gen.ts` (`pnpm build`), `Cargo.lock`.

## Risks / open questions

- Le portail n'a pas encore de vraie utilité sans le livret ; il pose la chaîne d'accès. Propriétaire : sous-projet 2.
- Les vues apprenti et tuteur de Studea n'ont pas été observées ; le sous-projet 2 en aura besoin. Propriétaire : Baptiste (comptes de test).
- Droit : on reprend des fonctions et un modèle de données, pas l'interface ni les libellés de Studea.

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored
cd apps/webapp && pnpm check && pnpm test && pnpm build
```

Amended: 2026-11-09 `POST /participants` crée la fiche apprenti ou tuteur dans la même transaction, pour ne jamais laisser un participant sans rôle.
