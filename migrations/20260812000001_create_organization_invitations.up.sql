-- A shareable, single-use link that grants access to an organization.
--
-- `member_id` is nullable: `Some` targets a seat already created by #181's
-- member API (grants login access to a named-but-vacant seat); `None` means
-- nobody has a seat yet — acceptance creates one, named from the FerrisKey
-- account (see `InvitationService::accept_invitation`).
--
-- The token is stored hashed, never in clear (`token_hash`) — it is checked,
-- never read back. A consumed invitation is marked `consumed_at`, never
-- deleted: the row stays auditable, unlike `organization_members`' own
-- `ON DELETE CASCADE` (see `20260425000004_create_members.up.sql`).
CREATE TABLE organization_invitations (
    id                  UUID        PRIMARY KEY,
    organization_id     UUID        NOT NULL REFERENCES organizations(id),
    member_id           UUID        NULL REFERENCES organization_members(id),
    token_hash          BYTEA       NOT NULL,
    expires_at          TIMESTAMPTZ NOT NULL,
    consumed_at         TIMESTAMPTZ NULL,
    consumed_by_user_id UUID        NULL REFERENCES users(id),
    created_by_user_id  UUID        NOT NULL REFERENCES users(id),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_organization_invitations_expires_after_created
        CHECK (expires_at > created_at)
);

-- The lookup path `accept_invitation` runs on every attempt: hash the
-- presented token, then look up by exact match. Unique so two invitations
-- can never collide onto the same hash undetected.
CREATE UNIQUE INDEX uq_organization_invitations_token_hash
    ON organization_invitations (token_hash);

-- The list-pending-invitations-for-an-organization path (`GET .../invitations`).
CREATE INDEX idx_organization_invitations_org_pending
    ON organization_invitations (organization_id)
    WHERE consumed_at IS NULL;
