-- Reconstitutes `name` by recombining the two columns in display order,
-- "{last_name} {first_name}" — the same order the domain formats with, so a
-- row that went through `up` unchanged (first_name still NULL) round-trips
-- to exactly its original `name`. Reverting is otherwise lossy: a
-- `first_name` set after the split is folded into `name` and cannot be
-- separated out again if `up` is re-applied later.

ALTER TABLE employees
    ADD COLUMN name TEXT;

UPDATE employees
    SET name = CASE
        WHEN first_name IS NULL OR btrim(first_name) = '' THEN last_name
        ELSE last_name || ' ' || first_name
    END;

ALTER TABLE employees
    ALTER COLUMN name SET NOT NULL;

ALTER TABLE employees
    ADD CONSTRAINT chk_employees_name_not_blank
        CHECK (length(btrim(name)) > 0);

ALTER TABLE employees
    DROP CONSTRAINT chk_employees_last_name_not_blank,
    DROP CONSTRAINT chk_employees_first_name_not_blank_when_present;

ALTER TABLE employees
    DROP COLUMN last_name,
    DROP COLUMN first_name;
