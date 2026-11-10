# Tables

État : **spec, 2026-11-11** (branche `feat/tables`). Deuxième chantier du module Connaissances, après le Drive (#668, #670).

## Goal

Une entreprise crée ses propres tables (suivi de chantiers, inventaire, contacts fournisseurs…) avec des colonnes typées, les consulte en grille ou en Kanban, les remplit à la main, par import CSV, par un workflow ou par son assistant, et chaque ligne créée ou modifiée peut déclencher un workflow.

## Contexte

- Vision validée le 2026-11-10 : Drive, puis Tables façon Airtable, puis Documents par blocs façon Notion qui afficheront des vues de table.
- Réponses du 2026-11-11 (recommandations retenues) : Tables avant Documents ; colonnes 2a (texte, texte long, nombre, montant, date, case à cocher, choix unique, choix multiple, lien vers client / projet / devis, fichier du Drive, membre) ; ni liens entre tables ni formules ; vues Grille et Kanban avec filtres, tris et colonnes masquées enregistrés ; événements « ligne créée » et « ligne modifiée » plus une action de workflow ; import et export CSV ; MCP en lecture et écriture de lignes, sans suppression.
- Existant réutilisé : permissions `VIEW_KNOWLEDGE` / `MANAGE_KNOWLEDGE` (`knowledge.view`, `knowledge.manage`), catalogue d'événements (`domain/automation/catalogue.rs`), catalogue d'actions (`domain/automation/connector/actions`), garde-fou du dispatcher (un workflow ne se relance pas sur ses propres événements), scope MCP `mestier.knowledge.write` et règle Cerbos par domaine `knowledge`, plateau Kanban de la planification (glisser-déposer HTML natif).

## Décisions

1. **Valeurs des lignes en JSONB**, une clé par id de colonne (ADR 0015). Pas de table par table, pas de ligne par cellule.
2. **Colonne principale** : la première colonne d'une table est de type texte, ne se supprime pas et ne change pas de type. Elle sert de titre à la ligne (Kanban, événements, MCP, liens).
3. **Types de colonnes** (`ColumnKind`) et valeur stockée :

   | Type | Valeur JSON | Bornes |
   |---|---|---|
   | `text` | chaîne | 1 000 caractères |
   | `long_text` | chaîne | 20 000 caractères |
   | `number` | nombre | décimales affichées 0 à 4 (réglage de la colonne) |
   | `currency` | entier, centimes d'euro | i64 |
   | `date` | `"AAAA-MM-JJ"` | |
   | `checkbox` | booléen | |
   | `single_select` | id d'option | option existante |
   | `multi_select` | liste d'ids d'option | options existantes, sans doublon |
   | `link` | `{ "kind": "customer" \| "project" \| "quote", "id": uuid }` | même organisation ; le genre est fixé par la colonne |
   | `file` | id d'un article `file` du Drive | même organisation |
   | `member` | id utilisateur | membre de l'organisation |

   Une cellule vide n'a pas de clé. `null` reçu = vider la cellule.
4. **Changer le type d'une colonne** n'est permis que sans perte : vers `text` ou `long_text` depuis n'importe quel type (la valeur devient son texte affiché), entre `text` et `long_text` (refusé si une valeur dépasse la borne), entre `single_select` et `multi_select` (vers le choix unique : refusé si une ligne a plusieurs options). Les autres changements : créer une nouvelle colonne.
5. **Écriture cellule par cellule** : une modification de ligne n'envoie que les cellules changées, fusionnées en base (`values || patch`, clés vidées retirées). Deux personnes sur deux cellules différentes ne s'écrasent pas ; sur la même cellule, la dernière écriture gagne. Pas de temps réel dans ce chantier : la grille se recharge après chaque écriture et au retour sur l'onglet.
6. **Vues** : `grid` ou `kanban`, chacune avec filtres (ET uniquement), jusqu'à 3 tris, colonnes masquées et largeurs ; le Kanban groupe par une colonne `single_select`, avec une pile « Sans valeur ». Chaque table a au moins une vue. Filtre et tri sont faits par le serveur.
7. **Opérateurs de filtre** par type : texte (contient, est, est vide, n'est pas vide) ; nombre et montant (=, ≠, <, ≤, >, ≥, vide, non vide) ; date (avant, après, le, vide, non vide) ; case à cocher (cochée, non cochée) ; choix unique (est, n'est pas, parmi, vide) ; choix multiple (contient l'un de, contient tous, vide) ; lien, fichier, membre (est, vide, non vide).
8. **Événements** `table_row.created` et `table_row.updated` (v1). Charge utile : la table (`id`, `name`), la ligne (`id`, `title`, `values` par id de colonne, `fields` par nom de colonne avec la valeur affichée) et, pour la modification, `changed` (noms des colonnes modifiées). Pour réagir à une seule table, le workflow filtre sur `table.id` avec un nœud de condition. **L'import CSV n'émet aucun événement de ligne** : importer 5 000 lignes ne lance pas 5 000 workflows.
9. **Actions de workflow** : `mestier.table_row.create`, `mestier.table_row.update` (par id de ligne), `mestier.table_row.find` (lecture : lignes dont une colonne vaut une valeur, 50 au plus). Les valeurs sont un objet JSON par nom de colonne, converties et validées comme une saisie.
10. **CSV** :
    - Import dans une nouvelle table (en-têtes = colonnes, type deviné : nombre, date `JJ/MM/AAAA` ou `AAAA-MM-JJ`, case à cocher `oui/non/vrai/faux/x`, sinon texte) ou dans une table existante (colonnes reconnues par nom sans tenir compte de la casse ; les autres sont ignorées et listées dans le compte rendu).
    - Séparateur `;` ou `,` détecté, UTF-8 avec ou sans BOM, repli Windows-1252 (exports Excel français). 5 Mo et 10 000 lignes au plus. Une cellule invalide est laissée vide et comptée dans le compte rendu ; le fichier n'est pas refusé pour ça.
    - Export d'une vue (ses filtres, tris et colonnes visibles) : `;`, UTF-8 avec BOM, décimales avec virgule, montants en euros, dates `JJ/MM/AAAA`, choix par leur libellé, liens par leur nom.
11. **Droits** : `knowledge.view` lit tables, vues, lignes et export ; `knowledge.manage` fait tout le reste (structure, lignes, vues, import).
12. **MCP** (domaine `knowledge`) : lecture `list_tables`, `get_table` (colonnes, options, vues), `list_table_rows` (vue facultative, recherche texte simple, 100 au plus) ; écriture `create_table_row`, `update_table_row`. Ni suppression, ni changement de structure.
13. **Liens morts** : un client, projet, devis, fichier ou membre supprimé laisse sa valeur ; la lecture l'affiche « élément supprimé » et ne tombe jamais en erreur.

## Invariants

1. Toute lecture et écriture est bornée par `org_id` ; une table, colonne, vue, ligne, ou une cible de lien, de fichier ou de membre d'une autre organisation est « introuvable ». [test]
2. Une valeur stockée respecte toujours le type de sa colonne. [type : `CellValue` construit seulement par `CellValue::parse(kind, config, json)` ; test]
3. Les clés de `values` sont des colonnes existantes de la table : supprimer une colonne retire sa clé de toutes les lignes, dans la même transaction. [test]
4. Supprimer une option de choix la retire des lignes qui l'utilisent. [test]
5. La colonne principale existe toujours, de type texte. [constructeur ; test]
6. Une table a toujours au moins une vue. [test]
7. Noms uniques sans tenir compte de la casse : table dans l'organisation, colonne dans la table, vue dans la table. [index unique ; test]
8. Bornes : 50 colonnes, 10 000 lignes, 100 options par colonne de choix, 20 vues par table, 200 tables par organisation. [constructeur ou use case, `Result` ; test]
9. Un changement de type ne perd jamais une valeur. [test]
10. L'import CSV n'émet aucun événement ; toute autre création ou modification de ligne en émet exactement un, dans la transaction de l'écriture. [test]
11. Le MCP ne supprime rien et ne change pas la structure. [catalogue `Tool` ; test Cerbos]
12. Migration réversible. [test : up, down, up]

## Hors périmètre

- Liens entre tables, formules, champs calculés, cumuls.
- Vues Calendrier et Galerie, filtres OU et groupes de filtres.
- Temps réel entre navigateurs, historique d'une ligne, commentaires.
- Droits par table, partage public, formulaire public de saisie.
- Plusieurs fichiers ou plusieurs liens dans une cellule.
- Vue de table dans un document (chantier Documents).
- Déclencheur filtré par table dans l'éditeur de workflow (on passe par un nœud de condition).

## Design

- **Migration** `20261119000001_create_knowledge_tables` :
  - `knowledge_tables (id, org_id, name, description, created_by, created_at, updated_at)`, unique `(org_id, lower(name))`.
  - `knowledge_table_columns (id, table_id → CASCADE, org_id, name, kind, config JSONB, position, is_primary, created_at)`, unique `(table_id, lower(name))`.
  - `knowledge_table_rows (id, table_id → CASCADE, org_id, values JSONB, created_by, updated_by, created_at, updated_at)`, index `(table_id, created_at)` et GIN sur `values`.
  - `knowledge_table_views (id, table_id → CASCADE, org_id, name, kind, config JSONB, position, created_at, updated_at)`, unique `(table_id, lower(name))`.
- **Core** `domain/knowledge/tables/` : `Table`, `Column`, `ColumnKind`, `ColumnConfig` (options, décimales, genre de lien), `CellValue`, `RowValues`, `Row`, `View`, `ViewKind`, `ViewConfig`, `Filter`, `Sort`, conversions de type, `csv` (lecture, détection, écriture), événements. Ports `KnowledgeTableRepository` (tables, colonnes, vues) et `KnowledgeTableRowRepository` (lignes : insertion, patch, suppression, liste filtrée triée paginée, comptage, insertion en lot). La requête filtrée est construite avec `QueryBuilder` (filtres dynamiques), chaque opérateur traduit en SQL paramétré, jamais de texte utilisateur dans le SQL.
- **Application** `application/knowledge/tables.rs` : use cases avec `knowledge.view` / `knowledge.manage`, `emitter` sur la création et la modification de ligne.
- **API** dans `handlers-knowledge`, sous `/api/v1/organizations/{org}/knowledge/tables` : tables (liste, création, lecture avec colonnes et vues, renommage, suppression), colonnes (création, modification, suppression, ordre), vues (création, modification, suppression), lignes (liste par vue avec `limit`/`offset` ou filtres ad hoc, création, modification partielle, suppression d'une ou plusieurs), import (`multipart`, nouvelle table ou table existante, compte rendu), export (`text/csv` d'une vue). Les lignes lues portent les libellés résolus des liens, fichiers et membres.
- **Automatisation** : `domain/knowledge/tables/events.rs` (`descriptors`, `emitted_events`) ajouté à `event_catalogue()` ; un fichier d'actions `tables.rs` dans `connector/actions` et son exécution dans `infrastructure/automation/connectors/actions`.
- **MCP** : 5 entrées dans `libs/mcp-tools` (`Tool`), outils dans `apps/mcp-server/src/tables.rs`, cas ajoutés aux tests Cerbos.
- **Webapp** : section « Tables » du module Connaissances.
  - `/connaissances/tables` : liste des tables, création (vide ou depuis un CSV).
  - `/connaissances/tables/$tableId?vue=` : onglets de vues, barre (filtres, tris, colonnes masquées, import, export), grille éditable (`@tanstack/react-table` + `@tanstack/react-virtual`, nouvelle dépendance pour 10 000 lignes ; chargement par pages de 100), menu d'en-tête de colonne (renommer, type, options, masquer, trier, supprimer), fiche d'une ligne avec tous les champs, Kanban par choix unique (glisser-déposer HTML natif, comme la planification).
  - Éditeurs de cellule par type, purs dans `ui/` ; validation partagée dans `lib/`.

## Lots

| Lot | Contenu | Qui |
|---|---|---|
| T1 | Migration, domaine (types, valeurs, conversions, bornes), dépôts, use cases, API tables / colonnes / vues / lignes, filtres et tris, tests | moi |
| T2 | Événements et actions de workflow | sous-agent, après T1 |
| T3 | CSV : import et export (domaine + API) | sous-agent, après T1 |
| T4 | MCP et tests Cerbos | sous-agent, après T1 |
| T5 | Webapp : liste, grille, vues, Kanban, fiche, import / export | sous-agent, après `pnpm gen:api` de T1 |

T2 à T5 se font en parallèle une fois T1 figé. Une branche `feat/tables`, une PR.

## Acceptance

T1
1. Créer une table « Chantiers » : elle a une colonne principale texte « Nom » et une vue Grille.
2. Ajouter une colonne de chaque type, puis une ligne avec une valeur valide dans chacune : la ligne relue rend les mêmes valeurs.
3. Une valeur invalide (texte dans un nombre, option inconnue, date `31/02/2026`, client d'une autre organisation) est refusée en 422 ; rien n'est enregistré.
4. Deux modifications de la même ligne sur deux cellules différentes, envoyées l'une après l'autre avec l'état initial : les deux valeurs sont gardées.
5. Supprimer une colonne : sa valeur disparaît de toutes les lignes. Supprimer une option : elle disparaît des lignes qui l'avaient.
6. Supprimer la colonne principale, ou la dernière vue : refusé.
7. Passer une colonne nombre en texte : les valeurs deviennent leur texte. Passer un choix multiple en choix unique alors qu'une ligne a deux options : refusé, rien ne change. Passer un nombre en date : refusé.
8. Filtrer « Montant > 1 000 € » et « Statut est En cours », trier par date décroissante : seules les bonnes lignes, dans le bon ordre, avec la pagination.
9. Une 51e colonne, une 10 001e ligne, deux colonnes « Statut » et « statut » : refusées.
10. Une autre organisation ne voit ni ne modifie aucune table, colonne, vue ou ligne (404).
11. Sans `MANAGE_KNOWLEDGE`, toute écriture est refusée (403) ; avec `VIEW_KNOWLEDGE`, la lecture marche.
12. Un client lié puis supprimé : la ligne se lit toujours, la cellule indique « élément supprimé ».

T2
13. Créer une ligne : un événement `table_row.created` est enregistré avec `table.name`, `row.title` et `fields` par nom de colonne.
14. Modifier deux cellules : `table_row.updated` avec `changed` = ces deux colonnes. Une écriture qui ne change rien n'émet rien.
15. Un workflow « ligne créée » → `mestier.table_row.update` (cocher « Traité ») tourne une fois, sans se relancer sur sa propre modification.
16. `mestier.table_row.find` rend les lignes dont « Statut » vaut « En cours ».
17. Le catalogue d'événements et le catalogue d'actions restent alignés (tests anti-dérive existants).

T3
18. Un CSV exporté d'Excel français (`;`, Windows-1252, « Désignation », « 12,50 », « 05/11/2026 ») crée une table avec une colonne nombre et une colonne date justes, et les accents intacts.
19. Import dans une table existante avec une colonne inconnue et une date invalide : les lignes sont créées, la colonne inconnue et la cellule invalide sont dans le compte rendu.
20. Un fichier de plus de 5 Mo ou de plus de 10 000 lignes est refusé, sans rien créer.
21. L'import n'émet aucun événement de ligne.
22. L'export d'une vue filtrée ne contient que ses lignes et ses colonnes visibles, et s'ouvre dans Excel avec les accents.

T4
23. L'assistant liste les tables, lit une table, liste ses lignes, en crée une et en modifie une.
24. Sans `mestier.knowledge.write`, l'écriture est refusée par Cerbos ; aucun outil de suppression ou de structure n'existe.

T5
25. Créer une table, ajouter des colonnes, saisir des lignes au clavier dans la grille (Entrée, Tab, Échap), recharger : tout est là.
26. Créer une vue Kanban sur « Statut », déplacer une carte : la cellule change.
27. Enregistrer un filtre et un tri dans une vue, changer de vue, revenir : ils sont toujours là.
28. Importer un CSV, exporter la vue.
29. Une table de 10 000 lignes défile sans saccade.

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings
SQLX_OFFLINE=true cargo test --workspace
cargo test -p mestier-core -- --ignored knowledge
cargo test -p handlers-knowledge --test http_e2e -- --ignored
cd apps/webapp && pnpm test && pnpm build
```

Plus une recette navigateur de T5 et un appel MCP réel pour T4.

## Risques

| Risque | Parade |
|---|---|
| Filtres JSONB lents sur 10 000 lignes | Borne de 10 000 lignes, index GIN, mesure sur 10 000 lignes dans les tests d'intégration ; colonnes générées plus tard si besoin |
| Injection par les filtres dynamiques | `QueryBuilder` avec paramètres liés ; ids de colonne validés contre la table avant de construire la requête |
| Un workflow qui modifie une ligne relance une chaîne A → B → A | Comportement existant du dispatcher (seul le retour sur soi est bloqué), inchangé ici |
| Un import massif bloque l'API | Bornes 5 Mo / 10 000 lignes, insertion en lot dans une transaction |
| Conflits d'écriture sans temps réel | Écriture par cellule ; temps réel hors périmètre |
