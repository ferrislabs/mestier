-- Absence types (motifs) and the approval workflow of absences.
--
-- `kind` stays the coarse class planning reads; the type is the
-- organization-defined reason that carries it, its counting mode and its
-- approval and proof rules.
--
-- `absence_types.account_id` is a plain UUID for now: the leave accounts table
-- arrives with lot B4, whose migration adds the foreign key.
--
-- Existing absences were all effective, so they migrate to APPROVED, pointed
-- at the default type of their kind. The default types are created here for
-- every organization that already has absences; the others get them on their
-- first read of the types.

CREATE TABLE absence_types (
    id             UUID          PRIMARY KEY,
    org_id         UUID          NOT NULL REFERENCES organizations(id),
    name           TEXT          NOT NULL,
    hue            SMALLINT      NOT NULL,
    count_mode     TEXT          NOT NULL,
    account_id     UUID          NULL,
    paid           BOOLEAN       NOT NULL,
    needs_approval BOOLEAN       NOT NULL,
    needs_proof    BOOLEAN       NOT NULL,
    kind           absence_kind  NOT NULL,
    archived_at    TIMESTAMPTZ   NULL,
    created_at     TIMESTAMPTZ   NOT NULL DEFAULT now(),

    CONSTRAINT uq_absence_types_id_org UNIQUE (id, org_id),
    CONSTRAINT chk_absence_types_name_not_blank CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_absence_types_hue_range CHECK (hue BETWEEN 0 AND 360),
    CONSTRAINT chk_absence_types_count_mode
        CHECK (count_mode IN ('WORKING', 'WORKABLE', 'CALENDAR'))
);

CREATE INDEX idx_absence_types_org_id ON absence_types(org_id, created_at, id);

CREATE TEMP TABLE absence_type_defaults (
    position       INT     PRIMARY KEY,
    name           TEXT    NOT NULL,
    hue            INT     NOT NULL,
    count_mode     TEXT    NOT NULL,
    paid           BOOLEAN NOT NULL,
    needs_approval BOOLEAN NOT NULL,
    needs_proof    BOOLEAN NOT NULL,
    kind           absence_kind NOT NULL
) ON COMMIT DROP;

INSERT INTO absence_type_defaults VALUES
    (0, 'Congés payés',       155, 'WORKING',  true,  true,  false, 'LEAVE'),
    (1, 'RTT',                250, 'WORKING',  true,  true,  false, 'LEAVE'),
    (2, 'Arrêt maladie',       25, 'CALENDAR', true,  false, true,  'SICK'),
    (3, 'Intempéries',        215, 'WORKING',  true,  false, false, 'UNAVAILABLE'),
    (4, 'Événement familial', 320, 'WORKING',  true,  true,  true,  'LEAVE'),
    (5, 'Formation',          190, 'WORKING',  true,  true,  false, 'UNAVAILABLE'),
    (6, 'Congé sans solde',    60, 'WORKING',  false, true,  false, 'LEAVE');

INSERT INTO absence_types (
    id, org_id, name, hue, count_mode, paid, needs_approval, needs_proof, kind, created_at
)
SELECT
    gen_random_uuid(),
    o.org_id,
    d.name,
    d.hue,
    d.count_mode,
    d.paid,
    d.needs_approval,
    d.needs_proof,
    d.kind,
    now() + d.position * interval '1 millisecond'
FROM (SELECT DISTINCT org_id FROM absences) o
CROSS JOIN absence_type_defaults d;

ALTER TABLE absences
    ADD COLUMN type_id              UUID,
    ADD COLUMN start_date           DATE,
    ADD COLUMN start_part           TEXT,
    ADD COLUMN end_date             DATE,
    ADD COLUMN end_part             TEXT,
    ADD COLUMN status               TEXT,
    ADD COLUMN proof_file_key       TEXT        NULL,
    ADD COLUMN created_by_member_id UUID        NULL,
    ADD COLUMN decided_by_member_id UUID        NULL,
    ADD COLUMN decided_at           TIMESTAMPTZ NULL,
    ADD COLUMN decision_comment     TEXT        NULL;

UPDATE absences a
SET type_id = t.id
FROM (
    SELECT DISTINCT ON (org_id, kind) id, org_id, kind
    FROM absence_types
    ORDER BY org_id, kind, created_at, id
) t
WHERE t.org_id = a.org_id AND t.kind = a.kind;

-- Same mapping as `AbsenceSpan::from_window`: a window starting from noon
-- starts in the afternoon, one ending at midnight ends the previous day's
-- afternoon, one ending at or before noon ends in the morning.
UPDATE absences a
SET start_date = timezone(o.timezone, a.starts_at)::date,
    start_part = CASE
        WHEN timezone(o.timezone, a.starts_at)::time >= TIME '12:00'
            THEN 'AFTERNOON' ELSE 'MORNING' END,
    end_date = CASE
        WHEN timezone(o.timezone, a.ends_at)::time = TIME '00:00'
            THEN timezone(o.timezone, a.ends_at)::date - 1
        ELSE timezone(o.timezone, a.ends_at)::date END,
    end_part = CASE
        WHEN timezone(o.timezone, a.ends_at)::time = TIME '00:00' THEN 'AFTERNOON'
        WHEN timezone(o.timezone, a.ends_at)::time <= TIME '12:00' THEN 'MORNING'
        ELSE 'AFTERNOON' END,
    status = 'APPROVED'
FROM organizations o
WHERE o.id = a.org_id;

ALTER TABLE absences
    ALTER COLUMN type_id    SET NOT NULL,
    ALTER COLUMN start_date SET NOT NULL,
    ALTER COLUMN start_part SET NOT NULL,
    ALTER COLUMN end_date   SET NOT NULL,
    ALTER COLUMN end_part   SET NOT NULL,
    ALTER COLUMN status     SET NOT NULL;

ALTER TABLE absences
    ADD CONSTRAINT fk_absences_type
        FOREIGN KEY (type_id, org_id) REFERENCES absence_types(id, org_id),
    ADD CONSTRAINT fk_absences_created_by
        FOREIGN KEY (created_by_member_id, org_id)
        REFERENCES organization_members(id, organization_id),
    ADD CONSTRAINT fk_absences_decided_by
        FOREIGN KEY (decided_by_member_id, org_id)
        REFERENCES organization_members(id, organization_id),
    ADD CONSTRAINT chk_absences_parts
        CHECK (start_part IN ('MORNING', 'AFTERNOON') AND end_part IN ('MORNING', 'AFTERNOON')),
    ADD CONSTRAINT chk_absences_status
        CHECK (status IN ('PENDING', 'APPROVED', 'REJECTED', 'CANCELLED')),
    ADD CONSTRAINT chk_absences_end_not_before_start
        CHECK (
            end_date > start_date
            OR (end_date = start_date AND NOT (start_part = 'AFTERNOON' AND end_part = 'MORNING'))
        ),
    ADD CONSTRAINT chk_absences_proof_file_key_not_blank
        CHECK (proof_file_key IS NULL OR length(btrim(proof_file_key)) > 0),
    ADD CONSTRAINT chk_absences_decision_comment_not_blank
        CHECK (decision_comment IS NULL OR length(btrim(decision_comment)) > 0);

CREATE INDEX idx_absences_type_id ON absences(type_id);
CREATE INDEX idx_absences_org_status ON absences(org_id, status);
