-- migrations/20260622000002_create_chat_message_attachments.up.sql

CREATE TABLE chat.message_attachments (
    id              UUID    PRIMARY KEY,
    org_id          UUID    NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    message_id      UUID    NOT NULL REFERENCES chat.messages(id) ON DELETE CASCADE,
    storage_key     TEXT    NOT NULL,
    filename        TEXT    NOT NULL,
    mime_type       TEXT    NOT NULL,
    size_bytes      BIGINT  NOT NULL CHECK (size_bytes >= 0),
    position        INTEGER NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_message_attachments_message_id
    ON chat.message_attachments(message_id);

CREATE INDEX idx_message_attachments_org_id
    ON chat.message_attachments(org_id);
