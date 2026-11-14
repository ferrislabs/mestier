# Questionnaires S1 : persistance, services, cas d'usage, horloge. Plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Faire vivre le domaine `survey` de S0 : services de domaine, adaptateurs Postgres, cas d'usage transactionnels avec événements, permissions, horloge de campagne.

**Architecture:** Même découpage que `extra_period` : services génériques sur les ports dans `domain/survey/service.rs`, adaptateurs sous `infrastructure/survey/postgres/` (`#[repository(domain = X, backend = Postgres)]`, marqueurs dans `infrastructure/registry.rs`), cas d'usage `#[transactional(...)]` dans `application/survey/`. L'horloge de campagne tourne au tick du worker d'automatisation existant.

**Tech Stack:** Rust, sqlx (requêtes vérifiées à la compilation, cache `.sqlx/`), `mockall` (ports), `cucumber` (scénarios).

**Spec:** `docs/chantiers/questionnaires.md` (sections « Frozen contracts » et « À savoir pour S1 » obligatoires). Base : branche `feat/surveys` (PR #688). Cette branche `feat/surveys-s1` est empilée dessus.

## Global Constraints

- Zéro commentaire dans le code. Dispatch statique d'abord, pas de `async-trait`, pas de `dyn` sans raison, pas de `unwrap`/`expect` hors tests.
- Une transaction persiste ses événements avant de valider (`with_tx_emitting`, attribut `emitter`). Un événement existe si et seulement si sa transaction a validé.
- Toute requête est filtrée par `org_id`. Exceptions déclarées : recherche par `public_slug` (l'organisation est inconnue avant), horloge de campagne (toutes organisations, une transaction par campagne).
- Les contrats figés de S0 ne changent pas. Les ports peuvent recevoir des méthodes en plus.
- Le domaine ne génère ni aléa ni heure : les services reçoivent `now` ; les identifiants, jetons de reprise et slugs viennent de la couche application.
- Toute nouvelle `query!` : base vivante, puis `cargo sqlx prepare --workspace -- --all-targets` pour mettre à jour `.sqlx/`.
- Migrations réversibles, `.down.sql` testé.
- Vocabulaire : « questionnaire » (`survey`), « campagne », « ouvrir », « réponse », « fenêtre ». Jamais « sondage », « formulaire », « envoyer ».

## Review Focus

- Deux compléments simultanés de la même réponse : un seul réussit, l'autre renvoie « réponse déjà complète » (violation d'index unique traduite). Tâche 3 et 4.
- Une réponse après la fermeture ou hors fenêtre de campagne : refusée, même si la requête a lu le questionnaire juste avant. Tâche 2.
- Le plafond de réponses : la complétion qui dépasse est refusée, pas la création. Tâche 2.
- Une organisation ne voit jamais les réponses, questionnaires ou campagnes d'une autre, y compris via `find_by_public_slug`. Tâche 3.
- La campagne dont la fenêtre est déjà passée émet `started` puis `ended`. Tâche 5.
- Supprimer un questionnaire ayant des réponses est refusé. Tâche 2.

---

## Orchestration

Une seule branche, `feat/surveys-s1`, des tâches dans l'ordre, un sous-agent `ferrislabs:ferris-implementer` sur `opus` par groupe : (1) tâches 1 et 2, (2) tâche 3, (3) tâches 4 et 5, (4) tâche 6. Les groupes sont séquentiels (même crate). Relecture : une seule, sur la branche entière, par un relecteur `opus`, plus la vérification de l'orchestrateur (mode allégé validé par le demandeur).

**Points de convergence (orchestrateur, ou à reporter) :** `libs/core/src/domain/mod.rs`, `libs/core/src/lib.rs` (exports), `libs/core/src/application/mod.rs` (modules, actions d'autorisation, démarrage de l'horloge), `libs/core/src/infrastructure/registry.rs`. Le sous-agent peut les éditer seulement pour la ligne d'enregistrement de son propre module, et la liste dans son rapport.

**Régénérés :** `.sqlx/` (commande `cargo sqlx prepare --workspace -- --all-targets`), `Cargo.lock` (cargo, jamais à la main, jamais commité ici : il porte déjà des modifications qui ne sont pas de ce travail).

**Environnement :** `DATABASE_URL` défini (`source .env`), PostgreSQL 17.10.

---

### Task 1 : permissions

**Files:**
- Modify: `libs/core/src/domain/role/mod.rs`, `libs/core/src/application/mod.rs`
- Create: `migrations/20261123000001_grant_survey_permissions.{up,down}.sql`

**Interfaces:**
- Produces : `Permissions::MANAGE_SURVEYS`, `Permissions::VIEW_SURVEYS` (prochains bits libres), actions d'autorisation `survey.manage` et `survey.view` dans `application/mod.rs`, accordées par migration aux rôles administrateur (même schéma que `migrations/20260829000002_grant_business_permissions.up.sql`). Le rôle membre n'a aucune permission survey : répondre à un questionnaire `MEMBERS` ne demande que d'être membre de l'organisation.

- [ ] **Step 1: Test** : un test unitaire dans `domain/role` vérifie que les deux bits sont distincts des autres et que `all()` les contient. Run : `SQLX_OFFLINE=true cargo test -p mestier-core --lib role::` ; attendu : FAIL.
- [ ] **Step 2: Implémenter** les deux constantes, les actions `survey.manage`/`survey.view`, la migration up/down (admin seulement). Imiter la migration de `planning.manage`.
- [ ] **Step 3: Vérifier** : `SQLX_OFFLINE=true cargo test -p mestier-core --lib role::` PASS ; `sqlx migrate run`, `revert`, `run` réussissent.
- [ ] **Step 4: Commit** `feat(survey): add the survey permissions`.

### Task 2 : services de domaine

**Files:**
- Create: `libs/core/src/domain/survey/service.rs` (et `commands.rs` si besoin), tests dans `libs/core/src/domain/survey/service_tests.rs`
- Modify: `libs/core/src/domain/survey/{mod.rs,ports.rs,error.rs}`

**Interfaces:**
- Consumes : les types et ports figés de S0.
- Produces :
  - Ports ajoutés : `SurveyRepository::{find_by_public_slug(&mut self, slug: &PublicSlug) -> Option<SurveyRecord>, delete(&mut self, organization_id, id), has_responses(&mut self, organization_id, id) -> bool}` ; `CampaignRepository::{delete(&mut self, organization_id, id), list_due(&mut self, now: DateTime<Utc>, limit: u64) -> Vec<Campaign>}` (`list_due` est inter-organisations : campagnes dont `status_at(now)` diffère du statut stocké) ; `ResponseRepository::{find_in_progress_by_member(&mut self, organization_id, survey_id, MemberId) -> Option<ResponseRecord>, delete(&mut self, organization_id, id)}`.
  - `SurveyService<S, C, R, E>`, `CampaignService<C, S, E>`, `ResponseService<R, S, C, E>` génériques sur les ports et l'`EventEmitter`, tous les temps en paramètre `now`.
  - SurveyService : `create(org, id, title, visibility)`, `update_draft(org, id, patch)`, `duplicate(org, id, new_id)` (clone un questionnaire ouvert ou fermé en brouillon, sans slug ni dates), `open(org, id, now, public_slug: Option<PublicSlug>)` (émet `survey.opened`, campagne chargée pour fusionner les métadonnées), `close(org, id, now)`, `delete(org, id)` (refuse si `has_responses`), `attach_campaign(org, id, Option<CampaignId>)` (brouillon seulement).
  - CampaignService : `create`, `update`, `delete` (refuse si des questionnaires y sont rattachés), `advance(org, id, now)` (une campagne, sous verrou ; `advance_due` a été supprimé après relecture) qui applique `advance` et émet `survey_campaign.started` puis `survey_campaign.ended` dans l'ordre, y compris le saut `Scheduled` vers `Ended` (lire `status()` avant `advance`).
  - ResponseService : `start_public(org, survey_id, id, resume_token, now)`, `start_member(org, survey_id, id, member_id, now)` (si une réponse en cours existe pour ce membre, la renvoie), `save(org, response_id | resume_token, answers, now)`, `complete(org, ..., now)`, `delete(org, id)`. Chaque opération charge le questionnaire ouvert et la campagne, appelle `accepts(campaign, now)`, applique le plafond (`count_completed >= response_cap` refuse à la complétion), l'unicité (membre : `find_completed_by_member` ; public : `find_completed_by_identifier` quand `unique_respondent`), puis émet `survey.response_submitted` après `save` et `survey.response_completed` après `complete`.
  - `SurveyError` reçoit les variants manquants (questionnaire introuvable, déjà complet, plafond atteint, déjà répondu, suppression refusée), avec `From<SurveyError> for CoreError` mis à jour.

- [ ] **Step 1: Tests (mocks, `mockall`)** nommés d'après les critères : `opening_a_survey_emits_survey_opened_with_campaign_metadata`, `saving_emits_response_submitted`, `completing_emits_response_completed_once`, `a_member_cannot_complete_twice_when_multiple_responses_are_off`, `a_member_can_complete_twice_when_multiple_responses_are_on`, `a_public_identifier_already_used_is_refused`, `a_response_after_the_window_is_refused`, `a_response_before_the_window_is_refused`, `the_response_cap_refuses_the_completion_beyond_it`, `deleting_a_survey_with_responses_is_refused`, `advance_due_emits_started_then_ended_on_a_skipped_step`, `advance_due_emits_ended_only_from_running`, `duplicate_creates_a_draft_without_slug_or_dates`. Run : `SQLX_OFFLINE=true cargo test -p mestier-core --lib survey::service` ; attendu : FAIL.
- [ ] **Step 2: Implémenter** les ports en plus, les trois services et les variants d'erreur, en imitant `domain/extra_period/service.rs` (émetteur passé au constructeur, `emit(org_id, &event)?`).
- [ ] **Step 3: Vérifier** : `SQLX_OFFLINE=true cargo test -p mestier-core --lib survey::` et `cargo clippy -p mestier-core --all-targets -- -D warnings` PASS.
- [ ] **Step 4: Commit** `feat(survey): add the survey, campaign and response services`.

### Task 3 : adaptateurs Postgres

**Files:**
- Create: `libs/core/src/infrastructure/survey/{mod.rs,postgres/{mod.rs,model.rs,survey.rs,campaign.rs,response.rs,tests.rs}}`
- Modify: `libs/core/src/infrastructure/registry.rs` (trois marqueurs `Survey`, `Campaign`, `SurveyResponse` dans `pub mod domain`), `libs/core/src/infrastructure/mod.rs`, `.sqlx/` (régénéré)

**Interfaces:**
- Produces : `PgSurveyRepository`, `PgCampaignRepository`, `PgResponseRepository`, chacun `#[repository(domain = X, backend = Postgres)]`, qui implémentent les ports de la tâche 2.

- [ ] **Step 1: Test d'intégration** (`#[ignore]`, `DATABASE_URL`, une base ou un schéma par test, commande `cargo test -p mestier-core -- --ignored survey`) : une fonction de contrat générique `fn survey_contract<R: SurveyRepository>` appelée par le test Postgres ; cas : insertion puis lecture de chaque état (`Draft`, `Opened`, `Closed`), `find_by_public_slug`, filtre par `org_id` (une organisation ne lit pas l'autre), `list_due`, unicité membre et identifiant violées sur deux compléments (la violation d'index unique devient `SurveyError` « déjà complète »), `ON DELETE SET NULL (member_id)`, `delete` d'un questionnaire sans réponse. Attendu : FAIL.
- [ ] **Step 2: Implémenter.** Les lignes sont reconstruites par désérialisation serde (JSON assemblé depuis les colonnes : `graph` frère et nul en brouillon, `window` imbriquée, `unique_respondent` fourni). `ResponseBody.started_at` ↔ `created_at`, écrit explicitement. `response_cap` : `i32` vers `u32` vers `NonZeroU32` par `try_from`. `updated_at` écrit par le dépôt. `survey.status` déduit de la variante.
- [ ] **Step 3: Régénérer** `.sqlx/` : `source .env && cargo sqlx prepare --workspace -- --all-targets`, puis `SQLX_OFFLINE=true cargo check --workspace`.
- [ ] **Step 4: Vérifier** : `cargo test -p mestier-core -- --ignored survey` PASS.
- [ ] **Step 5: Commit** `feat(survey): add the postgres adapters`.

### Task 4 : cas d'usage

**Files:**
- Create: `libs/core/src/application/survey/{mod.rs,tests.rs}`
- Modify: `libs/core/src/application/mod.rs` (`pub mod survey;`), `libs/core/src/lib.rs` (exports de commandes et de sorties)

**Interfaces:**
- Produces : méthodes `#[transactional(survey, survey_campaign, survey_response, authz, emitter)]` sur `MestierUseCase` : gestion (`create_survey`, `update_survey`, `open_survey`, `close_survey`, `duplicate_survey`, `delete_survey`, `list_surveys`, `get_survey`, `create_campaign`, `update_campaign`, `delete_campaign`, `list_campaigns`, `list_survey_responses`, `get_survey_response`, `delete_survey_response`, permission `survey.manage` ou `survey.view`), réponse membres (`start_member_response`, `save_member_response`, `complete_member_response` : l'acteur doit être membre de l'organisation du questionnaire, aucune permission survey), réponse publique (`open_public_survey(slug)`, `start_public_response`, `save_public_response`, `complete_public_response` : sans utilisateur). Les identifiants, le jeton de reprise et le slug sont générés ici (couche application). Le flag `surveys` est testé à l'entrée de chaque méthode de gestion et de réponse (`is_feature_enabled`), et la route publique d'un questionnaire dont l'organisation n'a pas le flag répond « introuvable ».
- À trancher en lisant le code : comment une méthode sans utilisateur s'appelle sur `MestierUseCase` (imiter l'entrée de déclenchement par webhook de l'automatisation, `libs/handlers-automation/src/webhook/`). Le noter dans le rapport.

- [ ] **Step 1: Tests** (`application/survey/tests.rs`, sur le modèle de `application/extra_period/tests.rs`) : un test par méthode de gestion pour l'autorisation (refus sans `survey.manage`), un pour le flag, un pour « membre d'une autre organisation refusé », un pour la route publique sans compte, un pour la route publique d'une organisation sans le flag. Attendu : FAIL.
- [ ] **Step 2: Implémenter** en imitant `application/extra_period/mod.rs`.
- [ ] **Step 3: Vérifier** : `SQLX_OFFLINE=true cargo test -p mestier-core --lib application::survey` PASS.
- [ ] **Step 4: Commit** `feat(survey): add the survey use cases`.

### Task 5 : horloge de campagne

**Files:**
- Create: `libs/core/src/application/survey/clock.rs`
- Modify: `libs/core/src/application/mod.rs` (démarrage), tests

**Interfaces:**
- Produces : `MestierUseCase::advance_survey_campaigns(&self, now) -> Result<usize, CoreError>` (une transaction par campagne, via `CampaignService::advance_due`), appelé à chaque tick du worker d'automatisation (`run_automation_worker`, `libs/core/src/application/mod.rs`, au même endroit que `ensure_recurrence_horizon_runs`). Décision à consigner dans la spec : tâche de fond ou déclencheur `Schedule` (question ouverte de la spec) tranchée pour **le tick du worker existant**, parce qu'il survit aux redémarrages et n'ajoute aucun processus.

- [ ] **Step 1: Test d'intégration** : une campagne dont la fenêtre vient de s'ouvrir émet `survey_campaign.started` une seule fois sur deux ticks ; une campagne dont la fenêtre est déjà passée émet `started` puis `ended` ; la fin émet `ended` ; rien ne se passe pour une campagne sans fenêtre. Attendu : FAIL.
- [ ] **Step 2: Implémenter**, vérifier, **Step 3: Commit** `feat(survey): advance campaigns on the automation tick`.

### Task 6 : scénarios Gherkin

**Files:**
- Create: `libs/core/tests/survey_bdd/{main.rs,world.rs,steps.rs,repository.rs}`, `libs/core/tests/features-survey/survey.feature`
- Modify: `libs/core/Cargo.toml` (cible de test `survey_bdd`, comme `deal_bdd`) : ligne à reporter à l'orchestrateur si le sous-agent ne doit pas la poser.

**Interfaces:**
- Consumes : les services de la tâche 2 avec des dépôts en mémoire (`Arc<Mutex<State>>`, comme `deal_bdd/repository.rs`) et un `RecordingEmitter`.

- [ ] **Step 1: Écrire le `.feature`** avec les scénarios de la spec taggés `@spec-survey-N` : 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 16, 17, 18, 19, 20, 24, 25, 26 (les scénarios 1, 2, 3, 15, 21, 22, 23 relèvent des routes et des photos : S2 et S3 ; 14 relève de S4 ; 27 à 33 sont couverts par les tests de domaine de S0). Une ligne de la spec sans scénario ici est listée dans le rapport avec son workstream.
- [ ] **Step 2: Implémenter les steps et le monde**, lancer : `SQLX_OFFLINE=true cargo test -p mestier-core --test survey_bdd` ; attendu : PASS, aucun scénario ignoré (`fail_on_skipped`).
- [ ] **Step 3: Commit** `test(survey): add the survey scenarios`.

---

## Self-review

- **Couverture de la spec (S1)** : permissions (1), cas d'usage et services avec événements, plafond, unicité, fenêtre, suppression, duplication (2 et 4), adaptateurs, isolation par organisation, contraintes d'unicité (3), horloge (5), acceptation par scénarios (6). Hors S1 et assumé : routes HTTP (S2, S3), actions d'automatisation et contrats d'événements côté webapp (S4), pages (S5, S6), photos (S3).
- **Pièges de S0 couverts** : `started` puis `ended` sur saut (tâches 2 et 5), `accepts` et non `status()` pour accepter une réponse (2), réhydratation par serde et mapping des colonnes (3), port manquant « réponse en cours d'un membre » (2), traduction de la violation d'index unique (3), ordre `save` puis événement (2).
- **Cohérence des noms** : `SurveyService`, `CampaignService`, `ResponseService`, `advance_due`, `find_by_public_slug`, `find_in_progress_by_member`, `list_due` employés à l'identique entre les tâches 2 à 5.
- **Proportion** : signatures, noms de tests et règles, sans corps de fonction.
