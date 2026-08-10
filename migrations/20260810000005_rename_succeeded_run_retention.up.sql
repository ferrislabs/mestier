-- `succeeded_delivery_retention_seconds` has named a retention that no
-- longer belongs to a "delivery" since #201 retired the webhook delivery
-- pipeline in favor of workflow runs (20260810000004). #203 exposes this
-- column through the automation API (`AutomationSettingsBody`), which is
-- the last moment the name can change without breaking a client already
-- relying on it.
--
-- A plain rename: PostgreSQL updates `chk_automation_settings_retention`'s
-- stored definition automatically (it tracks columns by attnum, not name),
-- so nothing else needs touching.
ALTER TABLE automation.settings
    RENAME COLUMN succeeded_delivery_retention_seconds TO succeeded_run_retention_seconds;
