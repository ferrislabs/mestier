-- migrations/20260622000002_create_chat_message_attachments.down.sql

DROP INDEX IF EXISTS chat.idx_message_attachments_org_id;
DROP INDEX IF EXISTS chat.idx_message_attachments_message_id;
DROP TABLE IF EXISTS chat.message_attachments;
