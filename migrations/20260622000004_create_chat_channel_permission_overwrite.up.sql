-- migrations/20260622000004_create_chat_channel_permission_overwrite.up.sql

CREATE TABLE chat.channel_permission_overwrite (
    id          UUID        PRIMARY KEY,
    channel_id  UUID        NOT NULL REFERENCES chat.channels(id) ON DELETE CASCADE,
    org_id      UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    target_type TEXT        NOT NULL CHECK (target_type IN ('EVERYONE','ROLE','MEMBER')),
    target_id   UUID        NULL,
    allow       BIGINT      NOT NULL DEFAULT 0,
    deny        BIGINT      NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_overwrite_target CHECK (
        (target_type = 'EVERYONE' AND target_id IS NULL) OR
        (target_type IN ('ROLE','MEMBER') AND target_id IS NOT NULL)
    )
);

-- at most one row per (channel, role/member target)
CREATE UNIQUE INDEX uq_overwrite_target
    ON chat.channel_permission_overwrite (channel_id, target_type, target_id)
    WHERE target_type IN ('ROLE','MEMBER');

-- at most one EVERYONE row per channel
CREATE UNIQUE INDEX uq_overwrite_everyone
    ON chat.channel_permission_overwrite (channel_id)
    WHERE target_type = 'EVERYONE';

CREATE INDEX idx_overwrite_channel_id ON chat.channel_permission_overwrite (channel_id);
