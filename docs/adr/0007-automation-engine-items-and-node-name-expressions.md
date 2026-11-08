# 0007. The automation engine works on lists of items and references nodes by name

Status: accepted
Date: 2026-11-06

## Context
The prototype writes expressions as `$('Node name').item.json.x` and shows per-node item counts ("1 → 3 items", "true 2, false 1"). The engine passes one JSON value per step and references connectors by id (`connectors.c1.output.x`). The user chose conformity over keeping the engine.

## Decision
Rewrite the engine so a step receives and produces a list of items, and expressions resolve nodes by name. Stored workflows written with `connectors.<id>.output` expressions are not converted: they stop validating and have to be rewritten (user decision, 2026-11-06). Connectors that return one value return a list of one item.

## Alternatives rejected
- Convert stored expressions in a migration: recommended, declined by the user.
- Accept both syntaxes during a transition: two languages in the engine, declined by the user.
- Front-end facade that shows names and writes ids: leaves the item counts, branch counts and the item selector impossible, and every error message keeps ids.
- Keep the current model and drop the prototype's item views: not the product the user validated.

## Consequences
- Gains: identical expressions, counts and branching behaviour as the prototype.
- Costs: engine and every connector change; existing workflows that reference another node stop working.
- What becomes harder: renaming a node rewrites the references to it.
