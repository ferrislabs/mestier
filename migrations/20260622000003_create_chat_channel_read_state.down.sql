-- migrations/20260622000003_create_chat_channel_read_state.down.sql

DROP INDEX IF EXISTS chat.idx_channel_read_state_org_id;
DROP TABLE IF EXISTS chat.channel_read_state;
