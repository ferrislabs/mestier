-- How many days ahead a recurrence is materialized, per organization.
--
-- A setting, not a constant in code
-- (`domain::task_recurrence::service::DEFAULT_HORIZON_DAYS`): a shop with a
-- lot of monthly recurrences and a horizon-extension worker that only ticks
-- once a day needs more margin than the default gives, and there is no one
-- number that is right for every organization. An organization absent from
-- this table has never configured anything and gets the code default —
-- exactly the same "created on first write, defaulted until then" shape as
-- `automation.settings` (see `AutomationSettingsRepository::settings_for`).
--
-- No write path is exposed yet — that is a settings-UI workstream of its
-- own, out of scope here. The table exists so the horizon-extension worker
-- (`MestierUseCase::extend_recurrence_horizons_for_organization`) already
-- reads through a setting rather than a hard-coded number, so shipping the
-- UI later touches no Rust at all, only this table.

CREATE TABLE task_recurrence_horizon_settings (
    org_id       UUID        PRIMARY KEY REFERENCES organizations(id) ON DELETE CASCADE,
    horizon_days SMALLINT    NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_task_recurrence_horizon_settings_positive
        CHECK (horizon_days > 0)
);
