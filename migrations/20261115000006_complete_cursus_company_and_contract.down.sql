DROP INDEX cursus.uq_cursus_contract_tutor_main;

ALTER TABLE cursus.contract_tutor ADD COLUMN rank TEXT;

UPDATE cursus.contract_tutor SET rank = 'MAIN' WHERE is_main;

UPDATE cursus.contract_tutor ct
SET rank = 'SECONDARY'
FROM (
    SELECT contract_id, company_tutor_id,
           ROW_NUMBER() OVER (PARTITION BY contract_id ORDER BY position, company_tutor_id) AS rn
    FROM cursus.contract_tutor
    WHERE NOT is_main
) ranked
WHERE ct.contract_id = ranked.contract_id
  AND ct.company_tutor_id = ranked.company_tutor_id
  AND ranked.rn = 1;

DELETE FROM cursus.contract_tutor WHERE rank IS NULL;

ALTER TABLE cursus.contract_tutor
    ALTER COLUMN rank SET NOT NULL,
    DROP CONSTRAINT contract_tutor_pkey,
    DROP COLUMN is_main,
    DROP COLUMN position,
    ADD PRIMARY KEY (contract_id, rank),
    ADD CONSTRAINT chk_cursus_contract_tutor_rank CHECK (rank IN ('MAIN', 'SECONDARY'));

ALTER TABLE cursus.contract
    ADD COLUMN cerfa_file_key TEXT,
    DROP COLUMN comment;

ALTER TABLE cursus.host_company ADD COLUMN address TEXT;

UPDATE cursus.host_company h
SET address = a.line1
FROM (
    SELECT DISTINCT ON (host_company_id) host_company_id, line1
    FROM cursus.host_company_address
    ORDER BY host_company_id, is_main DESC, position ASC
) a
WHERE a.host_company_id = h.id;

DROP TABLE cursus.host_company_address;

ALTER TABLE cursus.host_company
    DROP CONSTRAINT chk_cursus_host_company_group_employee_count,
    DROP COLUMN group_name,
    DROP COLUMN capital,
    DROP COLUMN phone,
    DROP COLUMN email,
    DROP COLUMN website,
    DROP COLUMN group_employee_count,
    DROP COLUMN activity_domains,
    DROP COLUMN missions;
