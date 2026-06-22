-- migrations/20260622000004_create_chat_channel_permission_overwrite.down.sql

DROP INDEX IF EXISTS chat.idx_overwrite_channel_id;
DROP INDEX IF EXISTS chat.uq_overwrite_everyone;
DROP INDEX IF EXISTS chat.uq_overwrite_target;
DROP TABLE IF EXISTS chat.channel_permission_overwrite;
