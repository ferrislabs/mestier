-- A new enum value cannot be used in the transaction that adds it: the table
-- and constraints that mention DIRECT live in the next migration.
ALTER TYPE chat.channel_type ADD VALUE IF NOT EXISTS 'DIRECT';
