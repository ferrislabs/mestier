ALTER TABLE chat.channels DROP CONSTRAINT chk_channels_thread_no_project;
DROP INDEX chat.uq_channels_project_id;
ALTER TABLE chat.channels DROP CONSTRAINT fk_channels_project;
ALTER TABLE chat.channels DROP COLUMN project_id;
