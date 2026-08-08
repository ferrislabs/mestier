CREATE TABLE automation.settings (
    org_id                              UUID        PRIMARY KEY REFERENCES organizations(id),
    event_retention_seconds             BIGINT      NOT NULL DEFAULT 7776000,
    succeeded_delivery_retention_seconds BIGINT     NOT NULL DEFAULT 2592000,
    -- The number of attempts is the length of this array. Storing a separate
    -- count would let the two disagree.
    retry_schedule_seconds              BIGINT[]    NOT NULL DEFAULT ARRAY[5, 30, 120, 600, 3600, 21600]::BIGINT[],
    -- NULL never disables a target: legitimate for a self-hosted instance
    -- calling a machine on its own LAN.
    disable_target_after                INTEGER     NULL DEFAULT 20,
    created_at                          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_automation_settings_retention CHECK (
        event_retention_seconds > 0 AND succeeded_delivery_retention_seconds > 0
    ),
    -- A schedule with no entry is not "no retries"; it is a delivery that
    -- fails once and is never looked at again, silently.
    CONSTRAINT chk_automation_settings_schedule CHECK (
        cardinality(retry_schedule_seconds) > 0
    ),
    CONSTRAINT chk_automation_settings_disable_after CHECK (
        disable_target_after IS NULL OR disable_target_after > 0
    )
);

COMMENT ON TABLE automation.settings IS
    'One row per organization, created on first read. Instance bounds are enforced in the domain, not here: they are deployment configuration, not a schema constant.';

-- Consecutive failures per subscription, so a target that keeps failing can be
-- disabled without scanning its whole delivery history.
ALTER TABLE automation.subscription
    ADD COLUMN consecutive_failures INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN disabled_at TIMESTAMPTZ NULL;
