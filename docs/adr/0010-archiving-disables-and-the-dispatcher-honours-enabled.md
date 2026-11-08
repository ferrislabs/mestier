# 0010. Archiving disables a workflow, and the dispatcher honours `enabled`

Status: accepted
Date: 2026-11-06

## Context
`workflow.enabled` was read by neither the event dispatcher nor `start_run`, so a disabled workflow kept running on events. The prototype's "Archiver" removes the workflow from the list.

## Decision
`archived_at` set implies `enabled = false` (CHECK constraint). The dispatcher selects only enabled, non-archived workflows. An archived workflow cannot be started, replayed or saved (409 `E_WORKFLOW_ARCHIVED`); a disabled one can still be started by hand.

## Alternatives rejected
- Archive only hides the workflow in the list: it would keep reacting to events.
- Archive deletes: the history of runs would go with it.

## Consequences
- Gains: "Actif" and "Archiver" do what they say.
- Costs: workflows already disabled in production stop receiving events, which is the intended behaviour.
- What becomes harder: nothing noticeable.
