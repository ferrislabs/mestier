-- Restores the pre-#182 shape: `employees` points at a user and carries the
-- person's name again, and the satellite tables come back to `employee_id`.
--
-- Lossy where the forward migration gained expressiveness, and necessarily so:
-- a member with no HR profile has no `employees` row to come back to, so any
-- assignment, absence or work slot belonging to such a member cannot be
-- represented in the old shape and is dropped. Those rows could not have
-- existed before the forward migration ran.

ALTER TABLE employees
    ADD COLUMN user_id UUID NULL REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN last_name TEXT,
    ADD COLUMN first_name TEXT;

UPDATE employees e
SET user_id = m.user_id,
    last_name = m.last_name,
    first_name = m.first_name
FROM organization_members m
WHERE m.id = e.member_id;

ALTER TABLE employees ALTER COLUMN last_name SET NOT NULL;

ALTER TABLE employees
    ADD CONSTRAINT chk_employees_last_name_not_blank
        CHECK (length(btrim(last_name)) > 0),
    ADD CONSTRAINT chk_employees_first_name_not_blank_when_present
        CHECK (first_name IS NULL OR length(btrim(first_name)) > 0);

CREATE INDEX idx_employees_user_id ON employees(user_id);
CREATE UNIQUE INDEX uq_employees_org_user_active
    ON employees(org_id, user_id)
    WHERE user_id IS NOT NULL AND deleted_at IS NULL;

-- -- satellites: back onto the profile ---------------------------------------

ALTER TABLE work_slots RENAME TO employee_work_slots;
ALTER TABLE employee_work_slots ADD COLUMN employee_id UUID;

UPDATE employee_work_slots w
SET employee_id = e.id
FROM employees e
WHERE e.member_id = w.member_id AND e.deleted_at IS NULL;

DELETE FROM employee_work_slots WHERE employee_id IS NULL;

ALTER TABLE employee_work_slots ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE employee_work_slots
    ADD CONSTRAINT employee_work_slots_employee_id_fkey
        FOREIGN KEY (employee_id) REFERENCES employees(id);

ALTER TABLE employee_work_slots DROP CONSTRAINT fk_work_slots_member;
DROP INDEX idx_work_slots_member_id_work_date;
CREATE INDEX idx_employee_work_slots_employee_id_work_date
    ON employee_work_slots(employee_id, work_date);

ALTER TABLE employee_work_slots DROP COLUMN member_id;

ALTER INDEX idx_work_slots_org_id RENAME TO idx_employee_work_slots_org_id;
ALTER TABLE employee_work_slots
    RENAME CONSTRAINT chk_work_slots_ends_after_starts
        TO chk_employee_work_slots_ends_after_starts;

ALTER TABLE absences RENAME TO employee_absences;
ALTER TABLE employee_absences ADD COLUMN employee_id UUID;

UPDATE employee_absences a
SET employee_id = e.id
FROM employees e
WHERE e.member_id = a.member_id AND e.deleted_at IS NULL;

DELETE FROM employee_absences WHERE employee_id IS NULL;

ALTER TABLE employee_absences ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE employee_absences
    ADD CONSTRAINT employee_absences_employee_id_fkey
        FOREIGN KEY (employee_id) REFERENCES employees(id);

ALTER TABLE employee_absences DROP CONSTRAINT fk_absences_member;
DROP INDEX idx_absences_member_id;
CREATE INDEX idx_employee_absences_employee_id ON employee_absences(employee_id);

ALTER TABLE employee_absences DROP COLUMN member_id;

ALTER INDEX idx_absences_org_id_starts_at RENAME TO idx_employee_absences_org_id_starts_at;
ALTER INDEX idx_absences_deleted_at RENAME TO idx_employee_absences_deleted_at;
ALTER TABLE employee_absences
    RENAME CONSTRAINT chk_absences_ends_at_after_starts_at
        TO chk_employee_absences_ends_at_after_starts_at;
ALTER TABLE employee_absences
    RENAME CONSTRAINT chk_absences_note_not_blank_when_present
        TO chk_employee_absences_note_not_blank_when_present;

ALTER TABLE task_assignments ADD COLUMN employee_id UUID;

UPDATE task_assignments a
SET employee_id = e.id
FROM employees e
WHERE e.member_id = a.member_id AND e.deleted_at IS NULL;

DELETE FROM task_assignments WHERE employee_id IS NULL;

ALTER TABLE task_assignments ALTER COLUMN employee_id SET NOT NULL;
ALTER TABLE task_assignments
    ADD CONSTRAINT task_assignments_employee_id_fkey
        FOREIGN KEY (employee_id) REFERENCES employees(id) ON DELETE CASCADE;

ALTER TABLE task_assignments DROP CONSTRAINT fk_task_assignments_member;
ALTER TABLE task_assignments DROP CONSTRAINT uq_task_assignments_task_member;
ALTER TABLE task_assignments
    ADD CONSTRAINT uq_task_assignments_task_employee UNIQUE (task_id, employee_id);

DROP INDEX idx_task_assignments_member_id;
CREATE INDEX idx_task_assignments_employee_id ON task_assignments(employee_id);

ALTER TABLE task_assignments DROP COLUMN member_id;

-- -- employees: shed the seat pointer ----------------------------------------

DROP INDEX uq_employees_member_active;
ALTER TABLE employees DROP CONSTRAINT fk_employees_member;
ALTER TABLE employees DROP COLUMN member_id;
