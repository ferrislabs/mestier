-- migrations/20260622000003_create_chat_channel_read_state.up.sql

CREATE TABLE chat.channel_read_state (
    user_id              UUID        NOT NULL REFERENCES users(id)          ON DELETE CASCADE,
    channel_id           UUID        NOT NULL REFERENCES chat.channels(id)  ON DELETE CASCADE,
    org_id               UUID        NOT NULL REFERENCES organizations(id)  ON DELETE CASCADE,
    last_read_message_id UUID        NULL,
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT pk_channel_read_state PRIMARY KEY (user_id, channel_id)
);

CREATE INDEX idx_channel_read_state_org_id ON chat.channel_read_state(org_id);
