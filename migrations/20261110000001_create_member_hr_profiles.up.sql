CREATE TABLE member_hr_profiles (
    member_id           UUID        PRIMARY KEY,
    org_id              UUID        NOT NULL REFERENCES organizations(id),
    work_email          TEXT        NULL,
    personal_email      TEXT        NULL,
    mobile              TEXT        NULL,
    landline            TEXT        NULL,
    street              TEXT        NOT NULL DEFAULT '',
    complement          TEXT        NOT NULL DEFAULT '',
    postal_code         TEXT        NULL,
    city                TEXT        NOT NULL DEFAULT '',
    country             TEXT        NOT NULL DEFAULT 'France',
    job_title           TEXT        NULL,
    manager_member_id   UUID        NULL,
    work_permit_end_date DATE       NULL,
    archived_at         TIMESTAMPTZ NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_member_hr_profiles_member
        FOREIGN KEY (member_id, org_id) REFERENCES organization_members(id, organization_id),
    CONSTRAINT fk_member_hr_profiles_manager
        FOREIGN KEY (manager_member_id, org_id) REFERENCES organization_members(id, organization_id),
    CONSTRAINT chk_member_hr_profiles_manager_not_self
        CHECK (manager_member_id IS NULL OR manager_member_id <> member_id)
);

CREATE INDEX idx_member_hr_profiles_org_id ON member_hr_profiles(org_id);
CREATE INDEX idx_member_hr_profiles_manager ON member_hr_profiles(manager_member_id)
    WHERE manager_member_id IS NOT NULL;
