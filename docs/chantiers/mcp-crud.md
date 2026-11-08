# MCP — CRUD courant

Étend le catalogue de `libs/mcp-tools` (voir `mcp.md`) aux actions que l'on effectue couramment dans Mestier. Audit de départ : 20 endpoints HTTP sur 327 avaient un outil MCP.

## Périmètre

34 outils ajoutés, en 4 lots. Aucun outil ne supprime, n'émet ni n'envoie : on archive ou on change un statut.

| Lot | Domaine (`Tool::domain`) | Scope | Outils |
|---|---|---|---|
| 1 | `crm` | `mestier.crm.write` | `list_customers`, `get_customer`, `create_customer`, `update_customer`, `create_customer_contact`, `update_customer_contact`, `list_deals`, `get_deal`, `create_deal`, `update_deal`, `move_deal_stage` |
| 2 | `quote` | `mestier.quote.write` | `create_quote`, `update_quote`, `set_quote_status`, `create_quote_revision`, `export_quote_pdf` (lecture) |
| 3 | `planning` | `mestier.planning.write` | `list_tasks`, `get_task`, `get_planning`, `create_project`, `update_project`, `create_task`, `update_task`, `add_task_comment` |
| 4 | `reference` et `purchase` (écriture), `expense`, `reporting`, `invoice` (lecture) | `mestier.reference.write` | `create_product`, `update_product`, `list_suppliers`, `create_supplier`, `update_supplier`, `list_supplier_invoices`, `list_expenses`, `list_outstanding_invoices`, `get_profitability`, `get_worked_hours` |

`create_customer_contact` et `update_customer_contact` appartiennent au domaine `crm`. Les lectures restent sous `mestier.read`.

## Hors périmètre

- Les 4 outils de facturation (`create_invoice`, `record_invoice_payment`, `issue_invoice`, `send_invoice`) : reportés, ils auraient eu un scope `mestier.billing.write` séparé.
- Suppressions (client, deal, devis, tâche, produit).
- Webhooks, callback OAuth, API d'intake support, clés API, rotation de secrets, rôles, invitations, membres.
- Chat temps réel, automation, terrain.

## Décisions

1. **Un scope d'écriture par domaine.** `SCOPES_SUPPORTED` passe de 3 à 7 entrées. Chaque domaine d'écriture a sa règle dans `deploy/cerbos/policies/mcp_tool.yaml`, recopiée dans `deploy/helm/mestier/files/mcp_tool.yaml` (les deux fichiers doivent rester identiques).
2. **Pas de suppression.** Un test du même style que `nothing_activates_validates_or_deletes_maintenance` interdit `delete` dans le nom de tout outil des nouveaux domaines.
3. **Écriture sur un id global : vérifier l'organisation d'abord.** Les endpoints `customers/{id}`, `deals/{id}`, `quotes/{id}` ne portent pas l'organisation dans l'URL. Avant toute écriture, l'outil lit la ressource par `get_global` et refuse si `organization_id` diffère de celle choisie. La lecture seule le faisait déjà ; l'écriture s'y aligne.
4. **Statuts : le modèle ne peut pas franchir un état irréversible.** `set_quote_status` accepte `SENT`, `DECLINED` et `CANCELLED`. `ACCEPTED` (engagement commercial, déclenche la suite) et `SUPERSEDED` (posé par la révision) restent à la main.
5. **Arguments typés, comme l'existant.** Chaque outil a sa struct `schemars` avec des descriptions utiles au modèle. Pas de corps JSON libre.
6. **Un module par domaine** dans `apps/mcp-server/src` (`crm.rs`, `quote.rs`, `planning.rs`, `reference.rs`) pour les requêtes, comme `maintenance.rs` et `edit.rs`. `ApiClient` gagne `patch`.

## Acceptance

Pour chaque outil ajouté :

- Il est classé dans `access()`, `domain()` et `is_organization_scoped()` sans arme joker.
- `every_tool_round_trips_through_its_name` le couvre via `Tool::ALL`.
- Un test `apps/mcp-server/tests/server.rs` appelle l'outil contre une API simulée et vérifie le chemin, la méthode et le corps envoyés.
- Un test vérifie qu'une ressource d'une autre organisation est refusée (écritures sur id global).
- Cerbos : un test autorise l'outil avec son scope, le refuse avec seulement `mestier.read`, le refuse avec le scope d'un autre domaine.

Transverse :

- `the_write_tools_are_exactly_these` liste les nouveaux outils d'écriture.
- Aucun outil d'écriture ne contient `delete`, `issue`, `send`, `cancel`.
- `set_quote_status` refuse `ACCEPTED`, `SUPERSEDED`, `DRAFT` et toute valeur inconnue avec un message en français.
- `SCOPES_SUPPORTED` contient les 7 scopes, dans l'ordre : read, doe, maintenance, crm, quote, planning, reference.
- Aucun outil d'écriture hors DOE ne contient `delete`, `archive`, `issue`, `send` ou `cancel`. Les `delete_doe_*` existaient avant ce chantier.

## Plan

Lot 1 (`crm`) en pilote : il fixe le patron (module, `patch`, vérification d'organisation, tests). Les lots 2 à 4 le reprennent.

Chaque lot se termine par `cargo fmt --all --check`, `SQLX_OFFLINE=true cargo check --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, et `cerbos compile` sur les politiques si l'outil est disponible. Aucun nouveau `query!` n'est attendu : les outils passent par l'API HTTP.

## État

Les 4 lots sont implémentés (34 outils, catalogue de 29 à 63). Sont écrits mais non exécutés ici : les tests Cerbos, faute du binaire `cerbos`.

À faire hors du dépôt : déclarer les 4 nouveaux scopes (`mestier.crm.write`, `mestier.quote.write`, `mestier.planning.write`, `mestier.reference.write`) côté FerrisKey.

Écarts entre l'API et les outils, voulus :

- Les `PATCH` de client, deal, devis, projet, produit sont des remplacements complets : les outils lisent, fusionnent, puis envoient. Les `PATCH` de tâche sont partiels et partent tels quels. Le `PUT` d'un fournisseur reprend ses contacts et adresses.
- Aucun champ ne peut être vidé via MCP.
- `update_product` conserve l'état d'archivage du produit.
