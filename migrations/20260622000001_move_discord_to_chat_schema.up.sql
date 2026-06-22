-- migrations/20260622000001_move_discord_to_chat_schema.up.sql

CREATE SCHEMA IF NOT EXISTS chat;

-- Move tables (indexes, constraints, and cross-schema FKs to public.organizations/public.users remain valid)
ALTER TABLE categories        SET SCHEMA chat;
ALTER TABLE channels          SET SCHEMA chat;
ALTER TABLE messages          SET SCHEMA chat;
ALTER TABLE message_reactions SET SCHEMA chat;
ALTER TABLE webhooks          SET SCHEMA chat;
ALTER TABLE member_presence   SET SCHEMA chat;

-- Move enum types
ALTER TYPE channel_type    SET SCHEMA chat;
ALTER TYPE author_type     SET SCHEMA chat;
ALTER TYPE presence_status SET SCHEMA chat;
