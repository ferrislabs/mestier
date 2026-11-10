CREATE TABLE knowledge_folders (
    id         UUID        PRIMARY KEY,
    org_id     UUID        NOT NULL REFERENCES organizations(id),
    parent_id  UUID        NULL,
    name       TEXT        NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_knowledge_folders_name_not_blank CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_knowledge_folders_not_own_parent CHECK (parent_id IS NULL OR parent_id <> id),
    CONSTRAINT uq_knowledge_folders_id_org UNIQUE (id, org_id),
    -- A parent of another organization cannot be referenced, and a folder
    -- that still has children cannot be removed.
    CONSTRAINT fk_knowledge_folders_parent FOREIGN KEY (parent_id, org_id)
        REFERENCES knowledge_folders (id, org_id)
);

-- Siblings never share a name, whatever its case. The root has no parent, so
-- a fixed uuid stands in for it.
CREATE UNIQUE INDEX uq_knowledge_folders_sibling_name ON knowledge_folders (
    org_id,
    coalesce(parent_id, '00000000-0000-0000-0000-000000000000'::uuid),
    lower(name)
);

ALTER TABLE knowledge_articles ADD COLUMN folder_id UUID NULL;
ALTER TABLE knowledge_articles ADD CONSTRAINT fk_knowledge_articles_folder
    FOREIGN KEY (folder_id, org_id) REFERENCES knowledge_folders (id, org_id);

CREATE INDEX idx_knowledge_articles_folder ON knowledge_articles (org_id, folder_id);
