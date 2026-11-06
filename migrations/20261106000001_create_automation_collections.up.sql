CREATE TABLE automation.collection (
    id           UUID        PRIMARY KEY,
    org_id       UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    slug         TEXT        NOT NULL,
    title        TEXT        NOT NULL,
    version      TEXT        NOT NULL,
    description  TEXT        NOT NULL DEFAULT '',
    base_url     TEXT        NOT NULL,
    source_kind  TEXT        NOT NULL,
    source_value TEXT        NOT NULL,
    imported_at  TIMESTAMPTZ NOT NULL,
    security     JSONB       NOT NULL DEFAULT '[]',
    operations   JSONB       NOT NULL,
    schemas      JSONB       NOT NULL DEFAULT '{}',

    CONSTRAINT uq_automation_collection_slug UNIQUE (org_id, slug),
    CONSTRAINT chk_automation_collection_slug
        CHECK (slug ~ '^[a-z0-9]+(-[a-z0-9]+)*$' AND char_length(slug) <= 220),
    CONSTRAINT chk_automation_collection_title
        CHECK (char_length(btrim(title)) BETWEEN 1 AND 200),
    CONSTRAINT chk_automation_collection_source_kind
        CHECK (source_kind IN ('url', 'file')),
    CONSTRAINT chk_automation_collection_security
        CHECK (jsonb_typeof(security) = 'array'),
    CONSTRAINT chk_automation_collection_operations
        CHECK (jsonb_typeof(operations) = 'array'),
    CONSTRAINT chk_automation_collection_schemas
        CHECK (jsonb_typeof(schemas) = 'object')
);

CREATE INDEX idx_automation_collection_org
    ON automation.collection (org_id, lower(title), id);

CREATE TABLE automation.collection_version (
    id            BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    collection_id UUID        NOT NULL REFERENCES automation.collection(id) ON DELETE CASCADE,
    version       TEXT        NOT NULL,
    imported_at   TIMESTAMPTZ NOT NULL,
    source_kind   TEXT        NOT NULL,
    source_value  TEXT        NOT NULL,
    base_url      TEXT        NOT NULL,
    security      JSONB       NOT NULL DEFAULT '[]',

    CONSTRAINT chk_automation_collection_version_source_kind
        CHECK (source_kind IN ('url', 'file')),
    CONSTRAINT chk_automation_collection_version_security
        CHECK (jsonb_typeof(security) = 'array')
);

CREATE INDEX idx_automation_collection_version_collection
    ON automation.collection_version (collection_id, id DESC);

CREATE FUNCTION automation.collection_version_forbid_update() RETURNS trigger AS $$
BEGIN
    RAISE EXCEPTION 'automation.collection_version is append-only';
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_automation_collection_version_forbid_update
    BEFORE UPDATE ON automation.collection_version
    FOR EACH ROW
    EXECUTE FUNCTION automation.collection_version_forbid_update();
