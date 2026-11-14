# Questionnaires S0 : domaine, schéma, événements. Plan d'implémentation

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Poser le domaine `survey` (types, graphe, états typés, ports, événements) et le schéma SQL `survey`, sans cas d'usage ni HTTP.

**Architecture:** Un bounded context `survey` dans `libs/core/src/domain/survey/`, pur (aucun import d'infrastructure). Les invariants vivent dans des types (`Window`, `Graph`, `PhoneNumber`, `EmailAddress`, états typés). Les événements suivent le modèle de `domain/extra_period/events.rs`.

**Tech Stack:** Rust (workspace `mestier-core`), `serde`, `thiserror`, `chrono`, `uuid`, `mockall` (ports), `events` (crate du workspace), sqlx (migration seulement).

**Spec:** `docs/chantiers/questionnaires.md` (validée le 2026-11-14). Ce plan implémente le workstream S0. S1 à S6 auront chacun leur plan, écrit après le gel des contrats de la dernière tâche.

## Global Constraints

- Aucun commentaire dans le code, sauf réponse contraire à la question posée avant le premier code (skill `architecture`).
- Dispatch statique d'abord, pas de `async-trait`, pas de `dyn` sans raison (`rust-dev`).
- Ports : `#[cfg_attr(any(test, feature = "mock"), mockall::automock)]`, `&mut self`, `-> impl Future<Output = Result<_, CoreError>> + Send`, comme `domain/extra_period/ports.rs`.
- Le payload d'un événement est le modèle de domaine sérialisé, jamais la ligne SQL.
- Événements : `survey.opened`, `survey.response_submitted`, `survey.response_completed` (sujet `survey` ou `survey_response`), `survey_campaign.started`, `survey_campaign.ended`. Tous en version 1.
- Les trois événements de questionnaire portent `survey_id` et `campaign_id` (`null` si indépendant, champ toujours présent). Les deux de campagne portent `campaign_id`.
- Schéma SQL `survey`. Migration réversible, `.down.sql` testé.
- Dix types de question : `TEXT`, `LONG_TEXT`, `NUMBER`, `SINGLE_CHOICE`, `MULTIPLE_CHOICE`, `DATE`, `DATETIME`, `PHONE`, `EMAIL`, `PHOTO`.
- Nœuds du graphe : `Question`, `Condition`, `End`. Condition : branches `Then` et `Else`, uniquement sur une question de choix.
- Vocabulaire : « questionnaire » (code `survey`), « campagne », « ouvrir », « réponse », « fenêtre ». Jamais « sondage », « formulaire », « envoyer ».

## Review Focus

- Un graphe vide, ou sans nœud `End` : refusé, jamais un panic. Test dans la tâche 3.
- Deux nœuds de départ, ou un nœud orphelin inaccessible depuis le départ : refusé. Tâche 3.
- Une condition sur une option qui n'existe pas dans la question référencée : refusée. Tâche 3.
- Une fenêtre avec seulement un début, seulement une fin, ou début égal à la fin : comportement défini. Tâche 4.
- Un e-mail avec espaces ou majuscules : normalisé (minuscules, espaces retirés), pour que l'unicité sur l'identifiant tienne. Tâche 2.
- Deux clés identiques dans les métadonnées de la campagne et du questionnaire : celle du questionnaire gagne. Tâche 4.

---

## Orchestration

Un seul workstream, donc pas de parallélisme à ce stade. Les tâches 1 à 5 vont à un sous-agent `ferrislabs:ferris-implementer` (modèle `sonnet`), une tâche à la fois, dans l'ordre. L'orchestrateur fait la tâche 6 (convergence) et la tâche 7 (gel des contrats), puis relit le diff avant intégration.

**Fichiers du workstream (écrits par le sous-agent) :**
`migrations/20261122000001_create_survey_schema.{up,down}.sql`, `docs/adr/0018-surveys-have-their-own-schema-and-graph-definition.md`, `libs/core/src/domain/survey/**`.

**Points de convergence (écrits par l'orchestrateur) :** `libs/core/src/domain/mod.rs`, `libs/core/src/domain/automation/catalogue.rs`, `libs/core/src/domain/feature_flag/mod.rs`, `docs/glossary.md`, `docs/chantiers/questionnaires.md`.

**Artefacts régénérés à l'intégration :** `Cargo.lock` (par cargo, jamais à la main).

**Environnement :** `DATABASE_URL` est défini (vérifié dans `.env`). La migration est appliquée sur cette base.

---

### Task 1 : ADR, schéma SQL, glossaire

**Files:**
- Create: `docs/adr/0018-surveys-have-their-own-schema-and-graph-definition.md`
- Create: `migrations/20261122000001_create_survey_schema.up.sql`
- Create: `migrations/20261122000001_create_survey_schema.down.sql`
- Modify (orchestrateur) : `docs/glossary.md`

**Interfaces:**
- Produces: tables `survey.campaign`, `survey.survey`, `survey.response`, que les dépôts de S1 liront.

- [ ] **Step 1: Écrire l'ADR 0018** au gabarit `spec-driven/references/adr-template.md`. Décision : réponses dans un schéma `survey` dédié ; définition du questionnaire = un JSONB `graph`, entier, jamais normalisé (comme `automation.workflow_version`). Alternatives rejetées : tables du module Connaissances (usage de persistance LLM ou humain, pas de stockage de réponses) ; nœuds et arêtes normalisés en tables (le graphe est figé à l'ouverture et toujours lu en entier). Statut `proposed`.

- [ ] **Step 2: Écrire `…create_survey_schema.up.sql`.** `CREATE SCHEMA survey`. Tables, toutes avec `org_id UUID NOT NULL REFERENCES organizations(id)` :
  - `survey.campaign(id, org_id, name, starts_at TIMESTAMPTZ NULL, ends_at TIMESTAMPTZ NULL, status TEXT, metadata JSONB NOT NULL DEFAULT '{}', created_at, updated_at)`, `CHECK (starts_at IS NULL OR ends_at IS NULL OR starts_at < ends_at)`, `CHECK (status IN ('SCHEDULED','RUNNING','ENDED'))`.
  - `survey.survey(id, org_id, campaign_id NULL REFERENCES survey.campaign, title, description, visibility TEXT CHECK IN ('PUBLIC','MEMBERS'), graph JSONB NOT NULL, closing_message, allow_multiple_responses BOOLEAN NOT NULL DEFAULT false, response_cap INTEGER NULL CHECK (response_cap > 0), metadata JSONB NOT NULL DEFAULT '{}', status TEXT CHECK IN ('DRAFT','OPENED','CLOSED'), public_slug TEXT NULL UNIQUE, opened_at, closed_at, created_at, updated_at)`.
  - `survey.response(id, org_id, survey_id REFERENCES survey.survey, member_id NULL REFERENCES members(id), answers JSONB NOT NULL DEFAULT '{}', status TEXT CHECK IN ('IN_PROGRESS','COMPLETED'), resume_token TEXT NULL UNIQUE, identifier_value TEXT NULL, completed_at, created_at, updated_at)`.
  - Contraintes d'unicité partielles : `UNIQUE (survey_id, member_id) WHERE status = 'COMPLETED' AND member_id IS NOT NULL` n'est pas posée ici, car elle dépend du réglage `allow_multiple_responses` : S1 la fait respecter dans le cas d'usage et pose un index d'appui. Index : `(org_id, survey_id)` sur `survey.response`, `(org_id, campaign_id)` sur `survey.survey`.
  - Vérifier les vrais noms des tables `organizations` et `members` avant d'écrire les clés étrangères.

- [ ] **Step 3: Écrire `.down.sql`** : `DROP SCHEMA survey CASCADE`.

- [ ] **Step 4: Tester la migration.**
  Run: `source .env && sqlx migrate run --source migrations && sqlx migrate revert --source migrations && sqlx migrate run --source migrations`
  Expected: les trois commandes réussissent, sans erreur.

- [ ] **Step 5: Commit** `feat(survey): add the survey schema and ADR 0018`.

### Task 2 : types valeur

**Files:**
- Create: `libs/core/src/domain/survey/{mod.rs,value.rs,tests.rs}`

**Interfaces:**
- Produces (dans `value.rs`) :
  - `pub enum Visibility { Public, Members }` (`serde` `SCREAMING_SNAKE_CASE`, `as_str`).
  - `pub struct QuestionId(pub Uuid)` (comme `ExtraPeriodId`).
  - `pub struct EmailAddress(String)` : `TryFrom<&str>`, normalise (trim, minuscules), `as_str`.
  - `pub struct PhoneNumber(String)` : `TryFrom<&str>`, accepte chiffres, espaces, `+`, tirets, parenthèses, normalise en `+` et chiffres, 6 à 15 chiffres.
  - `pub enum QuestionKind` (les dix types).
  - `pub enum Answer { Text(String), Number(f64), Choice(String), Choices(Vec<String>), Date(NaiveDate), DateTime(DateTime<Utc>), Phone(PhoneNumber), Email(EmailAddress), Photos(Vec<String>) }`.
  - `pub enum SurveyError` (`thiserror`), variantes ajoutées au fil des tâches.
- Consumes: rien.

- [ ] **Step 1: Écrire les tests qui échouent** dans `tests.rs` : `an_email_is_trimmed_and_lowercased`, `an_email_without_an_at_sign_is_refused`, `a_phone_keeps_only_a_plus_and_digits`, `a_phone_with_five_digits_is_refused`, `a_phone_with_letters_is_refused`, plus un test `proptest` `an_email_normalization_is_idempotent` (normaliser deux fois égale normaliser une fois).
- [ ] **Step 2: Lancer et vérifier l'échec.**
  Run: `cargo test -p mestier-core survey::tests::`
  Expected: FAIL (types non définis).
- [ ] **Step 3: Implémenter** `value.rs` et déclarer les modules dans `mod.rs` (`pub mod value;`, `#[cfg(test)] mod tests;`). Dériver `Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema` comme `ExtraPeriodId`. Un `Answer` est validé à sa construction : pas de variante qui accepte une chaîne brute pour un e-mail.
- [ ] **Step 4: Lancer et vérifier le succès.**
  Run: `cargo test -p mestier-core survey::tests:: && cargo clippy -p mestier-core --all-targets -- -D warnings`
  Expected: PASS, aucun warning.
- [ ] **Step 5: Commit** `feat(survey): add the value types`.

### Task 3 : le graphe

**Files:**
- Create: `libs/core/src/domain/survey/graph.rs`, `libs/core/src/domain/survey/fixtures/paths.json`
- Modify: `libs/core/src/domain/survey/{mod.rs,tests.rs}`

**Interfaces:**
- Consumes: `QuestionId`, `QuestionKind`, `Answer`, `SurveyError`.
- Produces :
  - `pub struct Graph` (champs privés, `Serialize`/`Deserialize` via une forme brute `GraphDraft`).
  - `pub enum Node { Question(QuestionNode), Condition(ConditionNode), End(EndNode) }`.
  - `pub struct QuestionNode { pub id: QuestionId, pub kind: QuestionKind, pub label: String, pub required: bool, pub options: Vec<String>, pub max_photos: Option<u8>, pub identifier: bool }`.
  - `pub struct ConditionNode { pub id: QuestionId, pub question: QuestionId, pub option: String, pub checked: bool }`.
  - `pub struct EndNode { pub id: QuestionId, pub message: Option<String> }`.
  - `pub enum Branch { Next, Then, Else }`, `pub struct Edge { pub from: QuestionId, pub to: QuestionId, pub branch: Branch }`.
  - `Graph::new(nodes: Vec<Node>, edges: Vec<Edge>, start: QuestionId) -> Result<Graph, SurveyError>`.
  - `Graph::path(&self, answers: &Answers) -> Result<Vec<QuestionId>, SurveyError>` : suit le graphe depuis `start`, retourne les questions visitées dans l'ordre. `Answers` est un alias de `BTreeMap<QuestionId, Answer>`.

- [ ] **Step 1: Écrire les tests qui échouent** (un par règle de la spec) : `a_graph_without_nodes_is_refused`, `a_graph_without_an_end_is_refused`, `a_cycle_is_refused`, `a_node_with_no_way_to_an_end_is_refused`, `a_node_unreachable_from_the_start_is_refused`, `two_starts_are_impossible_because_start_is_explicit_and_orphans_are_refused`, `a_condition_on_an_unknown_option_is_refused`, `a_condition_on_a_text_question_is_refused`, `a_condition_on_a_question_not_asked_on_every_path_is_refused`, `two_branches_may_rejoin_on_a_common_question`, `the_then_branch_is_followed_when_the_option_is_checked`, `the_else_branch_is_followed_otherwise`.
- [ ] **Step 2: Lancer et vérifier l'échec.**
  Run: `cargo test -p mestier-core survey::tests::graph`
  Expected: FAIL.
- [ ] **Step 3: Implémenter `Graph::new`** : détection de cycle (tri topologique ou DFS à trois couleurs), accessibilité depuis `start`, chemin vers un `End` depuis chaque nœud (parcours inverse depuis les `End`), et pour chaque condition l'ensemble des questions posées sur tous les chemins qui y mènent (intersection des ensembles des prédécesseurs, calculée dans l'ordre topologique). `Question` a une sortie `Next`, `Condition` a exactement `Then` et `Else`, `End` n'en a aucune.
- [ ] **Step 4: Écrire le jeu de cas partagé** `fixtures/paths.json` : liste de `{ name, graph, answers, expected_path }` couvrant au moins : chemin linéaire, branche `Then`, branche `Else`, branches qui se rejoignent, choix multiple coché et non coché, réponse manquante sur une question obligatoire (chemin s'arrête à cette question). Un test Rust `the_shared_path_cases_hold` le charge avec `include_str!` et vérifie `path`. Le webapp le lira à S6.
- [ ] **Step 5: Lancer et vérifier le succès.**
  Run: `cargo test -p mestier-core survey::tests:: && cargo clippy -p mestier-core --all-targets -- -D warnings`
  Expected: PASS.
- [ ] **Step 6: Commit** `feat(survey): add the decision graph and its path`.

### Task 4 : états typés, fenêtre, métadonnées

**Files:**
- Create: `libs/core/src/domain/survey/{survey.rs,campaign.rs,response.rs,metadata.rs}`
- Modify: `libs/core/src/domain/survey/{mod.rs,tests.rs}`

**Interfaces:**
- Consumes: `Graph`, `Visibility`, `Answer`, `SurveyError`, `OrganizationId`, `MemberId` (de `crate`).
- Produces :
  - `pub struct SurveyId(pub Uuid)`, `CampaignId`, `ResponseId` (même forme que `ExtraPeriodId`).
  - `pub struct Metadata(BTreeMap<String, String>)` : `Metadata::merged(campaign: &Metadata, survey: &Metadata) -> Metadata` (clés du questionnaire prioritaires), `get`, `iter`.
  - `pub struct Window { starts_at: Option<DateTime<Utc>>, ends_at: Option<DateTime<Utc>> }` : `Window::new(starts_at, ends_at) -> Result<Window, SurveyError>` (refuse début ≥ fin quand les deux existent), `Window::contains(&self, now: DateTime<Utc>) -> bool` (début inclus, fin exclue).
  - `pub enum CampaignStatus { Scheduled, Running, Ended }` et `Campaign::status_at(&self, now) -> CampaignStatus`.
  - `pub struct Survey<S>` avec `Draft`, `Opened`, `Closed` : `Survey<Draft>::edit_*`, `Survey<Draft>::open(self, at) -> Result<Survey<Opened>, SurveyError>` (refuse un graphe absent), `Survey<Opened>::close(self, at) -> Survey<Closed>`, `Survey<Opened>::accepts(&self, campaign: Option<&Campaign>, now) -> Result<(), SurveyError>` (ouvert et, si campagne, dans la fenêtre). Pas de méthode d'édition sur `Opened` ni `Closed`, pas de réouverture.
  - `pub struct Response<S>` avec `InProgress`, `Completed` : `Response<InProgress>::save(&mut self, answers)`, `Response<InProgress>::complete(self, graph: &Graph, at) -> Result<Response<Completed>, SurveyError>` (valide les questions obligatoires du chemin, écarte les réponses hors chemin). `Response<Completed>` n'expose aucune modification.

- [ ] **Step 1: Écrire les tests qui échouent** : `a_window_with_an_end_before_its_start_is_refused`, `a_window_with_only_a_start_is_open_forever_after`, `a_window_with_only_an_end_is_open_until_then`, `a_window_includes_its_start_and_excludes_its_end`, `a_window_starting_when_it_ends_is_refused`, `campaign_status_follows_the_clock`, `merged_metadata_lets_the_survey_win`, `a_draft_cannot_open_without_a_graph`, `a_survey_in_a_running_campaign_accepts_a_response`, `a_survey_before_its_campaign_window_refuses`, `a_survey_after_its_campaign_window_refuses`, `a_closed_survey_refuses`, `a_response_cannot_complete_with_a_required_question_on_its_path_unanswered`, `a_response_ignores_a_required_question_off_its_path`, `completion_discards_answers_off_the_path`, `a_completed_response_has_no_way_to_change` (test de compilation : un `compile_fail` en doctest ou l'absence de méthode vérifiée à la revue).
- [ ] **Step 2: Lancer et vérifier l'échec.**
  Run: `cargo test -p mestier-core survey::tests::`
  Expected: FAIL.
- [ ] **Step 3: Implémenter** les types ci-dessus. `now` est toujours un paramètre : le domaine n'appelle jamais `Utc::now()`.
- [ ] **Step 4: Lancer et vérifier le succès.**
  Run: `cargo test -p mestier-core survey::tests:: && cargo clippy -p mestier-core --all-targets -- -D warnings`
  Expected: PASS.
- [ ] **Step 5: Commit** `feat(survey): add the typed states, the window and the metadata`.

### Task 5 : ports et événements

**Files:**
- Create: `libs/core/src/domain/survey/{ports.rs,events.rs,error.rs}`
- Modify: `libs/core/src/domain/survey/{mod.rs,tests.rs}`

**Interfaces:**
- Consumes: tous les types des tâches 2 à 4, `events::{DomainEvent, EventDescriptor, EventSubject}`, `common::CoreError`.
- Produces :
  - `SurveyRepository`, `CampaignRepository`, `ResponseRepository` (traits `Send`, méthodes `find`, `insert`, `update`, `list`, plus `ResponseRepository::find_by_resume_token`, `count_completed(survey_id)`, `find_completed_by_member(survey_id, member_id)`, `find_completed_by_identifier(survey_id, value)`).
  - Cinq structs d'événement : `SurveyOpened { survey, campaign_id }`, `SurveyResponseSubmitted { response, survey, campaign_id }`, `SurveyResponseCompleted { response, survey, campaign_id, metadata, path_answers }`, `SurveyCampaignStarted { campaign }`, `SurveyCampaignEnded { campaign }`, chacune `impl DomainEvent`.
  - `pub fn descriptors() -> Vec<EventDescriptor>` et `#[cfg(test)] pub fn emitted_events() -> Vec<(&'static str, u16)>`.
- Un `Clock` : vérifier d'abord s'il en existe un dans `libs/common` ou `libs/core` (`grep -rn "trait Clock"`). S'il n'existe pas, ne pas en créer en S0 : le domaine prend `now` en paramètre, et S1 décide de l'horloge de campagne (question ouverte de la spec).

- [ ] **Step 1: Écrire les tests qui échouent** : `survey_opened_payload_is_locked` (JSON exact, avec `survey_id` et `campaign_id` à `null`), `survey_opened_payload_carries_the_campaign_id`, `response_completed_payload_lists_only_the_path_in_order_with_labels`, `response_completed_payload_carries_merged_metadata`, `campaign_started_payload_carries_campaign_id`, `every_emitted_event_has_a_descriptor` (compare `emitted_events` à `descriptors`, sur le modèle de `extra_period`), et pour chaque descripteur : le `payload_example` est un JSON valide aux champs du payload réel.
- [ ] **Step 2: Lancer et vérifier l'échec.**
  Run: `cargo test -p mestier-core survey::tests::events`
  Expected: FAIL.
- [ ] **Step 3: Implémenter** `ports.rs` (même forme que `extra_period/ports.rs`), `events.rs` (même forme que `extra_period/events.rs`, payload = modèle de domaine sérialisé), `error.rs` (`SurveyError` migré ici si `value.rs` l'a défini, avec `From<SurveyError> for CoreError`). Le payload de `survey.response_completed` contient `response_id`, `survey_id`, `campaign_id`, `title`, `member_id`, `metadata`, et `answers`: une entrée par question du chemin, dans l'ordre, avec `id`, `label`, `kind`, `value` et, pour un choix, le libellé de l'option.
- [ ] **Step 4: Lancer et vérifier le succès.**
  Run: `cargo fmt --check && cargo clippy -p mestier-core --all-targets -- -D warnings && cargo test -p mestier-core survey::`
  Expected: tout passe.
- [ ] **Step 5: Commit** `feat(survey): add the ports and the five events`.

### Task 6 : convergence (orchestrateur)

**Files:**
- Modify: `libs/core/src/domain/mod.rs`, `libs/core/src/domain/automation/catalogue.rs`, `libs/core/src/domain/feature_flag/mod.rs`, `docs/glossary.md`

- [ ] **Step 1:** ajouter `pub(crate) mod survey;` et le `pub use survey::*;` utile dans `domain/mod.rs`, selon le modèle des autres modules.
- [ ] **Step 2:** ajouter `survey::events::descriptors()` à `descriptors()` et `survey::events::emitted_events()` au test `the_catalogue_describes_exactly_what_the_modules_emit`.
- [ ] **Step 3:** ajouter la variante `FeatureFlag::Surveys` (clé `surveys`, libellé « Questionnaires »), l'ajouter à `ALL`, `key`, `label` et `description`, et les tests existants de `feature_flag/mod.rs` doivent passer.
- [ ] **Step 4:** ajouter à `docs/glossary.md` les entrées Questionnaire, Campagne, Ouvrir, Visibilité, Réponse, Fenêtre (texte de la spec).
- [ ] **Step 5: Vérifier.**
  Run: `cargo fmt --all --check && SQLX_OFFLINE=true cargo check --workspace && cargo clippy -p mestier-core --all-targets -- -D warnings && cargo test -p mestier-core`
  Expected: PASS. Le test du catalogue vérifie les 5 nouveaux événements.
- [ ] **Step 6: Commit** `feat(survey): register the module, its events and its flag`.

### Task 7 : gel des contrats (orchestrateur)

- [ ] **Step 1:** copier verbatim dans `docs/chantiers/questionnaires.md`, section « Frozen contracts », les signatures publiques de `Visibility`, `Window`, `QuestionId`, `Answer`, `Graph` (`new`, `path`), `Survey<S>`, `Response<S>`, `Metadata`, les trois traits de dépôt et les 5 payloads (exemples des descripteurs).
- [ ] **Step 2:** passer le statut de S0 à « integrated » dans le tableau des workstreams et noter les écarts éventuels par rapport à la spec (décision explicite : accepter et consigner, ou réexpédier avec un briefing corrigé).
- [ ] **Step 3: Commit** `docs(survey): freeze the S0 contracts`.

---

## Self-review

- **Couverture de la spec (S0)** : schéma (tâche 1), types `Visibility`, `Window`, `QuestionId`, `Answer`, `Graph`, états typés (tâches 2 à 4), ports et événements avec `survey_id` et `campaign_id` (tâche 5), glossaire et flag (tâche 6), gel des contrats (tâche 7). L'ADR est ajouté pour le schéma séparé et la définition JSONB. Hors S0 et assumé : cas d'usage, HTTP, horloge, scénarios Gherkin (S1, car ils passent par les cas d'usage).
- **Acceptation couverte en S0 par des tests unitaires** : 4, 6 (payload), 8, 13, 25, 26, 27, 28, 29, 30, 31, 33. Les autres scénarios arrivent avec S1 à S6.
- **Cohérence des noms** : `Graph::new`, `Graph::path`, `Window::new`, `Window::contains`, `Survey<Draft|Opened|Closed>`, `Response<InProgress|Completed>`, `Metadata::merged` sont employés à l'identique dans les tâches 3 à 5.
- **Proportion** : le plan donne signatures, noms de tests et valeurs de la spec, sans corps de fonction.
