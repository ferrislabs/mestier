# CRM conformity

Goal
  The CRM of the real Mestier behaves like the prototype's CRM module, and every value on its screens comes from the API.

Context
  Frontend lot merged as #622: URLs, navigation and every screen follow the prototype, with localStorage mocks for what the API does not carry (docs/chantiers/crm-conformite.md lists each mock and the lot that replaces it). This spec covers the backend lots B1 to B7. Stripe stays a mock (decision 2026-11-07). ADRs 0011 (deals), 0012 (expenses), 0013 (SIRET).
  The prototype's pure logic and tests under `mestier-rework/src/modules/crm/lib` and `features/*/*.ts` are the reference for behaviour. Rules below cite them as `proto:<file>:<lines>`. Where the real service already satisfies a rule, the lot changes nothing and says so in its report; it adds only the gap.

Glossary
  Deal         Sales opportunity of a customer, shown as "opportunité". Rejected: "lead" (a prospect, not an opportunity), "affaire".
  Expense      Company cost recorded in the treasury journal, shown as "dépense". Not a supplier invoice.
  Credit note  Invoice of kind CREDIT_NOTE, shown as "avoir", numbered in its own series.
  Series       Numbering sequence per organization, kind and year (quote, invoice, credit note).

Invariants
  - Every query is scoped by org_id and an id from another organization answers not found before anything is written.  [test]
  - Money is integer cents. A line net is rounded half away from zero, VAT is computed once per rate on the summed base, never per line.  [test: proto money.ts:70-102]
  - Prices and totals are computed in the backend only.  [type: responses carry the totals, requests carry none]
  - A number is assigned once, never reused, never reassigned. A series counter only moves forward.  [constraint: unique index per org, kind, year + counter in the insert]
  - A quote that is not a draft and an invoice that is not a draft are immutable.  [test]
  - A deal is lost only with a non-empty reason.  [type: `DealStage::Lost(LostReason)` through a `TryFrom`]
  - A deal linked to a quote is never deleted.  [test]
  - An invoice balance is total minus credit notes minus net payments, and a payment never exceeds the balance.  [test]
  - A credit note line never exceeds what remains on its source line.  [test]
  - A customer registration number holds 9 or 14 digits.  [type: `RegistrationNumber` newtype, ADR 0013]
  - A migration reverts and re-applies on the lot's database.  [test: sqlx migrate run, revert, run]
  - No CRM screen reads localStorage for backend data once its lot is merged.  [test]

Out of scope
  - Stripe: events, webhook, customer id, settings. Mocks stay.
  - "Données de démonstration", prototype-only seed data.
  - Dropping `customers.pipeline_stage` (ADR 0011): the API stops exposing it, the column goes in a later migration.
  - Sending e-mail for invoice send and remind: the lot records the event and `sent_at`, delivery is not built.
  - Anything the real CRM has that the prototype lacks (stays as is).

Design
  Dispatch: generic over ports, no `dyn`, no `async-trait`. Layering as `customer`: domain/customer, application/customer, infrastructure/customer/postgres, libs/handlers-customer. Same macros (`#[transactional]`, `#[repository]`), one marker per new aggregate in `infrastructure/registry.rs`.
  Permissions: reuse the existing bits (customers: VIEW_CUSTOMERS, MANAGE_CUSTOMERS; quotes: VIEW_QUOTES; invoices and payments: VIEW_INVOICES; catalogue: MANAGE_REFERENCE; settings: MANAGE_ORG). A new bit only if none fits, reported.
  Clock: use the time source the neighbouring services use. Every rule below that says "today" takes it from there so tests can fix it.

  B1  Billing settings. New table `organization_billing_settings` (org_id primary key). Fields and bounds: payment_terms_days 0..60 (default 30), quote_validity_days >= 1 (30), default_vat_rate_bp in {0, 550, 1000, 2000} (2000), late_penalty non-empty text, recovery_fee_cents >= 0 (4000), invoice_footer text, number_padding 1..6 (4, the real padding today), iban and bic nullable. Credit notes get their own series with prefix AV and counter table beside the quote and invoice ones. Setting the next number of a series is allowed only above the last assigned one. The issuer identity frozen on a quote or invoice gains iban, bic, late_penalty, recovery_fee_cents, invoice_footer. Endpoints under `/organizations/{id}/billing-settings`.
  B2  Customer. Columns: kind (INDIVIDUAL | PROFESSIONAL), website, vat_number, owner_member_id, lead_source, notes, payment_terms_days (0..60, null), billing_contact_id, billing_address_id, converted_at, archived_at. Contact gains mobile. Customer context gains kind (BILLING | SITE | OTHER), country, address_line2. registration_number accepts 9 or 14 digits, Factur-X sends the first 9 under scheme 0002 (ADR 0013). Status moves PROSPECT -> CLIENT -> ARCHIVED set converted_at once (first conversion kept) and archived_at; restoring goes to CLIENT if converted_at is set, else PROSPECT. lead_source values are those of `LeadSource` in proto types.ts:49.
  B3  Deals (ADR 0011). Table `deals`: id, org_id, customer_id, title, amount_cents >= 0, stage (NEW, VISIT, QUOTE_SENT, NEGOTIATION, WON, LOST), expected_close_date, owner_member_id, source, lost_reason, notes, created_at, stage_changed_at, closed_at. Migration: one deal per customer whose pipeline_stage is set (NEW->NEW, CONTACTED and QUALIFIED->VISIT, QUOTE_SENT->QUOTE_SENT, WON->WON, LOST->LOST, title "Opportunité <customer name>", amount 0). Quotes gain deal_id. Endpoints: CRUD, POST `/deals/{id}/stage`, GET `/deals/summary`.
  B4  Quotes. Columns: valid_until, deposit_percent (0..100, one decimal), owner_member_id, notes, deal_id, billing_contact_id, billing_address_id, responded_at, responded_by, decline_reason, revision_of_id; status gains SUPERSEDED. Lines gain discount_percent (0..100), billing_frequency (ONE_SHOT, MONTHLY, QUARTERLY, YEARLY) and a catalogue reference (product_id or service_rate_id). The response adds `recurring` totals per frequency (MONTHLY, QUARTERLY, YEARLY; ONE_SHOT excluded). Frequency never changes the totals.
  B5  Invoices, payments, credit notes. Invoice columns: title, payment_terms_days (0..60), legal_notice, internal_note, sent_at, quote_id, issued_on (the chosen issue date). New table `invoice_events` (kind: SENT | REMINDER | NOTE, detail, at, actor). Payments gain method (CARD, TRANSFER, CHECK, CASH, DIRECT_DEBIT) and may be negative (refund). Credit notes get reason (required), full_cancellation and the AV series. Responses and list rows carry paid_cents, credited_cents, balance_cents and display_state (DRAFT, UNPAID, PARTIAL, OVERDUE, PAID, CANCELLED, REFUND_DUE). Invoice lines gain the catalogue reference. Endpoints: POST `/quotes/{id}/invoices`, DELETE draft, POST send, POST remind, POST note.
  B6  Expenses (ADR 0012) and reference data. Tables `expense_categories` (id, org_id, name, hue) and `expenses` (date, label, supplier_id, category_id, reference, amount_ht_cents, vat_cents, due_date, status TO_PAY | PAID, paid_on, method, advanced_by_member_id, reimbursed_on, customer_id, proof_file_key, note). Read model GET `/ledger` (journal, summary, trend, VAT). Suppliers gain iban, payment_terms_days (0..999), default_category_id, notes, archived_at, and tables `supplier_contacts` and `supplier_addresses` (typed). Products and service rates gain category, billing_frequency, cost_cents, archived_at, and a derived usage_count (lines referencing them).
  B7  Customer list read model: city, primary contact, revenue_12m_cents, overdue_cents, last_activity_at on each row of the customer list.

Rules (acceptance source), by lot
  B1: proto settings.ts:53-80, settings-model.ts:206, numbering.ts:21-69, numbering.test.ts:44-46.
  B2: proto account-draft.ts:104-166, accounts.test.ts:58-184.
  B3: proto pipeline.ts:59-233, pipeline-model.ts:144-171, deal-form.ts:81-94, pipeline.test.ts:93-132.
  B4: proto quotes.ts:23-365, money.ts:70-102, frequency.ts, quote-draft.ts, quote-actions.ts:15-32.
  B5: proto invoices.ts:21-524, payments.ts:19-151, credit-notes.ts:44-222, invoice-new.ts, invoice-issue.ts, credit-request.ts.
  B6: proto expenses.ts:13-262, ledger.ts:62-222, supplier-form.ts, catalog.ts:15-146, catalog-form.ts.
  B7: proto stats.ts:26-58.

Acceptance (Gherkin)
  B1
  @spec-crm-1-1   Scenario: reading the settings of an organization that has none returns the defaults
  @spec-crm-1-2   Scenario: payment terms of 61 days, a padding of 7, a validity of 0 and a negative recovery fee are refused, naming the field
  @spec-crm-1-3   Scenario: the next number of a series cannot be set at or below the last assigned one
  @spec-crm-1-4   Scenario: a credit note takes AV-<year>-<padded n> from its own series and restarts at 1 each year
  @spec-crm-1-5   Scenario: a new padding applies to the next number only, numbers already assigned do not change
  @spec-crm-1-6   Scenario: an issued invoice keeps the iban, late penalty, recovery fee and footer it had at issue after the settings change
  @spec-crm-1-7   Scenario: the webapp settings screen reads and writes the API, not localStorage
  B2
  @spec-crm-2-1   Scenario: a customer stores kind, website, vat number, owner, source, notes, payment terms and the billing contact and address
  @spec-crm-2-2   Scenario: converting a prospect sets converted_at once, converting again keeps the first date
  @spec-crm-2-3   Scenario: archiving sets archived_at, restoring returns to CLIENT if converted_at is set, else to PROSPECT
  @spec-crm-2-4   Scenario: a registration number of 9 or 14 digits is accepted, spaces ignored, any other length refused
  @spec-crm-2-5   Scenario: the Factur-X issuer and customer identifier is the first 9 digits of a 14-digit number
  @spec-crm-2-6   Scenario: a contact stores a mobile and a phone, an address stores kind, country and line 2
  @spec-crm-2-7   Scenario: a billing contact or address from another customer is refused
  @spec-crm-2-8   Scenario: payment terms of 61 days on a customer are refused
  B3
  @spec-crm-3-1   Scenario: creating a deal starts at NEW, refuses an empty title, an unknown customer and a negative amount
  @spec-crm-3-2   Scenario: moving a deal to LOST without a reason changes nothing, with a reason sets lost_reason and closed_at
  @spec-crm-3-3   Scenario: moving a deal to WON sets closed_at and converts a prospect customer
  @spec-crm-3-4   Scenario: reopening a closed deal into an open stage clears closed_at and lost_reason
  @spec-crm-3-5   Scenario: deleting a deal that has a quote is refused
  @spec-crm-3-6   Scenario: archiving a customer loses its open deals with the reason "Compte archivé" and keeps the won ones
  @spec-crm-3-7   Scenario: the summary counts won and lost over the last 90 days and the rate is round(won / (won + lost) * 100), null when none
  @spec-crm-3-8   Scenario: the summary counts customers converted over the last 90 days
  @spec-crm-3-9   Scenario: the migration creates one deal per customer with a pipeline stage and reverts cleanly
  @spec-crm-3-10  Scenario: the customer response no longer carries pipeline_stage
  @spec-crm-3-11  Scenario: the pipeline screen reads the API, not the deals mock
  B4
  @spec-crm-4-1   Scenario: a quote is editable only as a draft, deleting is allowed only for a draft and uses no number
  @spec-crm-4-2   Scenario: sending is refused with an empty title, an invalid line, a total of zero, an empty billing street or city, a valid_until in the past, a deposit outside 0..100
  @spec-crm-4-3   Scenario: sending assigns the number, sets valid_until to today plus the validity days when empty, and freezes issuer and recipient
  @spec-crm-4-4   Scenario: sending moves a NEW or VISIT deal to QUOTE_SENT and sets its amount to the quote subtotal
  @spec-crm-4-5   Scenario: a quote is expired when SENT and valid_until is before today, the day itself stays valid
  @spec-crm-4-6   Scenario: accepting needs SENT and not expired, records responded_at and responded_by, wins the deal or converts the prospect
  @spec-crm-4-7   Scenario: declining needs SENT, records responded_at and decline_reason
  @spec-crm-4-8   Scenario: a revision is possible from SENT or DECLINED, copies the lines as a draft without number or valid_until
  @spec-crm-4-9   Scenario: sending a revision supersedes the source quote only if it is still SENT
  @spec-crm-4-10  Scenario: a line discount of 10 percent applies before VAT and rounds half away from zero
  @spec-crm-4-11  Scenario: two lines of 100,33 euros at 20 percent give a base of 20066 cents and VAT of 4013 cents
  @spec-crm-4-12  Scenario: billing frequency leaves the totals unchanged and the response lists recurring totals per MONTHLY, QUARTERLY, YEARLY
  @spec-crm-4-13  Scenario: a line references its catalogue item and the catalogue usage count includes it
  @spec-crm-4-14  Scenario: changing the customer of a quote that has lines is refused
  @spec-crm-4-15  Scenario: the quote screens read the API, not the quote-extras mock
  B5
  @spec-crm-5-1   Scenario: issuing needs a draft, a known customer, valid lines, a positive total, a billing street and city, a SIRET for a professional customer
  @spec-crm-5-2   Scenario: issuing with an issue date in the future, or before the last issued invoice, is refused
  @spec-crm-5-3   Scenario: the number is assigned at issue with the year of the issue date
  @spec-crm-5-4   Scenario: the due date is the issue date plus payment_terms_days, 0 to 60
  @spec-crm-5-5   Scenario: a draft invoice carries the late penalty, plus the recovery fee for a non-individual customer
  @spec-crm-5-6   Scenario: an issued invoice cannot be edited or deleted, a draft can be deleted
  @spec-crm-5-7   Scenario: creating a full invoice from an accepted quote copies its lines and deducts each issued deposit per rate, drafts excluded
  @spec-crm-5-8   Scenario: a deposit invoice has one lump line per rate priced round(base x percent / 100) and is refused above the remaining percent
  @spec-crm-5-9   Scenario: a quote with an active standard invoice cannot be invoiced again, a cancelled one does not block
  @spec-crm-5-10  Scenario: a payment is refused above the balance and a refund is refused above the net amount collected
  @spec-crm-5-11  Scenario: a payment has a method among CARD, TRANSFER, CHECK, CASH, DIRECT_DEBIT, a non-zero amount and a date not in the future
  @spec-crm-5-12  Scenario: display_state follows the order draft, refund due, cancelled, paid, overdue, partial, unpaid
  @spec-crm-5-13  Scenario: overdue means due date strictly before today
  @spec-crm-5-14  Scenario: a credit note needs an issued invoice, a reason, and lines within what remains on each source line
  @spec-crm-5-15  Scenario: a credit note is dated today, refused before the last credit note or the invoice issue date
  @spec-crm-5-16  Scenario: crediting every remaining line sets full_cancellation and leaves the source invoice unchanged
  @spec-crm-5-17  Scenario: sending records sent_at and an event, reminding records a reminder with the days late, neither changes the balance
  @spec-crm-5-18  Scenario: list rows carry paid, credited, balance and display_state
  @spec-crm-5-19  Scenario: the invoice screens read the API, not the invoice-extras or invoice-activity mocks
  B6
  @spec-crm-6-1   Scenario: an expense needs a label, a date, an amount without VAT above zero, a VAT of zero or more, an existing category and a due date not before its date
  @spec-crm-6-2   Scenario: an advanced expense is created PAID, paid_on is its date, the method is CARD, there is no due date
  @spec-crm-6-3   Scenario: marking paid works only on a TO_PAY expense that is not advanced and records paid_on and a method
  @spec-crm-6-4   Scenario: reimbursing works only on an advanced expense not yet reimbursed, a second call changes nothing
  @spec-crm-6-5   Scenario: the journal signs receipts (refunds negative), expenses as minus total, sorted by date descending
  @spec-crm-6-6   Scenario: the journal summary gives receipts, expenses, net, pending and unallocated amounts
  @spec-crm-6-7   Scenario: the trend returns the last 6 months, empty months included
  @spec-crm-6-8   Scenario: VAT collected is the VAT of issued invoices minus credit notes, deductible is the VAT of expenses, due is the difference
  @spec-crm-6-9   Scenario: a payment without invoice and above zero is unallocated and can be allocated to an issued invoice within its balance
  @spec-crm-6-10  Scenario: a supplier stores iban, payment terms 0..999, default category, notes, several contacts and typed addresses, and archives
  @spec-crm-6-11  Scenario: a supplier registration number of 9 or 14 digits or empty is accepted
  @spec-crm-6-12  Scenario: a catalogue item stores category, billing frequency, cost and archived, a negative price or cost is refused
  @spec-crm-6-13  Scenario: editing a catalogue item leaves the lines already copied into quotes and invoices unchanged
  @spec-crm-6-14  Scenario: the treasury, suppliers and catalogue screens read the API, not the ledger, supplier-extras, catalog-extras or catalog-usage mocks
  B7
  @spec-crm-7-1   Scenario: a customer row carries its city and primary contact
  @spec-crm-7-2   Scenario: revenue_12m is issued invoices minus credit notes over 365 days, floored at zero
  @spec-crm-7-3   Scenario: overdue_cents sums the balances of issued invoices past their due date, a partly paid one counts its balance only
  @spec-crm-7-4   Scenario: last_activity_at is the latest of the quote and invoice dates, null when none
  @spec-crm-7-5   Scenario: the client list screen makes one request per page, not two per row

Simulation
  No. Nothing here is concurrent or time-dependent beyond fixed-clock unit tests. The one concurrency point, number assignment, is covered by the unique index and a counter updated in the insert, tested by two sequential inserts and a constraint violation.

Workstreams
  Sequential, one branch and one PR per lot, each stacked on the previous (B1 on feature/crm-front, B2 on B1, and so on). One sub-agent per lot, run one after another, each holding a coherent tree, so each runs the full exit block itself. A lot owns, during its turn, the convergence files it needs (Cargo.toml, lib.rs, registry.rs, router.rs, openapi.rs, its migration). Its frontend half: regenerate the API client, replace the mocks of its scope with real hooks, delete the mock files, adapt the tests.
  Branches: backend/crm-b1-settings, backend/crm-b2-customer, backend/crm-b3-deals, backend/crm-b4-quotes, backend/crm-b5-invoices, backend/crm-b6-expenses, backend/crm-b7-list.
  Migrations, assigned: 20261109000001 (B1), 20261109000002 (B2), 20261109000003 (B3), 20261109000004 (B4), 20261109000005 (B5), 20261109000006 and 20261109000007 (B6). B7 adds none.
  Regenerated at integration: `.sqlx` by `cargo sqlx prepare --workspace -- --all-targets`, `apps/webapp/src/api/api.client.ts` and `api.tanstack.ts` by `pnpm gen:api` against a running API, `apps/webapp/src/routeTree.gen.ts` by `pnpm build`. Nobody edits them by hand.
  Database: each agent works on database `mestier_crm` of the local Postgres container, never on the developer's database. `.env` is not edited.

Frozen contracts
  JSON is snake_case, ids are UUIDs, money is `*_cents` integers, dates are `YYYY-MM-DD`, instants are RFC 3339.
  Enums (wire form): customer kind INDIVIDUAL | PROFESSIONAL; address kind BILLING | SITE | OTHER; deal stage NEW | VISIT | QUOTE_SENT | NEGOTIATION | WON | LOST; quote status DRAFT | SENT | ACCEPTED | DECLINED | CANCELLED | SUPERSEDED; billing frequency ONE_SHOT | MONTHLY | QUARTERLY | YEARLY; payment method CARD | TRANSFER | CHECK | CASH | DIRECT_DEBIT; invoice display_state DRAFT | UNPAID | PARTIAL | OVERDUE | PAID | CANCELLED | REFUND_DUE; expense status TO_PAY | PAID; invoice event kind SENT | REMINDER | NOTE.
  Error codes: a rule refused answers 422 with a named code and the field, an id of another organization 404, a state conflict 409. New codes: E_DEAL_LOST_REASON_REQUIRED, E_DEAL_HAS_QUOTE, E_QUOTE_LOCKED, E_QUOTE_EXPIRED, E_QUOTE_NOT_SENT, E_INVOICE_LOCKED, E_INVOICE_DATE_RANGE, E_PAYMENT_EXCEEDS_BALANCE, E_REFUND_EXCEEDS_COLLECTED, E_CREDIT_EXCEEDS_REMAINING, E_DEPOSIT_EXCEEDS_REMAINING, E_QUOTE_ALREADY_INVOICED, E_NUMBER_NOT_FORWARD.
  Endpoint shapes are fixed by the lot's section in Design. Field names are those of the prototype types (camelCase there, snake_case here), minus Stripe fields and minus `history` (derived from events).

Risks / open questions
  - Existing VAT rounding may differ from the rule above (summed base per rate). Owner: B4 agent reports the current behaviour before changing it; default is to change only if the E19 example fails, and to apply it to documents created after the migration.
  - Credit notes share a number series with invoices today. Moving them to AV: existing credit notes keep their numbers. Owner: B1 agent.
  - Consumers of `pipeline_stage` (automation connectors, MCP tools, webapp) are not inventoried. Owner: B3 agent greps first and lists them in its report.
  - The default padding is 4 in the real code and 3 in the prototype's tests. Decision: keep 4.
  - H18: the prototype lets an expense be edited or deleted at any status. Decision: same, no lock.
  - The `.sqlx` directory is large and regenerated: each PR diff buries the real change there. Owner: orchestrator, noted in each PR.

Verify (exit)
  cargo fmt --all --check
  SQLX_OFFLINE=true cargo check --workspace
  cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  cargo test -p mestier-core --lib -- --ignored --test-threads=1   (DATABASE_URL on mestier_crm; the cucumber harnesses reject --ignored)
  sqlx migrate run, revert, run                  (--source migrations)
  cd apps/webapp && ./node_modules/.bin/biome check src && ./node_modules/.bin/vitest run && pnpm build

Amended: 2026-11-07 B1 stored the settings on the Organization aggregate and added the handlers to handlers-organization (no new aggregate marker, no new crate); the padding goes through allocate_number of quotes and invoices; only invoices freeze the billing mentions, quotes freeze no issuer identity today.
Amended: 2026-11-07 B3 creates the deal of a customer only when the customer is not archived (and not deleted) at migration time, and leaves `customers.pipeline_stage` in the table with its default while the API neither exposes nor accepts it; lost deals of the migration carry the reason "Non renseigné"; the summary window starts at midnight UTC ninety days before today; "won" converts a customer only when it is a prospect.
Amended: 2026-11-07 B2 keeps the existing 403 for an id of another organization on a customer (the membership barrier answers before the lookup); the 404 rule applies to ids nested in a body. The customer PATCH replaces every field, an absent field is cleared; the owner is checked only when it changes.
