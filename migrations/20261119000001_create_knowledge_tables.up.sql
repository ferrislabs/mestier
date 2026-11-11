CREATE TABLE knowledge_tables (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id),
    name        TEXT        NOT NULL,
    description TEXT        NULL,
    created_by  UUID        NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_knowledge_tables_name_not_blank CHECK (length(btrim(name)) > 0),
    CONSTRAINT uq_knowledge_tables_id_org UNIQUE (id, org_id)
);

CREATE UNIQUE INDEX uq_knowledge_tables_name ON knowledge_tables (org_id, lower(name));

CREATE TABLE knowledge_table_columns (
    id         UUID        PRIMARY KEY,
    table_id   UUID        NOT NULL,
    org_id     UUID        NOT NULL,
    name       TEXT        NOT NULL,
    kind       TEXT        NOT NULL,
    config     JSONB       NOT NULL DEFAULT '{}'::jsonb,
    position   INTEGER     NOT NULL,
    is_primary BOOLEAN     NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_knowledge_table_columns_name_not_blank CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_knowledge_table_columns_kind CHECK (kind IN (
        'text', 'long_text', 'number', 'currency', 'date', 'checkbox',
        'single_select', 'multi_select', 'link', 'file', 'member'
    )),
    CONSTRAINT chk_knowledge_table_columns_primary_is_text CHECK (NOT is_primary OR kind = 'text'),
    CONSTRAINT chk_knowledge_table_columns_config_object CHECK (jsonb_typeof(config) = 'object'),
    CONSTRAINT fk_knowledge_table_columns_table FOREIGN KEY (table_id, org_id)
        REFERENCES knowledge_tables (id, org_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX uq_knowledge_table_columns_name ON knowledge_table_columns (table_id, lower(name));
CREATE UNIQUE INDEX uq_knowledge_table_columns_primary ON knowledge_table_columns (table_id) WHERE is_primary;
CREATE INDEX idx_knowledge_table_columns_table ON knowledge_table_columns (table_id, position);

CREATE TABLE knowledge_table_rows (
    id         UUID        PRIMARY KEY,
    table_id   UUID        NOT NULL,
    org_id     UUID        NOT NULL,
    "values"   JSONB       NOT NULL DEFAULT '{}'::jsonb,
    created_by UUID        NULL REFERENCES users(id) ON DELETE SET NULL,
    updated_by UUID        NULL REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_knowledge_table_rows_values_object CHECK (jsonb_typeof("values") = 'object'),
    CONSTRAINT fk_knowledge_table_rows_table FOREIGN KEY (table_id, org_id)
        REFERENCES knowledge_tables (id, org_id) ON DELETE CASCADE
);

CREATE INDEX idx_knowledge_table_rows_table ON knowledge_table_rows (table_id, created_at, id);
CREATE INDEX idx_knowledge_table_rows_values ON knowledge_table_rows USING GIN ("values");

CREATE TABLE knowledge_table_views (
    id         UUID        PRIMARY KEY,
    table_id   UUID        NOT NULL,
    org_id     UUID        NOT NULL,
    name       TEXT        NOT NULL,
    kind       TEXT        NOT NULL,
    config     JSONB       NOT NULL DEFAULT '{}'::jsonb,
    position   INTEGER     NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_knowledge_table_views_name_not_blank CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_knowledge_table_views_kind CHECK (kind IN ('grid', 'kanban')),
    CONSTRAINT chk_knowledge_table_views_config_object CHECK (jsonb_typeof(config) = 'object'),
    CONSTRAINT fk_knowledge_table_views_table FOREIGN KEY (table_id, org_id)
        REFERENCES knowledge_tables (id, org_id) ON DELETE CASCADE
);

CREATE UNIQUE INDEX uq_knowledge_table_views_name ON knowledge_table_views (table_id, lower(name));
CREATE INDEX idx_knowledge_table_views_table ON knowledge_table_views (table_id, position);
