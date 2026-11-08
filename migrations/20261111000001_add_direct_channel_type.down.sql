-- Postgres cannot drop an enum value: rebuild the type without it. Every
-- constraint and index that compares the column to a literal is bound to the
-- old type, so each is dropped and recreated around the column change.
DELETE FROM chat.channels WHERE channel_type = 'DIRECT';

ALTER TABLE chat.channels
    DROP CONSTRAINT chk_channels_category_text_only,
    DROP CONSTRAINT chk_channels_text_no_parent,
    DROP CONSTRAINT chk_channels_thread_requires_parent,
    DROP CONSTRAINT chk_channels_thread_no_project;
DROP INDEX chat.idx_channels_type;

ALTER TYPE chat.channel_type RENAME TO channel_type_old;
CREATE TYPE chat.channel_type AS ENUM ('TEXT', 'THREAD');
ALTER TABLE chat.channels
    ALTER COLUMN channel_type TYPE chat.channel_type
    USING channel_type::text::chat.channel_type;
DROP TYPE chat.channel_type_old;

ALTER TABLE chat.channels
    ADD CONSTRAINT chk_channels_category_text_only
        CHECK (category_id IS NULL OR channel_type = 'TEXT'),
    ADD CONSTRAINT chk_channels_text_no_parent
        CHECK (channel_type <> 'TEXT' OR parent_id IS NULL),
    ADD CONSTRAINT chk_channels_thread_requires_parent
        CHECK (channel_type <> 'THREAD' OR parent_id IS NOT NULL),
    ADD CONSTRAINT chk_channels_thread_no_project
        CHECK (channel_type <> 'THREAD' OR project_id IS NULL);
CREATE INDEX idx_channels_type ON chat.channels(org_id, channel_type);
