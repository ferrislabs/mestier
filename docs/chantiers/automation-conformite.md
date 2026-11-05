# Automatisation : conformité au prototype

Objectif : l'automatisation du vrai Mestier (front et back) est identique, en fonctions et en UI, au module Workflows du prototype `mestier-rework` (`src/modules/automatisation`). Le prototype fait foi.

## Décisions (2026-11-05)
- Les fonctions du réel absentes du prototype (supprimer ou renommer un workflow, historique, relance, boucle, configuration, Odoo, rotation et origine générée des secrets, formulaire d'identification dans un nœud, rétention) sont gardées et restylées.
- Le moteur est réécrit pour le modèle du prototype : expressions `$('Nom du nœud').item.json.x` et listes d'éléments par nœud.
- « Tester le workflow » devient un mode à blanc côté backend (graphe courant, connecteurs renvoyant leur exemple de sortie, aucun effet de bord).
- URLs du prototype, anciennes URLs redirigées. Identifiant d'exécution dans l'URL : UUID du run jusqu'au lot 2 (numéro séquentiel ensuite).
- Libellés de branches `Each` et `After` : « chaque élément » et « terminé » (choix de l'agent orchestrateur, le prototype n'a pas de boucle).
- Une catégorie du créateur sans type disponible (« Validation humaine » avant le lot 6) n'est pas affichée.

## Lots
| Lot | Branche | Contenu | État |
|---|---|---|---|
| 1 | automation-front | Front seul : URLs, libellés, rendu, éditeur, exécutions, identifications | intégré, non commité |
| 2 | automation-persistence | Dossiers, tags, archivage, champs de nœud dans le graphe, champs de run | à faire |
| 3 | automation-collections | Stockage et exécution des collections OpenAPI | à faire |
| 4 | automation-dry-run | Test à blanc | à faire |
| 5 | automation-triggers | Webhook typé, planificateur | à faire |
| 6 | automation-connectors | Attendre, fusionner, code, note, devis, projet, canal, SMTP, validation | à faire |
| 7 | automation-credentials | Description, statut, secrets séparés, 14 types | à faire |
| 8 | automation-engine | Expressions par nom de nœud, listes d'éléments | à faire |

## Lot 1 : étapes
1. Routage et URLs (R) : intégré.
2. Parallèle : L liste, E éditeur et créateur, X exécutions, I identifications.
3. Après E : N détail de nœud.
4. Comparaison visuelle 3100/3000 et correctifs.

## Partition lot 1
- L : feature/{automation-workflows-feature,use-workflow-actions,projects-sidebar-feature}, ui/{automation-workflows-list,workflow-row,workflow-list-toolbar,workflow-list-empty,workflow-editor-dialog,folder-row,folder-breadcrumb,projects-sidebar}, lib/{workflow-list,folders,download}.
- E : feature/{workflow-canvas-feature,editor-canvas-feature,editor-actions,use-workflow-graph,workflow-top-bar-feature,automation-flow-node,node-creator-feature,event-picker-feature,run-now-feature,node-issues-context,workflow-page-feature}, ui/{editor-top-bar,automation-node-card,node-creator-panel,empty-canvas-prompt,canvas-menu-items,context-menu-at,rename-node-dialog,workflow-not-found,mestier-event-dialog,run-now-dialog}, lib/{creator,editor-nodes,connector-appearance,node-appearance,trigger-events,event-search,node-issues,validation,graph,layout,history,shortcuts,references,operation-search}.
- X : feature/{executions-log-feature,workflow-executions-feature,run-node-details-feature}, ui/{automation-executions-log,executions-table,executions-toolbar,execution-list,execution-parameters-panel,execution-steps,execution-canvas,execution-flow-node,execution-legend,execution-status,run-node-details,run-status-label,workflow-executions}, lib/{execution-log,execution-status,run-view,run-node,workflow-runs,run-appearance}.
- I : feature/{automation-credentials-feature,use-credential-meta}, ui/{automation-credentials-ui,credentials-table,credential-dialog,credential-kind-aside,credential-kind-names,credential-kind-picker,credential-secret-reveal,credential-status-label,secret-field,inline-credential-form,credential-picker-field,kind-icon-tile}, lib/{credential-appearance,credential-form,credential-meta,credential-picker}.
- N : feature/{node-details-feature,connector-parameters-feature,use-resolved-parameters}, ui/{ndv-*,node-details-dialog,node-input-*,node-output-panel,node-parameters-panel,event-input-panel,event-payload-tabs,variables-panel,trigger-parameters,connector-config-*,expression-preview,operation-*,data-viewer,json-view,typescript-view,text-area}, lib/{connector-config,parameters,variables,expression,data-tree,items,node-view,payload-contract,schema-typescript,upstream-entries,run-upstream}.
- Orchestrateur : `docs/chantiers/automation-conformite.md`, `components/ui/sidebar.tsx`, `routeTree.gen.ts` (régénéré), `lib/automation-paths.ts`, `lib/branch-labels.ts`.
- Régénéré à l'intégration : `apps/webapp/src/routeTree.gen.ts` par le plugin du routeur (serveur 3000, sinon `pnpm build`).

## Journal
- Étape 1 intégrée : Biome propre, 739 tests verts (automation, modules, home), erreurs `tsc` préexistantes inchangées.
- Lot 1 intégré (branche `feature/automation-front`, rien de commité) : URLs, liste, éditeur et créateur (`lib/connector-copy.ts`), exécutions (statuts d'étape, durée en direct), identifications (libellés français), détail de nœud (`lib/connector-field-copy.ts`, `lib/event-contracts.ts`), correctifs shell (en-tête de page, sidebar 15 rem, launcher). Vérifié : Biome propre, 3919 tests verts (364 fichiers), `pnpm build` passe, 46 erreurs `tsc` préexistantes (fixtures sans `mcp_enabled`, hors périmètre).
- Comparaison visuelle 3100/3000 faite sur l'organisation `baptistep`, qui n'a qu'un workflow vide, aucune exécution ni identification : ces écrans sont comparés par le code.

## À reprendre dans les lots suivants
- Lot 2 : remplacer les mocks `localStorage` (`mocks/workflow-placements.ts`, `mocks/node-meta.ts`, `mocks/execution-node-counts.ts`, `mocks/collections-store.ts`, `mocks/activity.ts`) ; création inactive (le brouillon fait POST puis PATCH `enabled: false`) ; supprimer le contournement d'invalidation de la requête du workflow (`useUpdateWorkflow`) ; numéro d'exécution séquentiel dans l'URL ; défauts de champs écrits à la création du nœud (déjà fait pour http.request).
- Lot 5 : le texte « Utilisable dans … » d'une identification annonce « Webhook » (prototype) ; il est retiré du réel tant qu'aucun déclencheur webhook n'existe.
- Lot 6 : la catégorie « Validation humaine » du créateur apparaît avec le type « Demander une validation » ; aujourd'hui masquée car vide.
- Lot 7 : remplacer les tables front de libellés d'identification par des descripteurs serveur ; statut « Expirée » et « Utilisée par N workflows » viennent d'un hachage d'id (`lib/credential-meta.ts`).
- Lot 8 : syntaxe d'expression `$('Nom').item.json.x` (astuce du nœud HTTP, panneau Variables, aperçu d'expression).
- Décision à confirmer par l'utilisateur : nom et interrupteur « Actif » passent par « Enregistrer » (le prototype ne persiste rien).
