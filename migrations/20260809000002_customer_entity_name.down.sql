-- Splits `name` back on its first space. The reverse is lossy by nature: a
-- company named "Mairie de Saint-Julien" comes back as first_name "Mairie",
-- last_name "de Saint-Julien", and a single-word name gets a placeholder
-- last_name to satisfy the NOT NULL it used to carry. That asymmetry is the
-- point of the forward migration — the old shape could not represent an entity.

ALTER TABLE customers ADD COLUMN first_name TEXT;
ALTER TABLE customers ADD COLUMN last_name TEXT;

UPDATE customers
SET first_name = split_part(btrim(name), ' ', 1),
    last_name  = NULLIF(btrim(substr(btrim(name), strpos(btrim(name), ' ') + 1)), '');

UPDATE customers
SET last_name = first_name
WHERE last_name IS NULL OR btrim(last_name) = '' OR last_name = first_name;

ALTER TABLE customers ALTER COLUMN first_name SET NOT NULL;
ALTER TABLE customers ALTER COLUMN last_name SET NOT NULL;

ALTER TABLE customers
    ADD CONSTRAINT chk_customers_last_name_not_blank
        CHECK (length(btrim(last_name)) > 0);
ALTER TABLE customers
    ADD CONSTRAINT chk_customers_first_name_not_blank
        CHECK (length(btrim(first_name)) > 0);

ALTER TABLE customers DROP CONSTRAINT chk_customers_name_not_blank;
ALTER TABLE customers DROP COLUMN name;
