-- migrations/20260622000005_create_chat_notification.down.sql

DROP INDEX IF EXISTS chat.idx_notification_user_unread;
DROP TABLE IF EXISTS chat.notification;
