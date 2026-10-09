# MCP : Mestier dans ChatGPT et les autres LLM

État : **W0 intégré le 2026-10-09 ; W5a en PR #540 ; W1 confié à un agent FerrisKey par l'utilisateur ; W2, W3, W4 en cours**.

## Goal

Un utilisateur Mestier colle l'URL du serveur MCP dans son client LLM (ChatGPT web ou autre), se connecte avec son compte Mestier, et l'IA travaille sur toutes les organisations auxquelles il a accès, avec ses droits à lui et pas plus.

## Context

- Demande du 2026-10-09, après la démo du DOE au client.
- Architecture imposée : serveur MCP en Rust, gateway MCP en Rust qui fait PEP, PDP externe, échange de token pour que l'API soit appelée avec l'identité de l'utilisateur, OAuth2 côté client LLM, et rien d'autre à saisir que l'URL.
- ADR : [0004, gateway PEP et échange de token](../adr/0004-mcp-gateway-pep-token-exchange.md), [0005, Cerbos comme PDP provisoire](../adr/0005-cerbos-interim-pdp.md).
- Décisions du 2026-10-09 :
  1. CIMD et DCR s'ajoutent dans FerrisKey même ; ils font partie du chantier.
  2. PDP simple pour l'instant (Cerbos), FerrisKey à terme. La gateway parle AuthZEN 1.0, le changement de PDP sera une URL.
  3. L'échange de token se fait dans la gateway.
  4. Gateway et serveur MCP vivent dans le dépôt Mestier pour l'instant.
  5. Outils v1 : lecture sur projets, DOE, catalogue, devis, factures ; écriture DOE seulement ; ni suppression ni émission de facture.
  6. Un connecteur, un compte, toutes les organisations de l'utilisateur : chaque outil est scopé par organisation.
  7. Chaque organisation autorise ou non l'usage du MCP par ses membres (interrupteur dans Paramètres › Organisation, désactivé par défaut, modifiable avec `MANAGE_ORG`).

### Faits vérifiés

- Spec MCP 2025-11-25 : le serveur MCP est une ressource OAuth 2.1, publie ses métadonnées RFC 9728, répond 401 avec `WWW-Authenticate: Bearer resource_metadata=...`, refuse tout token qui ne lui est pas destiné, ne repasse jamais le token reçu à une API en aval.
- Spec MCP 2026-07-28 : protocole sans état (plus de handshake ni de session), le serveur d'autorisation renvoie `iss` dans la réponse d'autorisation (RFC 9207) et le client le vérifie, préférence pour CIMD sur DCR, routage possible par en-têtes sans lire le corps.
- ChatGPT : préfère CIMD (`client_id` = URL d'un document de métadonnées), sinon DCR (RFC 7591). Sans l'un des deux, il ne peut pas finir le parcours. En CIMD il utilise `none` (client public + PKCE) ou `private_key_jwt`.
- FerrisKey (`/opt/ferrislabs/ferriskey/ferriskey`, branche `main`) : échange de token RFC 8693 présent (`audience`, `resource`, `scope`, `actor_token`, politiques par client et audience cible, `allow_delegation`), PKCE présent, `openid-configuration` présent. **Absents** : CIMD, DCR. **Non vérifiés** : métadonnées RFC 8414, `resource` (RFC 8707) dans le parcours d'autorisation, `iss` RFC 9207 dans la réponse d'autorisation.
- Mestier : `libs/authz` parle AuthZEN 1.0. **L'API ne vérifie pas l'audience des tokens** (`FerrisKeyRepository::new(issuer, None)`, `libs/core/src/application/mod.rs:232`) : un token émis pour la gateway serait accepté par l'API.
- Cerbos PDP ≥ 0.48 implémente l'API AuthZEN 1.0. `rmcp` (SDK MCP officiel en Rust) est en 3.5.1.

## Glossary

| Terme | Définition | Synonyme rejeté |
|---|---|---|
| Gateway | Le service Rust exposé aux clients LLM : ressource OAuth, PEP, échange de token, proxy vers le serveur MCP | proxy, BFF |
| Serveur MCP | Le service Rust qui expose les outils et appelle l'API Mestier ; jamais exposé sur Internet | agent, connecteur |
| PEP | Le point qui applique la décision : la gateway | — |
| PDP | Le point qui décide : Cerbos, interrogé en AuthZEN | moteur de règles |
| Token client | Token émis par FerrisKey au client LLM, audience = la gateway | token utilisateur |
| Token délégué | Token issu de l'échange, audience = l'API Mestier, sujet = l'utilisateur, `act` = la gateway | token technique |
| Outil | Une fonction MCP (`tools/call`), lecture ou écriture | tool, action |

## Invariants

1. Le token client n'atteint jamais l'API Mestier ni le serveur MCP. [test : e2e, le serveur MCP reçoit un token d'audience API avec `act`]
2. L'API Mestier refuse un token dont l'audience n'est pas la sienne. [test : intégration `auth`, un token d'audience gateway est refusé 401]
3. La gateway refuse un token dont l'audience n'est pas la sienne, expiré, ou d'un autre émetteur. [test : unitaires gateway]
4. Aucun outil ne s'exécute sans décision `true` du PDP ; PDP injoignable = refus. [test : gateway, PDP coupé → refus]
5. Un outil n'agit que sur une organisation dont l'utilisateur est membre ; l'API reste juge des droits métier (`planning.manage`, etc.). [test : e2e, organisation étrangère → erreur, pas de donnée]
6. Le catalogue d'outils est fermé : un nom inconnu est refusé par la gateway avant le PDP. [type : enum `Tool` côté gateway, `FromStr`]
7. Aucun outil de suppression ni d'émission de facture n'existe en v1, et la politique Cerbos refuse toute action d'écriture hors DOE. [type : absence dans l'enum ; test : politique Cerbos]
8. Le serveur MCP n'accepte que le trafic de la gateway. [déploiement : réseau privé ; test : le serveur refuse un appel sans token délégué valide]
10. Une organisation qui n'a pas activé le MCP est inaccessible par un token délégué : l'API refuse (403) tout appel portant le claim `act` de la gateway sur cette organisation, et ne la liste pas. [test : intégration API ; e2e @spec-mcp-12]
9. Les tokens délégués en cache ne servent qu'au couple (utilisateur, client LLM) qui les a obtenus et expirent avant leur `exp`. [test : unitaires cache]

## Out of scope

- PDP FerrisKey (prévu à terme, la gateway n'aura qu'à changer d'URL AuthZEN).
- Déploiement Kubernetes, Helm, DNS public : un lot à part, avec `devops-safety`.
- Outils d'écriture hors DOE, suppressions, émission de devis ou de factures.
- Ressources et prompts MCP : outils seulement.
- Interface de consentement fine par outil côté FerrisKey (le consentement OAuth standard suffit en v1).

## Design

### Parcours

1. Le client LLM appelle `POST https://mcp.<domaine>/mcp` sans token → la gateway répond 401, `WWW-Authenticate: Bearer resource_metadata="https://mcp.<domaine>/.well-known/oauth-protected-resource"`.
2. Métadonnées RFC 9728 : `resource` = l'URL de la gateway, `authorization_servers` = le realm FerrisKey, `scopes_supported` = `mestier.read`, `mestier.doe.write`.
3. Le client lit les métadonnées RFC 8414 du realm, s'identifie par CIMD (ou s'enregistre en DCR), fait code d'autorisation + PKCE avec `resource` = la gateway. L'utilisateur se connecte avec son compte Mestier. FerrisKey renvoie `iss` (RFC 9207).
4. Chaque requête MCP porte le token client. La gateway : valide le token (JWKS, `iss`, `aud`, `exp`) → identifie l'outil (en-têtes 2026-07-28 si présents, sinon corps JSON-RPC) → refuse un outil inconnu → interroge le PDP en AuthZEN → si refus, erreur JSON-RPC → sinon obtient (ou relit en cache) un token délégué par échange RFC 8693 (`subject_token` = token client, `audience` = API Mestier, `actor_token` = token client-credentials de la gateway) → transmet la requête au serveur MCP avec `Authorization: Bearer <token délégué>`.
5. Le serveur MCP exécute l'outil en appelant l'API Mestier avec le token délégué. L'API applique ses propres contrôles (appartenance à l'organisation, permissions).

### Organisation

- Interrupteur `mcp_enabled` sur l'organisation (colonne `organizations.mcp_enabled BOOLEAN NOT NULL DEFAULT false`, migration réversible). Lu par l'API, jamais par la gateway ni le PDP : la donnée vit là où elle est administrée.
- L'API reconnaît un appel MCP au claim `act` du token délégué dont le `client_id` (ou `sub`) est le client de la gateway (configuré : `AUTH_MCP_GATEWAY_CLIENT_ID`). Sur une telle requête : organisation sans `mcp_enabled` → 403 « l'accès par IA est désactivé pour cette organisation » ; liste des organisations de l'utilisateur filtrée.
- Outil `list_organizations` (lecture) ; tous les autres prennent `organization` (slug). Facultatif quand l'utilisateur n'a qu'une organisation : le serveur MCP la résout. Sinon, erreur qui liste les slugs possibles.
- Le PDP reçoit l'organisation dans la ressource AuthZEN ; l'API reste juge de l'appartenance (invariant 5).

### Décision PDP (AuthZEN 1.0, `POST /access/v1/evaluation`)

```json
{
  "subject":  { "type": "user", "id": "<sub>", "properties": { "roles": ["user"], "client_id": "<client LLM>", "scopes": ["mestier.read"] } },
  "action":   { "name": "<outil>" },
  "resource": { "type": "mcp_tool", "id": "<outil>", "properties": { "access": "read|write", "domain": "doe|project|...", "organization": "<slug|null>" } },
  "context":  {}
}
```

Politique Cerbos v1 : tout outil `read` autorisé avec le scope `mestier.read` ; outil `write` autorisé seulement si `domain == "doe"` et scope `mestier.doe.write` ; tout le reste refusé. Les droits métier restent dans l'API (pas de duplication des rôles Mestier dans Cerbos en v1).

### Outils v1 (catalogue fermé)

| Outil | Accès | Domaine | API Mestier |
|---|---|---|---|
| `list_organizations` | read | org | organisations de l'utilisateur |
| `list_projects`, `get_project` | read | project | projets |
| `list_does`, `get_doe` | read | doe | `GET /does`, `GET /does/{id}` |
| `list_doe_models` | read | doe | modèles |
| `search_products`, `get_product` | read | catalog | produits (avec fiche technique) |
| `list_quotes`, `get_quote` | read | quote | devis |
| `list_invoices`, `get_invoice` | read | invoice | factures |
| `create_doe` | write | doe | `POST /projects/{id}/doe` |
| `add_doe_supply_from_product` | write | doe | `POST /does/{id}/supplies/draft` puis `PUT /does/{id}` |
| `update_doe_header` | write | doe | `PUT /does/{id}` (titre, lieu, lot, date, rédacteur, indice) |
| `export_doe_pdf` | read | doe | `GET /does/{id}/pdf`, renvoyé en ressource intégrée `application/pdf` (plafond 10 Mo, sinon erreur explicite) |

### Composants

- `apps/mcp-gateway` (binaire) : axum ; ports `TokenVerifier`, `PolicyDecisionPoint` (adapter AuthZEN HTTP), `TokenExchanger` (adapter FerrisKey RFC 8693), cache des tokens délégués ; proxy HTTP en flux vers le serveur MCP. Dispatch statique (génériques), pas de `dyn`, pas d'`async-trait`.
- `apps/mcp-server` (binaire) : `rmcp` 3.x, transport Streamable HTTP, outils via `#[tool]` ; client HTTP vers l'API Mestier (DTO du crate `handlers-*` réutilisés en lecture seule si la dépendance reste légère, sinon structs locales).
- `deploy/cerbos/` : configuration et politiques Cerbos, avec leurs tests (`cerbos compile`), et un service `cerbos` dans `docker-compose.yml`.
- `libs/auth` : l'API Mestier vérifie l'audience (invariant 2). Le webapp doit continuer à marcher : vérifier l'audience de ses tokens avant d'activer.
- FerrisKey (dépôt `ferriskey`, PR séparée) : CIMD, DCR, métadonnées RFC 8414 avec `registration_endpoint` et `client_id_metadata_document_supported`, `iss` RFC 9207, `resource` RFC 8707 dans le code d'autorisation (audience du token = `resource`).
- Configuration FerrisKey (realm `mestier`) : client confidentiel `mestier-mcp-gateway` avec politique d'échange `target_audience = <API Mestier>`, `allow_delegation = true` ; audience de l'API Mestier déclarée.

### Errors

- Gateway → client LLM : 401 avec `WWW-Authenticate` (token absent ou invalide), 403 `insufficient_scope` (scope manquant), erreur JSON-RPC `-32001` « refusé par la politique » (PDP), `-32002` « outil inconnu », 502 si l'échange de token ou le serveur MCP échoue (sans détail interne).
- Serveur MCP → LLM : résultat d'outil `isError: true` avec le message de l'API en français quand il est utile (409 du DOE, organisation inconnue).

## Acceptance (Gherkin)

```gherkin
@spec-mcp-1 Scenario: découverte depuis la seule URL
  When un client appelle /mcp sans token
  Then la réponse est 401 avec resource_metadata pointant vers /.well-known/oauth-protected-resource
  And ce document désigne le realm FerrisKey et la ressource de la gateway

@spec-mcp-2 Scenario: enregistrement sans configuration manuelle
  Given un document CIMD publié par un client
  When le client démarre le code d'autorisation avec client_id = l'URL du document
  Then FerrisKey accepte le client et termine le parcours avec PKCE, et la réponse porte iss
  And sans CIMD, un POST sur registration_endpoint (DCR) crée un client utilisable

@spec-mcp-3 Scenario: l'API refuse le token client
  Given un token client d'audience gateway
  When il appelle directement l'API Mestier
  Then la réponse est 401

@spec-mcp-4 Scenario: appel d'outil de bout en bout avec l'identité de l'utilisateur
  Given un utilisateur membre de deux organisations
  When le client appelle list_organizations puis list_does pour la seconde
  Then il reçoit les DOE de la seconde organisation
  And le serveur MCP a reçu un token d'audience API avec sub = l'utilisateur et act = la gateway

@spec-mcp-5 Scenario: organisation étrangère
  When le client appelle list_does pour une organisation dont l'utilisateur n'est pas membre
  Then l'outil renvoie une erreur et aucune donnée

@spec-mcp-6 Scenario: politique
  Given un token avec le seul scope mestier.read
  When le client appelle create_doe
  Then la gateway renvoie le refus de politique sans contacter le serveur MCP

@spec-mcp-7 Scenario: PDP injoignable
  Given Cerbos arrêté
  When le client appelle list_projects
  Then la gateway refuse

@spec-mcp-8 Scenario: outil inconnu
  When le client appelle delete_project
  Then la gateway renvoie « outil inconnu » sans interroger le PDP

@spec-mcp-9 Scenario: droits métier
  Given un membre sans planning.manage avec le scope mestier.doe.write
  When il appelle create_doe
  Then la politique autorise, l'API refuse 403, et l'outil renvoie une erreur lisible

@spec-mcp-10 Scenario: export PDF
  When le client appelle export_doe_pdf sur un DOE existant
  Then il reçoit une ressource application/pdf qui commence par %PDF

@spec-mcp-12 Scenario: organisation qui n'a pas activé le MCP
  Given un utilisateur membre de A (MCP activé) et de B (MCP désactivé, valeur par défaut)
  When le client appelle list_organizations puis list_does pour B
  Then list_organizations ne renvoie que A
  And list_does pour B renvoie une erreur indiquant que l'accès par IA est désactivé
  And le même utilisateur garde l'accès à B depuis le webapp

@spec-mcp-13 Scenario: réglage de l'organisation
  Given un membre avec MANAGE_ORG
  When il active « Autoriser l'accès par IA (MCP) » dans Paramètres › Organisation
  Then l'organisation devient accessible par MCP
  And un membre sans MANAGE_ORG ne voit pas l'interrupteur modifiable et l'API lui refuse la modification (403)

@spec-mcp-11 Scenario: ChatGPT
  Given le serveur exposé sur une URL publique de test
  When on ajoute l'URL comme connecteur dans ChatGPT
  Then la connexion FerrisKey aboutit et ChatGPT liste les DOE de l'utilisateur (vérification manuelle)
```

## Simulation

Non. Pas d'état concurrent propre : le cache de tokens délégués est testé en unitaire (expiration, isolation par couple).

## Workstreams

| Id | Mission | Dépend de | Où |
|---|---|---|---|
| W0 | Contrats : spec, ADR, squelettes `apps/mcp-gateway` et `apps/mcp-server`, enum `Tool` et catalogue, contrat gateway → serveur, service Cerbos dans compose. **Orchestrateur.** | — | mestier |
| W1 | FerrisKey : CIMD, DCR, RFC 8414, `iss` RFC 9207, `resource` RFC 8707. Mené par l'utilisateur avec un agent dédié, à partir de `mcp-w1-ferriskey-prompt.md` | — | ferriskey |
| W2 | Gateway : métadonnées RFC 9728, 401, validation JWT, identification de l'outil, PEP AuthZEN, échange de token + cache, proxy | W0 | mestier |
| W3 | Serveur MCP : `rmcp`, outils v1, client API Mestier, scoping organisation | W0 | mestier |
| W4 | Politiques Cerbos + tests | W0 | mestier |
| W5a | API Mestier : vérification d'audience sans casser le webapp. **PR à part**, depuis `main` | — | mestier |
| W5b | `mcp_enabled` : migration, réglage, refus 403 et filtrage sur token délégué | W5a | mestier |
| W7 | Webapp : interrupteur MCP dans Paramètres › Organisation | W5 + client API régénéré | mestier |
| W6 | Bout en bout : realm FerrisKey local configuré, e2e @spec-mcp-1 à 10, recette ChatGPT @spec-mcp-11 | W1 à W5 | mestier |

W1, W2, W3, W4, W5 partent ensemble après W0. W7 dès W5b intégré. W6 en dernier.

Fichiers : W2 possède `apps/mcp-gateway/**`, son `Cargo.toml` compris ; W3 `apps/mcp-server/**` idem ; W4 `deploy/cerbos/**` et le service `cerbos` de `docker-compose.yml` ; W5 `libs/auth/**` et la ligne de construction dans `libs/core/src/application/mod.rs` (signalée). Orchestrateur : `Cargo.toml` du workspace, `libs/mcp-tools/**` (catalogue figé), `libs/args/**`, docs. Régénérés : `Cargo.lock`, `.sqlx/` (W5 n'en a normalement pas besoin).

## Frozen contracts

Écrits en W0 dans `libs/mcp-tools/src/lib.rs`, partagé par la gateway et le serveur :

- `enum Tool` (les 16 outils ci-dessus), `name()`, `access() -> Access { Read, Write }`, `domain()`, `is_organization_scoped()`, `FromStr` (nom MCP exact, sinon `UnknownTool`), constantes `SCOPE_READ`, `SCOPE_DOE_WRITE`, `CLIENT_HEADER`.
- Contrat gateway → serveur : `POST /mcp` interne, corps JSON-RPC inchangé, `Authorization: Bearer <token délégué>`, `X-Mestier-Mcp-Client: <client_id LLM>` (journalisation seulement).
- Requête AuthZEN telle que ci-dessus.
- Scopes : `mestier.read`, `mestier.doe.write`.
- `subject.properties.roles` vaut toujours `["user"]` en v1 (Cerbos exige un rôle de principal) ; les décisions portent sur les attributs, pas sur le rôle.
- Branche `feat/mcp` posée sur `feat/doe-backend` (#538) : le serveur MCP appelle les routes du DOE.

## Risks / open questions

| Risque | Porteur | Parade |
|---|---|---|
| ChatGPT parle-t-il 2025-11-25 ou 2026-07-28 ? | W3 / W6 | `rmcp` 3.x ; la gateway lit le corps JSON-RPC si les en-têtes 2026-07-28 manquent |
| Activer l'audience dans l'API casse le webapp si ses tokens n'ont pas l'audience attendue | W5 | Mesurer l'`aud` des tokens webapp avant ; accepter la liste {webapp, API} si besoin |
| FerrisKey : RFC 8414, `resource`, `iss` RFC 9207 non vérifiés | W1 | Le lot W1 commence par l'état des lieux |
| CIMD : FerrisKey doit aller chercher une URL externe (SSRF, cache, taille) | W1 | HTTPS seul, IP privées refusées, taille et durée plafonnées, cache |
| Recette ChatGPT : il faut une URL publique HTTPS | W6 / user | Tunnel de test ou environnement éphémère (voir mémoire envs éphémères) |
| Une PR dans le dépôt ferriskey suit ses propres règles | W1 | Lire son CLAUDE.md avant |

## Verify (exit)

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p mcp-gateway -p mcp-server -- --ignored     # e2e, stack locale (FerrisKey, Cerbos, Postgres, Redis, S3)
docker compose run --rm cerbos compile /policies          # tests de politiques
# ferriskey : la suite de vérification de son dépôt
```

## Statut

| Lot | État |
|---|---|
| W0 | intégré |
| W1 | pending |
| W2 | pending |
| W3 | pending |
| W4 | pending |
| W5a | pending |
| W5b | pending |
| W6 | pending |
| W7 | pending |

## Decision log

- 2026-10-09 : Cerbos plutôt qu'OPA comme PDP provisoire : AuthZEN 1.0 natif (≥ 0.48), politiques YAML lisibles. OPA demande Rego et une couche AuthZEN à part.
- 2026-10-09 : l'autorisation du MCP par organisation est appliquée par l'API Mestier, pas par la gateway ni Cerbos. Rejeté : passer `mcp_enabled` au PDP, qui obligerait la gateway à lire la configuration des organisations. Désactivé par défaut : une organisation choisit d'ouvrir l'accès.
- 2026-10-09 : la politique v1 ne recopie pas les rôles Mestier. Elle décide sur l'outil, son accès et les scopes ; l'API reste juge des droits métier. Rejeté : alimenter Cerbos avec les permissions Mestier, qui dupliquerait la source de vérité.
