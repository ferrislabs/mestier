# 0017. Feature flags are declared in code, activations live in the database

Status: proposed
Date: 2026-11-12

## Context

The Mestier staff must be able to open a feature to some organizations only (`docs/chantiers/backoffice.md`). A gated feature is closed to every organization until the staff enables it for one. The gate is checked by the API, and the webapp hides what the organization cannot use.

## Decision

The domain declares the flags as a closed enum, `FeatureFlag`. Each variant has a stable `snake_case` key, a label and a description. The database stores only activations, one row per enabled flag in `organization_feature_flags (org_id, flag, enabled_at, enabled_by)`. A missing row means disabled. A key in the database that the code no longer knows is ignored.

## Alternatives rejected

- Flags created from the back-office, stored in a `feature_flags` table: a flag has no effect until code checks it, so a flag created from a screen guards nothing. The code would also have to compare against free strings, and a typo would fail silently.
- A JSONB column on `organizations`: no `enabled_by` or `enabled_at` per flag, and every write rewrites the whole set.
- An external flag service: one more system to run, for a few flags toggled by hand.

## Consequences

- Adding a flag takes a deploy. Removing one means deleting the variant, then its rows in a migration.
- `require_feature` takes a `FeatureFlag`, so a misspelled flag does not compile.
- The back-office lists the catalogue from the code. It cannot show a flag the running binary does not know.
