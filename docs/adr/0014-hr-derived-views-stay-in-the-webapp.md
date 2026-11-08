# 0014. HR stores facts in the API, the leave and planning views stay computed in the webapp

Status: proposed
Date: 2026-11-08

## Context
The prototype computes leave balances, the planning grid, the "today" status and the extra-hours recap from pure, tested functions (`mestier-rework/src/modules/rh/lib`, `features/*/*-model.ts`). The repository rule asks that prices and profitability be computed in the backend. These views count days and hours; they carry no price.

## Decision
The API persists facts only: member profile, contracts and their schedule, absence types and requests, leave accounts and adjustments, day overrides, surcharges, extra periods. Balances, planning cells and recaps remain pure functions in `pages/hr/lib/rh`, fed by list endpoints. Any amount in euros stays out of these views and keeps coming from the profitability module.

## Alternatives rejected
- Port the balance and planning engines to Rust: about 2000 lines of tested logic duplicated, with a second implementation to keep identical to the prototype, for no price-integrity gain.
- Store balances as columns: a stale-value risk on every absence decision.

## Consequences
- Gains: one source of behaviour (the prototype's tests), thin endpoints.
- Costs: list endpoints must return a whole period (members, absences, adjustments) unpaginated or with a high cap.
- What becomes harder: a server-side export of balances (payslip hand-off) would need the port later; this ADR is then superseded.
