-- One row per direct message conversation. The pair is stored ordered
-- (user_low < user_high) so that "exactly two distinct people" and "one
-- conversation per pair" are the schema's own rules, not a convention.
CREATE TABLE chat.direct_channels (
    channel_id UUID NOT NULL PRIMARY KEY REFERENCES chat.channels(id) ON DELETE CASCADE,
    org_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_low   UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_high  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    CONSTRAINT chk_direct_channels_ordered_pair CHECK (user_low < user_high),
    CONSTRAINT uq_direct_channels_pair UNIQUE (org_id, user_low, user_high)
);

CREATE INDEX idx_direct_channels_user_low  ON chat.direct_channels(org_id, user_low);
CREATE INDEX idx_direct_channels_user_high ON chat.direct_channels(org_id, user_high);

-- A direct channel hangs off nothing: no category, parent thread or project.
ALTER TABLE chat.channels
    ADD CONSTRAINT chk_channels_direct_standalone
    CHECK (
        channel_type <> 'DIRECT'
        OR (category_id IS NULL AND parent_id IS NULL AND project_id IS NULL)
    );
