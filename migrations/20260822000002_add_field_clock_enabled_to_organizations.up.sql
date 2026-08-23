-- Whether the field app's home screen offers clocking in/out at all.
--
-- Since ADR 0002 the plan, not the clock, is what the profitability
-- calculation reads — nothing computes money from `time_entries` any more.
-- Defaulting this to `false` demotes the clock from a worker's first
-- impression of the product; an organization that still wants a punch clock
-- for payroll or for a client's records flips it back on. `time_entries`,
-- `time_entry_photos` and `day_logs` are untouched either way — this gates
-- the field screen's own UI, not the API routes or the data they still hold.
--
-- Organization-wide behaviour belongs on `organizations`, same as
-- `timezone` (see 20260807000001_planning_harness.up.sql) — not on
-- `automation.settings`, which is the automation engine's own table.

ALTER TABLE organizations
    ADD COLUMN field_clock_enabled BOOLEAN NOT NULL DEFAULT false;
