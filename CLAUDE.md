# CLAUDE.md - Mestier ERP

This file provides guidance to Claude Code (claude.ai/code) when working with the **Mestier** repository. Mestier is an open-source, high-performance ERP for artisans and SMEs.

## Repository layout

Cargo workspace plus a pnpm app:

- `apps/api` — the Axum binary. Assembles the router and the OpenAPI document; holds no business logic.
- `apps/webapp` — React 19 + TanStack (Router, Query, Table, Form).
- `libs/core` — the whole backend domain, split `domain/` (entities, ports, pure services) → `application/` (use cases) → `infrastructure/` (adapters). One directory per bounded context in each layer.
- `libs/handlers-*` — HTTP handlers, one crate per **area**, not per bounded context: `handlers-planning` holds task/project/label/work-time, `handlers-reference` holds the referentials. Each crate is split by aggregate into submodules, and adding one touches two lines of its `lib.rs` (a `pub mod` and a `.merge(...)`). `libs/handlers` holds `AppState` and the shared error type.
- `libs/events` — the `DomainEvent` trait, the persisted envelope and the event catalogue. Depends on nothing in `libs/core`.
- `libs/macros` — `#[transactional]` and `#[repository]`. Read it before touching a use case.
- Others: `args`, `auth` (FerrisKey), `authz`, `common` (ids, `CoreError`, config), `discord` (chat domain), `iam`, `pagination`, `rate-limit`, `server`.

## Where things register

Adding a module means editing a registry, never a `match`:

- Crate → `Cargo.toml` (workspace `members` **and** `workspace.dependencies`).
- Repository → a marker in `libs/core/src/infrastructure/registry.rs`, plus `#[repository(domain = X, backend = Postgres)]` on the adapter.
- Route and OpenAPI tag → `apps/api/src/router.rs` and `apps/api/src/openapi.rs`.
- Frontend module and its sidebar, settings sections included → `apps/webapp/src/modules/registry.ts`.

## Commands

Backend, from the repository root:

```bash
cargo fmt --all --check
SQLX_OFFLINE=true cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                      # unit tests, no database
cargo test -p mestier-core -- --ignored     # integration tests, needs DATABASE_URL
```

Queries are compile-time checked. **Any new `query!` needs a live database and
`cargo sqlx prepare --workspace -- --all-targets`** before the offline build passes again.
`source .env` exports `DATABASE_URL`; migrations run with `sqlx migrate run --source migrations`.

Frontend, from `apps/webapp` (pnpm, Node ≥ 22.13):

```bash
pnpm install
pnpm dev                      # vite dev on port 3000
pnpm build
pnpm check                    # biome: lint + format + organize imports
pnpm test                     # vitest
pnpm dlx shadcn@latest add <component>
```

## Architecture & Conventions (Frontend)

### Separation of Concerns (Domain-Driven)
For every page in `src/pages/<domain>/`, we apply a strict Feature/UI split:

1. **Feature (`feature/`)**: Handler business logic, data fetching (Tanstack Query), state management, and side effects.
2. **UI (`ui/`)**: Pure presentational components. No hooks, no fetch. Receives data and callbacks via props.

### Example: `src/pages/inventory/`
- `feature/inventory-list-feature.tsx`: Manages filters, pagination state, and calls `useQuery`
- `ui/inventory-list.tsx`: Receives inventory items and renders the table. No hooks or data fetching logic.


### Routing (`src/routes/`)
- TanStack Router (file-based).
- Route files must be thin. They call the Feature component of a page.
- Layouts: _app.tsx for authenticated artisan space (uses Ferriskey for auth).

### Tech Stack Highlights
- TanStack Table: Crucial for ERP grids (inventory, customer lists).
- TanStack Form: Used for all complex business inputs (quotes, invoices).
- React 19: Use use and Action patterns for form submissions.
- Tailwind v4: Styling with utility-first approach.

### Backend (Rust)
- Framework: Axum. Database: PostgreSQL with SQLx. Auth: FerrisKey — ferrislabs' own IAM, **not** Keycloak; never model its API on Keycloak's.
- Hexagonal: the domain imports nothing from infrastructure. Ports are traits defined by the domain, implemented by adapters.
- **A use case is a `#[transactional(...)]` method on `MestierUseCase`.** The macro opens the transaction, injects the repositories you list, and commits. Two names are special: `authz` (the policy engine) and `events` (a realtime publisher scoped to this transaction).
- **Every transaction persists its domain events before committing**, via `with_tx_emitting`. An event exists if and only if its transaction committed. List `emitter` to emit one; see `libs/events` and the `automation` schema.
- Never hold anything request-scoped on `MestierUseCase`: it is long-lived and cloned into every request. A shared buffer there once leaked chat events across organizations.
- Multi-tenancy: every query is scoped by `org_id`. Migrations are reversible — write and test the `.down.sql`.


### Development Guidelines
- Imports: Always use `#/*` alias for `src/*.
- Naming: PascalCase for components, kebab-case for files.
- Formatting: Biome is the source of truth (tabs, double quotes).
- ERP Logic: Every price calculation or rentability metric must be computed in the backend or a dedicated lib to ensure single source of truth between UI and PDF exports.
