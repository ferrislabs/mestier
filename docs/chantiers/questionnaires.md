# Questionnaires

État : **spec validée le 2026-11-14. S0 intégré sur `feat/surveys` (voir Workstreams). S1 à S6 à planifier.**

Taille : **L**. Le découpage en workstreams est à valider avant tout lancement (voir Workstreams).

## Goal

Un utilisateur crée un questionnaire, l'ouvre à son audience (n'importe qui avec le lien, ou les membres de l'organisation), reçoit les réponses, et l'automatisation réagit aux événements émis (ouverture, réponse enregistrée, réponse complète, début et fin de campagne).

## Context

Deux usages réels :

1. **Intervenant de formation.** Un questionnaire envoyé à un intervenant pour qu'il renseigne des informations. Sa réponse déclenche la création de ressources métier. Aujourd'hui : Drag'n Survey, qui rappelle un webhook entrant.
2. **QCM de fin de session.** Les élèves ouvrent un lien, saisissent leur nom et répondent. Le formateur calcule une note.

Ce qui existe et qu'on réutilise :

- Le moteur d'automatisation et son déclencheur `Events` : chaque module décrit ses événements dans `libs/core/src/domain/<module>/events.rs`, assemblés par `event_catalogue()` (voir `docs/events.md`).
- Le connecteur SMTP et le catalogue d'actions (`libs/core/src/domain/automation/connector/`). L'envoi d'e-mail passe par là, jamais par le module questionnaire.
- Le déclencheur `Schedule` des workflows et le code à date de `application/task_recurrence/`.
- `libs/rate-limit`, pour protéger les routes sans authentification.

Ce qui n'existe pas : aucune route ni page ouverte sans compte (tout est sous `_app` côté webapp).

Décisions prises avec le demandeur (2026-11-14) :

1. Deux visibilités seulement : **publique** et **membres**.
2. En public, l'identité du répondant n'est pas gérée par le système. « Nom » et « prénom » sont des questions que l'auteur ajoute s'il le souhaite.
3. En membres, l'authentification est obligatoire et la réponse est rattachée au membre.
4. Un questionnaire est **indépendant** ou **dans une campagne**. Une campagne a une date de début et une date de fin.
5. La date de début d'une campagne est le moment à partir duquel les réponses sont acceptées. La date de fin est le moment après lequel elles sont refusées.
6. **On n'envoie pas un questionnaire, on l'ouvre.** L'envoi par e-mail est fait par une automatisation (bouton, récurrence, événement) via le connecteur SMTP.
7. **La notation est hors module.** Le module stocke les réponses fidèlement. Un workflow calcule la note à partir du payload de l'événement.
8. Les réponses vivent **uniquement dans le module**, dans un **schéma SQL `survey`**. Pas de réutilisation des tables du module Connaissances.
9. **Métadonnées privées.** Un questionnaire (et une campagne) porte un jeu de métadonnées clé-valeur que le répondant ne voit jamais. Elles sont recopiées dans les payloads d'événements pour servir de contexte à l'automatisation (session de formation, intervenant, etc.).
10. **Types de question** : texte, texte long, nombre, choix unique, choix multiple, date, date et heure, téléphone, e-mail, photo.
11. **Une même personne ne répond qu'une fois**, sauf si le questionnaire l'autorise : le questionnaire porte un réglage « réponses multiples » (refusées par défaut).

Choix de l'assistant (à corriger) :

- Une campagne regroupe un ou plusieurs questionnaires. Un questionnaire appartient à une campagne au plus.
- Une réponse est « en cours » dès son premier enregistrement, puis « complète » à la soumission finale.
- La définition des questions est un JSON versionné, figé à l'ouverture. Les réponses sont en JSONB.
- Un questionnaire de campagne accepte des réponses s'il est ouvert **et** que la campagne est dans sa fenêtre.
- Le répondant ne voit pas de note. Il voit un message de fin configuré par l'auteur.

## Glossary

Questionnaire : ensemble de questions soumis à une audience, affiché « questionnaire ». Rejeté : « sondage » et « formulaire », trop larges ; « survey » reste le nom du code (module, schéma, crate).
Campagne : regroupe des questionnaires et fixe la fenêtre pendant laquelle leurs réponses sont acceptées. Rejeté : « envoi ».
Ouvrir : rendre un questionnaire disponible à son audience. Rejeté : « envoyer », « publier ».
Visibilité : `PUBLIC` (n'importe qui avec le lien) ou `MEMBERS` (membres authentifiés de l'organisation).
Réponse : l'ensemble des réponses d'un répondant à un questionnaire. Rejeté : « soumission », réservé à l'acte d'enregistrer.
Fenêtre : intervalle début-fin d'une campagne pendant lequel les réponses sont acceptées.

## Invariants

- Toute requête est filtrée par `org_id`.  [test : une requête sans `org_id` ne compile pas avec `query!` ; le filtre est vérifié par test d'intégration]
- Une campagne dont la fin précède le début n'existe pas.  [type : constructeur `Window::new` renvoie `Result`]
- Un questionnaire n'a des questions modifiables qu'à l'état brouillon. Après ouverture la définition est figée.  [type : `Survey<Draft>` expose l'édition, pas `Survey<Opened>`]
- Une réponse complète ne change plus.  [type : état `Completed` sans méthode de modification]
- Une réponse n'est acceptée que si le questionnaire est ouvert et, s'il est dans une campagne, que l'instant est dans la fenêtre.  [test : garde dans le cas d'usage de soumission ; l'horloge est injectée]
- Une réponse ne peut pas être complète si une question obligatoire **du chemin parcouru** est sans réponse, ou si le chemin ne se termine pas sur un nœud `End`.  [test : validation de la réponse contre la définition figée]
- Un graphe n'a pas de cycle, a un seul nœud de départ, et chaque nœud mène à un `End`.  [type : `Graph::new` renvoie `Result`, la définition ne se construit pas autrement]
- Une condition ne référence qu'une question de choix posée sur tous les chemins qui y mènent, et une option qui existe.  [type : vérifié dans `Graph::new`]
- Une réponse hors du chemin parcouru n'est ni conservée ni envoyée dans un événement.  [test]
- Un téléphone et un e-mail enregistrés sont valides ; une date et une date et heure sont bien formées.  [type : `PhoneNumber`, `EmailAddress`, `Answer` construits par `TryFrom`]
- Si `allow_multiple_responses` est faux, un membre ne complète qu'une réponse par questionnaire.  [test d'intégration : contrainte unique `(survey_id, member_id)` sur les réponses complètes]
- Si `allow_multiple_responses` est faux, deux réponses publiques ne partagent pas la même valeur de la question « identifiant ».  [test d'intégration : contrainte unique sur la valeur normalisée]
- Une photo n'est acceptée que si c'est une image JPEG, PNG ou WebP dans la limite de taille ; elle est normalisée avant stockage.  [test]
- Les métadonnées ne sont jamais renvoyées par une route de réponse (publique ou membres).  [test]
- Un événement existe si et seulement si sa transaction a validé (`with_tx_emitting`).  [test : règle du projet, vérifiée pour chaque événement du lot]
- `survey.response_completed` part au plus une fois par réponse.  [test]
- Un questionnaire `MEMBERS` ne reçoit jamais de réponse d'un non-membre de son organisation.  [test d'intégration sur la route]
- Une route publique ne révèle rien de l'organisation au-delà du questionnaire ouvert (pas d'identifiant d'organisation, pas de liste).  [test]
- La visibilité d'un questionnaire ne change plus après sa première ouverture.  [type : visibilité absente de `Survey<Opened>` en écriture] — *choix de l'assistant, à confirmer*
- Les réponses ne sont écrites que dans le schéma `survey`.  [test : migration et revue, aucun autre schéma ne référence ses tables en écriture]

## Out of scope

- Envoi d'e-mail, de SMS ou de notification : tout passe par l'automatisation.
- Notation, points, bonnes réponses, corrigé : un workflow calcule.
- Affichage d'une note au répondant.
- Statistiques et tableaux de bord de résultats (la lecture brute des réponses suffit à ce lot).
- Export CSV ou PDF.
- Boucles dans le graphe (retour à une question précédente) et pages regroupant plusieurs questions. Le branchement conditionnel est dans le lot.
- Conditions sur autre chose que des questions de choix (nombre, texte, date) : à ajouter plus tard si besoin.
- Import depuis Drag'n Survey.
- Garantie forte d'unicité en mode public : sans compte, seul un identifiant déclaré par l'auteur (voir Unicité) ou un jeton navigateur permet de reconnaître un répondant.
- Métadonnées propres à chaque lien ou à chaque répondant (paramètres d'URL) : les métadonnées sont portées par le questionnaire et la campagne.
- Photos placées dans la photothèque de l'organisation : elles restent attachées à la réponse.
- Questionnaires envoyés à des destinataires connus avec lien personnel : écarté par le demandeur (deux typologies seulement).

## Design

### Domain (`libs/core/src/domain/survey/`)

Entités :

- `Survey` : id, `org_id`, titre, description, visibilité, définition (questions), message de fin, réglages (`allow_multiple_responses`, plafond de réponses optionnel), métadonnées privées, état (`Draft`, `Opened`, `Closed`), `campaign_id` optionnel, `opened_at` et `closed_at` (audit).
- `Question` (dans la définition JSON versionnée) : id stable, type (`TEXT`, `LONG_TEXT`, `NUMBER`, `SINGLE_CHOICE`, `MULTIPLE_CHOICE`, `DATE`, `DATETIME`, `PHONE`, `EMAIL`, `PHOTO`), libellé, obligatoire, options pour les choix, indicateur « identifiant » (voir Unicité), nombre maximal de photos pour `PHOTO`.
- `Graph` (la définition d'un questionnaire, JSONB versionné, voir Arbre de décision) : nœuds `Question`, `Condition` et `End`, et arêtes.
- `Campaign` : id, `org_id`, nom, `Window` optionnelle (début et fin chacun optionnel), métadonnées privées, état (`Scheduled`, `Running`, `Ended`). Les métadonnées d'un questionnaire de campagne se superposent à celles de la campagne (celles du questionnaire gagnent).
- `Response` : id, `org_id`, `survey_id`, `member_id` optionnel (rempli en mode membres), réponses (JSONB indexé par id de question), état (`InProgress`, `Completed`), jeton de reprise (mode public).

Valeurs : `Visibility`, `Window`, `QuestionId`, `Answer` (enum aligné sur les types de question).

Cas d'usage (`application/survey/`, méthodes `#[transactional]` sur `MestierUseCase`) :

- gestion : créer, modifier (brouillon), ouvrir, fermer, supprimer un questionnaire ; créer, modifier, supprimer une campagne ; rattacher un questionnaire à une campagne ;
- réponse : enregistrer une réponse (création au premier appel, mise à jour ensuite), compléter une réponse ;
- lecture : lister les réponses d'un questionnaire, lire une réponse ;
- horloge : avancer l'état des campagnes (`Scheduled` → `Running` → `Ended`) et émettre `survey_campaign.started` et `survey_campaign.ended`.

### Arbre de décision

La définition d'un questionnaire est un **graphe orienté sans cycle**, construit dans un canevas comme l'éditeur d'automatisation (`@xyflow/react`), avec un modèle propre au module : on ne réutilise pas le code de l'éditeur d'automatisation, couplé à ses connecteurs. *Modèle : choix de l'assistant, à confirmer.*

- **Nœuds** : `Question` (un des dix types, libellé, obligatoire, options, etc.), `Condition` et `End` (avec le message de fin). Un nœud de départ unique : la première question.
- **Condition** : compare la réponse d'une question de choix (`SINGLE_CHOICE` ou `MULTIPLE_CHOICE`) déjà posée sur **tous les chemins** qui mènent à la condition. Elle a une branche `Then` et une branche `Else`. Formes : « l'option X est cochée » ou « n'est pas cochée ».
- **Arêtes** : une seule forme, `from` / `to` / branche, comme dans l'automatisation. Une question a une seule sortie ; une condition en a deux.
- **Rejoindre** : plusieurs arêtes peuvent arriver sur le même nœud (une question commune après deux branches). Ce n'est donc pas un arbre strict. *Choix de l'assistant : sans cela, une question commune doit être copiée dans chaque branche.*
- **Chemin** : le chemin d'une réponse est calculé par le backend en suivant le graphe à partir des réponses. Une réponse donnée sur un chemin abandonné (le répondant est revenu en arrière et a changé une réponse de choix) est écartée à l'enregistrement.
- **Validation de la complétion** : seules les questions obligatoires **du chemin parcouru** doivent être renseignées, et le chemin doit se terminer sur un nœud `End`.
- Les payloads d'événements ne contiennent que les réponses du chemin parcouru, dans l'ordre du chemin.
- Le graphe fait partie de la définition figée à l'ouverture.

**Où se fait le calcul du chemin.** Le backend fait foi, à chaque enregistrement et à la complétion. Le navigateur doit aussi trouver la question suivante sans attendre le réseau, donc il suit le graphe lui aussi. Pour qu'il n'y ait pas deux vérités, un même jeu de cas de test (fichier JSON de graphes, réponses et chemins attendus) est exécuté par les tests Rust et par les tests vitest.

### Interface de gestion

- **Éditeur** : le canevas du graphe. Un panneau de détails édite la question ou la condition sélectionnée.
- **Ajout d'un nœud** : un sélecteur présente les dix types de question et le nœud `Condition` sous forme de cartes. Chaque carte a une **icône** du type, un **titre** et une **description** courte en français. Le catalogue (icône, titre, description) est une donnée du front, une seule source pour le sélecteur et le canevas.
- **Validation à l'édition** : le canevas signale un graphe invalide (nœud sans sortie, chemin sans `End`, condition sur une question non posée, cycle) avant l'ouverture.
- **Réponse** : le répondant voit **une question à la fois**, dans l'ordre du chemin, avec un bouton pour revenir en arrière.
- Réglages du questionnaire : visibilité, message de fin, réponses multiples, plafond de réponses, métadonnées, campagne de rattachement.

### Unicité

Le réglage `allow_multiple_responses` vaut faux par défaut.

- **Mode membres** : unicité par `(survey_id, member_id)`. Le membre reprend sa réponse en cours.
- **Mode public** : le système ne connaît pas le répondant. L'auteur peut marquer **une** question `EMAIL`, `PHONE` ou `TEXT` comme « identifiant » : la valeur normalisée est unique parmi les réponses complètes, et un second dépôt est refusé avec un message clair. Sans question identifiant, seul le jeton de reprise (navigateur) évite le doublon, ce qui n'est pas une garantie. *Choix de l'assistant, à confirmer.*

### Photos

Une réponse `PHOTO` stocke les clés d'objets, jamais des octets dans le JSONB. Le dépôt passe par une route du module (publique ou membres selon la visibilité) qui normalise l'image comme la photothèque (rotation EXIF, redimensionnement, retrait du GPS, JPEG) et écrit via le port `FileStorage` sous un préfixe propre au questionnaire. La logique de normalisation de `domain/photo_library/normalize.rs` doit être partagée plutôt que copiée : c'est sa deuxième utilisation. Un fichier dont la réponse n'est jamais complétée est orphelin : un nettoyage des fichiers de réponses abandonnées est à prévoir (S3).

### Événements

Ajoutés au catalogue (un `EventDescriptor` par événement, une ligne dans `emitted_events()`, le nom dans `EVENT_DOMAINS` du webapp, le contrat dans `event-contracts.ts`) :

| Événement | Libellé | Sujet | Quand |
|---|---|---|---|
| `survey.opened` | Questionnaire ouvert | `survey` | passage `Draft` → `Opened` |
| `survey.response_submitted` | Réponse enregistrée | `survey_response` | chaque enregistrement d'une réponse en cours |
| `survey.response_completed` | Réponse complète | `survey_response` | soumission finale ; le payload porte toutes les réponses |
| `survey_campaign.started` | Campagne démarrée | `survey_campaign` | la fenêtre s'ouvre |
| `survey_campaign.ended` | Campagne terminée | `survey_campaign` | la fenêtre se ferme |

`docs/events.md` passe de 46 à 51 événements.

**Identifiants portés par chaque événement :**

- Les trois événements de questionnaire (`survey.opened`, `survey.response_submitted`, `survey.response_completed`) portent `survey_id` et `campaign_id`. `campaign_id` est `null` pour un questionnaire indépendant, jamais absent du payload.
- Les deux événements de campagne portent `campaign_id`.
- Le contrat est figé en S0 et décrit dans `event-contracts.ts`, pour que le sélecteur de l'éditeur propose ces deux champs.

### Ports

- `SurveyRepository`, `CampaignRepository`, `ResponseRepository` : définis par le domaine, adaptateurs Postgres dans `infrastructure/survey/postgres/`, marqueurs dans `infrastructure/registry.rs` avec `#[repository(domain = X, backend = Postgres)]`. Choix de dispatch : générique statique, comme les autres dépôts.
- `Clock` : port d'horloge pour les gardes de fenêtre. *Vérifier s'il en existe déjà un avant d'en créer un.*

### Adaptateurs et routes

Crate `libs/handlers-survey`, découpée par agrégat (`survey/`, `campaign/`, `response/`), branchée dans `apps/api/src/router.rs` et `openapi.rs`.

- **Gestion** (authentifiée, permission dédiée `MANAGE_SURVEYS`, lecture `VIEW_SURVEYS`) : CRUD des questionnaires et campagnes, ouverture, fermeture, lecture des réponses.
- **Réponse membres** (authentifiée, membre de l'organisation) : lire un questionnaire ouvert, enregistrer, compléter.
- **Réponse publique** (sans authentification, limitée par `libs/rate-limit`) : lire un questionnaire ouvert par son lien, enregistrer, compléter avec le jeton de reprise. Le lien public contient un identifiant non devinable, jamais l'`org_id`.

Webapp :

- Modules dans `apps/webapp/src/modules/registry.ts` (menu de gestion), pages `src/pages/survey/` en séparation `feature/` et `ui/`.
- Page de réponse publique : une **route hors de `_app`**, la première du produit. Design à faire avec soin (mobile d'abord, les élèves répondent sur téléphone).
- Page de réponse membres : sous `_app`.

Automatisation : actions ajoutées au catalogue des connecteurs (`connector/actions/`), au minimum « ouvrir un questionnaire », « fermer un questionnaire », « lire le lien d'un questionnaire », « lister les réponses ». La liste définitive est fixée dans le workstream S4.

### Données

Migration `survey` : `CREATE SCHEMA survey`, tables `survey.survey`, `survey.campaign`, `survey.response`. Migrations réversibles, `.down.sql` écrit et testé. Index sur `(org_id, survey_id)` et sur le lien public.

Chaque nouveau `query!` demande une base vivante et `cargo sqlx prepare --workspace -- --all-targets`.

### Erreurs

Une énumération par frontière : `SurveyError` (domaine), erreurs de handler vers `CoreError`/type partagé de `libs/handlers`. Cas visibles côté répondant : questionnaire introuvable, fermé, hors fenêtre, réponse déjà complète, question obligatoire manquante, réponse invalide pour le type de la question.

## Acceptance (Gherkin)

```
@spec-survey-1  Scenario: un questionnaire public ouvert reçoit une réponse sans compte
@spec-survey-2  Scenario: un questionnaire membres refuse un répondant non authentifié
@spec-survey-3  Scenario: un questionnaire membres refuse un membre d'une autre organisation
@spec-survey-4  Scenario: ouvrir un questionnaire fige sa définition et émet survey.opened
@spec-survey-5  Scenario: enregistrer une réponse en cours émet survey.response_submitted
@spec-survey-6  Scenario: compléter une réponse émet survey.response_completed avec toutes les réponses, une seule fois
@spec-survey-7  Scenario: compléter sans réponse à une question obligatoire est refusé
@spec-survey-8  Scenario: une réponse complète ne peut plus être modifiée
@spec-survey-9  Scenario: un questionnaire de campagne refuse une réponse avant le début de la fenêtre
@spec-survey-10 Scenario: un questionnaire de campagne refuse une réponse après la fin de la fenêtre
@spec-survey-11 Scenario: l'horloge démarre la campagne à sa date de début et émet survey_campaign.started
@spec-survey-12 Scenario: l'horloge termine la campagne à sa date de fin et émet survey_campaign.ended
@spec-survey-13 Scenario: une campagne dont la fin précède le début est refusée
@spec-survey-14 Scenario: un workflow abonné à survey.response_completed s'exécute avec les réponses dans son payload
@spec-survey-15 Scenario: une route publique n'expose ni l'org_id ni la liste des questionnaires
@spec-survey-16 Scenario: le jeton de reprise permet de reprendre une réponse publique en cours
@spec-survey-17 Scenario: les réponses d'une organisation ne sont jamais visibles d'une autre
@spec-survey-18 Scenario: un membre ne peut compléter deux fois un questionnaire qui n'autorise pas les réponses multiples
@spec-survey-19 Scenario: un membre peut répondre plusieurs fois si le questionnaire l'autorise
@spec-survey-20 Scenario: une réponse publique avec un identifiant déjà utilisé est refusée
@spec-survey-21 Scenario: un e-mail, un téléphone ou une date invalides sont refusés à l'enregistrement
@spec-survey-22 Scenario: une photo est normalisée et stockée, et la réponse en porte la clé
@spec-survey-23 Scenario: un fichier qui n'est pas une image est refusé
@spec-survey-24 Scenario: les métadonnées du questionnaire et de la campagne sont dans le payload des événements et jamais dans une route de réponse
@spec-survey-25 Scenario: chaque événement de questionnaire porte survey_id et campaign_id, null pour un questionnaire indépendant
@spec-survey-26 Scenario: chaque événement de campagne porte campaign_id
@spec-survey-27 Scenario: la branche Then ou Else d'une condition décide de la question suivante
@spec-survey-28 Scenario: une question obligatoire hors du chemin parcouru n'empêche pas la complétion
@spec-survey-29 Scenario: une réponse donnée sur un chemin abandonné est écartée et absente du payload de survey.response_completed
@spec-survey-30 Scenario: un graphe avec un cycle, un nœud sans issue ou une condition sur une question non posée est refusé à l'édition
@spec-survey-31 Scenario: deux branches qui se rejoignent sur une question commune sont acceptées
@spec-survey-32 Scenario: le navigateur et le backend calculent le même chemin sur le jeu de cas partagé
@spec-survey-33 Scenario: un chemin qui ne se termine pas sur un nœud End empêche la complétion
```

Chaque ligne devient un test avant le code. Les scénarios 1 à 17 sont écrits en Gherkin quand le workstream S0 démarre.

## Simulation

Non pour la fenêtre : l'horloge est injectée, des tests unitaires à instants fixes suffisent (1 seconde avant le début, au début, à la fin, 1 seconde après). À reconsidérer pour la concurrence « deux compléments simultanés de la même réponse » : couvert par un test d'intégration sur la contrainte, pas par une simulation.

## Workstreams

À valider avant lancement (attente bloquante de `dev-methodology`).

| Id | Mission | Dépend de | Fichiers |
|---|---|---|---|
| S0 (**intégré**) | Migration `survey`, entités, états typés, ports, événements et descripteurs, glossaire | rien | `migrations/`, `libs/core/src/domain/survey/` |
| S1 | Dépôts Postgres, cas d'usage de gestion et de réponse, horloge de campagne | S0 | `libs/core/src/application/survey/`, `infrastructure/survey/` |
| S2 | Crate `handlers-survey` : gestion et réponse membres, permissions, OpenAPI | S1 | `libs/handlers-survey/`, `apps/api/src/{router,openapi}.rs` |
| S3 | Routes publiques, limitation de débit, jeton de reprise, dépôt de photos, nettoyage des fichiers abandonnés | S2 | `libs/handlers-survey/src/response/` |
| S4 | Actions d'automatisation, contrats d'événements côté webapp, `docs/events.md` | S1 | `connector/actions/`, `event-contracts.ts`, `event-search.ts` |
| S5 | Webapp de gestion : liste, éditeur de graphe (`@xyflow/react`) avec nœuds question et condition et sélecteur de type (cartes icône, titre, description), campagnes, lecture des réponses | S2 | `apps/webapp/src/pages/survey/` |
| S6 | Webapp de réponse : page publique hors `_app`, page membres, une question à la fois, suivi du chemin côté navigateur | S3 | `apps/webapp/src/routes/`, `pages/survey/` |

S0 d'abord seul. S1 puis S2 et S4 en parallèle. S3 après S2. S5 et S6 en parallèle côté webapp. Aucun fichier n'a deux auteurs.

## Frozen contracts

Figés à l'intégration de S0 (commit `c2b684c6` sur `feat/surveys`). Les types complets, avec leurs champs et variants, sont dans `libs/core/src/domain/survey/` : ces fichiers font foi. Aucun workstream ne modifie un type ou une signature ci-dessous. Un besoin nouveau passe par un ajout (méthode, variant, méthode de port) et est annoncé dans le rapport du workstream.

Exports à la racine de `mestier-core` (liste réelle, de `domain/mod.rs`) : `pub use survey::{ Answer, Answers, Branch as SurveyBranch, Campaign, CampaignId, CampaignStatus, Closed, Completed, ConditionNode as SurveyConditionNode, Draft, Edge as SurveyEdge, EmailAddress as SurveyEmailAddress, EndNode as SurveyEndNode, Graph as SurveyGraph, GraphPath, InProgress, Metadata, Node as SurveyNode, NodeRef as SurveyNodeRef, Opened, PhoneNumber as SurveyPhoneNumber, PublicSlug, QuestionId, QuestionKind, QuestionNode as SurveyQuestionNode, Response as SurveyResponse, ResponseId, ResponseRecord, ResumeToken, Survey, SurveyError, SurveyId, SurveyRecord, Visibility, Window as SurveyWindow, };`. Les noms trop génériques ou déjà pris sont préfixés : `SurveyGraph`, `SurveyWindow`, `SurveyResponse`, `SurveyNode`, `SurveyEdge`, `SurveyBranch`, `SurveyQuestionNode`, `SurveyConditionNode`, `SurveyEndNode`, `SurveyNodeRef`, `SurveyEmailAddress`, `SurveyPhoneNumber`. `domain` est `pub(crate)` : tout ce que S2 doit toucher passe par la racine.

Signatures publiques (extraites du code à l'intégration finale de S0) :

```text
// value.rs
pub enum Visibility
pub fn as_str(self) -> &'static str
pub struct QuestionId(pub Uuid);
pub enum QuestionKind
pub fn as_str(self) -> &'static str
pub struct EmailAddress(String);
pub fn as_str(&self) -> &str
pub struct PhoneNumber(String);
pub fn as_str(&self) -> &str
pub enum Answer
pub fn is_blank(&self) -> bool
pub struct PublicSlug(String);
pub fn new(value: String) -> Result<Self, SurveyError>
pub fn as_str(&self) -> &str

// error.rs
pub enum SurveyError

// metadata.rs
pub struct Metadata(BTreeMap<String, String>);
pub fn new(entries: BTreeMap<String, String>) -> Result<Self, SurveyError>
pub fn merged(campaign: &Metadata, survey: &Metadata) -> Metadata
pub fn get(&self, key: &str) -> Option<&str>
pub fn iter(&self) -> impl Iterator<Item = (&str, &str)>

// campaign.rs
pub struct Window
pub fn new( starts_at: Option<DateTime<Utc>>, ends_at: Option<DateTime<Utc>>, ) -> Result<Self, SurveyError>
pub fn starts_at(&self) -> Option<DateTime<Utc>>
pub fn ends_at(&self) -> Option<DateTime<Utc>>
pub fn contains(&self, now: DateTime<Utc>) -> bool
pub enum CampaignStatus
pub fn as_str(self) -> &'static str
pub struct Campaign
pub fn new( id: CampaignId, organization_id: OrganizationId, name: String, window: Window, metadata: Metadata, now: DateTime<Utc>, ) -> Result<Self, SurveyError>
pub fn id(&self) -> CampaignId
pub fn organization_id(&self) -> OrganizationId
pub fn name(&self) -> &str
pub fn window(&self) -> &Window
pub fn metadata(&self) -> &Metadata
pub fn status(&self) -> CampaignStatus
pub fn status_at(&self, now: DateTime<Utc>) -> CampaignStatus
pub fn advance(&mut self, now: DateTime<Utc>) -> Option<CampaignStatus>
pub fn rename(&mut self, name: String) -> Result<(), SurveyError>
pub fn reschedule(&mut self, window: Window, now: DateTime<Utc>) -> Option<CampaignStatus>
pub fn edit_metadata(&mut self, metadata: Metadata)

// graph.rs
pub type Answers = BTreeMap<QuestionId, Answer>;
pub struct QuestionNode
pub struct ConditionNode
pub struct EndNode
pub enum Node
pub fn id(&self) -> QuestionId
pub enum Branch
pub struct Edge
pub struct GraphPath
pub struct Graph
pub enum NodeRef<'a>
pub fn new(nodes: Vec<Node>, edges: Vec<Edge>, start: QuestionId) -> Result<Self, SurveyError>
pub fn path(&self, answers: &Answers) -> GraphPath
pub fn start(&self) -> QuestionId
pub fn nodes(&self) -> impl Iterator<Item = NodeRef<'_>>
pub fn question(&self, id: &QuestionId) -> Option<&QuestionNode>
pub fn edges(&self) -> impl Iterator<Item = Edge>

// aggregate.rs
pub struct Draft;
pub struct Opened;
pub struct Closed;
pub struct Survey<S: Stage>
pub fn id(&self) -> SurveyId
pub fn organization_id(&self) -> OrganizationId
pub fn title(&self) -> &str
pub fn description(&self) -> Option<&str>
pub fn visibility(&self) -> Visibility
pub fn allow_multiple_responses(&self) -> bool
pub fn response_cap(&self) -> Option<NonZeroU32>
pub fn metadata(&self) -> &Metadata
pub fn campaign_id(&self) -> Option<CampaignId>
pub fn graph(&self) -> Option<&Graph>
pub fn opened_at(&self) -> Option<DateTime<Utc>>
pub fn closed_at(&self) -> Option<DateTime<Utc>>
pub fn new( id: SurveyId, organization_id: OrganizationId, title: String, visibility: Visibility, ) -> Result<Self, SurveyError>
pub fn edit_title(&mut self, title: String) -> Result<(), SurveyError>
pub fn edit_description(&mut self, description: Option<String>)
pub fn edit_visibility(&mut self, visibility: Visibility)
pub fn edit_allow_multiple_responses(&mut self, allowed: bool)
pub fn edit_response_cap(&mut self, cap: Option<NonZeroU32>)
pub fn edit_metadata(&mut self, metadata: Metadata)
pub fn edit_campaign(&mut self, campaign_id: Option<CampaignId>)
pub fn edit_graph(&mut self, graph: Option<Graph>)
pub fn open( mut self, at: DateTime<Utc>, public_slug: Option<PublicSlug>, ) -> Result<Survey<Opened>, SurveyError>
pub fn definition(&self) -> &Graph
pub fn public_slug(&self) -> Option<&PublicSlug>
pub fn close(mut self, at: DateTime<Utc>) -> Result<Survey<Closed>, SurveyError>
pub fn accepts( &self, campaign: Option<&Campaign>, now: DateTime<Utc>, ) -> Result<(), SurveyError>
pub fn public_slug(&self) -> Option<&PublicSlug>
pub fn accepts( &self, _campaign: Option<&Campaign>, _now: DateTime<Utc>, ) -> Result<(), SurveyError>

// response.rs
pub struct InProgress;
pub struct Completed;
pub struct ResumeToken(String);
pub fn new(value: String) -> Result<Self, SurveyError>
pub fn as_str(&self) -> &str
pub struct Response<S>
pub fn id(&self) -> ResponseId
pub fn survey_id(&self) -> SurveyId
pub fn organization_id(&self) -> OrganizationId
pub fn member_id(&self) -> Option<MemberId>
pub fn resume_token(&self) -> Option<&ResumeToken>
pub fn answers(&self) -> &Answers
pub fn started_at(&self) -> DateTime<Utc>
pub fn completed_at(&self) -> Option<DateTime<Utc>>
pub fn start( id: ResponseId, survey: &Survey<Opened>, member_id: Option<MemberId>, resume_token: Option<ResumeToken>, at: DateTime<Utc>, ) -> Result<Self, SurveyError>
pub fn save( &mut self, survey: &Survey<Opened>, mut answers: Answers, ) -> Result<(), SurveyError>
pub fn complete( mut self, survey: &Survey<Opened>, at: DateTime<Utc>, ) -> Result<Response<Completed>, SurveyError>
pub fn identifier_value(&self) -> Option<&str>
pub fn unique_respondent(&self) -> bool

// record.rs
pub enum SurveyRecord
pub fn id(&self) -> SurveyId
pub fn organization_id(&self) -> OrganizationId
pub enum ResponseRecord
pub fn id(&self) -> ResponseId
pub fn survey_id(&self) -> SurveyId
pub fn organization_id(&self) -> OrganizationId

// ports.rs
pub trait SurveyRepository: Send
pub trait CampaignRepository: Send
pub trait ResponseRepository: Send

// events.rs
pub struct SurveyOpened<'a>
pub fn new( survey: &'a Survey<Opened>, campaign: Option<&Campaign>, ) -> Result<Self, SurveyError>
pub struct SurveyResponseSubmitted<'a>
pub fn new( survey: &'a Survey<Opened>, campaign: Option<&Campaign>, response: &'a Response<InProgress>, ) -> Result<Self, SurveyError>
pub struct SurveyResponseCompleted<'a>
pub fn new( survey: &'a Survey<Opened>, campaign: Option<&Campaign>, response: &'a Response<Completed>, ) -> Result<Self, SurveyError>
pub struct SurveyCampaignStarted
pub struct SurveyCampaignEnded
```

Payloads des cinq événements (exemples complets dans `survey/descriptors.rs`) :

```text
survey.opened                sujet survey          {survey_id, campaign_id|null, metadata, survey: {id, organization_id, title, description, visibility, allow_multiple_responses, response_cap|null, opened_at}}
survey.response_submitted    sujet survey_response {survey_id, campaign_id|null, metadata, response: {id, member_id|null, status}, answered_questions}
survey.response_completed    sujet survey_response {survey_id, campaign_id|null, survey: {id, title}, response: {id, member_id|null, completed_at, identifier_value|null}, metadata, answers: [{question_id, label, kind, value}]}
survey_campaign.started      sujet survey_campaign {campaign_id, campaign: {id, organization_id, name, starts_at|null, ends_at|null, metadata}}
survey_campaign.ended        sujet survey_campaign {campaign_id, campaign: {id, organization_id, name, starts_at|null, ends_at|null, metadata}}
```

Écarts acceptés par rapport au plan de S0 (décisions de l'orchestrateur) :

- `Graph::path` renvoie `GraphPath { questions, end }` et ne s'arrête jamais : une question sans réponse est traversée, une condition sur une source sans réponse prend `Else`.
- `Response::start`, `save` et `complete` prennent le `&Survey<Opened>` (et non le graphe) : ils refusent un autre questionnaire, imposent `member_id` pour `MEMBERS` et jeton de reprise pour `PUBLIC` (jamais les deux), et partagent une validation : question inconnue, type de réponse, option inconnue, photos au plus `max_photos` (5 par défaut), nombres finis. `save` écarte les réponses hors chemin et les réponses vides (texte vide, choix ou photos vides = non répondu). `complete` fixe `unique_respondent = !allow_multiple_responses`.
- `Campaign` a des champs privés, un `new` validé, un statut persisté et `advance(now)`/`reschedule` qui ne reculent jamais.
- `Survey<Draft>::open(at, public_slug)` : un questionnaire `PUBLIC` exige un `PublicSlug`, un `MEMBERS` en refuse un. `Survey<Opened>::accepts` refuse aussi une campagne qui n'est pas celle du questionnaire.
- Les trois événements de questionnaire se construisent depuis `&Survey<Opened>` et `Option<&Campaign>` : `campaign_id` et la fusion des métadonnées (campagne puis questionnaire, le questionnaire gagne) viennent de là. Le payload porte un `metadata` fusionné au premier niveau ; `survey.metadata` a disparu de `survey.opened`. Versions inchangées (1) : rien n'est publié.
- La migration porte des clés étrangères avec `org_id`, un `unique_respondent` et deux index uniques partiels : l'unicité « une réponse par personne » est tenue par la base, pas seulement par le cas d'usage.
- `Survey<Opened>::close` renvoie un `Result` et refuse une date antérieure à l'ouverture.
- `Graph::new` refuse aussi : plus d'une question « identifiant », un identifiant hors `TEXT`/`EMAIL`/`PHONE`, `max_photos` hors `PHOTO` ou nul, une question à choix sans option.
- `SurveyRecord` et `ResponseRecord` portent un questionnaire ou une réponse dans l'état où il est stocké. La conversion depuis et vers les lignes SQL est du ressort de S1.
- Les ports acceptent des ajouts de méthodes par S1. Les quatre recherches de `ResponseRepository` prennent `organization_id` en premier.
- La mise en page du canevas (positions des nœuds) n'est pas dans `Graph` : S5 la stocke dans un JSON de disposition séparé.
- Les réponses du jeu de cas partagé `survey/fixtures/paths.json` gardent l'étiquetage serde de `Answer` (`{"Choice":"Oui"}`) : S6 le lit tel quel.
- Aucun port d'horloge n'existe dans le domaine : S1 tranche pour l'horloge de campagne.

À savoir pour S1 (relecture finale de S0) :

1. **Horloge de campagne.** Lire `status()` avant `advance` ou `reschedule`. Si le statut était `Scheduled` et que l'appel renvoie `Some(Ended)`, émettre `survey_campaign.started` puis `survey_campaign.ended` dans la même transaction, sinon `started` est perdu.
2. **Statut persisté et `accepts` peuvent diverger.** `accepts` suit la fenêtre, le statut ne recule jamais. Ne jamais décider d'accepter une réponse d'après `status()`.
3. **Réhydratation par serde seulement** (`Survey`, `Campaign`, `Response` : champs privés, pas de constructeur depuis les colonnes). S1 construit la forme JSON : `graph` frère, nul en brouillon, `window` imbriquée, `unique_respondent` obligatoire.
4. **Colonnes.** `ResponseBody.started_at` correspond à `created_at` (l'écrire explicitement). `response_cap` : `i32` vers `u32` vers `NonZeroU32`. `survey.status` se déduit de la variante de `SurveyRecord`.
5. **Suppression d'un membre** met `member_id` à nul : les lignes restent lisibles mais aucune recherche ne peut les reprendre, et l'index unique par membre ne les couvre plus.
6. **Port manquant** : aucune recherche de la réponse en cours d'un membre (reprise en mode membres). S1 l'ajoute, avec `organization_id`, ou décide ce que « reprendre » veut dire pour un membre.
7. **Ordre.** Émettre `survey.response_submitted` juste après `save`. Traduire une violation d'index unique à la complétion en « réponse déjà complète ».
8. **Éditeur (S5).** `Graph::new` refuse un libellé de question vide : un brouillon ne s'enregistre pas tant qu'une question n'a pas de libellé.
9. **Événements (S4).** `event-contracts.ts` doit décrire les payloads actuels ; `docs/events.md` passe de 46 à 51 événements.

## Risks / open questions

Choix de l'assistant après relecture (à corriger) :

- **Dates d'un questionnaire** : `opened_at` et `closed_at` sont des horodatages d'audit. Seule une campagne a une fenêtre programmée. Un questionnaire indépendant s'ouvre et se ferme à la main ou par une action d'automatisation.
- **Cycle de vie** : `Draft` → `Opened` → `Closed`. Pas de réouverture. Pour corriger une faute dans un questionnaire ouvert, on le **duplique** en brouillon. Un questionnaire sans réponse peut être supprimé ; avec des réponses, il peut seulement être fermé.
- **Payload de `survey.response_completed`** : `response_id`, `survey_id`, titre du questionnaire, `member_id` (mode membres), les métadonnées fusionnées (campagne puis questionnaire), et pour chaque question son id, son libellé, son type, la réponse et, pour un choix, le libellé de l'option. Le payload est lisible sans la définition.
- **Abus du public** : limitation de débit, plus un plafond de réponses optionnel par questionnaire (valeur choisie par l'auteur). Pas de captcha dans ce lot. `survey.response_submitted` part aussi pour une réponse anonyme.
- **Données personnelles** : l'auteur peut supprimer une réponse. Pas de purge automatique dans ce lot. À rouvrir si les payloads d'événements doivent aussi être purgés.
- **Feature flag** : le module est derrière un flag par organisation (ADR 0017). Une route publique dont l'organisation n'a pas le flag répond comme un questionnaire introuvable, sans rien révéler.
- **Événement `survey.closed`** : absent du lot, non demandé.

Questions ouvertes :

| Question | Propriétaire |
|---|---|
| Les métadonnées sont-elles portées uniquement par le questionnaire et la campagne (mon interprétation), ou faut-il aussi un contexte propre à chaque lien ? | demandeur |
| Unicité en mode public : « question identifiant » choisie par l'auteur (ma proposition) convient-elle ? | demandeur |
| Plafond de photos par réponse et taille maximale par photo (je propose 5 photos et `file_storage.max_upload_bytes`) | demandeur |
| Quels fichiers de réponses abandonnés purger, et après combien de temps ? | S3 |
| Où vit le lien public (domaine, préfixe de route) et comment il est servi derrière la passerelle ? | S3 |
| Existe-t-il déjà un port d'horloge dans le domaine ? | S0 |
| L'horloge de campagne : tâche de fond dans le binaire `apps/api`, ou déclencheur `Schedule` d'un workflow ? Le premier est plus fiable, le second réutilise l'existant. | S1 |

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored        # intégration, DATABASE_URL requis
cd apps/webapp && pnpm check && pnpm test && pnpm build
```

Amended: 2026-11-14 les événements portent survey_id et campaign_id (null si questionnaire indépendant), demandé pour l'automatisation.
Amended: 2026-11-14 questions conditionnelles (affichage selon les cases cochées) et sélecteur de type en cartes ajoutés au lot ; `Condition` rejoint les contrats figés en S0.
Amended: 2026-11-14 la définition du questionnaire devient un graphe sans cycle (nœuds question, condition, fin) édité dans un canevas `@xyflow/react` propre au module ; remplace l'affichage conditionnel `visible_when`. Les branches peuvent se rejoindre (choix de l'assistant, à confirmer).
Amended: 2026-11-14 contrats figés régénérés après la relecture finale de S0 (réponse liée au questionnaire ouvert, campagne à statut persisté, métadonnées fusionnées dans les événements, clés étrangères avec org_id, index d unicité par répondant).
