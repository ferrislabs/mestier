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

- Une campagne regroupe un ou plusieurs questionnaires. Un questionnaire appartient à zéro, une ou plusieurs campagnes (amendé en C1).
- Une réponse est « en cours » dès son premier enregistrement, puis « complète » à la soumission finale.
- La définition des questions est un JSON versionné, modifiable après l'ouverture (amendé en C1, voir Décisions C1). Les réponses sont en JSONB.
- Un questionnaire de campagne accepte des réponses s'il est ouvert **et** que l'une au moins de ses campagnes est dans sa fenêtre.
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
- Un questionnaire reste modifiable dans tous ses états (titre, description, graphe, réglages, métadonnées, campagnes). Un questionnaire ouvert ou fermé a toujours un graphe valide.  [type : les éditeurs sont communs à `Survey<S: Stage>` ; hors brouillon, `edit_graph` prend un `Graph` et non un `Option<Graph>`]
- Un graphe modifié alors que le questionnaire a une réponse complète sur sa version courante conserve la version précédente et incrémente `graph_version` ; sinon il est remplacé sur place.  [test : service et intégration Postgres]
- Une réponse complète ne change plus.  [type : état `Completed` sans méthode de modification]
- Une réponse n'est acceptée que si le questionnaire est ouvert et, s'il est dans des campagnes, que l'instant est dans la fenêtre d'au moins une.  [test : garde dans le cas d'usage de soumission ; l'horloge est injectée]
- Une réponse ne peut pas être complète si une question obligatoire **du chemin parcouru** est sans réponse, ou si le chemin ne se termine pas sur un nœud `End`.  [test : validation de la réponse contre le graphe courant]
- Une réponse complète porte la version du graphe sur laquelle elle a été complétée, et ce graphe reste lisible.  [test : `graph_version` requis par une contrainte quand la réponse est complète ; `get_survey_graph_version`]
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
- La visibilité et le lien public d'un questionnaire ne changent plus après sa première ouverture.  [type : `edit_visibility` n'existe que sur `Survey<Draft>` ; le service refuse une autre visibilité avec `survey_visibility_frozen`, 409]
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

- `Survey` : id, `org_id`, titre, description, visibilité, définition (questions) et sa version `graph_version`, message de fin, réglages (`allow_multiple_responses`, plafond de réponses optionnel), métadonnées privées, état (`Draft`, `Opened`, `Closed`), `campaign_ids` (ordre de rattachement, sans doublon), `opened_at` et `closed_at` (audit).
- `Question` (dans la définition JSON versionnée) : id stable, type (`TEXT`, `LONG_TEXT`, `NUMBER`, `SINGLE_CHOICE`, `MULTIPLE_CHOICE`, `DATE`, `DATETIME`, `PHONE`, `EMAIL`, `PHOTO`), libellé, obligatoire, options pour les choix, indicateur « identifiant » (voir Unicité), nombre maximal de photos pour `PHOTO`.
- `Graph` (la définition d'un questionnaire, JSONB versionné, voir Arbre de décision) : nœuds `Question`, `Condition` et `End`, et arêtes.
- `Campaign` : id, `org_id`, nom, `Window` optionnelle (début et fin chacun optionnel), métadonnées privées, état (`Scheduled`, `Running`, `Ended`). Les métadonnées d'un questionnaire se superposent à celles de ses campagnes, prises dans l'ordre de rattachement (une campagne plus tardive gagne, le questionnaire gagne sur toutes).
- `Response` : id, `org_id`, `survey_id`, `member_id` optionnel (rempli en mode membres), réponses (JSONB indexé par id de question), état (`InProgress`, `Completed`), jeton de reprise (mode public), `graph_version` (réponse complète seulement).

Valeurs : `Visibility`, `Window`, `QuestionId`, `Answer` (enum aligné sur les types de question).

Cas d'usage (`application/survey/`, méthodes `#[transactional]` sur `MestierUseCase`) :

- gestion : créer, modifier (tous états), ouvrir, fermer, supprimer un questionnaire ; créer, modifier, supprimer une campagne ; fixer les campagnes d'un questionnaire (`set_survey_campaigns`) ; lire une version du graphe (`get_survey_graph_version`) ; lister les questionnaires d'une campagne ;
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
- Le graphe reste modifiable après l'ouverture ; il est versionné (voir Décisions C1).

**Où se fait le calcul du chemin.** Le backend fait foi, à chaque enregistrement et à la complétion. Le navigateur doit aussi trouver la question suivante sans attendre le réseau, donc il suit le graphe lui aussi. Pour qu'il n'y ait pas deux vérités, un même jeu de cas de test (fichier JSON de graphes, réponses et chemins attendus) est exécuté par les tests Rust et par les tests vitest.

### Interface de gestion

- **Éditeur** : une liste de questions, les unes sous les autres, comme un formulaire classique (amendé en C2, remplace le canevas). Chaque question peut porter une règle d'affichage ; la liste se compile en graphe de décision (voir Décisions C2).
- **Ajout d'une question** : un menu présente les dix types de question. Chaque entrée a une **icône** du type, un **titre** et une **description** courte en français. Le catalogue (icône, titre, description) est une donnée du front, une seule source pour le menu et les cartes.
- **Validation à l'édition** : chaque carte signale ses problèmes (libellé, options, règle incomplète, plafond de photos, identifiant en double) ; un compteur « N problèmes à corriger » mène au premier. Le graphe compilé passe aussi le validateur client avant l'enregistrement.
- **Réponse** : le répondant voit **une question à la fois**, dans l'ordre du chemin, avec un bouton pour revenir en arrière.
- Réglages du questionnaire : titre, description, visibilité (brouillon seulement), réponses multiples, plafond de réponses, métadonnées, campagnes de rattachement. Le message de fin se règle sur la dernière carte de l'éditeur.

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

- Les trois événements de questionnaire (`survey.opened`, `survey.response_submitted`, `survey.response_completed`) portent `survey_id`, `campaign_ids` et `graph_version`. `campaign_ids` est `[]` pour un questionnaire indépendant, jamais absent du payload (amendé en C1, remplace `campaign_id`). `graph_version` n'est jamais nul (un questionnaire ouvert a toujours un graphe) ; pour `survey.response_completed` c'est la version de la réponse.
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
@spec-survey-4  Scenario: ouvrir un questionnaire émet survey.opened avec la version de son graphe ; un questionnaire ouvert reste modifiable
@spec-survey-5  Scenario: enregistrer une réponse en cours émet survey.response_submitted
@spec-survey-6  Scenario: compléter une réponse émet survey.response_completed avec toutes les réponses, une seule fois
@spec-survey-7  Scenario: compléter sans réponse à une question obligatoire est refusé
@spec-survey-8  Scenario: une réponse complète ne peut plus être modifiée
@spec-survey-9  Scenario: un questionnaire de campagne refuse une réponse avant le début de la fenêtre (raison campaign_not_started)
@spec-survey-10 Scenario: un questionnaire de campagne refuse une réponse après la fin de la fenêtre (raison campaign_ended)
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
@spec-survey-24 Scenario: les métadonnées du questionnaire et de ses campagnes sont dans le payload des événements et jamais dans une route de réponse
@spec-survey-25 Scenario: chaque événement de questionnaire porte survey_id et campaign_ids, [] pour un questionnaire indépendant
@spec-survey-26 Scenario: chaque événement de campagne porte campaign_id
@spec-survey-27 Scenario: la branche Then ou Else d'une condition décide de la question suivante
@spec-survey-28 Scenario: une question obligatoire hors du chemin parcouru n'empêche pas la complétion
@spec-survey-29 Scenario: une réponse donnée sur un chemin abandonné est écartée et absente du payload de survey.response_completed
@spec-survey-30 Scenario: un graphe avec un cycle, un nœud sans issue ou une condition sur une question non posée est refusé à l'édition
@spec-survey-31 Scenario: deux branches qui se rejoignent sur une question commune sont acceptées
@spec-survey-32 Scenario: le navigateur et le backend calculent le même chemin sur le jeu de cas partagé
@spec-survey-33 Scenario: un chemin qui ne se termine pas sur un nœud End empêche la complétion
@spec-survey-34 Scenario: un questionnaire dans deux campagnes accepte tant que l'une est dans sa fenêtre ; refuse campaign_ended si les deux sont finies, campaign_not_started si l'une est finie et l'autre à venir
@spec-survey-35 Scenario: les métadonnées de deux campagnes sont fusionnées dans l'ordre de rattachement, puis le questionnaire gagne
@spec-survey-36 Scenario: une campagne contient deux questionnaires
@spec-survey-37 Scenario: après modification d'un questionnaire ouvert, une réponse en cours perd la réponse à une question supprimée et se complète
@spec-survey-38 Scenario: les réponses complètes gardent leur graph_version et l'ancien graphe reste lisible ; deux modifications sans nouvelle réponse complète écrasent sur place ; un questionnaire fermé se modifie et reste fermé
@spec-survey-39 Scenario: la visibilité d'un questionnaire ouvert ne change pas (survey_visibility_frozen)
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
pub fn merged_all(campaigns: &[&Metadata], survey: &Metadata) -> Metadata  // C1
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
pub fn campaign_ids(&self) -> &[CampaignId]                       // C1, remplace campaign_id()
pub fn graph(&self) -> Option<&Graph>
pub fn graph_version(&self) -> Option<u32>                        // C1
pub fn opened_at(&self) -> Option<DateTime<Utc>>
pub fn closed_at(&self) -> Option<DateTime<Utc>>
pub fn new( id: SurveyId, organization_id: OrganizationId, title: String, visibility: Visibility, ) -> Result<Self, SurveyError>
pub fn edit_title(&mut self, title: String) -> Result<(), SurveyError>       // C1 : tous états
pub fn edit_description(&mut self, description: Option<String>)             // C1 : tous états
pub fn edit_visibility(&mut self, visibility: Visibility)                   // brouillon seulement
pub fn edit_allow_multiple_responses(&mut self, allowed: bool)              // C1 : tous états
pub fn edit_response_cap(&mut self, cap: Option<NonZeroU32>)                // C1 : tous états
pub fn edit_metadata(&mut self, metadata: Metadata)                         // C1 : tous états
pub fn edit_campaigns(&mut self, campaign_ids: Vec<CampaignId>)             // C1, remplace edit_campaign
pub fn edit_graph(&mut self, graph: S::Input)                               // C1 : Option<Graph> en brouillon, Graph sinon
pub fn revise_graph(&mut self, graph: S::Input) -> Option<GraphVersion>     // C1
pub fn open( mut self, at: DateTime<Utc>, public_slug: Option<PublicSlug>, ) -> Result<Survey<Opened>, SurveyError>
pub fn definition(&self) -> &Graph
pub fn version(&self) -> u32                                                // C1
pub fn public_slug(&self) -> Option<&PublicSlug>
pub fn close(mut self, at: DateTime<Utc>) -> Result<Survey<Closed>, SurveyError>
pub fn accepts(&self, campaigns: &[Campaign], now: DateTime<Utc>) -> Result<(), SurveyError>   // C1
pub fn public_slug(&self) -> Option<&PublicSlug>
pub fn accepts(&self, _campaigns: &[Campaign], _now: DateTime<Utc>) -> Result<(), SurveyError> // C1
pub struct GraphVersion                                                      // C1, exporté SurveyGraphVersion
pub fn new(version: u32, graph: Graph) -> Result<Self, SurveyError>
pub fn version(&self) -> u32
pub fn graph(&self) -> &Graph

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
pub fn graph_version(&self) -> u32                                          // C1, Response<Completed>

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
pub fn new(survey: &'a Survey<Opened>, campaigns: &[Campaign]) -> Result<Self, SurveyError>                                              // C1
pub struct SurveyResponseSubmitted<'a>
pub fn new(survey: &'a Survey<Opened>, campaigns: &[Campaign], response: &'a Response<InProgress>) -> Result<Self, SurveyError>        // C1
pub struct SurveyResponseCompleted<'a>
pub fn new(survey: &'a Survey<Opened>, campaigns: &[Campaign], response: &'a Response<Completed>) -> Result<Self, SurveyError>         // C1
pub struct SurveyCampaignStarted
pub struct SurveyCampaignEnded
```

Payloads des cinq événements (exemples complets dans `survey/descriptors.rs`) :

```text
survey.opened                sujet survey          {survey_id, campaign_ids: [], graph_version, metadata, survey: {id, organization_id, title, description, visibility, allow_multiple_responses, response_cap|null, opened_at}}
survey.response_submitted    sujet survey_response {survey_id, campaign_ids: [], graph_version, metadata, response: {id, member_id|null, status}, answered_questions}
survey.response_completed    sujet survey_response {survey_id, campaign_ids: [], graph_version, survey: {id, title}, response: {id, member_id|null, completed_at, identifier_value|null}, metadata, answers: [{question_id, label, kind, value}]}
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

Décisions S1 (branche `feat/surveys-s1`) :

- **Horloge** : tick du worker d'automatisation, une transaction par campagne verrouillée. Une campagne créée dans ou après sa fenêtre émet `started` (et `ended`) à la création. L'horloge ignore le flag `surveys`.
- **Réponse en deux temps** : `start_*` crée la réponse (aucun événement), `save_*` enregistre (émet `survey.response_submitted`), `complete_*` termine. Le jeton de reprise public est toujours généré côté serveur ; `resume_public_response(slug, token)` ne fait que retrouver la réponse.
- **Vue du répondant** : `open_public_survey` et `open_member_survey` renvoient `RespondentSurvey` (id, titre, description, visibilité, graphe, slug) : jamais d'`organization_id`, de métadonnées, de réglages ni de campagne. S2 et S3 ne renvoient que ce type.
- **Verrous** : toute transition de questionnaire (modifier un brouillon, rattacher, ouvrir, fermer, supprimer, dupliquer) et toute complétion chargent le questionnaire `FOR UPDATE` ; la base refuse qu'un état recule. Le plafond de réponses est donc exact.
- **Unicité** : une violation d'un des deux index uniques devient `RespondentAlreadyAnswered`.
- **Erreurs** : `CoreError` n'a pas de variante de validation, donc toute erreur de questionnaire autre que « introuvable » arrive en `Conflict`. S2 décide s'il faut en ajouter une pour distinguer 400/422 de 409.
- **Permissions** : `survey.manage` et `survey.view` (lecture : l'une ou l'autre) pour les administrateurs ; répondre en mode membres ne demande que d'être membre de l'organisation.
- **Reste à traiter** : deux réponses en cours simultanées pour un même membre (index à ajouter), `list_due` ne résout pas la famine d'une campagne qui échoue en boucle (index partiel seulement), `advance_due` supprimé.

Décisions S2 et S4 :

- **Routes (S2)** : gestion sous `/api/v1/organizations/{organization_id}/...` (questionnaires, campagnes, réponses) ; réponse membres sous `/member-surveys/...` et `/member-survey-responses/...`, à garder séparées. Une route membres sur un questionnaire public répond 404. `start_member` répond 200 quand elle reprend une réponse en cours, 201 quand elle en crée une. Les routes publiques (S3) vivent à une racine à part, jamais sous `/organizations/{id}`.
- **Erreurs** : `CoreError` a deux variantes de plus, `Validation` (422) et `Refused`, qui porte une raison stable. L'enveloppe d'erreur expose `details.reason` (code `snake_case` de `SurveyError::code()`) et `details.question_id` pour les erreurs de question. Les 422 levés par le handler avant le cas d'usage (forme du graphe, e-mail dans `answers`, plafond à 0) n'ont pas de `details.reason`, et les 404 n'ont pas de `details`.
- **OpenAPI** : le graphe et les réponses sont typés dans le document par des types miroirs côté handler. Le client généré type encore `answers` en `Record<string, unknown>` (limite de typed-openapi).
- **Lien public** : la page publique est servie à `/s/{slug}`, à la racine du webapp, hors `_app` et hors `/organizations/{id}` (fichier de route `s.$slug.tsx`). `PublicSlug::path()` est la seule source du format. Aucun réglage d'adresse publique n'existe : les actions renvoient le chemin relatif ; quand un réglage arrivera, un champ `url` s'ajoutera à côté de `path`.
- **Actions d'automatisation (S4)** : `mestier.survey.open`, `.close`, `.get_link`, `.list_responses`, exécutées en acteur système, la portée d'organisation vient du workflow. `open` rattache à la campagne et ouvre dans une seule transaction. `list_responses` renvoie `{ total, responses }`, réponses complètes seulement, les plus récentes d'abord, `limit` par défaut 50, plafond 100, mêmes entrées `answers` que `survey.response_completed`.
- **Pour S5 et S6** : la disposition du canevas n'a pas encore de stockage (colonne et API à décider au début de S5) ; aucun listing des questionnaires ouverts d'un membre ; pas de filtre sur les listes ; `Graph::new` refuse un libellé vide, donc un brouillon ne s'enregistre pas avec une question sans libellé.

Décisions S3 :

- **Routes publiques** (sans compte) sous `/api/v1/public/surveys/{slug}` : lire le questionnaire, `POST .../responses` (démarrer, le seul endroit qui renvoie le jeton de reprise), `GET`/`PUT .../responses/{jeton}`, `POST .../responses/{jeton}/complete`, `POST .../responses/{jeton}/photos`. Le slug et le jeton sont retirés des traces (chemin et requête). Un slug inconnu ou invalide, un brouillon, un questionnaire `MEMBERS`, une organisation sans le flag et un jeton invalide répondent par le même 404, au même corps. Un questionnaire fermé répond 409 `details.reason` (`survey_closed`, `campaign_not_started`, `campaign_ended`, `response_already_completed`).
- **Limites** : par adresse client (adresse lue dans `X-Forwarded-For` seulement si elle se parse comme une IP, sinon l'adresse du pair) et par slug (indépendante de l'en-tête) sur démarrage, envoi de photo et enregistrement ; 429 au-delà. Hypothèse de confiance : la passerelle réécrit `X-Forwarded-For` avec l'adresse réelle du client. Les réponses en cours restent non bornées par questionnaire.
- **Photos** : 20 envois au plus par réponse (409 `response_photo_cap_reached`), distincts de `max_photos` par question (5 par défaut). Octets bruts JPEG, PNG ou WebP (HEIC converti dans le navigateur), type détecté sur le contenu, EXIF et GPS retirés, 50 mégapixels et 12 000 px de côté au plus, vérifiés avant décodage ; normalisation bornée (2 à la fois en public, délai de 2 s puis 503). Clé stockée : `<racine du stockage>/survey-<org>/<survey>/<réponse>/<uuid>.jpg`. Une réponse `PHOTO` n'accepte qu'une clé enregistrée dans `survey.response_photo` pour cette réponse (422 `answer_foreign_photo` sinon). La suppression d'une réponse efface ses fichiers d'abord, puis ses lignes.
- **Purge** : les réponses en cours, publiques ou membres, non modifiées depuis 30 jours sont supprimées avec leurs fichiers enregistrés, sur le tick du worker ; un échec de stockage sur l'une ne bloque pas les autres. Une réponse reprise après purge répond 404 : S6 la traite comme « expirée ». Migration `20261125000001` (`survey.response_photo`).

Décisions S5 :

- **Routes** : `/o/$organizationSlug/questionnaires` (liste), `/questionnaires/$surveyId` (éditeur), `/questionnaires/$surveyId/reponses`, `/questionnaires/campagnes` et `/questionnaires/campagnes/$campaignId` (le préfixe du module doit rester `/questionnaires`).
- Le module est visible avec `VIEW_SURVEYS` et le flag `surveys`. La disposition du canevas se recalcule à chaque chargement : rien n'est stocké. L'éditeur contrôle côté client les règles de `Graph::new` ; un brouillon ne s'enregistre pas tant que son graphe est refusé par le serveur.
- **Manques de l'API** : pas de nombre de réponses ni de nombre de questionnaires par campagne (comptés côté client sur les 100 premiers), pas de route de gestion pour lire une photo de réponse (l'interface affiche la clé). Une route de gestion qui présigne uniquement les clés de `survey.response_photo` reste à écrire.

Décisions S6 :

- **Page publique** : `/s/$slug`, hors `_app`, sans en-tête d'autorisation ni cookie (`credentials: 'omit'`, `referrerPolicy: 'no-referrer'`), sans hook d'authentification ni d'organisation. La réponse ne démarre qu'à la première étape validée. Le jeton de reprise est gardé dans `localStorage` par slug (accès protégé), jamais dans l'URL, effacé à la complétion et sur un 404 « expiré ».
- **Page membres** : `/o/$organizationSlug/repondre/$surveyId`, sans permission de questionnaire. Elle démarre ou reprend la réponse au chargement (200 reprise, 201 création) pour restituer les réponses et afficher « déjà répondu » avant la saisie ; aucun garde de route ne lit le gating des modules, la page affiche donc la navigation de l'accueil.
- **Parcours** : une question à la fois, suivant le chemin du même parcoureur que l'éditeur (jeu de cas `paths.json`), retour arrière, enregistrement en quittant une étape, validation par type alignée sur le backend. Un identifiant déjà utilisé à la complétion renvoie à la question identifiant avec le message, réponses conservées (en public) ; au démarrage, l'écran « Déjà répondu » reste bloquant (membres).
- **Photos** : HEIC converti dans le navigateur par `heic-to` (chargé à la demande, 3,1 Mo), envoi en octets bruts, délai de 60 s par envoi avec annulation, retrait possible d'une photo en cours d'envoi. Une photo déjà envoyée n'a pas de route de relecture : elle s'affiche comme « Photo envoyée précédemment ».
- **Non vérifié sans navigateur** : barre d'actions collante face au clavier iOS, invite caméra, conversion HEIC sur un vrai fichier iPhone, contrastes dans les deux thèmes, annonces de lecteur d'écran, CORS derrière la passerelle.
- **Suites à donner** : exposer `allow_multiple_responses` dans la vue du répondant pour afficher « Réponse déjà envoyée » au rechargement d'un questionnaire à réponse unique (aujourd'hui le jeton est effacé à la complétion, un rechargement rouvre un formulaire vierge) ; route de gestion qui présigne les photos de réponse ; listing des questionnaires ouverts d'un membre ; index d'unicité des réponses en cours par membre ; lien membre copiable depuis la liste de gestion.

Décisions C1 (branche `feat/surveys-editable`) :

- **Questionnaire modifiable** : en brouillon, ouvert ou fermé, le titre, la description, le graphe, `allow_multiple_responses`, le plafond, les métadonnées et les campagnes se modifient. La visibilité et le lien public sont figés dès l'ouverture : une autre visibilité est refusée avec `survey_visibility_frozen` (409) ; la même visibilité est acceptée sans effet. Un questionnaire ouvert ou fermé ne perd pas son graphe (`survey_without_graph`). Chaque modification charge le questionnaire `FOR UPDATE` et garde l'état (un questionnaire fermé reste fermé). Aucun événement de modification.
- **Versions du graphe** : `survey.survey.graph_version` (nul seulement en brouillon sans graphe, contrainte `(graph IS NULL) = (graph_version IS NULL)`) et la table `survey.survey_version (org_id, survey_id, version, graph, created_at)`. Règle exacte (amendée par la vague de correction C1) : **une version est figée si et seulement si sa ligne existe dans `survey_version`**. La ligne est écrite à la complétion : quand une réponse est complétée, elle reçoit la version courante et la ligne de cette version est créée avec le graphe courant si elle manque (si elle existe, elle porte déjà ce graphe, puisqu'une version figée ne s'écrase jamais). Quand le graphe d'un questionnaire ouvert ou fermé change, `has_graph_version(version courante)` décide : ligne présente, la version est figée et le nouveau graphe prend `version + 1` ; ligne absente, le graphe est remplacé sur place, sans incrément. Supprimer des réponses ne défige jamais une version : une réponse de test complétée puis supprimée coûte seulement un numéro de version, et les vraies réponses suivantes sont lues sur leur propre graphe. Un brouillon démarre à la version 1 dès qu'il a un graphe ; une copie repart à 1. La version est typée `GraphVersion(NonZeroU32)` dans le domaine ; l'incrément est vérifié (`checked_add`) et refuse avec `graph_version_exhausted` (409) au-delà de `u32::MAX` ; la colonne `INTEGER` est convertie par un `try_from` vérifié. L'événement `survey.response_completed` refuse (`response_graph_version_mismatch`, erreur interne) une réponse dont la version n'est pas la version courante du questionnaire.
- **Réponses en cours** : elles suivent toujours le graphe courant. À la complétion, une réponse enregistrée (non renvoyée dans la requête) est écartée sans erreur quand sa question n'existe plus ou ne l'accepte plus après une modification du graphe : type changé (`answer_kind_mismatch`), option retirée (`answer_unknown_option`) ou plafond de photos abaissé (`answer_too_many_photos`) ; le répondant n'est donc pas bloqué par une modification. Une réponse hors chemin est écartée comme avant. Une photo étrangère ou vide et un nombre non fini stockés restent refusés. Une réponse envoyée par le client (sauvegarde) est toujours validée et refusée si elle ne convient pas (`answer_unknown_question`, `answer_kind_mismatch`, etc.). Une question supprimée n'est par construction sur aucun chemin : le filtre du chemin l'écarte déjà.
- **Lecture d'une version** : `get_survey_graph_version(survey_id, version)` (lecture de gestion) renvoie le graphe courant quand la version est la courante, sinon la ligne de `survey_version`, sinon 404 `graph_version_not_found` (une version 0 dans le chemin donne aussi 404). Route `GET /api/v1/organizations/{organization_id}/surveys/{survey_id}/graph-versions/{version}` → `{version, graph}`. `get_survey_response` et `list_survey_responses` exposent `graph_version` (nul pour une réponse en cours). Une ligne de version manquante n'est jamais silencieuse : `list_survey_responses` refuse avec `graph_version_not_found` (404) si une réponse complète porte une version ancienne sans ligne ; `list_completed_survey_responses` joint les graphes des versions anciennes en lisant chaque version une seule fois (les absences sont mémorisées) et `CompletedSurveyResponses::graph_of` renvoie `graph_version_not_found` pour une version sans ligne ; l'action `mestier.survey.list_responses` calcule `answers` sur le graphe de chaque réponse, renvoie `graph_version` et échoue en nommant la réponse si son graphe manque, au lieu de renvoyer `answers: []`.
- **Plusieurs campagnes** : la table `survey.survey_campaign (org_id, survey_id, campaign_id, position, attached_at)` remplace `survey.survey.campaign_id` (migration avec copie, `.down.sql` qui recopie la première campagne). `position` garde l'ordre de rattachement, `attached_at` seul ne le départage pas dans une même transaction. `Survey::campaign_ids()` suit cet ordre, sans doublon ; `edit_campaigns` dédoublonne en gardant la première occurrence. Supprimer une campagne reste refusé tant qu'un questionnaire y est rattaché. Concurrence de l'ensemble de campagnes : `find_for_update` pose d'abord un verrou nu (`SELECT 1 … FOR UPDATE`) puis relit le questionnaire par une nouvelle requête, donc un nouvel instantané sous READ COMMITTED : la sous-requête corrélée sur `survey_campaign` ne renvoie plus un ensemble périmé après l'attente du verrou. Les lignes de `survey_campaign` ne sont réécrites que par le port dédié `save_campaigns(organization_id, survey_id, &[CampaignId])`, appelé seulement par `set_campaigns`, `add_campaign` (ouverture dans une campagne) et la création (insertion, copie comprise) ; `update` ne touche plus la table de jointure, si bien qu'une modification du titre qui attendait le verrou ne perd pas les campagnes posées entre-temps. Chaque sous-requête corrélée filtre aussi `sc.org_id = s.org_id`.
- **Acceptation** : `accepts(&[Campaign], now)` sur un questionnaire ouvert. Sans campagne, il accepte ; sinon il accepte si l'une au moins des fenêtres contient `now`. Sinon `campaign_not_started` si l'une des campagnes n'a pas encore commencé, `campaign_ended` sinon. Les campagnes passées doivent être exactement celles du questionnaire (`survey_campaign_mismatch`, `survey_campaign_organization_mismatch`).
- **Métadonnées** : `Metadata::merged_all(campagnes, questionnaire)` fusionne les campagnes dans l'ordre de rattachement (une plus tardive gagne), puis le questionnaire gagne sur toutes.
- **API** : les vues de questionnaire exposent `campaign_ids` et `graph_version` ; création et modification acceptent `campaign_ids`. En modification, `campaign_ids` absent ou `null` laisse les campagnes inchangées et un tableau (même vide) remplace tout l'ensemble, dans l'ordre donné. `UpdateSurveyRequest` est `#[serde(deny_unknown_fields)]` : un ancien client qui envoie `campaign_id` reçoit 422 au lieu d'être ignoré en silence (documenté dans la description OpenAPI). `set_survey_campaigns` (toutes doivent exister dans l'organisation, sinon 404). Listes de campagne paginées en SQL : `GET .../survey-campaigns/{campaign_id}/surveys` renvoie un résumé `{id, title, status}` lu par une requête dédiée sans charger les graphes, borné par `LIMIT/OFFSET` (`per_page` 50 par défaut, 200 au plus, `page` à partir de 1) avec un bloc `pagination` comme les autres listes ; `GET .../surveys?campaign_id=` pagine en SQL par la jointure (plus de pagination en mémoire). L'action `mestier.survey.open` garde son champ `campaign_id` optionnel et **ajoute** la campagne à celles du questionnaire sous le verrou du questionnaire (`SurveyService::add_campaign` : chargement `find_for_update`, ajout si absente, `save_campaigns`) avant de l'ouvrir, dans une transaction ; ses sorties portent `campaign_ids`. Les routes de répondant n'exposent toujours ni campagne ni version.

Décisions C2 (branche `feat/surveys-list-editor`, webapp seulement) :

- **Éditeur en liste** : le canevas du graphe et sa grande vue de détails sont retirés. L'éditeur montre une colonne d'au plus 720 px : en-tête (titre modifiable sur place, statut, Aperçu, Réglages, Réponses, Enregistrer, Ouvrir ou Fermer, menu Dupliquer et lien public), cartes de question, carte « Message de fin », bouton « Ajouter une question », et un « + » entre deux cartes pour insérer à cet endroit. Une question à choix est créée **sans aucune option**. `@xyflow/react` reste dans `package.json` pour l'automatisation.
- **Ordre** : boutons Monter et Descendre, et flèches haut et bas sur la poignée ; chaque déplacement est annoncé (`aria-live`). Pas de glisser-déposer à la souris : aucune bibliothèque de glisser-déposer n'est dans `package.json`.
- **Règles d'affichage** : « Afficher cette question seulement si » une ou plusieurs conditions « question : l'option X est cochée / n'est pas cochée », jointes par « toutes » ou « au moins une ». Une condition ne porte que sur une question à choix **antérieure et toujours posée** (sans règle), parce que le backend exige que la question testée soit posée sur tous les chemins. Une question qui sert de condition ne peut pas recevoir de règle. Monter une question au-dessus de sa source, ou descendre une source sous une question qui en dépend, est refusé avec un message. Supprimer une source demande confirmation et retire les conditions qui en dépendent. Renommer une option met à jour les conditions ; supprimer une option ou passer la question à un type sans option retire les conditions concernées, avec un message.
- **Compilation** (`lib/form-graph.ts`, `buildGraph`) : les questions s'enchaînent par `NEXT` jusqu'à un seul nœud `End` qui porte le message de fin. Pour une question à règle : « toutes » donne une chaîne de conditions dont chaque `Then` mène à la suivante puis à la question, et chaque `Else` à l'entrée suivante ; « au moins une » donne une chaîne par `Else` dont chaque `Then` mène à la question et le dernier `Else` à l'entrée suivante. L'entrée d'une question est sa première condition, ou elle-même. L'id d'une question est l'id de son nœud ; l'id d'une condition est un UUID (version 8) dérivé de l'id de la question et du rang de la condition : un formulaire inchangé donne un graphe identique. Un formulaire vide donne un graphe nul (brouillon seulement).
- **Décompilation** (`parseGraph`) : reconnaît les formes produites par `buildGraph` (dont les chaînes linéaires), quels que soient les ids des conditions existantes ; le graphe recompilé, ids de condition renommés, doit être identique à l'original. Une règle d'une seule condition se relit « toutes ». Tout autre graphe (plusieurs fins, condition qui saute plusieurs questions, branches qui ne se rejoignent pas à la question suivante, nœud inaccessible, cycle) est un **parcours avancé** : la page affiche « Ce questionnaire utilise un parcours avancé qui ne peut pas être modifié ici. » avec le nombre de questions, n'envoie jamais son graphe à l'enregistrement des réglages, et garde l'Aperçu et les Réponses.
- **Aperçu** : une fenêtre plein écran rejoue le formulaire **non enregistré** avec les composants et la machine à états du répondant (`survey-respond`) sur une source en mémoire (`previewSource`, mode `preview` : aucune requête, aucun jeton, photos gardées en URL locales libérées à la fermeture). Bascule Téléphone (390 px) et Bureau, Recommencer. Un formulaire avec des problèmes affiche la liste des problèmes à la place. Le titre du document n'est pas changé en aperçu.
- **Questionnaire ouvert** : il reste modifiable, avec le bandeau « Ce questionnaire est ouvert. Vos modifications s'appliquent immédiatement aux nouvelles réponses ; les réponses déjà reçues gardent la version qu'elles ont vue. » La visibilité est en lecture seule dès l'ouverture (infobulle « La visibilité d'un questionnaire ouvert ne peut pas changer : dupliquez-le. ») et n'est plus envoyée par l'enregistrement. Le bandeau « Dupliquer pour corriger » disparaît ; « Dupliquer » reste dans le menu de l'en-tête et dans la liste. La confirmation d'ouverture ne parle plus de définition figée.
- **Campagnes** : les réglages présentent les campagnes en cases à cocher (`campaign_ids`, ordre de rattachement conservé). La liste des questionnaires affiche toutes les campagnes en pastilles et se filtre par campagne (côté client, « Sans campagne » compris). La page d'une campagne liste ses questionnaires rattachés (`GET .../survey-campaigns/{id}/surveys`).
- **Réponses** : une réponse complète d'une version antérieure est relue sur le graphe de sa version (`GET .../graph-versions/{version}`), avec une pastille « Version N » dans la liste et le détail. En cas d'échec de lecture de la version, le détail retombe sur le graphe courant.

## Risks / open questions

Choix de l'assistant après relecture (à corriger) :

- **Dates d'un questionnaire** : `opened_at` et `closed_at` sont des horodatages d'audit. Seule une campagne a une fenêtre programmée. Un questionnaire indépendant s'ouvre et se ferme à la main ou par une action d'automatisation.
- **Cycle de vie** : `Draft` → `Opened` → `Closed`. Pas de réouverture. Un questionnaire ouvert ou fermé se corrige sur place (amendé en C1) ; on ne le **duplique** que pour changer sa visibilité. Un questionnaire sans réponse peut être supprimé ; avec des réponses, il peut seulement être fermé.
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
| *Tranché en S1 :* l'horloge de campagne tourne au tick du worker d'automatisation existant (`infrastructure/automation/worker.rs`), une transaction par campagne, relecture `FOR UPDATE` avant `advance`. | S1 |

## Décisions C3 (blocs de questions)

Un **bloc** affiche N questions en même temps sur un seul écran du répondant.

- Le nœud question porte `block: Option<BlockId>` (UUID, `null` hors bloc ; absent = `null`). Les questions d'un même bloc se suivent : pour chaque identifiant de bloc, ses nœuds forment une chaîne `q1 -NEXT-> q2 -NEXT-> … -NEXT-> qn`, sans condition entre deux membres, et `q1` est le seul membre qui reçoit des arêtes entrantes de l'extérieur du bloc. Un bloc a donc au moins deux membres (un seul est refusé) et s'affiche en entier ou pas du tout.
- Une règle d'affichage porte sur le bloc entier : les conditions précèdent `q1`, et la branche `ELSE` saute au nœud qui suit `qn`. Les membres `q2…qn` n'ont pas de règle propre. Une condition ne peut pas avoir pour source un membre d'un bloc qui suit ou contient la question dépendante, et la source reste « posée à chaque parcours » comme avant.
- Le répondant : tous les champs du bloc sont affichés ensemble, « Suivant » valide l'écran (chaque question obligatoire du bloc doit être répondue), l'enregistrement et la complétion restent par réponse, sans changement d'API de réponse. Le parcours calculé par `answeredPath` ne change pas ; seul le regroupement des questions consécutives de même `block` est nouveau, côté client.
- Erreurs de graphe : `graph_block_not_contiguous`, `graph_block_too_small`, `graph_block_member_has_condition` (422, `details.reason`, `question_id`).
- Éditeur : un bloc est un conteneur de cartes de question (liseré, intitulé « Bloc de N questions »), ajouté par « Ajouter un bloc » (deux questions vides au départ) ; chaque carte offre « Regrouper avec la précédente » et « Sortir du bloc » ; la logique d'affichage se règle sur le bloc, pas sur ses membres.
- Aucun changement de modèle de réponse ni d'événement. `buildGraph` / `parseGraph` portent `block` ; les graphes sans bloc restent identiques octet pour octet.

## Décisions C4 (description de question)

- Le nœud question porte `description: Option<String>` (texte libre, `null` = absente). Normalisée à l'écriture : espaces de tête et de fin retirés, vide devient `null`, 500 caractères au plus (au-delà : `graph_question_description_too_long`, 422, `details.reason`, `question_id`). Aucun autre invariant, aucun changement de réponse ni d'événement ; un graphe stocké sans le champ reste valide.
- Éditeur : champ « Description (facultative) » sous le libellé de chaque carte de question ; dupliquer une question copie sa description ; `buildGraph` / `parseGraph` la portent, les graphes sans description restent identiques octet pour octet (le serveur renvoie `null` : traité comme absent).
- Répondant et aperçu : la description s'affiche sous le titre de la question, en texte secondaire, avec la même mise en forme dans les écrans de bloc compacts (retours à la ligne conservés).

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
Amended: 2026-11-14 décisions de S1 consignées (horloge, deux temps, vue du répondant, verrous, erreurs).
Amended: 2026-11-14 décisions de S2 et S4 consignées (routes, raisons d erreur, chemin public, actions).
Amended: 2026-11-14 décisions de S3 et S5 consignées.
Amended: 2026-11-14 décisions de S6 consignées.
Amended: 2026-11-15 C1 : un questionnaire ouvert ou fermé reste modifiable (visibilité et lien figés), son graphe est versionné et une réponse complète porte sa version ; un questionnaire appartient à plusieurs campagnes, les événements portent `campaign_ids` et `graph_version` au lieu de `campaign_id`.
Amended: 2026-11-15 vague de correction C1 : une version est figée si et seulement si sa ligne existe (écrite à la complétion, jamais défigée par une suppression) ; verrou nu puis relecture et `save_campaigns` dédié pour l'ensemble de campagnes ; réponses enregistrées périmées écartées à la complétion ; `GraphVersion(NonZeroU32)` ; versions manquantes signalées ; `deny_unknown_fields` sur la modification ; listes de campagne paginées en SQL.
Amended: 2026-11-15 C3 : blocs de questions affichées ensemble (`block` sur le nœud question, règle portée par le bloc, regroupement côté répondant).
Amended: 2026-11-15 C4 : description facultative sur chaque question (`description` sur le nœud question).
