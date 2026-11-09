DROP TABLE IF EXISTS cursus.contract_tutor;
DROP TABLE IF EXISTS cursus.contract;
DROP TABLE IF EXISTS cursus.triad;
ALTER TABLE cursus.closure_reason DROP CONSTRAINT IF EXISTS uq_cursus_closure_reason_org_id;
