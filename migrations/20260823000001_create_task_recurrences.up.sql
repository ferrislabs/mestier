-- Task recurrence: the rule that says "every Tuesday" or "the 1st of the
-- month", and the horizon up to which it has already been turned into real
-- `tasks` rows.
--
-- Two tables, because a rule and an occurrence are different things with
-- different lifetimes: the rule survives edits to the series, an occurrence
-- is a `tasks` row that can be edited or deleted on its own without touching
-- the rule at all.
--
-- The rule is explicit columns per frequency (`frequency` plus
-- `weekly_weekdays`/`monthly_day`, exactly one populated depending on
-- `frequency`), not an RRULE string. Making invalid states unrepresentable
-- is the house rule (see CLAUDE.md), and a TEXT column holding a grammar is
-- the opposite of that: nothing would stop `'FREQ=NOPE;BYDAY=8'` from being
-- stored, and every reader would have to parse and re-validate it. Rejected
-- on purpose, even though RRULE is the "standard" shape — the trade only
-- pays off once the product needs RRULE's full generality (COUNT, BYSETPOS,
-- multiple BYDAY-with-ordinal...), and the artisan-facing trade only ever
-- needs daily, weekly-on-these-weekdays, or monthly-on-this-day.
--
-- `starts_on`/`ends_on` are calendar dates, not instants: a recurrence is a
-- wall-clock claim ("every Tuesday at 9am, in the organization's own
-- timezone"), and `timezone` plus `start_time` is what turns a date into the
-- actual UTC instant a task needs, DST included.
--
-- `horizon_filled_to` is the watermark: every occurrence up to and including
-- this date already has a materialized `tasks` row. It moves forward with
-- the same transaction that inserts the rows for the new date range (see
-- `TaskRecurrenceService`), so it is never ahead of what was actually
-- persisted.

CREATE TYPE task_recurrence_frequency AS ENUM (
    'DAILY',
    'WEEKLY',
    'MONTHLY'
);

CREATE TABLE task_recurrences (
    id                   UUID                       PRIMARY KEY,
    org_id               UUID                       NOT NULL REFERENCES organizations(id),
    frequency            task_recurrence_frequency  NOT NULL,
    -- ISO weekday numbers, 1 (Monday) through 7 (Sunday). Populated only for
    -- WEEKLY, and never empty when it is — a weekly recurrence with no
    -- weekday would produce nothing, which is not "weekly", it is "never".
    weekly_weekdays      SMALLINT[]                 NULL,
    -- 1 through 31. Populated only for MONTHLY. A month shorter than this
    -- clamps to its own last day (`TaskRecurrenceService::expand_occurrences`)
    -- rather than skipping the month outright, so "monthly" always means
    -- once a month.
    monthly_day          SMALLINT                   NULL,
    starts_on            DATE                       NOT NULL,
    ends_on              DATE                       NULL,
    horizon_filled_to    DATE                       NOT NULL,
    -- IANA name (e.g. `Europe/Paris`), not a fixed UTC offset: the whole
    -- point is that "9am" stays 9am local across a DST change.
    timezone             TEXT                       NOT NULL,
    start_time           TIME                       NOT NULL,
    duration_minutes     INTEGER                    NOT NULL,
    all_day              BOOLEAN                    NOT NULL DEFAULT false,
    -- The template every materialized occurrence copies onto its own `tasks`
    -- row. `assignee_member_ids` is the complete set applied to every
    -- occurrence, mirroring the `PATCH /tasks/{id}` contract's own
    -- `assignees`: always the full list, never a delta.
    title                TEXT                       NOT NULL,
    description          TEXT                       NULL,
    blocks_availability  BOOLEAN                    NOT NULL DEFAULT true,
    customer_id          UUID                       NULL REFERENCES customers(id),
    customer_context_id  UUID                       NULL REFERENCES customer_contexts(id),
    project_id           UUID                       NULL,
    assignee_member_ids  UUID[]                     NOT NULL DEFAULT '{}',
    deleted_at           TIMESTAMPTZ                NULL,
    created_at           TIMESTAMPTZ                NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ                NOT NULL DEFAULT now(),

    CONSTRAINT chk_task_recurrences_title_not_blank
        CHECK (btrim(title) <> ''),
    CONSTRAINT chk_task_recurrences_weekly_days
        CHECK (
            (frequency = 'WEEKLY' AND weekly_weekdays IS NOT NULL AND array_length(weekly_weekdays, 1) > 0)
            OR (frequency <> 'WEEKLY' AND weekly_weekdays IS NULL)
        ),
    CONSTRAINT chk_task_recurrences_monthly_day
        CHECK (
            (frequency = 'MONTHLY' AND monthly_day BETWEEN 1 AND 31)
            OR (frequency <> 'MONTHLY' AND monthly_day IS NULL)
        ),
    CONSTRAINT chk_task_recurrences_ends_on_not_before_starts_on
        CHECK (ends_on IS NULL OR ends_on >= starts_on),
    CONSTRAINT chk_task_recurrences_duration_positive
        CHECK (duration_minutes > 0),
    -- Same relaxed pairing as `tasks` since `relax_task_customer_context_pairing`:
    -- a context needs a customer, a customer does not need a context.
    CONSTRAINT chk_task_recurrences_context_requires_customer
        CHECK (customer_context_id IS NULL OR customer_id IS NOT NULL),
    -- The target of the composite foreign keys `tasks.recurrence_id` and
    -- `tasks.project_id` use below — same device as `projects`'s own
    -- `uq_projects_id_org`.
    CONSTRAINT uq_task_recurrences_id_org UNIQUE (id, org_id)
);

CREATE INDEX idx_task_recurrences_org_id ON task_recurrences(org_id);

-- What the horizon-extension pass (#293) scans: every recurrence whose
-- watermark needs pushing forward. Partial on `deleted_at IS NULL` so a
-- deleted series never costs that scan a row.
CREATE INDEX idx_task_recurrences_horizon
    ON task_recurrences(horizon_filled_to)
    WHERE deleted_at IS NULL;

ALTER TABLE task_recurrences
    ADD CONSTRAINT fk_task_recurrences_project
        FOREIGN KEY (project_id, org_id) REFERENCES projects(id, org_id);

CREATE INDEX idx_task_recurrences_project_id ON task_recurrences(project_id);

-- `occurrence_date` is the local calendar date the row was materialized for
-- — kept even once a task detaches from its series (see below), so its own
-- history stays readable. It lets "does this date already have a row"
-- resolve on an index instead of a window scan over `starts_at`.
ALTER TABLE tasks ADD COLUMN recurrence_id UUID NULL;
ALTER TABLE tasks ADD COLUMN occurrence_date DATE NULL;

ALTER TABLE tasks
    ADD CONSTRAINT fk_tasks_recurrence
        FOREIGN KEY (recurrence_id, org_id) REFERENCES task_recurrences(id, org_id);

-- One occurrence per date per recurrence. Partial on `deleted_at IS NULL`:
-- soft-deleting an occurrence must free its date for a later re-fill to use
-- again, or a horizon extension that runs twice (or races a claim) could
-- never recreate a date somebody removed on purpose versus one that was
-- simply never filled. This is also the arbiter
-- `TaskRepository::insert_occurrence_if_absent`'s `ON CONFLICT` targets,
-- which is what makes re-running the fill idempotent.
CREATE UNIQUE INDEX uq_tasks_recurrence_occurrence
    ON tasks(recurrence_id, occurrence_date)
    WHERE recurrence_id IS NOT NULL AND deleted_at IS NULL;
