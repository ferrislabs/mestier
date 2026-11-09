-- New enum values cannot be used in the transaction that adds them: the
-- constraints that mention ANNOUNCEMENT and VOICE live in the next migration.
ALTER TYPE chat.channel_type ADD VALUE IF NOT EXISTS 'ANNOUNCEMENT';
ALTER TYPE chat.channel_type ADD VALUE IF NOT EXISTS 'VOICE';
