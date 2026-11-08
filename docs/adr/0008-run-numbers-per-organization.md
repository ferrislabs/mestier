# 0008. Run numbers are sequential per organization, allocated from a counter row

Status: accepted
Date: 2026-11-06

## Context
The prototype shows executions as `#1285` and puts the number in the url. The API identified runs by UUID only.

## Decision
`automation.run_counter(org_id, last_value)` is incremented in the statement that inserts the run, `run_number` is unique per organization, and the migration numbers existing runs by creation order.

## Alternatives rejected
- `max(run_number) + 1`: races between concurrent inserts.
- One Postgres sequence for all organizations: numbers leak other tenants' volume and are not contiguous per organization.
- Number per workflow: the executions log lists runs across workflows.

## Consequences
- Gains: per-tenant numbers that are never reused and safe under concurrency. Gaps are possible after a purge or a replayed fan-out.
- Costs: a counter row per organization; the migration needs the new binary, since the column is required.
- What becomes harder: bulk-inserting runs outside the use case.
