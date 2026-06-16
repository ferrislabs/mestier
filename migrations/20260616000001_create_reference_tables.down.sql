DROP INDEX IF EXISTS uq_service_rates_org_label_active;
DROP INDEX IF EXISTS idx_service_rates_deleted_at;
DROP INDEX IF EXISTS idx_service_rates_org_id;
DROP TABLE IF EXISTS service_rates;

DROP INDEX IF EXISTS uq_equipment_org_name_active;
DROP INDEX IF EXISTS idx_equipment_deleted_at;
DROP INDEX IF EXISTS idx_equipment_org_id;
DROP TABLE IF EXISTS equipment;

DROP INDEX IF EXISTS uq_employees_org_user_active;
DROP INDEX IF EXISTS idx_employees_deleted_at;
DROP INDEX IF EXISTS idx_employees_user_id;
DROP INDEX IF EXISTS idx_employees_org_id;
DROP TABLE IF EXISTS employees;
