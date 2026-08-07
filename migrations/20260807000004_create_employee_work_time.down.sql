DROP INDEX IF EXISTS idx_employee_work_slots_org_id;
DROP INDEX IF EXISTS idx_employee_work_slots_employee_id_work_date;
DROP TABLE IF EXISTS employee_work_slots;

DROP INDEX IF EXISTS idx_employee_rhythm_slots_rhythm_id;
DROP TABLE IF EXISTS employee_rhythm_slots;

DROP INDEX IF EXISTS idx_employee_rhythms_org_id;
DROP INDEX IF EXISTS idx_employee_rhythms_employee_id_effective_from;
DROP TABLE IF EXISTS employee_rhythms;
