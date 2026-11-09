# Automation conformity

Goal
  The automation module of the real Mestier behaves like the prototype's Workflows module, and every value on screen comes from the API.

Context
  #594 ported the look of the prototype. An audit of the code on 2026-11-05 found 278 gaps between the two (72 blocking), most of them data the frontend simulates in localStorage or by id hash, and backend features the prototype shows that do not exist. Epic #596. Issues: #597 (lot 1), #598 (lot 2); lots 3 to 8 get an issue once specified. ADRs 0007 to 0010.

Glossary
  Dossier      Folder grouping workflows, shown as "projet" in the UI. Rejected: "project", already the ERP's construction project.
  Run number   Sequential number of a run within its organization, shown as #N.
  Dry run      Execution of the current graph where each connector answers with its sample output and nothing leaves the process.
  Item         One element of the list a node receives or produces (lot 8).

Invariants
  - A disabled or archived workflow never starts a run from an event.  [test]
  - A workflow with archived_at set is never enabled.  [constraint: CHECK in the table]
  - A folder is never its own ancestor.  [test: cannot be carried by a type]
  - Node names are unique in a graph, ignoring case and padding.  [test: graph validation]
  - A run number is unique per organization and never reused.  [constraint: unique index + counter in the insert]
  - A graph stored before a field existed still loads.  [test]
  - A disabled connector is never called.  [test]
  - updated_at moves only on a name, description or version change.  [test]
  - No automation screen reads localStorage or an id hash for backend data.  [test]
  - A dry run has no side effect (lot 4).  [type: connectors expose a separate dry-run entry point]
  - A webhook path is unique across organizations (lot 5).  [constraint]
  - A credential secret is never returned by the API (lot 7).  [type: response type without the secret field]

Out of scope
  - A screen to create or move folders and tags: the prototype has none.
  - Prototype-only mock data (seed workflows, sample runs).
  - Anything the real module has that the prototype lacks (stays as is).
  - The JavaScript code node (deferred) and the approval node (dropped), user decisions of 2026-11-06.

Design
  Lot 1  Frontend: routes, copy tables (connector-copy, connector-field-copy, event-contracts), shared header and side nav. Interim tables replaced by server descriptors in lots 6 and 7.
  Lot 2  Domain: Workflow gains folder_id, tags, archived_at; WorkflowFolder aggregate; PlacedConnector/PlacedTrigger gain name, description, disabled; RunOverview read model with number and counts.
         Ports: WorkflowRepository (extended), WorkflowFolderRepository (new), run repository (overviews).  Dispatch: generic, no dyn.
         Errors: AutomationError at the handler boundary (E_WORKFLOW_ARCHIVED, E_FOLDER_NOT_EMPTY); GraphError gains EmptyNodeName, NodeNameTooLong, NodeDescriptionTooLong, DuplicateNodeName.
  Lots 3 to 8  To be completed after the question batch of 2026-11-06: collections storage and a collection operation connector (3), dry-run entry point on connectors (4), webhook and schedule triggers (5), nine connectors (6), credential kinds and descriptors (7), engine rewrite (8).

Acceptance (Gherkin), lot 2
  @spec-auto-2-1  Scenario: an event does not start a run for a disabled or archived workflow
  @spec-auto-2-2  Scenario: a manual start of a disabled workflow works, of an archived one answers 409 E_WORKFLOW_ARCHIVED
  @spec-auto-2-3  Scenario: deleting a non-empty folder answers 409 E_FOLDER_NOT_EMPTY
  @spec-auto-2-4  Scenario: moving a folder under its descendant is refused
  @spec-auto-2-5  Scenario: changing enabled, folder, tags or archived leaves updated_at untouched
  @spec-auto-2-6  Scenario: a disabled connector records a succeeded step without a call and passes its predecessor's output
  @spec-auto-2-7  Scenario: a graph stored before the node fields existed loads unchanged
  @spec-auto-2-8  Scenario: run numbers start at 1 per organization, existing runs are numbered by creation order, the migration reverts and re-applies
  @spec-auto-2-9  Scenario: duplicate node names are rejected with the node and the field named
  @spec-auto-2-10 Scenario: no automation screen reads localStorage for backend data

Lot 3  Collections (spec written 2026-11-06)
  Goal: OpenAPI collections are stored per organization with their version history, and a node made from an operation is run by the backend from the stored collection.
  Invariants
    - A collection id is unique per organization, a clash gets the suffix -2, -3.  [constraint]
    - The version history is append-only and written in the same transaction as the replacement.  [test]
    - A collection never leaves its organization.  [test]
    - The request built for an operation never targets an address the http connector already refuses (private ranges, metadata endpoints).  [test: reuses the address policy]
    - A node stores the reference (collection, operation) and the values entered, never a copy of the url or the schema.  [type: OperationRef in the graph]
  Design
    Domain: Collection, CollectionVersion, Operation, Parameter, SecurityScheme; PlacedConnector gains `operation: Option<OperationRef>`; connector kind `collection.operation`.
    Ports: CollectionRepository (generic dispatch). Adapter: Postgres, table automation.collection (model as JSONB) and automation.collection_version.
    Use cases: list, get, import, replace, delete, resolve_operation (used by validation and the engine).
    Errors: CollectionError (NotFound, Oversized, InvalidModel); GraphError gains UnknownCollection, UnknownOperation, MissingParameter, UnknownParameter.
    Parsing stays in the frontend (pure, already ported); the backend validates and stores the parsed model.
  Acceptance
    @spec-auto-3-1  importing from a url or a file stores the collection for the organization and no other organization sees it
    @spec-auto-3-2  a clashing title gets a suffixed id
    @spec-auto-3-3  replacing keeps the previous version in the history, atomically
    @spec-auto-3-4  deleting removes the collection and its history; a workflow using it fails the next save and the next run with the collection named
    @spec-auto-3-5  saving a collection.operation node with an unknown operation, a missing required parameter or an unknown field is rejected, naming the node and the field
    @spec-auto-3-6  running the node substitutes path parameters, sends query, headers and body, applies the credential for the declared scheme and returns the connector output
    @spec-auto-3-7  updating the collection changes what the next run sends, with no change to the workflow
    @spec-auto-3-8  a node whose collection or operation disappeared, or whose body no longer fits, shows a red badge in the editor
    @spec-auto-3-9  a base url on a refused address is rejected at run time with a named error
    @spec-auto-3-10 collections read needs VIEW_AUTOMATION, writes need MANAGE_AUTOMATION; a spec over the size limit is rejected
    @spec-auto-3-11 no automation screen reads localStorage for collections or for the operation behind a node
  Risks
    - The two sample collections (address API, company search) were injected by the mock for every organization. They are not seeded any more; the empty state invites to import. Owner: user.
    - An OAuth2 or query-string apiKey scheme cannot be applied; the node shows it as unsupported. Owner: lot 7.

Lot 4  Dry run (spec written 2026-11-06)
  Goal: "Tester le workflow" runs the current graph, saved or not, without side effect, and shows each node's status and output.
  Invariants
    - A dry run never calls a connector's execute path.  [type: connectors expose a separate dry_run entry point, the engine in dry mode only calls it]
    - A dry run writes no row in the run tables.  [test: row counts unchanged]
    - A dry run needs MANAGE_AUTOMATION.  [test]
  Design
    Domain: ConnectorDescriptor gains sample_output; trait entry point dry_run(config, input) -> Output defaulting to the sample; collection.operation answers with the response example of the stored operation.
    Application: dry_run_graph(org, graph, trigger_payload?) validates the graph, walks it from the triggers, evaluates conditions and loops on sample data (loops capped at 3 items), passes through disabled nodes, and returns one step per reachable node (status, output, error naming the node); nothing is persisted.
    Endpoint: POST .../automation/workflows/dry-run { graph, trigger_id?, trigger_payload? } -> steps, each with connector_id, status, iteration_path, input, output, error. The frontend reveals the steps one node at a time, as the prototype does. The first failed step stops the walk and every reachable node left unsettled gets a skipped step.
    Errors: GraphError for an invalid graph; a step carries a broken reference as its error.
  Acceptance
    @spec-auto-4-1  a dry run of an unsaved graph returns one step per reachable node
    @spec-auto-4-2  no connector is executed and no run row exists afterwards
    @spec-auto-4-3  an invalid node returns its error naming the node and the field
    @spec-auto-4-4  a condition follows the branch its sample data selects, a loop iterates at most 3 sample items
    @spec-auto-4-5  a disabled node passes the previous output through
    @spec-auto-4-6  a reference to a node output that does not exist is an error on the referencing node
    @spec-auto-4-7  the button is active on a draft and on unsaved changes, runs the current graph, animates node by node and fills the Exécution tabs
    @spec-auto-4-8  a member without MANAGE_AUTOMATION cannot dry-run
  Risks: the old engine and lot 8 differ on data shape; the dry run is rewritten with the engine.

Lot 5  Triggers (spec written 2026-11-06)
  Goal: a typed webhook and a scheduler start runs, as in the prototype.
  Invariants
    - A webhook path is unique across organizations.  [constraint]
    - A webhook request is authenticated by the credential chosen on the trigger before its body is read.  [test]
    - A schedule fires once per due time, even with several workers or a restart.  [test: simulation of time and restarts]
    - A disabled or archived workflow never fires from a schedule or a webhook.  [test]
  Design
    Domain: TriggerKind gains Webhook { method, path, credential_id, contract[], on_invalid } and Schedule { frequency, time, weekday?, timezone }; the contract is a list of keys with type, required and example.
    Public route /webhooks/{path} outside the authenticated api prefix, size-capped and rate-limited by the existing rate-limit crate; the body is validated against the contract; on_invalid is reject (400 listing the keys) or accept.
    Scheduler: a worker pass computes due times from the organization timezone and records next_run_at per schedule trigger in a table, claimed with a row lock like the run queue.
    Simulation: yes, a deterministic clock for the scheduler (fire once, restart, DST change, disabled workflow).
  Acceptance
    @spec-auto-5-1  a valid POST on the webhook path starts a run whose trigger payload is the body
    @spec-auto-5-2  a body that breaks the contract answers 400 with the keys when on_invalid is reject, and starts a run when it is accept
    @spec-auto-5-3  a request without the right credential answers 401 and starts nothing
    @spec-auto-5-4  two workflows cannot hold the same path
    @spec-auto-5-5  a daily schedule at 09:00 Europe/Paris fires once a day, once after a restart, and at 09:00 local after a DST change
    @spec-auto-5-6  a disabled or archived workflow does not fire
    @spec-auto-5-7  the creator offers Planificateur and Webhook; the webhook node shows its url, the contract editor and the expected request; the credential usage text lists Webhook again
  Risks
    - A public endpoint is new attack surface: a security review is part of the lot. Owner: orchestrator.
    - A webhook with no credential can be started by anyone who knows its url, a read-only member included; a 401 shows that a path, method and credential exist. Accepted, the prototype allows no credential. Owner: user.
    - The rate limit key trusts client forwarding headers (issue #608); the webhook route adds per-path and per-workflow limits that do not depend on them.
    - A path stays reserved while its workflow is disabled; archiving frees it.

Lot 6  Credentials (spec written 2026-11-06)
  Goal: identifications match the prototype: a description, a real status, how many workflows use them, secrets kept apart from readable values, the 17 kinds, labels and hints from the server.
  Invariants
    - A secret is never returned by the API.  [type: response type without secret fields]
    - A credential of another organization is never read, applied or listed.  [test]
    - An OAuth state is unguessable, single use, bound to the organization and the credential, and expires; the member that started it is recorded for the trail only because the callback is public.  [test + constraint]
    - A token refresh happens once under concurrency.  [test: row lock]
    - Deleting a used credential is allowed; a node that uses it fails at run with the credential named.  [test]
    - A credential name is unique per organization and kind.  [constraint]
  Design
    Domain: Credential gains description, status (valid, expired, not_connected), account; secret fields and readable fields are separate (readable ones returned, secrets only their presence); kinds gain server descriptors (label, category, usages, fields with default, placeholder, hint, showWhen).
    Kinds: the four existing plus query api key, digest, OAuth1, OAuth2 client credentials, OAuth2 authorization code, JWT, mutual TLS, custom (JSON headers), HMAC (verifies an inbound webhook body), SMTP, IMAP, SSH, FTP, SQL. Applied at run where a connector consumes them (http.request, collection.operation, webhook); SMTP is consumed by the SMTP node of the next lot; IMAP, SSH, FTP and SQL are stored with no executor.
    OAuth: start (returns the authorization url with a state), callback route (public, checks the state, exchanges the code, stores tokens sealed), refresh under a row lock, disconnect; tested against a fake provider only.
    Status: expired when the stored expiry is past or a refresh fails; not_connected for an OAuth kind with no tokens.
  Acceptance
    @spec-auto-6-1  creating a credential returns readable values and the presence of each secret, never a secret
    @spec-auto-6-2  editing with only one secret changed keeps the others
    @spec-auto-6-3  the catalogue of kinds carries French labels, categories, usages, defaults, placeholders and hints
    @spec-auto-6-4  each of the 17 kinds can be created, and those with a consumer are applied to the request
    @spec-auto-6-5  an OAuth authorization code flow against a fake provider stores tokens, shows the account, refreshes once under concurrency and can be disconnected
    @spec-auto-6-6  a replayed, expired or foreign OAuth state is refused
    @spec-auto-6-7  a credential past its expiry is listed as expired and the list says how many workflows use it
    @spec-auto-6-8  deleting a used credential succeeds and the node fails at run naming it
    @spec-auto-6-9  an HMAC credential authenticates an inbound webhook by its signature
    @spec-auto-6-10 the screens use no hash of the id for the status or the usage count
  Risks
    - OAuth needs real providers to be proven end to end; only a fake provider is tested. Owner: user.
    - IMAP, SSH, FTP and SQL kinds have no executor. Owner: user.
    - An inbound HMAC body is read before it is authenticated and a signed body can be replayed, there is no timestamp window. Owner: user.
    - OAuth2 and OpenID Connect schemes of a collection are applied as a bearer with the credential's token.

Lot 7  Missing nodes (spec written 2026-11-06)
  Goal: the workflow nodes of the prototype that the backend lacks exist: Attendre, Fusionner, Ajouter une note, Rechercher des devis, Créer un projet, Publier dans un canal, E-mail SMTP. The code node is deferred and the approval node dropped (user, 2026-11-06).
  Invariants
    - A wait never holds a worker: the step is parked with a resume time and the worker picks it up when due.  [test]
    - An SMTP node never connects to an address the address policy refuses, and never lets a value inject a header or a recipient list longer than the limit.  [test]
    - A node that writes to Mestier (note, project, channel post) acts as the workflow's organization and never another.  [test]
    - A search returns at most 200 items.  [type: a bounded list]
  Design
    Connectors: wait (minutes, hours, days; park the step with resume_at, new step status waiting), merge (append, combine by position, combine by field), add_note, search_quotes (list output), create_project (from a quote), post_channel (a chat channel of the organization), smtp (credential of the SMTP kind from lot 6, to, cc, subject, body, text or html, bounded recipients and size). Each has a descriptor (labels, descriptions, sample output, fields) so the dry run and the creator work unchanged; they call the existing use cases of their domain like support.assign_ticket does.
    Engine: a parked step is resumed by the worker pass; the run keeps status running while parked; replay and dry run know the waiting step (dry run answers at once with the sample).
  Acceptance
    @spec-auto-7-1  a wait parks its step with a resume time and the run continues after it, with no worker held in between
    @spec-auto-7-2  a merge appends, combines by position and combines by field the outputs of its incoming branches
    @spec-auto-7-3  add_note creates a note on the chosen record of the workflow's organization
    @spec-auto-7-4  search_quotes returns the matching quotes of the organization, at most 200, as a list
    @spec-auto-7-5  create_project creates a project from a quote and returns it
    @spec-auto-7-6  post_channel posts a message in a chat channel of the organization
    @spec-auto-7-7  smtp sends a message through the SMTP credential, refuses a refused address, rejects header injection and more recipients than the limit
    @spec-auto-7-8  the creator lists the seven nodes in the prototype's categories with their descriptions, and Logique, Actions Mestier and Outils are complete
    @spec-auto-7-9  the executions views show a parked step as "En pause" and its resume time
  Risks
    - Merge and search results are single values until the engine rewrite (lot 8). Owner: lot 8.
    - SMTP is proven against a local test server only. Owner: user.

Simulation
  Lot 2: no, the invariants are rows and constraints. Lot 5 (scheduler, time) and lot 8 (engine) need a deterministic simulation of time and of the engine pass.

Workstreams
  Lot 2: backend phases A (folders, workflow fields, dispatcher), B (graph fields, engine), C (run numbering and counts); frontend F1 (list, folders, tags, archive, create) and F2 (node fields, executions). Backend before client regeneration before frontend.

Frozen contracts
  Lot 2 DTOs: see the issue #598 and the OpenAPI document; client regenerated in the commit "chore(webapp): regenerate the api client".

Risks / open questions
  - Existing runs are renumbered when the migration is applied; the migration needs the new binary. Owner: release.
  - An archived recurrence-horizon workflow is no longer startable. Owner: task_recurrence maintainers.
  - A missing folder answers 409 E_CONFLICT, not 404. Owner: decided by the user.
  - Existing workflows with `connectors.<id>.output` expressions stop working at lot 8 (ADR 0007). Owner: user, decided.
  - Approval node: the answer read "Supprimer ce node"; whether it is also removed from the prototype is to be confirmed. Owner: user.
  - Order of lots 6 and 7 (SMTP needs the SMTP credential kind): default is credentials first. Owner: user, not understood yet.

Verify (exit)
  cargo fmt --all --check
  SQLX_OFFLINE=true cargo check --workspace
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  cargo test -p mestier-core -- --ignored
  (cd apps/webapp && pnpm check && pnpm test && pnpm build)

Amended: 2026-11-06 lot 4 gains trigger_payload, iteration_path and the stop-at-first-failure rule after the correctness review; the walk is bounded in bytes, time and structure after the security review.

Amended: 2026-11-06 lot 5 after the reviews: archiving frees routes and schedules, the scheduler payload also carries horodatage, jour and fuseau, an empty body is read as an empty object.

Amended: 2026-11-06 lot 6 after the reviews: OAuth state binding, OAuth2 collections as bearer, HMAC replay risk.
