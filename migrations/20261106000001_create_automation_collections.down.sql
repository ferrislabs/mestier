DROP TRIGGER IF EXISTS trg_automation_collection_version_forbid_update
    ON automation.collection_version;
DROP FUNCTION IF EXISTS automation.collection_version_forbid_update();

DROP INDEX IF EXISTS automation.idx_automation_collection_version_collection;
DROP TABLE IF EXISTS automation.collection_version;

DROP INDEX IF EXISTS automation.idx_automation_collection_org;
DROP TABLE IF EXISTS automation.collection;
