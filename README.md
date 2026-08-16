# Mestier

Open-source, high-performance ERP for artisans and SMEs.

Backend in Rust (Axum + SQLx + PostgreSQL), frontend in React 19 + TanStack.

## Workspace layout

```
apps/
  api/                 # Rust HTTP API binary (Axum)
  webapp/              # React 19 frontend (TanStack Router/Query/Table/Form)
libs/
  args/                # CLI / env args (clap)
  auth/                # Auth integration (Ferriskey)
  common/              # Shared config & utilities
  core/                # Domain (User, Organization, Role, Membership)
  macros/              # Internal proc-macros
  server/              # Axum server runtime helpers
deploy/                # docker-compose configs (SigNoz, OTel collector)
migrations/            # SQLx migrations
```

## Prerequisites

- Rust (edition 2024) and `cargo`
- `pnpm` (frontend)
- Docker + Docker Compose (Postgres + observability stack)

## Local stack

A `docker-compose.yml` at the root spins up Postgres and the full SigNoz observability stack (ClickHouse, query-service, frontend, OpenTelemetry collector).

```bash
docker compose up -d
```

Exposed ports on the host:

| Service              | Port        | Notes                                          |
| -------------------- | ----------- | ---------------------------------------------- |
| PostgreSQL           | 5433        | mapped to container `5432` (avoid clash)       |
| RustFS S3 API        | 9000        | file storage endpoint                          |
| RustFS Console       | 9001        | http://localhost:9001                          |
| OTLP gRPC            | 4317        | matches `OTLP_ENDPOINT` in `.env`              |
| OTLP HTTP            | 4318        |                                                |
| SigNoz UI            | 8080        | http://localhost:8080                          |

The first boot takes 1–2 minutes while `signoz-telemetrystore-migrator` provisions ClickHouse.

## Running the API

```bash
cargo run -p api
```

The API binary exposes two HTTP servers:

- **Public API** on `SERVER_PORT` (default `3456`) — business endpoints, `/scalar`, `/swagger`.
- **Internal API** on `SERVER_INTERNAL_PORT` (default `3457`) — `/health`, `/metrics`. Not meant to be exposed publicly.

Common flags / env vars (see `libs/args/src/`):

| Env                    | Flag                       | Default                            |
| ---------------------- | -------------------------- | ---------------------------------- |
| `SERVER_HOST`          | `--server-host`            | `0.0.0.0`                          |
| `SERVER_PORT`          | `--server-port`            | `3456`                             |
| `SERVER_INTERNAL_PORT` | `--server-internal-port`   | `3457`                             |
| `DATABASE_URL`         | —                          | `postgres://…@localhost:5433/mestier` |
| `ACTIVE_OBSERVABILITY` | `--active-observability`   | `false`                            |
| `OTLP_ENDPOINT`        | `--otlp-endpoint`          | `http://localhost:4317`            |
| `METRICS_ENDPOINT`     | `--metrics-endpoint`       | `http://localhost:4317`            |

File storage defaults target the local RustFS service:

| Env                              | Flag                                   | Default                 |
| -------------------------------- | -------------------------------------- | ----------------------- |
| `FILE_STORAGE_BUCKET`            | `--file-storage-bucket`                | `mestier-files`         |
| `FILE_STORAGE_ENDPOINT`          | `--file-storage-endpoint`              | `http://localhost:9000` |
| `FILE_STORAGE_REGION`            | `--file-storage-region`                | `us-east-1`             |
| `FILE_STORAGE_ACCESS_KEY_ID`     | `--file-storage-access-key-id`         | `rustfsadmin`           |
| `FILE_STORAGE_SECRET_ACCESS_KEY` | `--file-storage-secret-access-key`     | `rustfsadmin`           |
| `FILE_STORAGE_FORCE_PATH_STYLE`  | `--file-storage-force-path-style`      | `true`                  |
| `FILE_STORAGE_AUTO_CREATE_BUCKET` | `--file-storage-auto-create-bucket`   | `true`                  |
| `FILE_STORAGE_KEY_PREFIX`        | `--file-storage-key-prefix`            | `uploads`               |
| `FILE_STORAGE_MAX_UPLOAD_BYTES`  | `--file-storage-max-upload-bytes`      | `10485760`              |

## Database migrations

```bash
cargo install sqlx-cli --no-default-features --features rustls,postgres
sqlx migrate run
```

## Tests

```bash
cargo test --workspace                    # unit tests, no database
cargo test -p mestier-core -- --ignored   # integration tests, needs DATABASE_URL
cargo test -p mestier-core --test quote_bdd
```

The last one runs the Gherkin scenarios in `libs/core/tests/features/`, which
state business rules in plain sentences so they can be arbitrated without
reading Rust. To add one: write the scenario in a `.feature` file, then wire its
sentences in `libs/core/tests/quote_bdd/steps.rs`. Steps call a domain service
through its port and assert on the answer. They never hold a rule themselves,
and they never touch a database. Follow `tests/quote_bdd/repository.rs`, which
stubs a mock over a shared in-memory store, and declare any new suite as its
own `[[test]]` with `harness = false`.

### Mocks across crates

`mestier-core`, `discord`, `iam`, `rate-limit` and `authz` expose their ports'
`mockall` doubles behind a `mock` feature, so no crate writes a fake by hand:

```rust
#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait TaskRepository { /* … */ }
```

To use one from another crate's tests, enable the feature on the dev-dependency:

```toml
[dev-dependencies]
mestier-core = { workspace = true, features = ["mock"] }
```

For a crate's own `tests/`, it dev-depends on *itself* with the feature on,
which is what `mestier-core` does. Either way the resolver keeps a dev-only
feature out of `cargo build`, so mockall never reaches a production binary.
`cargo tree -p api --edges normal -i mockall` must stay empty. Do not reach for
`required-features` instead: it makes `cargo test --workspace` skip the target
in silence.

`mestier-core` re-exports all its mocks at the crate root, since its `domain`
module is `pub(crate)`. Adding a port means adding one line to that gated
`pub use` block in `libs/core/src/lib.rs`.

## Frontend

```bash
cd apps/webapp
pnpm install
pnpm dev          # vite dev on port 3000
pnpm check        # biome lint + format + organize imports
pnpm build
```

See `CLAUDE.md` for the architecture conventions (Feature/UI split, routing, tech stack).

## Observability

When `ACTIVE_OBSERVABILITY=true`, the API exports traces, metrics, and logs over OTLP gRPC. With the compose stack running, open SigNoz at http://localhost:8080 to inspect them.
