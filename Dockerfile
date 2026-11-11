# cargo-chef ships prebuilt in this image: no more `cargo install` on a cache miss.
FROM lukemathwalker/cargo-chef:0.1.77-rust-1.95.0-bookworm AS chef

WORKDIR /usr/local/src/mestier

# --- sqlx-cli: its own stage, so it never sits on the path of the workspace
# build. Only the `api` image pulls it in.
FROM rust:1.95.0-bookworm AS sqlx-cli

RUN cargo install sqlx-cli --version 0.8.6 --no-default-features --features postgres --locked

# --- sccache: its own stage, same reason as sqlx-cli. Caches rustc output
# across CI runs, so a commit only recompiles the crates it touched.
FROM rust:1.95.0-bookworm AS sccache

RUN cargo install sccache --version 0.10.0 --locked

# --- Tooling shared by the dependency and workspace builds -------------
# mold replaces the default linker: linking the big binaries is a visible
# share of a release build. RUSTFLAGS must stay identical in `cook` and
# `build`, or the cooked dependencies are rebuilt.
FROM chef AS tooling

RUN apt-get update && \
    apt-get install -y --no-install-recommends mold && \
    rm -rf /var/lib/apt/lists/*

COPY --from=sccache /usr/local/cargo/bin/sccache /usr/local/bin/sccache

ENV RUSTFLAGS="-C link-arg=-fuse-ld=mold"

# --- Plan: extract a recipe of the workspace dependency graph ----------
# Only the Rust inputs are copied: a change under apps/webapp, docs or deploy
# must not invalidate anything below.
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY apps/api apps/api
COPY apps/mcp-gateway apps/mcp-gateway
COPY apps/mcp-server apps/mcp-server
COPY apps/seed-demo apps/seed-demo
COPY libs libs
RUN cargo chef prepare --recipe-path recipe.json

# --- Cook: build every dependency from the recipe ----------------------
# This layer is cached as long as recipe.json (i.e. the dep graph) is
# stable. Source changes do not invalidate it.
FROM tooling AS builder

# CI passes its GitHub Actions cache credentials as build secrets. Without
# them (local build) sccache stays off and cargo runs as usual.
COPY --from=planner /usr/local/src/mestier/recipe.json recipe.json
RUN --mount=type=secret,id=ACTIONS_RESULTS_URL \
    --mount=type=secret,id=ACTIONS_RUNTIME_TOKEN \
    set -e; \
    if [ -s /run/secrets/ACTIONS_RUNTIME_TOKEN ]; then \
        export ACTIONS_RESULTS_URL="$(cat /run/secrets/ACTIONS_RESULTS_URL)" \
               ACTIONS_RUNTIME_TOKEN="$(cat /run/secrets/ACTIONS_RUNTIME_TOKEN)" \
               SCCACHE_GHA_ENABLED=true RUSTC_WRAPPER=sccache; \
    fi; \
    cargo chef cook --release --recipe-path recipe.json; \
    sccache --show-stats || true

# --- Build the actual workspace binaries -------------------------------
# The CI builds this stage once per platform (`build-rust`) and every Rust
# image reuses it from the cache. Keep the inputs in sync with the planner.
COPY Cargo.toml Cargo.lock ./
COPY apps/api apps/api
COPY apps/mcp-gateway apps/mcp-gateway
COPY apps/mcp-server apps/mcp-server
COPY apps/seed-demo apps/seed-demo
COPY libs libs
COPY migrations migrations
COPY .sqlx .sqlx
RUN --mount=type=secret,id=ACTIONS_RESULTS_URL \
    --mount=type=secret,id=ACTIONS_RUNTIME_TOKEN \
    set -e; \
    if [ -s /run/secrets/ACTIONS_RUNTIME_TOKEN ]; then \
        export ACTIONS_RESULTS_URL="$(cat /run/secrets/ACTIONS_RESULTS_URL)" \
               ACTIONS_RUNTIME_TOKEN="$(cat /run/secrets/ACTIONS_RUNTIME_TOKEN)" \
               SCCACHE_GHA_ENABLED=true RUSTC_WRAPPER=sccache; \
    fi; \
    cargo build --release; \
    sccache --show-stats || true

# --- CI warm-up target -------------------------------------------------
# Never shipped. It pulls in every expensive stage (workspace build and
# sqlx-cli) so a single `build-rust` job per platform fills one cache scope
# that all the Rust images then read.
FROM scratch AS rust-prebuild

COPY --from=builder /usr/local/src/mestier/target/release/api /api
COPY --from=sqlx-cli /usr/local/cargo/bin/sqlx /sqlx

FROM debian:bookworm-slim AS runtime

RUN \
    apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates=20230311+deb12u1 \
    libssl3=3.0.17-1~deb12u2 && \
    rm -rf /var/lib/apt/lists/* && \
    addgroup \
    --system \
    --gid 1000 \
    mestier && \
    adduser \
    --system \
    --no-create-home \
    --disabled-login \
    --uid 1000 \
    --gid 1000 \
    mestier

USER mestier

FROM runtime AS api

COPY --from=builder /usr/local/src/mestier/target/release/api /usr/local/bin/api
COPY --from=builder /usr/local/src/mestier/target/release/seed-demo /usr/local/bin/seed-demo
COPY --from=builder /usr/local/src/mestier/migrations /usr/local/mestier/migrations
COPY --from=sqlx-cli /usr/local/cargo/bin/sqlx /usr/local/bin/

EXPOSE 80

ENTRYPOINT [ "api" ]


FROM runtime AS mcp-gateway

COPY --from=builder /usr/local/src/mestier/target/release/mcp-gateway /usr/local/bin/mcp-gateway

EXPOSE 8080

ENTRYPOINT [ "mcp-gateway" ]


FROM runtime AS mcp-server

COPY --from=builder /usr/local/src/mestier/target/release/mcp-server /usr/local/bin/mcp-server

EXPOSE 8081

ENTRYPOINT [ "mcp-server" ]


FROM node:24.13.0-alpine AS webapp-deps

WORKDIR /usr/local/src/mestier

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"

RUN \
    corepack enable && \
    corepack prepare pnpm@9.15.0 --activate && \
    apk --no-cache add dumb-init=1.2.5-r3

COPY apps/webapp/package.json apps/webapp/pnpm-lock.yaml ./

RUN pnpm install --frozen-lockfile

FROM webapp-deps AS webapp-build

COPY apps/webapp/ ./

RUN pnpm build

FROM nginx:1.28.0-alpine3.21-slim AS webapp

COPY --from=webapp-build /usr/local/src/mestier/dist/client /usr/local/src/mestier
COPY apps/webapp/nginx.conf /etc/nginx/conf.d/default.conf
COPY apps/webapp/docker-entrypoint.sh /docker-entrypoint.d/docker-entrypoint.sh

EXPOSE 80

RUN chmod +x /docker-entrypoint.d/docker-entrypoint.sh
