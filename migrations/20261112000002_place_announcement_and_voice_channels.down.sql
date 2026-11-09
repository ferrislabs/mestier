ALTER TABLE chat.channels
    DROP CONSTRAINT chk_channels_category_not_thread_or_direct,
    DROP CONSTRAINT chk_channels_only_thread_has_parent;

ALTER TABLE chat.channels
    ADD CONSTRAINT chk_channels_category_text_only
        CHECK (category_id IS NULL OR channel_type = 'TEXT'),
    ADD CONSTRAINT chk_channels_text_no_parent
        CHECK (channel_type <> 'TEXT' OR parent_id IS NULL);
