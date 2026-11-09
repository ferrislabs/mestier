-- One row per explicit choice. Without a row the default applies: everything
-- in a direct message, mentions only in a channel.
CREATE TABLE chat.notification_preference (
    user_id    UUID        NOT NULL REFERENCES users(id)         ON DELETE CASCADE,
    channel_id UUID        NOT NULL REFERENCES chat.channels(id) ON DELETE CASCADE,
    org_id     UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    level      TEXT        NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, channel_id),
    CONSTRAINT chk_notification_preference_level
        CHECK (level IN ('ALL', 'MENTIONS', 'NOTHING'))
);

CREATE INDEX idx_notification_preference_org_user
    ON chat.notification_preference(org_id, user_id);
