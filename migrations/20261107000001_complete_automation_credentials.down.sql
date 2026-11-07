DO $$
DECLARE
    split_rows BIGINT;
BEGIN
    SELECT count(*) INTO split_rows
    FROM automation.credential
    WHERE fields_split AND origin = 'supplied';

    IF split_rows > 0 THEN
        RAISE EXCEPTION
            'cannot revert 20261107000001: % supplied credential(s) keep their readable fields apart from their secrets, and dropping those columns would lose them for good',
            split_rows;
    END IF;
END
$$;

DROP TABLE IF EXISTS automation.credential_tombstone;

ALTER TABLE automation.credential
    DROP CONSTRAINT IF EXISTS chk_automation_credential_data_object,
    DROP CONSTRAINT IF EXISTS chk_automation_credential_status,
    DROP CONSTRAINT IF EXISTS chk_automation_credential_description;

ALTER TABLE automation.credential
    DROP COLUMN IF EXISTS fields_split,
    DROP COLUMN IF EXISTS secret_names,
    DROP COLUMN IF EXISTS data,
    DROP COLUMN IF EXISTS expires_at,
    DROP COLUMN IF EXISTS account,
    DROP COLUMN IF EXISTS status,
    DROP COLUMN IF EXISTS description;
