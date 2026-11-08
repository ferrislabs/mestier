# RH conformity

Goal
  The RH module of the real Mestier behaves like the prototype's RH module, and every value on its screens comes from the API.

Context
  The prototype is `mestier-rework/src/modules/rh`. Its screens are already in the repo under `apps/webapp/src/pages/hr/ui/rh` (#594) on a localStorage store (`pages/hr/feature/mocks/rh-store.ts`) bound to the real members and the real absences. Frontend lots F1 and F2 finish the port (URLs, side nav, members list and detail). Backend lots B1 to B6 replace the mock by lot. ADR 0014 keeps balances, planning cells and recaps computed in the webapp.
  Rules cite the prototype as `proto:<file>` under `src/modules/rh`.

Glossary
  Member      Seat of an organization, shown as "membre". Already exists; the HR profile hangs on it.
  Contract    Dated employment contract of a member with a work schedule, shown as "contrat". Not the maintenance contract.
  Absence type  Organization-defined reason of absence (congés payés, maladie...), shown as "motif". Distinct from `AbsenceKind`, which stays the coarse class (LEAVE, SICK, UNAVAILABLE) used by planning.
  Leave account  Counter of acquired days (CP, RTT), shown as "compteur".
  Adjustment  Signed day movement on a leave account, shown as "ajustement" or "report".
  Override    One-day exception to a member's schedule (work or off), shown as "exception".
  Surcharge   Named majoration rate for extra hours, shown as "majoration".
  Extra period  Hours worked outside the schedule, shown as "heures extra".

Invariants
  - Every query is scoped by org_id; an id from another organization answers not found before anything is written.  [test]
  - A mutation by an unauthorized caller is refused 403.  [test]
  - A contract ends on or after its start; a trial end lies within the contract.  [type: `ContractPeriod` through `TryFrom`]
  - A schedule is weekly (7 days), rotation (anchor plus blocks of at least 1 week or month, each a 7-day week) or variable (one default day); a work day ends after it starts and its lunch lies inside it.  [type: `ContractSchedule`, `WorkDay`]
  - A member has at most one contract covering a given day.  [constraint: exclusion on the period + test]
  - An absence runs from start to end, end not before start (a half-day pair on one date is start morning, end afternoon at least).  [type]
  - A decided absence or extra (approved, rejected, cancelled) is never decided again.  [type: status transitions]
  - A rejection carries a non-empty comment.  [type]
  - A type that needs proof refuses approval without a proof file.  [test]
  - An archived absence type, account or surcharge stays readable by past records and cannot be picked for a new one.  [test]
  - An adjustment is non-zero and carries a reason.  [type]
  - One override per member and day.  [constraint: unique index]
  - A member archived stops appearing in new planning and picklists, and keeps its history.  [test]
  - A migration reverts and re-applies on the lot's database.  [test]
  - No RH screen reads localStorage for backend data once its lot is merged.  [test]

Out of scope
  - Computing balances, planning or recaps in Rust (ADR 0014).
  - Mapping the new contract schedule onto the existing `employee_rhythms` and weekly contract minutes: both stay as they are; the schedule is stored and shown, the planning read model keeps using rhythms.
  - Payslip export, e-mail notification of decisions.
  - "Données de démonstration" and the prototype seed.
  - What the real app has and the prototype lacks (Temps de travail, Matériel, cost history, invitations, roles): kept, restyled in the same tokens.

Design
  Layering as `employee`: domain/<x>, application/<x>, infrastructure/<x>/postgres, handler crate area `handlers-planning` or `handlers-reference` (one submodule per aggregate, two lines in lib.rs). Generic over ports, no `dyn`, no `async-trait`. `#[transactional]` use cases, one `#[repository]` marker per aggregate. Events: members and absences keep their current emission; new aggregates emit none in this chantier.
  Permissions: reuse the bits the current screens use. Profile, contracts, documents: MANAGE_MEMBERS. Absence types, accounts, surcharges: MANAGE_REFERENCE. Planning overrides, absence and extra decisions: MANAGE_PLANNING. A new bit only if none fits, reported.
  Actor: the caller's member id fills created_by and decided_by.
  Clock: the time source of the neighbouring services.
  Dates are ISO dates (`YYYY-MM-DD`) on the wire, times `HH:MM`; the schedule is JSONB validated by the domain type, never read as free JSON.

  B1  Member HR profile. On the employee aggregate (the HR profile of a member): work_email, personal_email, mobile, landline, address (street, complement, postal_code, city, country), job_title, manager_member_id (same org, not itself, no cycle), work_permit_end_date, archived_at. A member with no profile reads as empty defaults and a PUT creates it. Endpoints: `GET/PUT /organizations/{id}/members/{member_id}/profile`, `POST .../archive`, `POST .../restore`. proto:members.ts, member-draft.ts.
  B2  Contracts, schedule, documents. Table `member_contracts` (id, org_id, member_id, type CDI|CDD|APPRENTICESHIP|INTERIM|MANAGER, start_date, end_date, trial_end_date, schedule JSONB). Table `member_documents` (id, org_id, member_id, name, kind CONTRACT|IDENTITY|DIPLOMA|MEDICAL|OTHER, file_key, added_at); the file goes through `/files`. Endpoints: contracts CRUD, documents list, create, delete. proto:contracts.ts, contract-schedule.ts, rotation.ts, schedule.ts. Wire: contract `type` and document `kind` in SCREAMING_SNAKE, schedule tagged on `kind` (weekly|rotation|variable), times `HH:MM`; refusals are 422 naming `schedule.week[i]`, `start_date` (overlap) or `trial_end_date`; CDD, APPRENTICESHIP and INTERIM need an `end_date`; `GET /member-contracts` and `GET /member-documents` list the organization.
  B3  Absence types and the absence workflow. Table `absence_types` (id, org_id, name, hue, count_mode WORKING|WORKABLE|CALENDAR, account_id nullable, paid, needs_approval, needs_proof, kind AbsenceKind, archived_at). Defaults created on first read of an organization with none: congés payés, RTT, maladie, intempéries, indisponible, sans solde. `employee_absences` gains type_id, start_part, end_part (MORNING|AFTERNOON), status PENDING|APPROVED|REJECTED|CANCELLED, proof_file_key, created_by, decided_by, decided_at, decision_comment. Existing rows become APPROVED with the default type of their kind. A type with needs_approval false creates APPROVED. starts_at and ends_at stay, derived from the dates and parts in the organization timezone. A request giving only a kind and a window (the original shape) takes the default type of the kind and is approved at once. Only APPROVED absences block planning. Endpoints: types CRUD and archive, absence create with type, `POST /absences/{id}/approve|reject|cancel`. proto:absences.ts, absences-view.ts.
  B4  Leave accounts and adjustments. Tables `leave_accounts` (id, org_id, name, acquisition MONTHLY(per_month)|ANNUAL(days), period_start_month 1..12, carry_over_max >= 0, archived_at) and `leave_adjustments` (id, org_id, member_id, account_id, date, days non-zero, kind ADJUSTMENT|CARRY_OVER, reason, author_member_id). Defaults: CP 2.08 per month from June (25 days a year, carry-over 5), RTT 10 days a year from January. Days are stored as exact hundredths. Endpoints: accounts CRUD, adjustments list (by member, by account), create, delete. The webapp computes balances (ADR 0014). proto:balances.ts, rights-model.ts.
  B5  Day overrides and planning feed. Table `day_overrides` (org_id, member_id, date, kind WORK|OFF, start, end, lunch_start, lunch_end, note) unique on (member_id, date). Endpoints: list by period, upsert, delete. A single `GET /organizations/{id}/hr/planning-feed?from=&to=` returns members, active contracts, absences, overrides of the period so the planning page makes one call (period at most 62 days else 422; members carry an `archived` flag and no hue, absence types are included). An override carries `kind`, `start`, `end`, `lunch {start,end}` and `note`. proto:planning.ts, day-planning.ts.
  B6  Surcharges and extra hours. Tables `surcharges` (id, org_id, name, rate_bp >= 0, kind OVERTIME|SUNDAY|HOLIDAY|NIGHT|OTHER, archived_at) and `extra_periods` (id, org_id, member_id, date, start, end, lunch_start, lunch_end, reason, surcharge_id nullable, rate_bp, status, created_by, decided_by, decided_at, decision_comment). The rate is copied from the surcharge at creation. Defaults (the prototype list): heures supplémentaires 25 %, au-delà de 8 h 50 %, dimanche 100 %, jour férié 100 %, nuit 25 %. Endpoints: surcharges CRUD and archive, extras list, create, approve, reject, cancel. proto:extra.ts, extra-view.ts.

Acceptance (Gherkin)
  F1
  @spec-rh-f1-1   Scenario: the RH module lives at /rh with membres, planning, absences, heures extra, droits et RTT, plus the kept Temps de travail and Matériel
  @spec-rh-f1-2   Scenario: the old /hr URLs redirect to their /rh counterpart
  @spec-rh-f1-3   Scenario: the side nav shows the pending absence and extra counts and the upcoming deadlines
  @spec-rh-f1-4   Scenario: /rh/absences/nouvelle and /rh/extra/nouvelle open the form dialog over the list
  F2
  @spec-rh-f2-1   Scenario: /rh/membres lists the members with filters and /rh/membres/$memberId opens the detail with identity, contracts, schedule, calendar, absences, extra and documents
  @spec-rh-f2-2   Scenario: inviting a member, assigning roles and the cost history stay reachable
  B1
  @spec-rh-1-1    Scenario: a member without profile reads empty defaults, a PUT creates the profile
  @spec-rh-1-2    Scenario: a manager from another organization, the member itself, or a cycle is refused
  @spec-rh-1-3    Scenario: archiving sets archived_at, restoring clears it, history is kept
  @spec-rh-1-4    Scenario: a profile of another organization answers not found
  @spec-rh-1-5    Scenario: the members screen reads and writes the profile through the API
  B2
  @spec-rh-2-1    Scenario: a weekly, a rotation and a variable schedule round-trip unchanged
  @spec-rh-2-2    Scenario: a schedule with a work day ending before it starts, or a lunch outside the day, is refused naming the day
  @spec-rh-2-3    Scenario: two contracts overlapping for one member are refused
  @spec-rh-2-4    Scenario: a trial end outside the contract is refused
  @spec-rh-2-5    Scenario: a document is stored with its file key and deleted as a row only, its file is left in storage
  @spec-rh-2-6    Scenario: the contracts and documents sections read the API
  B3
  @spec-rh-3-1    Scenario: the first read of an organization creates the default types once
  @spec-rh-3-2    Scenario: an absence of a type needing approval is created PENDING, otherwise APPROVED
  @spec-rh-3-3    Scenario: approving a PENDING absence records decider and date, a second decision is refused
  @spec-rh-3-4    Scenario: rejecting needs a comment
  @spec-rh-3-5    Scenario: approving an absence whose type needs proof without a proof file is refused
  @spec-rh-3-6    Scenario: an archived type cannot be chosen for a new absence and still shows on past ones
  @spec-rh-3-7    Scenario: existing absences migrate to APPROVED with the default type of their kind, and the migration reverts
  @spec-rh-3-8    Scenario: a half-day start and end map to the organization-timezone window
  @spec-rh-3-9    Scenario: the absences screen reads types and requests from the API
  B4
  @spec-rh-4-1    Scenario: default accounts exist after the first read
  @spec-rh-4-2    Scenario: an adjustment of zero days or without a reason is refused
  @spec-rh-4-3    Scenario: deleting an adjustment removes it from the list
  @spec-rh-4-4    Scenario: an account with adjustments is archived, not deleted
  @spec-rh-4-5    Scenario: the rights screen reads accounts and adjustments from the API
  B5
  @spec-rh-5-1    Scenario: an override is upserted per member and day, a second one replaces the first
  @spec-rh-5-2    Scenario: a work override with end before start is refused
  @spec-rh-5-3    Scenario: the planning feed returns only the requested period
  @spec-rh-5-4    Scenario: the planning screen reads the feed, not the store
  B6
  @spec-rh-6-1    Scenario: default surcharges exist after the first read
  @spec-rh-6-2    Scenario: an extra period copies the rate of its surcharge, a later change of the surcharge leaves it unchanged
  @spec-rh-6-3    Scenario: approve and reject follow the absence rules (single decision, comment on reject)
  @spec-rh-6-4    Scenario: end not after start is refused
  @spec-rh-6-5    Scenario: the extra screen reads surcharges and periods from the API
  Final
  @spec-rh-9-1    Scenario: `rh-store`, `rh-seed` and `rh-binding` are deleted and nothing under pages/hr imports them

Verify (exit)
  cargo fmt --all --check
  SQLX_OFFLINE=true cargo clippy --workspace --all-targets -- -D warnings
  cargo test --workspace
  cargo test -p mestier-core --lib -- --ignored --test-threads=1   (needs DATABASE_URL on the lot's database)
  cd apps/webapp && pnpm check && pnpm test && pnpm build
