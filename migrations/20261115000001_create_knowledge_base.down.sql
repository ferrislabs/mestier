DROP TABLE IF EXISTS knowledge_links;
DROP TRIGGER IF EXISTS trg_knowledge_articles_search ON knowledge_articles;
DROP TABLE IF EXISTS knowledge_articles;
DROP FUNCTION IF EXISTS knowledge_articles_refresh_search();
