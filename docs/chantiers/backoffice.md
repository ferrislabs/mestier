# Backoffice : administration de la plateforme

État : **spec, 2026-11-12** (branche `feat/backoffice`).

## Goal

Le staff de Mestier administre la plateforme depuis un back-office : il voit les organisations et leurs membres, active des feature flags par organisation, voit les utilisateurs, publie les news et lit des statistiques globales.

## Context

- Le back-office des news existe déjà (`/admin/news`, crate `handlers-platform`). Il est ouvert par le rôle du royaume `mestier:admin`, lu dans `Identity::realm_roles()` et jamais donné au moteur de droits. La preuve de type s'appelle `NewsEditor`.
- Aucun mécanisme de feature flag n'existe. `organizations.mcp_enabled` et `field_clock_enabled` sont des réglages que l'organisation gère elle-même.
- `users` porte `created_at`, mais aucune date de connexion.
- Les événements de domaine sont enregistrés dans `automation.event` (`org_id`, `name`, `actor_kind`, `actor_id`, `payload`, `occurred_at`).
- Réponses du 2026-11-12 :
  1. un flag protège une fonctionnalité. Par défaut, aucune organisation n'y a accès. Le staff l'active organisation par organisation ;
  2. idem ;
  3. `mcp_enabled` et `field_clock_enabled` restent des réglages de l'organisation ;
  4. sur une organisation, le staff consulte et change les flags, rien d'autre ;
  5. les utilisateurs sont en lecture seule (nom, email, organisations, date d'inscription), avec une recherche ;
  6. les stats sont des comptages, sans montants ni activité ;
  7. un seul rôle, `mestier:admin`, et `NewsEditor` devient `PlatformAdmin` ;
  8. chaque changement de flag émet un événement avec son auteur, et l'historique s'affiche sur la fiche de l'organisation.

## Décisions

1. **Catalogue des flags dans le code** (ADR 0017). Le domaine déclare `FeatureFlag`, une enum fermée. Chaque variante a une clé stable (`snake_case`), un libellé et une description. Ajouter un flag = ajouter une variante. La base ne stocke que les activations.
2. **Désactivé par défaut, activé par une ligne.** La table `organization_feature_flags (org_id, flag, enabled_at, enabled_by)` contient une ligne par flag activé. L'absence de ligne vaut désactivé. Une clé en base inconnue du code est ignorée à la lecture.
3. **Appliqué par l'API, reflété par le webapp.**
   - API : `require_feature(state, org_id, FeatureFlag)` dans `libs/handlers` répond 403 avec un motif. Il existe aussi en couche axum, pour fermer d'un coup tout un routeur ou un groupe de routes.
   - Webapp : `GET /api/v1/organizations/{id}/features` renvoie les clés actives pour un membre. Un module, une section ou un onglet du registre peut porter `requiredFeature`, et il est alors caché si le flag est inactif.
4. **`PlatformAdmin` remplace `NewsEditor`.** Même construction (rôle `mestier:admin`, jeton propre, jamais délégué). Chaque use case du back-office l'exige en argument. `GET /api/v1/admin/access` → `{ can_access }` remplace `GET /api/v1/news/access`.
5. **Historique = événements.** `organization.feature_enabled` et `organization.feature_disabled` (payload `{ flag }`, acteur = l'admin) sont émis via `with_tx_emitting`. La fiche de l'organisation lit `automation.event` filtré par `org_id` et par ces deux noms. Ces deux événements ne sont pas proposés comme déclencheurs aux workflows de l'organisation.
6. **Lecture transverse, exception voulue au bornage par `org_id`.** Les requêtes du back-office (liste des organisations, des utilisateurs, stats) traversent les organisations. Elles vivent dans un port à part, `PlatformAdminRepository`, que seuls les use cases exigeant `PlatformAdmin` utilisent.
7. **Webapp : un espace `/admin`** avec sa propre barre latérale : Vue d'ensemble (stats), Organisations, Utilisateurs, News. On y entre par le menu utilisateur, et l'entrée n'apparaît qu'avec `can_access`. `/admin/news` garde son URL.

## Invariants

1. Sans ligne d'activation, une route protégée par un flag répond 403 pour toutes les organisations, et l'entrée du webapp est cachée. [test]
2. Activer un flag pour A ne donne rien à B. [test]
3. Seul `mestier:admin`, sur son propre jeton, atteint un use case du back-office. Un jeton délégué ou client n'y arrive jamais. [type : `PlatformAdmin` ; test sur `Identity`]
4. Le rôle `mestier:admin` ne donne aucun droit dans une organisation. [test existant : `Identity::roles` reste vide]
5. Un changement de flag sans événement n'existe pas, et un événement sans changement non plus. [`with_tx_emitting` ; test]
6. Activer un flag déjà actif ou désactiver un flag inactif ne change rien et n'émet rien. [test]
7. Un flag inconnu est refusé à l'API (422). [type : `FeatureFlag: TryFrom<&str>`]
8. Migration réversible. [test : up, down, up]

## Out of scope

- Suspendre, supprimer ou modifier une organisation. Agir sur un utilisateur (désactiver, se connecter à sa place).
- Montants agrégés (CA), activité et connexions.
- Rôles du back-office par section.
- Activation d'un flag pour toutes les organisations d'un coup, pourcentage de déploiement, flags par utilisateur.
- Interface pour créer un flag : un flag se crée dans le code.

## Design

- **Migration** `20261121000001_create_organization_feature_flags` : `organization_feature_flags (org_id → organizations ON DELETE CASCADE, flag TEXT, enabled_at, enabled_by → users, PRIMARY KEY (org_id, flag))`.
- **Core** :
  - `domain/platform` : `PlatformAdmin` (déplacé depuis `domain/news/editor.rs`), port `PlatformAdminRepository` (`list_organizations(search, page)`, `organization_overview(id)`, `list_users(search, page)`, `stats(now)`), `PlatformStats`, `OrganizationSummary`, `UserSummary`.
  - `domain/feature_flag` : `FeatureFlag` (enum + `ALL`, `key`, `label`, `description`, `TryFrom<&str>`), port `FeatureFlagRepository` (`list_enabled(org)`, `is_enabled(org, flag)`, `enable(org, flag, by)` → bool, `disable(org, flag)` → bool), événements `FeatureEnabled` / `FeatureDisabled`.
  - `application/platform` : `list_organizations_for_admin`, `get_organization_for_admin` (membres, flags, historique), `set_organization_feature(admin, org, flag, enabled)`, `list_users_for_admin`, `platform_stats`, `list_feature_catalogue`. Tous exigent `PlatformAdmin`.
  - `application/feature_flag` : `enabled_features(org)`, `is_feature_enabled(org, flag)`.
- **Infrastructure** : `infrastructure/platform/postgres`, `infrastructure/feature_flag/postgres`, et leurs marqueurs dans `registry.rs`.
- **Stats** (`PlatformStats`) :
  - organisations actives (sans `deleted_at`) ;
  - utilisateurs actifs ;
  - inscriptions par semaine sur 12 semaines, par `users.created_at` ;
  - organisations créées par semaine sur 12 semaines ;
  - devis, factures, projets et tâches créés, au total et sur 30 jours ;
  - nombre d'organisations par flag.
- **API**, crate `handlers-platform`, tag `admin` (les routes news gardent le tag `news`) :
  - `GET /api/v1/admin/access` ;
  - `GET /api/v1/admin/stats` ;
  - `GET /api/v1/admin/features` (catalogue) ;
  - `GET /api/v1/admin/organizations?search=&page=&per_page=` ;
  - `GET /api/v1/admin/organizations/{id}` ;
  - `PUT /api/v1/admin/organizations/{id}/features/{flag}` `{ enabled }` ;
  - `GET /api/v1/admin/users?search=&page=&per_page=` ;
  - côté membre : `GET /api/v1/organizations/{id}/features`, dans `handlers-organization`.
- **Webapp** :
  - `routes/_app.admin.tsx` : layout avec garde `can_access` et barre latérale ;
  - `pages/admin-stats`, `pages/admin-organizations` (liste + fiche : membres, flags avec interrupteurs, historique), `pages/admin-users`, en split `feature/` + `ui/` ;
  - `/admin/news` passe dans le layout ;
  - `hooks/use-features.ts` (`useFeature(flag)`) et `requiredFeature` dans `modules/types.ts`, filtré là où `requiredPermission` l'est déjà.

## Acceptance

1. Sans activation, `GET` sur une route protégée par le flag de test répond 403 pour l'organisation A. Après activation pour A, elle répond 200 pour A et toujours 403 pour B.
2. `PUT .../features/{flag}` sans `mestier:admin` → 403. Avec un jeton délégué portant le rôle → 403.
3. Activer puis désactiver un flag écrit deux événements dans `automation.event`, avec l'admin pour acteur. Les rejouer ne change rien et n'ajoute aucun événement.
4. `GET /admin/organizations/{id}` renvoie les membres, les flags (catalogue complet, état actif ou non) et l'historique dans l'ordre antichronologique.
5. `GET /admin/organizations?search=treyve` et `GET /admin/users?search=erwan` filtrent sur le nom, le slug et l'email, sans tenir compte de la casse, et paginent.
6. `GET /admin/stats` sur un jeu de données connu renvoie les comptages attendus.
7. `PUT .../features/inconnu` → 422.
8. Webapp : un module marqué `requiredFeature` disparaît du lanceur et de la barre latérale quand le flag est inactif.
9. Webapp : `/admin` sans le rôle redirige vers l'accueil. Avec le rôle, les quatre sections s'affichent.
10. Migration up, down, up.

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored
cd apps/webapp && pnpm check && pnpm test && pnpm build
```
