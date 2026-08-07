-- Split the free-text `employees.name` column into `last_name`/`first_name`.
--
-- `name` was always a single free-text field, filled inconsistently
-- ("Bonnal", "Parmantier", "baptiste" — surname only, no given name,
-- inconsistent case). It cannot be split on whitespace: "Le Guen" would
-- become "Le"/"Guen", "Van Damme" would become "Van"/"Damme". A name split
-- in the wrong place looks correct and is worse than one left incomplete.
--
-- Data migration: `last_name` takes the full, unmodified value of `name`.
-- `first_name` is left `NULL` — "not provided", distinct from an empty
-- string, which would be a `NULL` in disguise. Unlike `customers`, where
-- both `last_name` and `first_name` are `NOT NULL` (the customer form always
-- collects both), `first_name` here is nullable: we are migrating data we
-- cannot reliably split, not collecting fresh data from a form.

ALTER TABLE employees
    ADD COLUMN last_name TEXT,
    ADD COLUMN first_name TEXT;

UPDATE employees
    SET last_name = name,
        first_name = NULL;

ALTER TABLE employees
    ALTER COLUMN last_name SET NOT NULL;

ALTER TABLE employees
    DROP CONSTRAINT chk_employees_name_not_blank;

ALTER TABLE employees
    DROP COLUMN name;

ALTER TABLE employees
    ADD CONSTRAINT chk_employees_last_name_not_blank
        CHECK (length(btrim(last_name)) > 0),
    ADD CONSTRAINT chk_employees_first_name_not_blank_when_present
        CHECK (first_name IS NULL OR length(btrim(first_name)) > 0);
