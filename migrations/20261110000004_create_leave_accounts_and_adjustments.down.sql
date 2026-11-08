ALTER TABLE absence_types DROP CONSTRAINT fk_absence_types_account;

UPDATE absence_types SET account_id = NULL WHERE account_id IS NOT NULL;

DROP TABLE leave_adjustments;
DROP TABLE leave_accounts;
