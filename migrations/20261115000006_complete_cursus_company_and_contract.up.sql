ALTER TABLE cursus.host_company
    ADD COLUMN group_name           TEXT,
    ADD COLUMN capital              TEXT,
    ADD COLUMN phone                TEXT,
    ADD COLUMN email                TEXT,
    ADD COLUMN website              TEXT,
    ADD COLUMN group_employee_count INTEGER,
    ADD COLUMN activity_domains     TEXT,
    ADD COLUMN missions             TEXT,
    ADD CONSTRAINT chk_cursus_host_company_group_employee_count
        CHECK (group_employee_count IS NULL OR group_employee_count >= 0);

CREATE TABLE cursus.host_company_address (
    id              UUID        PRIMARY KEY,
    org_id          UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    host_company_id UUID        NOT NULL,
    label           TEXT,
    line1           TEXT        NOT NULL,
    line2           TEXT,
    postal_code     TEXT,
    city            TEXT,
    is_main         BOOLEAN     NOT NULL DEFAULT false,
    position        INTEGER     NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (org_id, host_company_id) REFERENCES cursus.host_company(org_id, id) ON DELETE CASCADE
);

CREATE INDEX idx_cursus_host_company_address_company
    ON cursus.host_company_address(org_id, host_company_id, position);
CREATE UNIQUE INDEX uq_cursus_host_company_address_main
    ON cursus.host_company_address(host_company_id) WHERE is_main;

INSERT INTO cursus.host_company_address (id, org_id, host_company_id, line1, is_main, position)
SELECT gen_random_uuid(), org_id, id, btrim(address), true, 0
FROM cursus.host_company
WHERE address IS NOT NULL AND btrim(address) <> '';

ALTER TABLE cursus.host_company DROP COLUMN address;

ALTER TABLE cursus.contract
    ADD COLUMN comment TEXT,
    DROP COLUMN cerfa_file_key;

ALTER TABLE cursus.contract_tutor
    ADD COLUMN is_main  BOOLEAN,
    ADD COLUMN position INTEGER;

UPDATE cursus.contract_tutor
SET is_main = (rank = 'MAIN'),
    position = CASE WHEN rank = 'MAIN' THEN 0 ELSE 1 END;

ALTER TABLE cursus.contract_tutor
    ALTER COLUMN is_main SET NOT NULL,
    ALTER COLUMN position SET NOT NULL,
    DROP CONSTRAINT chk_cursus_contract_tutor_rank,
    DROP CONSTRAINT contract_tutor_pkey,
    DROP COLUMN rank,
    ADD PRIMARY KEY (contract_id, company_tutor_id);

CREATE UNIQUE INDEX uq_cursus_contract_tutor_main
    ON cursus.contract_tutor(contract_id) WHERE is_main;
