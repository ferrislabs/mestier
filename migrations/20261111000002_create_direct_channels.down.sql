ALTER TABLE chat.channels DROP CONSTRAINT IF EXISTS chk_channels_direct_standalone;
DROP TABLE IF EXISTS chat.direct_channels;
