DROP INDEX IF EXISTS idx_knowledge_articles_folder;
ALTER TABLE knowledge_articles DROP CONSTRAINT IF EXISTS fk_knowledge_articles_folder;
ALTER TABLE knowledge_articles DROP COLUMN IF EXISTS folder_id;
DROP TABLE IF EXISTS knowledge_folders;
