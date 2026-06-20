-- migrations/20260620000001_create_discord_tables.down.sql

ALTER TABLE messages DROP CONSTRAINT IF EXISTS fk_messages_author_webhook;
ALTER TABLE channels DROP CONSTRAINT IF EXISTS fk_channels_origin_message;

DROP INDEX IF EXISTS idx_member_presence_org_id;
DROP TABLE IF EXISTS member_presence;

DROP INDEX IF EXISTS idx_message_reactions_message_id;
DROP TABLE IF EXISTS message_reactions;

DROP INDEX IF EXISTS idx_webhooks_org_id;
DROP INDEX IF EXISTS idx_webhooks_channel_id;
DROP TABLE IF EXISTS webhooks;

DROP INDEX IF EXISTS idx_messages_created_at;
DROP INDEX IF EXISTS idx_messages_author_user_id;
DROP INDEX IF EXISTS idx_messages_org_id;
DROP INDEX IF EXISTS idx_messages_channel_id;
DROP TABLE IF EXISTS messages;

DROP INDEX IF EXISTS idx_channels_type;
DROP INDEX IF EXISTS idx_channels_parent_id;
DROP INDEX IF EXISTS idx_channels_category_id;
DROP INDEX IF EXISTS idx_channels_org_id;
DROP TABLE IF EXISTS channels;

DROP INDEX IF EXISTS idx_categories_position;
DROP INDEX IF EXISTS idx_categories_org_id;
DROP TABLE IF EXISTS categories;

DROP TYPE IF EXISTS presence_status;
DROP TYPE IF EXISTS author_type;
DROP TYPE IF EXISTS channel_type;
