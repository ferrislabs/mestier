-- Announcement and voice channels sit in categories like text channels, and
-- like them have no parent: only a thread does.
ALTER TABLE chat.channels
    DROP CONSTRAINT chk_channels_category_text_only,
    DROP CONSTRAINT chk_channels_text_no_parent;

ALTER TABLE chat.channels
    ADD CONSTRAINT chk_channels_category_not_thread_or_direct
        CHECK (category_id IS NULL OR channel_type IN ('TEXT', 'ANNOUNCEMENT', 'VOICE')),
    ADD CONSTRAINT chk_channels_only_thread_has_parent
        CHECK (channel_type = 'THREAD' OR parent_id IS NULL);
