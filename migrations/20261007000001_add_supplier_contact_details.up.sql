-- A DOE prints the supplier's address and phone under every supply. Until
-- now a supplier only existed for a supplier invoice to point at, so it had
-- neither.

ALTER TABLE suppliers
    ADD COLUMN address TEXT NULL,
    ADD COLUMN phone   TEXT NULL,
    ADD COLUMN email   TEXT NULL,
    ADD CONSTRAINT chk_suppliers_address_not_blank CHECK (address IS NULL OR btrim(address) <> ''),
    ADD CONSTRAINT chk_suppliers_phone_not_blank   CHECK (phone IS NULL OR btrim(phone) <> ''),
    ADD CONSTRAINT chk_suppliers_email_not_blank   CHECK (email IS NULL OR btrim(email) <> '');
