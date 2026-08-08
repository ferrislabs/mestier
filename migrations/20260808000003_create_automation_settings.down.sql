ALTER TABLE automation.subscription
    DROP COLUMN IF EXISTS disabled_at,
    DROP COLUMN IF EXISTS consecutive_failures;

DROP TABLE IF EXISTS automation.settings;
