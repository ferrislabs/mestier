# 0012. Expenses are a separate aggregate from supplier invoices

Status: proposed
Date: 2026-11-07

## Context
The treasury journal of the prototype records expenses: a date, a label, an optional supplier, a category, amounts without and with VAT, a status (to pay, paid), a payment method, an advance by a member with a reimbursement date, and a proof. The backend has `supplier_invoices` (received, confirmed, rejected) allocated to projects, imported from Factur-X, and no expense entity.

## Decision
Add `expense` and `expense_category` aggregates scoped by `org_id`. An expense may reference a confirmed supplier invoice but does not require one. The ledger is a read model over payments and expenses.

## Alternatives rejected
- Reuse `supplier_invoices` as expenses: it has no category, no paid state, no advance/reimbursement, and its confirm/reject workflow and project allocation do not apply to a fuel receipt or a member's advance.
- Compute expenses from tasks (`expenses_cents` on `tasks`): those are costs charged to a task, not company expenses.

## Consequences
- Gains: a real journal with categories and reimbursements, independent of the purchase workflow.
- Costs: two overlapping ways to record a cost. The link from expense to supplier invoice is the reconciliation point.
- What becomes harder: profitability must not count a supplier invoice and the expense that references it twice.
