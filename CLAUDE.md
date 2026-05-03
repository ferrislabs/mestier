# CLAUDE.md - Mestier ERP

This file provides guidance to Claude Code (claude.ai/code) when working with the **Mestier** repository. Mestier is an open-source, high-performance ERP for artisans and SMEs.

## Repository layout

Monorepo-style workspace:

- `apps/webapp` — React 19 + TanStack (Router, Query, Table, Form).
- `libs/` — Shared logic and types (TBD).

## Commands (Webapp - `apps/webapp`)

Package manager is **pnpm**.

```bash
pnpm install
pnpm dev           # vite dev on port 3000
pnpm build          # production build
pnpm check          # biome check (lint + format + organize imports)
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

### Backend Context (Rust - apps/server)
- Framework: Axum (Asynchronous, Type-safe).
- Database: PostgreSQL with SQLx (compile-time checked queries).
- Auth: Ferriskey (WebAuthn/FIDO2) integration.
- Convention: Logic should be split into handlers/, models/, and services/ (for heavy business/IA logic).


### Development Guidelines
- Imports: Always use `#/*` alias for `src/*.
- Naming: PascalCase for components, kebab-case for files.
- Formatting: Biome is the source of truth (tabs, double quotes).
- ERP Logic: Every price calculation or rentability metric must be computed in the backend or a dedicated lib to ensure single source of truth between UI and PDF exports.
