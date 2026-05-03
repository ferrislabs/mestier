CREATE TABLE organization_members (
    id              UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id         UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_members_org_user UNIQUE (organization_id, user_id)
);

CREATE INDEX idx_members_organization_id ON organization_members(organization_id);
CREATE INDEX idx_members_user_id         ON organization_members(user_id);

CREATE TABLE member_roles (
    id        UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    member_id UUID        NOT NULL REFERENCES organization_members(id) ON DELETE CASCADE,
    role_id   UUID        NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_member_roles UNIQUE (member_id, role_id)
);

CREATE INDEX idx_member_roles_member_id ON member_roles(member_id);
CREATE INDEX idx_member_roles_role_id   ON member_roles(role_id);
