-- A customer is an entity — a company, a building manager, a town hall, or a
-- private individual — not a person. The people are already modelled: they live
-- in `customer_contacts`, which carries a role and a primary flag that the
-- customer row never had. Forcing `first_name`/`last_name` on the entity made a
-- company unnameable and duplicated the individual across two tables.

ALTER TABLE customers ADD COLUMN name TEXT;

UPDATE customers
SET name = btrim(first_name || ' ' || last_name);

ALTER TABLE customers ALTER COLUMN name SET NOT NULL;

ALTER TABLE customers
    ADD CONSTRAINT chk_customers_name_not_blank
        CHECK (length(btrim(name)) > 0);

ALTER TABLE customers DROP CONSTRAINT chk_customers_last_name_not_blank;
ALTER TABLE customers DROP CONSTRAINT chk_customers_first_name_not_blank;

ALTER TABLE customers DROP COLUMN first_name;
ALTER TABLE customers DROP COLUMN last_name;
