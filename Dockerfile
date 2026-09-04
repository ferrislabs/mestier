FROM rust:1.95.0-bookworm AS chef

WORKDIR /usr/local/src/mestier

RUN cargo install cargo-chef --version 0.1.77 --locked && \
    cargo install sqlx-cli --version 0.8.6 --no-default-features --features postgres --locked

# --- Plan: extract a recipe of the workspace dependency graph ----------
# Only Cargo.toml / Cargo.lock changes invalidate this layer.
FROM chef AS planner

COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# --- Cook: build every dependency from the recipe ----------------------
# This layer is cached as long as recipe.json (i.e. the dep graph) is
# stable. Source changes do not invalidate it.
FROM chef AS builder

COPY --from=planner /usr/local/src/mestier/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# --- Build the actual workspace binaries -------------------------------
COPY . .
RUN cargo build --release

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
COPY --from=builder /usr/local/cargo/bin/sqlx /usr/local/bin/

EXPOSE 80

ENTRYPOINT [ "api" ]


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
