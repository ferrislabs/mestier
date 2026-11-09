CREATE TABLE library_photos (
    id            UUID        PRIMARY KEY,
    org_id        UUID        NOT NULL REFERENCES organizations(id),
    file_key      TEXT        NOT NULL,
    thumbnail_key TEXT        NOT NULL,
    mime_type     TEXT        NOT NULL,
    width         INTEGER     NOT NULL,
    height        INTEGER     NOT NULL,
    size_bytes    INTEGER     NOT NULL,
    content_hash  TEXT        NOT NULL,
    caption       TEXT        NULL,
    tags          TEXT[]      NOT NULL DEFAULT '{}',
    project_id    UUID        NULL,
    taken_at      TIMESTAMPTZ NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_library_photos_mime_known CHECK (mime_type IN ('image/jpeg', 'image/png', 'image/webp')),
    CONSTRAINT chk_library_photos_dimensions_positive CHECK (width > 0 AND height > 0),
    CONSTRAINT chk_library_photos_size_positive CHECK (size_bytes > 0),
    CONSTRAINT chk_library_photos_caption_not_blank CHECK (caption IS NULL OR length(btrim(caption)) > 0),
    CONSTRAINT chk_library_photos_tags_max CHECK (cardinality(tags) <= 20),
    CONSTRAINT uq_library_photos_id_org UNIQUE (id, org_id),
    CONSTRAINT fk_library_photos_project FOREIGN KEY (project_id, org_id) REFERENCES projects (id, org_id)
);

CREATE UNIQUE INDEX uq_library_photos_org_hash ON library_photos (org_id, content_hash);
CREATE INDEX idx_library_photos_org_created ON library_photos (org_id, created_at DESC, id);
CREATE INDEX idx_library_photos_org_project ON library_photos (org_id, project_id) WHERE project_id IS NOT NULL;
CREATE INDEX idx_library_photos_tags ON library_photos USING GIN (tags);
