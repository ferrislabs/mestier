ALTER TABLE organizations
    ADD COLUMN deleted_at TIMESTAMPTZ NULL;

CREATE INDEX idx_organizations_deleted_at ON organizations(deleted_at);
