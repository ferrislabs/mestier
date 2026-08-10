-- `employees` stops being the profile of a *user* and becomes the contractual
-- profile of a *member* — a seat in the organization, which since #180 may be
-- occupied or free and carries its own name.
--
-- The old shape had `employees` point at `users` and hold the name itself.
-- That made a person plannable only if they had an HR record, and forced the
-- planning read model to reconcile two rosters by `user_id`. Once every member
-- is a plannable resource on its own, `employees` keeps only what is genuinely
-- contractual: `hourly_rate_cents` and `weekly_contract_minutes`.
--
-- Every satellite table is re-pointed by the same question: does this concern
-- *the person* or *their contract*? Assignments, absences and dated work slots
-- concern the person, and follow the member. `employee_rhythms` is the
-- translation of a contract into recurring slots, so it stays on the profile.

-- -- employees: the seat it profiles ----------------------------------------

ALTER TABLE employees ADD COLUMN member_id UUID;

-- An employee whose account already holds a seat in this organization profiles
-- that seat rather than opening a second one for the same person.
UPDATE employees e
SET member_id = m.id
FROM organization_members m
WHERE m.organization_id = e.org_id
  AND m.user_id IS NOT NULL
  AND m.user_id = e.user_id
  AND m.deleted_at IS NULL;

-- Everyone else gets a seat created for them, named from the employee record
-- while it still carries a name. The seat stays free unless the employee has
-- an account that holds no seat yet — `uq_employees_org_user_active` made two
-- active employees sharing a `user_id` impossible, so this can never collide
-- with `uq_members_org_user_active`.
--
-- A seat created for a soft-deleted profile is created soft-deleted too. Seats
-- are user-facing since #181; inventing a live one for someone the
-- organization already removed would put a ghost in the members list.
CREATE TEMP TABLE employee_seat_backfill ON COMMIT DROP AS
SELECT
    e.id                                       AS employee_id,
    gen_random_uuid()                          AS member_id,
    e.org_id,
    e.last_name,
    e.first_name,
    e.deleted_at,
    CASE
        WHEN e.deleted_at IS NULL
             AND e.user_id IS NOT NULL
             AND NOT EXISTS (
                 SELECT 1
                 FROM organization_members m
                 WHERE m.organization_id = e.org_id
                   AND m.user_id = e.user_id
                   AND m.deleted_at IS NULL
             )
        THEN e.user_id
    END                                        AS user_id
FROM employees e
WHERE e.member_id IS NULL;

INSERT INTO organization_members (
    id, organization_id, user_id, last_name, first_name, joined_at, created_at, deleted_at
)
SELECT
    b.member_id,
    b.org_id,
    b.user_id,
    b.last_name,
    b.first_name,
    CASE WHEN b.user_id IS NULL THEN NULL ELSE now() END,
    now(),
    b.deleted_at
FROM employee_seat_backfill b;

UPDATE employees e
SET member_id = b.member_id
FROM employee_seat_backfill b
WHERE b.employee_id = e.id;

ALTER TABLE employees ALTER COLUMN member_id SET NOT NULL;

-- Composite, so a profile referencing a member of another organization is
-- structurally impossible rather than checked in application code. The
-- `UNIQUE (id, organization_id)` target it needs was added by #180.
ALTER TABLE employees
    ADD CONSTRAINT fk_employees_member
        FOREIGN KEY (member_id, org_id)
        REFERENCES organization_members(id, organization_id);

CREATE UNIQUE INDEX uq_employees_member_active
    ON employees(member_id)
    WHERE deleted_at IS NULL;

-- -- satellites: the person, not the contract --------------------------------

ALTER TABLE task_assignments ADD COLUMN member_id UUID;

UPDATE task_assignments a
SET member_id = e.member_id
FROM employees e
WHERE e.id = a.employee_id;

ALTER TABLE task_assignments ALTER COLUMN member_id SET NOT NULL;

ALTER TABLE task_assignments
    DROP CONSTRAINT uq_task_assignments_task_employee;
ALTER TABLE task_assignments
    ADD CONSTRAINT uq_task_assignments_task_member UNIQUE (task_id, member_id);
ALTER TABLE task_assignments
    ADD CONSTRAINT fk_task_assignments_member
        FOREIGN KEY (member_id, org_id)
        REFERENCES organization_members(id, organization_id)
        ON DELETE CASCADE;

DROP INDEX idx_task_assignments_employee_id;
CREATE INDEX idx_task_assignments_member_id ON task_assignments(member_id);

ALTER TABLE task_assignments DROP COLUMN employee_id;

ALTER TABLE employee_absences RENAME TO absences;
ALTER TABLE absences ADD COLUMN member_id UUID;

UPDATE absences a
SET member_id = e.member_id
FROM employees e
WHERE e.id = a.employee_id;

ALTER TABLE absences ALTER COLUMN member_id SET NOT NULL;

ALTER TABLE absences
    ADD CONSTRAINT fk_absences_member
        FOREIGN KEY (member_id, org_id)
        REFERENCES organization_members(id, organization_id);

DROP INDEX idx_employee_absences_employee_id;
CREATE INDEX idx_absences_member_id ON absences(member_id);

ALTER TABLE absences DROP COLUMN employee_id;

ALTER INDEX idx_employee_absences_org_id_starts_at RENAME TO idx_absences_org_id_starts_at;
ALTER INDEX idx_employee_absences_deleted_at RENAME TO idx_absences_deleted_at;
ALTER TABLE absences
    RENAME CONSTRAINT chk_employee_absences_ends_at_after_starts_at
        TO chk_absences_ends_at_after_starts_at;
ALTER TABLE absences
    RENAME CONSTRAINT chk_employee_absences_note_not_blank_when_present
        TO chk_absences_note_not_blank_when_present;

ALTER TABLE employee_work_slots RENAME TO work_slots;
ALTER TABLE work_slots ADD COLUMN member_id UUID;

UPDATE work_slots w
SET member_id = e.member_id
FROM employees e
WHERE e.id = w.employee_id;

ALTER TABLE work_slots ALTER COLUMN member_id SET NOT NULL;

ALTER TABLE work_slots
    ADD CONSTRAINT fk_work_slots_member
        FOREIGN KEY (member_id, org_id)
        REFERENCES organization_members(id, organization_id);

DROP INDEX idx_employee_work_slots_employee_id_work_date;
CREATE INDEX idx_work_slots_member_id_work_date ON work_slots(member_id, work_date);

ALTER TABLE work_slots DROP COLUMN employee_id;

ALTER INDEX idx_employee_work_slots_org_id RENAME TO idx_work_slots_org_id;
ALTER TABLE work_slots
    RENAME CONSTRAINT chk_employee_work_slots_ends_after_starts
        TO chk_work_slots_ends_after_starts;

-- -- employees: shed what belongs to the person ------------------------------

DROP INDEX idx_employees_user_id;
DROP INDEX uq_employees_org_user_active;

ALTER TABLE employees
    DROP CONSTRAINT chk_employees_last_name_not_blank,
    DROP CONSTRAINT chk_employees_first_name_not_blank_when_present;

ALTER TABLE employees
    DROP COLUMN user_id,
    DROP COLUMN last_name,
    DROP COLUMN first_name;
