-- migrations/20260622000001_move_discord_to_chat_schema.down.sql

-- Move tables back to public
ALTER TABLE chat.categories        SET SCHEMA public;
ALTER TABLE chat.channels          SET SCHEMA public;
ALTER TABLE chat.messages          SET SCHEMA public;
ALTER TABLE chat.message_reactions SET SCHEMA public;
ALTER TABLE chat.webhooks          SET SCHEMA public;
ALTER TABLE chat.member_presence   SET SCHEMA public;

-- Move enum types back to public
ALTER TYPE chat.channel_type    SET SCHEMA public;
ALTER TYPE chat.author_type     SET SCHEMA public;
ALTER TYPE chat.presence_status SET SCHEMA public;

DROP SCHEMA IF EXISTS chat;
