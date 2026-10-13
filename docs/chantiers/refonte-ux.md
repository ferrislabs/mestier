# Refonte UX : navigation et DOE

État : **spec à valider le 2026-10-13, aucun code**. Remplace l'audit de `ux-simplification.md` (écrans, 2026-08) par un chantier de navigation.

## Goal

Retrouver n'importe quel écran courant en deux clics au plus depuis le menu, sans deviner dans quel module il se cache. Le DOE se trouve depuis le projet auquel il appartient. Les modèles et la mise en page des PDF se règlent à un seul endroit.

## Context

Retour du product owner (2026-10-13) : DOE, projets et modèles sont au même endroit et c'est confus ; sur l'ensemble du site « il faut cliquer un peu de partout, on s'y perd assez vite ».

### Faits vérifiés dans le code

- `modules/registry.ts` : 9 modules, 25 sections. `Planification` en porte 5 : Projets, DOE, Modèles de projet, Modèles de DOE, Tableau. Trois sont du travail quotidien, deux sont de la configuration.
- Le DOE est déjà rattaché à un projet : la route `/planification/projects/$projectId/doe` existe et la fiche projet a un bouton DOE. La liste `/planification/does` est donc un second chemin vers les mêmes objets.
- La mise en page du PDF (couleurs, polices, en-tête) est dans Paramètres › « Mise en page des documents ». Les modèles de DOE, qui peuvent la remplacer, sont ailleurs, dans Planification. Un même réglage vit à deux endroits.
- `Planning` (temps) et `Planification` (travail) sont deux modules distincts voulus (CLAUDE.md) ; le libellé proche est une source de confusion à traiter dans l'audit, pas une décision de les fusionner.
- Les anciennes adresses `/planning/projects*` redirigent déjà vers `/planification` : le projet sait garder des liens vivants.
- Le type `ModuleSection` accepte déjà des `tabs` (second niveau, sous le fil d'Ariane). Aucune section ne les utilise.
- Le chantier Entretien (#545) ajoute ses écrans sous `Planification` : Grille, Récap, À valider, Contrats.

## Décisions (2026-10-13)

1. **Le DOE devient un onglet du projet.** Plus d'entrée « DOE » ni « Modèles de DOE » dans le menu. La liste transversale disparaît ; la liste des projets montre l'état du DOE (aucun, brouillon, indice N) et se filtre dessus.
2. **Les modèles vont dans Paramètres › Modèles** : modèles de projet, modèles de DOE et mise en page des PDF, trois blocs d'une même page. Le formulaire de création garde un lien « Choisir un modèle ».
3. **Navigation d'abord.** Phase 1 : parcours et menu. Les écrans ne sont refaits qu'ensuite, un module à la fois.

## Out of scope

- Fusionner `Planning` et `Planification`.
- Refonte visuelle des écrans (phase 2, spécifiée après l'audit).
- Backend : aucun changement d'API.
- Le terrain (`/field`), qui a sa propre navigation.

## Workstreams

| Id | Mission | Dépend de | Fichiers possédés |
|---|---|---|---|
| W0 | Audit des parcours : pour 8 tâches courantes, le nombre de clics aujourd'hui et la cible. Résultat dans la section « Audit » ci-dessous. Lecture seule. | — | ce document |
| W1 | DOE en onglet du projet, retrait des deux entrées du menu, état du DOE dans la liste des projets, redirections | décisions 1 | `pages/projects/**`, `pages/does/**`, `pages/doe/**` (liens seulement), `routes/*planification.does*` |
| W2 | Page Paramètres › Modèles (projet, DOE, mise en page), retrait des entrées du menu, redirections | décision 2 | `pages/settings/**`, `pages/doe-models/**`, `pages/project-templates/**`, `routes/*project-templates*`, `routes/*doe-models*` |
| W3 | Menu : regroupement, libellés, ordre, d'après l'audit | W0, W1, W2 | `modules/registry.ts`, `modules/*.ts` |

W1 et W2 touchent le même `registry.ts` d'une ligne chacun (retrait d'une entrée) : un seul des deux le fait, W1, qui retire aussi `doe-models` et `project-templates` une fois W2 intégré. Ordre : W2 puis W1.

Tâches de l'audit : créer un devis ; planifier une tâche ; voir la rentabilité d'un chantier ; produire le DOE d'un chantier ; saisir une absence ; ajouter un fournisseur ; lancer un workflow ; inviter un membre.

## Acceptance (Gherkin)

```gherkin
@spec-ux-1 Scenario: le DOE s'ouvre depuis le projet
  Given le projet "Lapalisse" ouvert
  When je clique l'onglet "DOE"
  Then je vois le DOE du projet, ou le choix d'un modèle pour en créer un

@spec-ux-2 Scenario: le menu ne propose plus de liste de DOE
  When j'ouvre le module Planification
  Then le menu contient Projets et Tableau, ni "DOE" ni "Modèles de DOE" ni "Modèles de projet"

@spec-ux-3 Scenario: l'état du DOE dans la liste des projets
  Given 3 projets dont un avec un DOE à l'indice B
  When j'ouvre la liste des projets
  Then chaque ligne indique "Aucun DOE", "Brouillon" ou "Indice B"
  And le filtre "Avec DOE" ne garde qu'un projet

@spec-ux-4 Scenario: tous les modèles au même endroit
  When j'ouvre Paramètres › Modèles
  Then je vois les modèles de projet, les modèles de DOE et la mise en page des PDF

@spec-ux-5 Scenario: les anciennes adresses restent vivantes
  When j'ouvre /planification/does, /planification/doe-models ou /planification/project-templates
  Then je suis redirigé vers le projet, ou vers Paramètres › Modèles, sans erreur

@spec-ux-6 Scenario: droits
  Given un membre sans MANAGE_PLANNING
  Then il ne voit ni l'onglet DOE ni la page Modèles
```

Tests : 1 à 6 en vitest sur le registre, les redirections et les composants ; pas de e2e navigateur.

## Audit

À remplir par W0.

## Questions ouvertes

1. Les modèles de projet servent à créer un projet : faut-il aussi un accès depuis la liste des projets (« Gérer les modèles » → Paramètres › Modèles) ? Proposition : oui, un lien dans le menu « Nouveau projet ».
2. Le tableau (`/planification/board`) reste-t-il dans Planification ou monte-t-il en entrée d'accueil du module ? À trancher avec l'audit.
3. Le CRM et le chantier Customer (#482) ajoutent des écrans en parallèle : W3 attend leur intégration ou les range à part ?

## Verify (exit)

```bash
cd apps/webapp && pnpm check && pnpm test && pnpm build
```
