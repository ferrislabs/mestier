CREATE TABLE cursus.school_tutor (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    member_id   UUID        NOT NULL,
    job_title   TEXT,
    archived_at TIMESTAMPTZ,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (org_id, id),
    CONSTRAINT uq_cursus_school_tutor_member UNIQUE (org_id, member_id),
    CONSTRAINT fk_cursus_school_tutor_member
        FOREIGN KEY (member_id, org_id) REFERENCES organization_members(id, organization_id) ON DELETE CASCADE
);

CREATE INDEX idx_cursus_school_tutor_org ON cursus.school_tutor(org_id);

INSERT INTO cursus.school_tutor (id, org_id, member_id)
SELECT gen_random_uuid(), t.org_id, t.pedagogic_tutor_member_id
FROM cursus.triad t
WHERE t.pedagogic_tutor_member_id IS NOT NULL
GROUP BY t.org_id, t.pedagogic_tutor_member_id;

ALTER TABLE cursus.triad
    ADD CONSTRAINT fk_cursus_triad_school_tutor
        FOREIGN KEY (org_id, pedagogic_tutor_member_id)
        REFERENCES cursus.school_tutor(org_id, member_id)
        ON DELETE SET NULL (pedagogic_tutor_member_id);

CREATE INDEX idx_cursus_triad_school_tutor ON cursus.triad(org_id, pedagogic_tutor_member_id);
