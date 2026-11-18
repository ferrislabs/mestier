DROP INDEX IF EXISTS survey.idx_response_abandoned;
DROP TABLE IF EXISTS survey.response_photo;
ALTER TABLE survey.response DROP CONSTRAINT IF EXISTS uq_response_org_id;
