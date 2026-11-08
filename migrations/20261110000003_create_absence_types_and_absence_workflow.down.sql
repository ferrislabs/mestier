UPDATE absences SET deleted_at = now() WHERE status <> 'APPROVED' AND deleted_at IS NULL;

DROP INDEX idx_absences_org_status;
DROP INDEX idx_absences_type_id;

ALTER TABLE absences
    DROP COLUMN decision_comment,
    DROP COLUMN decided_at,
    DROP COLUMN decided_by_member_id,
    DROP COLUMN created_by_member_id,
    DROP COLUMN proof_file_key,
    DROP COLUMN status,
    DROP COLUMN end_part,
    DROP COLUMN end_date,
    DROP COLUMN start_part,
    DROP COLUMN start_date,
    DROP COLUMN type_id;

DROP TABLE absence_types;
