DROP INDEX IF EXISTS idx_employee_absences_deleted_at;
DROP INDEX IF EXISTS idx_employee_absences_employee_id;
DROP INDEX IF EXISTS idx_employee_absences_org_id_starts_at;
DROP TABLE IF EXISTS employee_absences;

DROP TYPE IF EXISTS absence_kind;
