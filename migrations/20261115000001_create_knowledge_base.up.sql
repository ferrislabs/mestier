CREATE TABLE knowledge_articles (
    id             UUID        PRIMARY KEY,
    org_id         UUID        NOT NULL REFERENCES organizations(id),
    kind           TEXT        NOT NULL,
    title          TEXT        NOT NULL,
    body           TEXT        NULL,
    file_key       TEXT        NULL,
    mime_type      TEXT        NULL,
    size_bytes     BIGINT      NULL,
    extracted_text TEXT        NULL,
    tags           TEXT[]      NOT NULL DEFAULT '{}',
    search         TSVECTOR    NOT NULL DEFAULT ''::tsvector,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_knowledge_articles_kind CHECK (kind IN ('file', 'note')),
    CONSTRAINT chk_knowledge_articles_title_not_blank CHECK (length(btrim(title)) > 0),
    CONSTRAINT chk_knowledge_articles_tags_max CHECK (cardinality(tags) <= 20),
    CONSTRAINT chk_knowledge_articles_shape CHECK (
        (kind = 'note' AND body IS NOT NULL AND length(btrim(body)) > 0
            AND file_key IS NULL AND mime_type IS NULL AND size_bytes IS NULL)
        OR
        (kind = 'file' AND file_key IS NOT NULL AND mime_type IS NOT NULL AND size_bytes > 0
            AND body IS NULL)
    ),
    CONSTRAINT uq_knowledge_articles_id_org UNIQUE (id, org_id)
);

CREATE INDEX idx_knowledge_articles_org_created ON knowledge_articles (org_id, created_at DESC, id);
CREATE INDEX idx_knowledge_articles_tags ON knowledge_articles USING GIN (tags);
CREATE INDEX idx_knowledge_articles_search ON knowledge_articles USING GIN (search);

-- `array_to_string` is only STABLE, which a generated column refuses, so the
-- search vector is kept by a trigger. Title weighs most, then tags, then text.
CREATE FUNCTION knowledge_articles_refresh_search() RETURNS trigger AS $$
BEGIN
    NEW.search :=
        setweight(to_tsvector('french', NEW.title), 'A')
        || setweight(to_tsvector('french', array_to_string(NEW.tags, ' ')), 'B')
        || setweight(to_tsvector('french', coalesce(NEW.body, '') || ' ' || coalesce(NEW.extracted_text, '')), 'C');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_knowledge_articles_search
    BEFORE INSERT OR UPDATE OF title, tags, body, extracted_text ON knowledge_articles
    FOR EACH ROW EXECUTE FUNCTION knowledge_articles_refresh_search();

CREATE TABLE knowledge_links (
    article_id    UUID        NOT NULL,
    org_id        UUID        NOT NULL,
    resource_type TEXT        NOT NULL,
    resource_id   UUID        NOT NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (article_id, resource_type, resource_id),
    CONSTRAINT chk_knowledge_links_resource_type CHECK (resource_type IN ('project', 'customer', 'quote')),
    CONSTRAINT fk_knowledge_links_article FOREIGN KEY (article_id, org_id)
        REFERENCES knowledge_articles (id, org_id) ON DELETE CASCADE
);

CREATE INDEX idx_knowledge_links_resource ON knowledge_links (org_id, resource_type, resource_id);
