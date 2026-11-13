DROP INDEX IF EXISTS cursus.idx_cursus_triad_school_tutor;
ALTER TABLE cursus.triad DROP CONSTRAINT IF EXISTS fk_cursus_triad_school_tutor;
DROP TABLE IF EXISTS cursus.school_tutor;
