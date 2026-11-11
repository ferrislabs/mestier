# 0015. Table rows store their values in one JSONB column

Status: accepted
Date: 2026-11-11

## Context

Users define their own tables in the knowledge module (`docs/chantiers/tables.md`): up to 50 typed columns and 10 000 rows per table, with columns added, retyped and removed at any time. Views filter and sort on any column on the server. Every query is scoped by `org_id` and checked at compile time with SQLx where possible.

## Decision

A row is one record in `knowledge_table_rows` with a `values JSONB` object keyed by column id. The domain owns the typing: a value is written only through `CellValue::parse(kind, config, json)`, and a column change rewrites the affected keys in the same transaction. Filters and sorts are built with `sqlx::QueryBuilder` from validated column ids and bound parameters, with a GIN index on `values`.

## Alternatives rejected

- One Postgres table per user table (dynamic DDL): migrations at runtime, a schema per customer that SQLx cannot check, and `org_id` scoping spread across thousands of tables.
- One row per cell (entity-attribute-value): a row read becomes a pivot, a 10 000-row sort joins 10 000 × columns cells, and a partial update touches many records.
- Typed columns per kind on a wide table (`text_1`, `number_1`…): a hard ceiling per kind and a remapping on every type change.

## Consequences

- Gains: one row read is one record; adding or removing a column is one metadata write plus one `values - key` update; per-cell writes merge with `values || patch`.
- Costs: the database does not enforce cell types; the domain parser is the only gate, so every write path (API, CSV, automation, MCP) must go through it.
- What becomes harder: indexing a single column (would need expression or generated columns), and cross-table links later (a link value is an id inside JSON, without a foreign key).
