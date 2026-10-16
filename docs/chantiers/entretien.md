# Entretien : contrats et échéances d'intervention

État : **W0 fait le 2026-10-11 (migration, types, ports, commandes, événements), W1 à W5 à lancer**. Mis à jour à chaque intégration.

## Goal

Le conducteur de travaux de l'entretien voit, pour la saison, chaque prestation d'entretien de chaque site, la date de la dernière intervention et la date de la prochaine. Le jeudi, il ouvre le récap des prestations dues la semaine suivante, les planifie en un clic, et valide les dates d'intervention saisies par le terrain.

## Context

- Besoin : Treyve Paysages, conducteur de travaux de l'entretien (agence Vichy). Aujourd'hui sur papier : une feuille A3 par conducteur, une ligne par prestation, une colonne par semaine (S12 à S48), le jour du mois écrit à la main dans la case. Oublis et cases non vues.
- Ligne de la feuille : code affaire (C25029), client, lieu, type de prestation, nombre d'interventions ou délai, heures totales, montant, contact.
- Deux cadences sur la feuille : glissante (« tonte toutes les 2 semaines ») et par fenêtres (« taille printemps / taille automne », « passage avril / juillet / novembre »).
- ADR : [0006, échéance glissante calculée](../adr/0006-maintenance-due-date.md).
- Décisions produit (2026-10-11) :
  1. La prochaine date se calcule depuis la dernière intervention **validée** (cadence glissante), pas depuis un calendrier fixe.
  2. Les deux cadences (glissante et fenêtres) sont en v1.
  3. Le terrain propose une date, le conducteur valide. Une saisie du conducteur est validée d'office.
  4. Écrans dans `Planification` (`/planification/maintenance`), droit `planning.manage`.
  5. Récap du jeudi : page et PDF à la demande. Pas d'envoi automatique en v1.
  6. Un contrat couvre une saison (une année). On le duplique pour l'année suivante.
  7. Le lien au devis est optionnel. Saisie manuelle en v1.

### Faits vérifiés dans le code

- `task_recurrence` fixe ses dates sur le calendrier depuis `starts_on` et ne lit jamais le statut des tâches : une intervention en retard ne décale rien. Il n'a ni « tous les N jours », ni « N par an » (`domain/task_recurrence/mod.rs:101`). Il ne convient pas, voir l'ADR 0006.
- Le site existe : `CustomerContext` (adresse, ville, photo). Les contacts (`CustomerContact`) sont rattachés au client, pas au site.
- Le coût vient des tâches planifiées d'un `Project` (ADR 0002). Rattacher les interventions à un projet donne la rentabilité du contrat sans nouveau calcul.
- Le terrain ne pointe presque pas (ADR 0002). L'application terrain a `my_tasks` et le signalement d'écart (`AssignmentReport`, signalé → appliqué / rejeté), mais aucun bouton « fait ».
- Automation : aucun déclencheur planifié, aucun canal email. Les événements tâche et projet ne sont pas au catalogue.

## Glossary

| Terme | Définition | Synonyme rejeté |
|---|---|---|
| Contrat d'entretien (`MaintenanceContract`) | Engagement d'entretenir un site d'un client pendant une saison | marché, abonnement |
| Saison (`Season`) | Année du contrat et sa plage de semaines (ex. S12 à S48) | campagne |
| Prestation (`MaintenanceLine`) | Une ligne de la feuille : un type de travail avec sa cadence | ligne, service |
| Cadence (`Cadence`) | Règle qui donne la prochaine échéance : glissante, par fenêtres ou à la demande | périodicité (UI : « périodicité ») |
| Intervention (`Intervention`) | Un passage daté sur une prestation, proposé puis validé | passage, visite |
| Échéance (`Due`) | Date à laquelle la prestation doit être faite, calculée, jamais stockée | rappel, prochaine date |
| Récap hebdo | Liste des prestations dues ou en retard pour une semaine donnée | rapport, synthèse |

## Invariants

1. Une échéance n'est jamais stockée : elle est recalculée depuis la cadence, la saison et les interventions validées. [type : `Due` n'a pas de colonne ; fonction pure `due_for(line, interventions, today)`]
2. Seule une intervention `Validated` fait avancer l'échéance. Une `Proposed` ou `Rejected` ne la change pas. [test unitaire]
3. Une cadence glissante a un intervalle entre 1 et 365 jours. Une cadence par fenêtres a entre 1 et 12 fenêtres, sans chevauchement, dans la saison. `OnDemand` n'a pas de paramètre. [type : `Cadence::try_from`]
4. Aucune échéance hors de la saison : après la dernière semaine, la prestation est `Terminée`. [test unitaire]
5. Une intervention est datée dans la saison de son contrat et au plus aujourd'hui. [constructeur `Result`]
6. Toute lecture et écriture est scopée par `organization_id`. [test d'intégration]
7. Un membre du terrain ne propose une intervention que sur une tâche qui lui est assignée. [test d'intégration, même règle que `report_assignment`]
8. Valider ou rejeter une intervention persiste son événement dans la même transaction. [`with_tx_emitting`]

## Out of scope (v1)

- Envoi automatique du récap (cron, email, Discord) : demande un déclencheur planifié et un canal, chantier à part.
- Création d'un contrat depuis un devis accepté, et l'import IA de la feuille (lot 2, voir « Usages IA »).
- Proposition automatique depuis le pointage : le terrain ne pointe presque pas (ADR 0002). Le bouton suffit.
- Météo, optimisation de tournée, contrats multi-sites.
- Contacts propres au site : on affiche le contact principal du client.

## Design

### Calcul de l'échéance (cœur du chantier)

`Cadence` :
- `Rolling { every_days }` : échéance = dernière intervention validée + `every_days`. Sans intervention dans la saison : premier jour de la saison.
- `OnDemand` : jamais d'échéance, statut `Done` en permanence. Les interventions s'enregistrent quand même (ligne « ND » de la feuille).
- `Windows([Window { from_week, to_week }])` : la première fenêtre sans intervention validée donne l'échéance = lundi de `from_week`. Elle est en retard après `to_week`. Une fenêtre passée et manquée reste en retard : elle ne s'efface pas au profit de la suivante.

`DueStatus`, par ordre de priorité :
1. `Planned` : une tâche ouverte (backlog, planifiée ou en cours) porte la prestation. Elle ne réapparaît pas dans le récap.
2. `Overdue` : échéance < lundi de la semaine courante.
3. `ThisWeek` / `NextWeek` : échéance dans la semaine courante ou la suivante (semaines ISO, fuseau de l'organisation).
4. `Later`.
5. `Done` : saison finie ou toutes les fenêtres faites. Sans échéance, le statut est `Done` même si une tâche est ouverte. Une fenêtre en cours, sans intervention, est `ThisWeek` : elle n'est en retard qu'une fois terminée.

Fonction pure dans `domain/maintenance/due.rs`, testée en unitaire. Le récap et la grille appellent la même fonction.

### Domain (`libs/core/src/domain/maintenance/`)

- `MaintenanceContract` : `customer_id`, `customer_context_id` (site, obligatoire), `project_id` (créé avec le contrat), `reference` (code affaire, texte libre, optionnel), `season: Season { year, first_week, last_week }`, `manager_member_id` (conducteur), `amount_cents` (optionnel), `quote_id` (optionnel), `notes`, `archived_at`.
- `MaintenanceLine` : `contract_id`, `label` (« Tonte avec ramassage »), `cadence`, `minutes_per_intervention`, `position`.
- `Intervention` : `line_id`, `done_on: NaiveDate`, `status: Proposed | Validated | Rejected`, `task_id` (optionnel), `proposed_by`, `resolved_by`, `resolved_at`, `note`.
- `Task` gagne `maintenance_line_id` (optionnel). Une tâche planifiée depuis le récap porte la prestation, le site et le projet du contrat.
- `MaintenanceService` : créer et modifier un contrat avec ses prestations, dupliquer pour la saison suivante, proposer, valider, rejeter, saisir directement, construire la grille et le récap.

### Persistance

Tables `maintenance_contracts`, `maintenance_lines`, `maintenance_interventions`, toutes avec `organization_id`. La cadence est en JSONB sur la ligne (valeur, jamais interrogée par l'intérieur). `tasks.maintenance_line_id` est nullable, avec `ON DELETE SET NULL`. `.down.sql` testé.

### HTTP (préfixe `/api/v1/organizations/{org}`)

| Méthode | Chemin | Crate | Droit |
|---|---|---|---|
| GET/POST | `/maintenance/contracts` (`?season=2026`) | handlers-planning | planning.manage |
| GET/PUT | `/maintenance/contracts/{id}` (contrat + prestations, remplacement complet) | handlers-planning | planning.manage |
| POST | `/maintenance/contracts/{id}/archive`, `/duplicate` (saison suivante, semaine 53 ramenée à 52 si l'année n'en a pas) | handlers-planning | planning.manage |
| GET | `/maintenance/grid?season=2026` (lignes × semaines, interventions, échéance) | handlers-planning | planning.manage |
| GET | `/maintenance/recap?week=2026-W43` (+ `/recap.pdf`) | handlers-planning | planning.manage |
| POST | `/maintenance/recap/plan` `{line_ids, week}` → tâches non datées dans la semaine, au backlog du board | handlers-planning | planning.manage |
| POST | `/maintenance/lines/{id}/interventions` `{done_on}` → `Validated` | handlers-planning | planning.manage |
| GET | `/maintenance/interventions?status=proposed` | handlers-planning | planning.manage |
| POST | `/maintenance/interventions/{id}/validate` (sans corps), `/reject` `{note?}` (non vide si présente) | handlers-planning | planning.manage |
| POST | `/field/tasks/{task_id}/intervention` `{done_on}` → `Proposed` | handlers-field | assigné |

Contrats (WS2) : le `PUT` ne change ni le client, ni le site, ni l'année (409). Le projet du contrat s'appelle « Entretien {année} - {client} - {site} ». La duplication reprend référence, conducteur, montant, notes, statut et prestations, mais pas le lien au devis, et ramène aussi à 52 les fenêtres en semaine 53. Archiver deux fois garde la première date.

### Précisions du récap et des visites (WS3)

- Le récap de la semaine W calcule l'échéance au lundi de W et garde `ThisWeek` et `Overdue` : les prestations dues dans W, une fenêtre encore ouverte en W, et celles dues avant W. Celles dues après W n'y sont pas. Communes en retard d'abord, puis alphabétique, sans commune en dernier ; dans une commune, en retard d'abord, puis par échéance, puis dans l'ordre du contrat.
- Les prestations d'un contrat `Draft` ne sont jamais dues : `Done` dans la grille, absentes du récap. Un contrat archivé n'est ni dans la grille ni dans le récap.
- `POST /maintenance/recap/plan` crée la tâche au backlog (titre = libellé de la prestation, « Entretien à prévoir en S43 2026 » en description) ; une prestation déjà portée par une tâche ouverte, ou d'un contrat archivé ou brouillon, renvoie 409 et ne crée rien.
- Une ligne, une intervention ou une tâche d'une autre organisation, ou une tâche qui n'est pas à l'appelant ou pas d'entretien, renvoie 404. Le chemin terrain est `/organizations/{org}/field/tasks/{task_id}/intervention`.

### Événements

`maintenance_intervention.proposed`, `.validated`, `.rejected`, ajoutés au catalogue. Un workflow pourra s'y brancher plus tard (Discord, Odoo).

### PDF du récap

Typst, comme le DOE (`libs/doc-render`, nouveau gabarit `maintenance_recap.typ`). Une ligne par prestation due : site, commune, prestation, dernière intervention, échéance, heures prévues, contact. Groupé par commune, en retard en tête : les communes qui ont une prestation en retard passent d'abord, puis l'ordre alphabétique, et dans une commune les lignes en retard précèdent les autres, par échéance. Le PDF sert les lignes du même cas d'usage que `GET /maintenance/recap`, et le site s'y imprime « client - site », comme le nom du projet.

### Front (`apps/webapp`)

Sous `Planification`, dans `modules/registry.ts` :
- **Entretien › Grille** : la feuille papier. Lignes = prestations groupées par contrat, colonnes = semaines de la saison, case = jour du mois de l'intervention. Couleurs : validée, proposée, échéance, en retard. Un clic sur une case vide saisit une intervention.
- **Entretien › Récap** : sélecteur de semaine, S+1 par défaut quel que soit le jour (le récap de S+1 contient déjà ce qui reste dû avant). Cases à cocher, bouton « Planifier », bouton « PDF ».
- **Entretien › À valider** : même UX que « Écarts signalés », avec Valider et Rejeter (motif facultatif) sur chaque ligne. La réponse ne porte que `line_id` : la prestation et le site viennent de la grille de la saison de la visite (un contrat archivé n'y est plus).
- **Entretien › Contrats** : liste par saison, formulaire contrat + prestations, « Dupliquer pour 2027 ».
- Terrain : sur une tâche d'entretien de « Mes tâches », bouton « Intervention faite » (date du jour par défaut, modifiable, jamais après aujourd'hui). La liste du terrain ne dit pas si une visite est déjà proposée : après l'envoi, le bouton laisse place à une confirmation jusqu'au rechargement.

Précisions de la grille et des contrats (WS5a) :
- Entretien est une seule entrée du menu Planification (`/planification/maintenance`), avec les onglets Grille et Contrats ; WS5b y ajoute Récap et À valider. La saison se choisit par `?season=` (année courante par défaut).
- Les colonnes couvrent la plus large des saisons des contrats de l'année ; les semaines hors saison d'un contrat sont hachurées et ne se cliquent pas.
- La case porte le jour du mois, et la semaine de la case dit de quel mois il s'agit : un jour que cette semaine ne contient pas est refusé. « Au plus aujourd'hui » et « dans la saison » restent le contrôle du backend (409 affiché tel quel).
- Une case qui a déjà une intervention (validée ou proposée) ne se clique plus : il n'existe ni modification ni suppression d'intervention en v1. Les rejetées n'apparaissent pas.
- Le formulaire de contrat renvoie tel quel le lien au devis et le statut, qu'il n'affiche pas : le `PUT` remplace tout.
- Brouillons (WS6b) : la grille ne montre pas les contrats `Draft` (l'API les renvoie, sans échéance). L'onglet Contrats les range dans « Brouillons à vérifier », avec « Activer » (un `PUT` qui renvoie tout tel quel, sauf `status: ACTIVE`) ; comme la liste, la section se lit par saison.

### Errors

`CoreError` existant. Une cadence ou une date invalide est un `Conflict`, comme ailleurs. Valider une intervention déjà résolue renvoie 409.

## Acceptance (Gherkin)

```gherkin
@spec-ent-1 Scenario: cadence glissante
  Given la prestation "Tonte" du contrat "GAUME 2026" avec une cadence glissante de 14 jours
  And une intervention validée le 2026-10-10
  When je calcule l'échéance le 2026-10-12
  Then l'échéance est le 2026-10-24 et son statut est NextWeek

@spec-ent-2 Scenario: seule une intervention validée compte
  Given la prestation précédente
  And une intervention proposée le 2026-10-20
  When je calcule l'échéance
  Then l'échéance reste le 2026-10-24

@spec-ent-3 Scenario: fenêtres
  Given la prestation "Passage" avec les fenêtres S14-S17, S27-S30, S44-S47
  And des interventions validées en S15 et S28
  When je calcule l'échéance en S31
  Then l'échéance est le lundi de S44 et le statut est Later
  When je calcule l'échéance en S48 sans intervention en S44-S47
  Then le statut est Overdue

@spec-ent-4 Scenario: fin de saison
  Given une saison S12-S48 et une tonte tous les 14 jours, dernière intervention en S47
  When je calcule l'échéance
  Then le statut est Done

@spec-ent-5 Scenario: récap du jeudi
  Given 3 prestations dues en S43, 1 en retard, 1 due en S43 déjà planifiée, 2 dues en S46
  When je demande le récap de S43
  Then il contient 4 lignes, celle en retard en tête, groupées par commune

@spec-ent-6 Scenario: planifier depuis le récap
  When je planifie 2 lignes du récap de S43
  Then 2 tâches existent dans le projet du contrat, avec le site et la prestation
  And le récap de S43 ne les montre plus

@spec-ent-7 Scenario: le terrain propose, le conducteur valide
  Given une tâche d'entretien assignée à Bernard
  When Bernard déclare l'intervention faite le 2026-10-14
  Then une intervention Proposed existe et l'échéance n'a pas bougé
  When le conducteur la valide
  Then l'échéance passe au 2026-10-28 et l'événement maintenance_intervention.validated est persisté

@spec-ent-8 Scenario: le terrain ne propose que sur ses tâches
  Given une tâche d'entretien assignée à Paul
  When Bernard déclare une intervention dessus
  Then la réponse est 404

@spec-ent-9 Scenario: isolation des organisations
  Given un contrat de l'organisation A
  When un membre de B le lit, le modifie ou valide une de ses interventions
  Then la réponse est 404

@spec-ent-10 Scenario: droits
  Given un membre sans planning.manage
  When il lit la grille ou valide une intervention
  Then la réponse est 403

@spec-ent-11 Scenario: duplication de saison
  Given le contrat "GAUME 2026" avec 2 prestations et 12 interventions
  When je le duplique pour 2027
  Then "GAUME 2027" a les 2 prestations, aucune intervention, un nouveau projet

@spec-ent-12 Scenario: PDF du récap
  When je télécharge le PDF du récap de S43
  Then c'est un PDF dont le texte contient chaque site et "Saint-Pourçain-sur-Sioule" avec accents

@spec-ent-13 Scenario: grille (front)
  Given la grille 2026 ouverte
  When je clique la case S43 de "Tonte GAUME" et saisis le 21
  Then la case affiche 21 en validé et l'échéance suivante apparaît en S45

@spec-ent-14 Scenario: prestation à la demande
  Given la prestation "Entretien T1" en cadence OnDemand, sans intervention
  When je calcule l'échéance
  Then le statut est Done et la prestation n'apparaît pas dans le récap
```

Tests : 1 à 4 et 14 en unitaires sur `due.rs` (plus des tests de propriété : l'échéance ne recule jamais quand on ajoute une intervention validée) ; 5 à 11 en intégration (`#[ignore]`, base réelle) ; 12 dans `doc-render` ; 13 en vitest.

## Simulation

Non. Le temps entre dans le calcul, mais par un paramètre `today` explicite, jamais par l'horloge système : les tests unitaires couvrent les bascules de semaine et d'année.

## Usages IA (lot 2, après la v1)

Règle commune : l'IA propose, l'humain valide. Les écritures passent par l'état `Proposed` ou par un contrat en brouillon.

| # | Usage | Outils MCP | Écriture |
|---|---|---|---|
| IA-1 | Préparer la séance du jeudi dans ChatGPT : « que dois-je programmer la semaine prochaine ? » | `list_maintenance_due(week)`, `get_maintenance_contract` | non |
| IA-2 | Import de la feuille papier ou Excel : le LLM lit la photo et crée les contrats en brouillon | `create_maintenance_contract_draft` | oui (brouillon), nouveau scope `mestier.maintenance.write` |
| IA-3 | Proposer les tournées : échéances × disponibilités (`get_availability`) × communes × heures | lecture + `propose_maintenance_plan` | oui (tâches au backlog) |
| IA-4 | Saisie terrain en langage naturel (« tondu chez Gaume ce matin ») | intervention `Proposed` | oui (proposée) |
| IA-5 | Fin de saison : contrats sous-consommés ou en dépassement d'heures, brouillon de devis de renouvellement | lecture + devis brouillon | oui (brouillon) |

IA-2 ajoute à `MaintenanceContract` un statut `Draft | Active`. On le prévoit dès la v1 (colonne `status`, `Active` par défaut) pour ne pas migrer ensuite.

## Workstreams

Epic : #545. Issues : W1 #546, W2 #547, W3 #548, W4 #549, W5a #550, W5b #551, W6 #552.

Contrats figés par W0 : `libs/core/src/domain/maintenance/{mod,ports,commands,events,due}.rs`, migration `20261011000002_create_maintenance`. `due_for` a sa signature, son corps est un `todo!` pour W1. Le champ `Task.maintenance_line_id` (colonne déjà en base) est ajouté au domaine `task` par W3, avec la création des tâches depuis le récap.


| Id | Mission | Dépend de | Modèle |
|---|---|---|---|
| W0 | Contrats : migration, types de domaine (`Cadence`, `Season`, `Due`), ports, commandes, marqueurs registry, événements au catalogue. **Orchestrateur.** | — | orchestrateur |
| W1 | `due.rs` : calcul d'échéance et tests (unitaires et propriété) | W0 | sonnet |
| W2 | Contrats, prestations, duplication : service, repo, use cases, handlers | W0 | sonnet |
| W3 | Interventions : proposer (terrain), valider, rejeter, saisir ; grille et récap ; planifier | W0, W1 | sonnet |
| W4 | PDF du récap (`doc-render`) | W0 | sonnet |
| W5a | Front : contrats et grille | W2 + W3 intégrés, client API régénéré | sonnet |
| W5b | Front : récap, À valider, bouton terrain | W3 + W4 intégrés | sonnet |
| W6 | Lot 2 : outils MCP IA-1 et IA-2 | v1 intégrée | sonnet |

## Questions fermées

1. Couleurs rouge et bleu de la feuille : sens inconnu, sans importance (2026-10-11). La grille a sa propre légende : validée, proposée, planifiée, échéance, en retard.
2. « ND » (ligne DREX) : sens inconnu (2026-10-11). On le traite comme une prestation à la demande : cadence `OnDemand`, jamais due, interventions enregistrées quand même.

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mestier-core -- --ignored
sqlx migrate revert --source migrations && sqlx migrate run --source migrations
cd apps/webapp && pnpm check && pnpm test && pnpm build
```
