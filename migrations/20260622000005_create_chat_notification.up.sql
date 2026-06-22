-- migrations/20260622000005_create_chat_notification.up.sql

CREATE TABLE chat.notification (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id)  ON DELETE CASCADE,
    user_id     UUID        NOT NULL REFERENCES users(id)          ON DELETE CASCADE,
    channel_id  UUID        NOT NULL REFERENCES chat.channels(id)  ON DELETE CASCADE,
    message_id  UUID        NOT NULL REFERENCES chat.messages(id)  ON DELETE CASCADE,
    kind        TEXT        NOT NULL CHECK (kind IN ('MENTION','REPLY')),
    read_at     TIMESTAMPTZ NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_notification_user_unread ON chat.notification (user_id, read_at);
