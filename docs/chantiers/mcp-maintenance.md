# MCP Entretien : préparer le jeudi et importer la feuille (WS6a)

État : **backend et outils faits sur `feat/maintenance-ws6a` ; reste la recette ChatGPT et l'opération FerrisKey ci-dessous**. Le bouton « Activer » du webapp est WS6b.

## Goal

Un client MCP (ChatGPT) prépare la réunion du jeudi (« que dois-je programmer la semaine prochaine ? ») et importe la feuille papier en contrats **brouillons**. Le modèle propose, une personne valide : l'assistant ne rend jamais un contrat actif et ne valide jamais une visite.

## Context

- Issue #552, usages IA-1 et IA-2 de `docs/chantiers/entretien.md`. Spec MCP : `docs/chantiers/mcp.md`. Patron d'un lot d'outils : `docs/chantiers/mcp-doe-lot1.md`.
- `POST /maintenance/contracts` accepte déjà `status: DRAFT` (WS2). Un contrat brouillon n'a jamais d'échéance (WS3) : absent du récap, `Done` dans la grille.
- Hors périmètre : proposer des tournées (IA-3), saisie terrain en langage naturel (IA-4), devis de renouvellement (IA-5).

## Outils (3 de plus, 29 au total)

| Outil | Accès | Scope | Route de l'API |
|---|---|---|---|
| `list_maintenance_due(week)` | read | `mestier.read` | `GET /maintenance/recap?week=2026-W43` |
| `get_maintenance_contract(contract_id)` | read | `mestier.read` | `GET /maintenance/contracts/{id}` |
| `create_maintenance_contract_draft(...)` | write | `mestier.maintenance.write` | `POST /maintenance/contracts` avec `status: DRAFT` |

Tous prennent `organization` (slug), facultatif quand l'utilisateur n'a qu'une organisation.

`create_maintenance_contract_draft` prend : `customer_id`, `customer_context_id` (le site), `season { year, first_week, last_week }`, `lines` (chacune `label`, `cadence`, `minutes_per_intervention?`), et `reference?`, `manager_member_id?`, `amount_cents?`, `notes?`. La cadence est `{kind: "ROLLING", every_days}`, `{kind: "WINDOWS", windows: [{from_week, to_week}]}` ou `{kind: "ON_DEMAND"}`. Le lien au devis n'est pas exposé.

## Invariants

1. Le contrat créé par l'outil est toujours `DRAFT`. L'outil n'a aucun paramètre `status` ; s'il en reçoit un, il l'ignore. [test : serveur MCP, corps envoyé et schéma de l'outil]
2. Un jeton délégué (claim `act`) ne crée qu'un brouillon : `POST /maintenance/contracts` renvoie 403 s'il demande `ACTIVE` ou n'indique pas de statut. [test : e2e API]
3. La liste blanche des routes déléguées n'ouvre, pour l'entretien, que `GET .../maintenance/recap`, `GET .../maintenance/contracts/{contract_id}` et `POST .../maintenance/contracts`. Ni `PUT` (qui change le statut), ni archivage, ni duplication, ni intervention, ni validation, ni rejet, ni suppression, ni planification. [test : unitaires `delegated`, e2e API : un `PUT` délégué répond 403 `E_DELEGATED_ROUTE_FORBIDDEN`]
4. Un jeton sans `mestier.maintenance.write` ne peut pas appeler l'outil d'écriture : la politique Cerbos n'autorise un outil `write` du domaine `maintenance` qu'avec ce scope. `mestier.read` seul, ou `mestier.doe.write` seul, est refusé ; `mestier.maintenance.write` seul n'ouvre aucune écriture DOE. [test : `deploy/cerbos/policies/tests/mcp_tool_test.yaml`, gateway : refus du PDP, rien n'atteint le serveur MCP]
5. Un brouillon n'apparaît pas dans le récap avant son activation par une personne. [test : e2e API, brouillon créé avec un jeton délégué, récap sans lui, puis avec lui après activation]
6. Les trois outils n'appellent que ces trois routes. [test : serveur MCP]
7. Le catalogue `Tool` classe chaque outil en lecture ou en écriture sans cas par défaut (un `match` exhaustif) : un outil ajouté sans classement ne compile pas. [type + test : la liste exacte des outils d'écriture]
8. Les identifiants (client, site, conducteur) sont vérifiés avant tout appel : un identifiant mal formé n'écrit rien. [test]

## Politique, scopes, gateway

- Scope `mestier.maintenance.write` (constante `SCOPE_MAINTENANCE_WRITE`, domaine `maintenance`). `SCOPES_SUPPORTED` de `libs/mcp-tools` alimente `scopes_supported` des métadonnées RFC 9728 de la gateway.
- Cerbos : une règle de plus dans `deploy/cerbos/policies/mcp_tool.yaml` (`write` + domaine `maintenance` + le scope). Tout le reste est refusé comme avant.
- `apps/mcp-gateway/e2e/local_stack.py` : le jeton demande le nouveau scope, 29 outils, un jeton sans le scope est refusé par la politique (-32001), avec le scope la politique laisse passer et l'API juge.

## Trouvé en route

`Tool::access()` rangeait en lecture neuf outils DOE du lot 1 (`update_doe_supply`, `delete_doe_supply`, `add_doe_section`, `update_doe_section`, `delete_doe_section`, `reorder_doe_sections`, `add_doe_file`, `update_doe_file`, `delete_doe_file`) à cause d'un cas par défaut `_ => Read`. Avec la politique v1, un jeton `mestier.read` seul pouvait donc les appeler. Corrigé dans ce lot (invariant 7) : ils demandent désormais `mestier.doe.write`, comme la spec l'a toujours prévu.

## Trou de spec

L'outil d'écriture exige `customer_id` et `customer_context_id`, et aucun outil du catalogue ne liste les clients ni les sites. Ils se trouvent aujourd'hui par `get_maintenance_contract` (contrat existant) ou `list_projects` / `get_project` (un projet porte son client et son site). Un client sans projet ni contrat ne peut pas encore être importé. Suite proposée : un outil de lecture `list_customer_sites` (routes `GET /customers` et `GET /customers/{id}/customer-contexts` à ouvrir aux jetons délégués), à décider à part car il expose des données clients.

## Opération FerrisKey à faire à la main

Le scope doit exister dans le realm et être accordé aux clients, sinon ChatGPT ne peut pas le demander (FerrisKey ne laisse un client dynamique demander que les scopes standards, « Default » ou ouverts aux clients dynamiques). Hors du dépôt, par un humain :

1. `POST /realms/mestier/client-scopes` avec `{"name": "mestier.maintenance.write", "description": "Create maintenance contract drafts", "protocol": "openid-connect", "is_default": true, "dynamic_registration_allowed": true}` (comme `mestier.read` et `mestier.doe.write`, voir `docs/chantiers/mcp.md`, W6 ; vérifier les champs acceptés par la version déployée).
2. Un client déjà enregistré (ChatGPT, le client de la recette) ne reçoit pas un scope par défaut créé après lui : lui rattacher le scope depuis la console d'administration, puis le déconnecter et le reconnecter dans ChatGPT pour obtenir un nouveau jeton.
3. Vérifier : le claim `scope` du jeton contient `mestier.maintenance.write`.

Tant que le scope n'est pas accordé, les deux outils de lecture marchent et l'outil d'écriture est refusé par la politique : l'échec est sûr.

## Recette manuelle (ChatGPT)

Prérequis : l'opération FerrisKey ci-dessus, `mcp_enabled` activé sur l'organisation, un utilisateur avec `planning.manage`, un client et son site déjà créés.

1. « Que dois-je programmer en S43 ? » : l'assistant appelle `list_maintenance_due` et résume par commune, retards en tête.
2. « Montre-moi le contrat de la ligne Tonte GAUME » : `get_maintenance_contract` avec le `contract_id` de la ligne.
3. Joindre la photo de la feuille : « Crée les contrats de cette feuille pour GAUME, saison 2026 ». L'assistant lit la feuille et appelle `create_maintenance_contract_draft`. Vérifier : la réponse porte `status: DRAFT`.
4. Rouvrir le récap de la semaine : les lignes du brouillon n'y sont pas. Dans le webapp (WS6b), « Activer » le brouillon : ses lignes apparaissent.
5. Avec un jeton sans `mestier.maintenance.write`, demander la même création : refus de la politique, rien n'est écrit.
6. Demander « active ce contrat » ou « valide la visite de lundi » : l'assistant n'a pas d'outil pour cela.

## Verify

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings
SQLX_OFFLINE=true cargo test --workspace
(set -a; source .env; set +a; cargo test -p mestier-core --lib -- --ignored --test-threads=1)
(set -a; source .env; set +a; cargo test -p handlers-planning --test http_e2e -- --ignored --test-threads=1)
docker run --rm -v "$PWD/deploy/cerbos:/conf" ghcr.io/cerbos/cerbos:0.56.0 compile --tests=/conf/policies/tests /conf/policies
python3 apps/mcp-gateway/e2e/local_stack.py     # pile complète : FerrisKey, Cerbos, gateway, serveur MCP, API
```
