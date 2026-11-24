-- DAF (demandes d'agrément de fourniture): the sheets of supplies submitted
-- to the project manager for approval. See docs/chantiers/daf.md.
--
-- Unlike the DOE, a DAF is a row with typed columns: the summary sorts by
-- number and filters by status, and the number must be unique per project.
-- Supplier and attachments are copies never queried from the inside, so
-- they stay JSONB.

CREATE TABLE daf_dossiers (
    project_id      UUID PRIMARY KEY,
    org_id          UUID NOT NULL,
    title           TEXT NOT NULL,
    issuer          TEXT NOT NULL,
    project_owner   TEXT NULL,
    project_manager TEXT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_daf_dossiers_project FOREIGN KEY (project_id, org_id)
        REFERENCES projects(id, org_id) ON DELETE CASCADE,
    CONSTRAINT chk_daf_dossiers_title_not_blank CHECK (btrim(title) <> ''),
    CONSTRAINT chk_daf_dossiers_issuer_not_blank CHECK (btrim(issuer) <> '')
);

CREATE INDEX idx_daf_dossiers_org ON daf_dossiers (org_id);

CREATE TABLE dafs (
    id                UUID PRIMARY KEY,
    org_id            UUID NOT NULL,
    project_id        UUID NOT NULL,
    number            INTEGER NOT NULL,
    designation       TEXT NOT NULL,
    price_ref         TEXT NULL,
    source_product_id UUID NULL,
    supplier          JSONB NULL,
    installer         TEXT NULL,
    specs             TEXT NULL,
    observations      TEXT NULL,
    photo_key         TEXT NULL,
    attachments       JSONB NOT NULL DEFAULT '[]',
    project_owner     TEXT NULL,
    project_manager   TEXT NULL,
    status            TEXT NOT NULL DEFAULT 'pending',
    sent_on           DATE NULL,
    issued_on         DATE NOT NULL,
    returned_on       DATE NULL,
    review_note       TEXT NULL,
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Composite FK: a DAF on another organization's project is unrepresentable.
    CONSTRAINT fk_dafs_project FOREIGN KEY (project_id, org_id)
        REFERENCES projects(id, org_id) ON DELETE CASCADE,
    CONSTRAINT uq_dafs_project_number UNIQUE (project_id, number),
    CONSTRAINT chk_dafs_number CHECK (number BETWEEN 1 AND 999),
    CONSTRAINT chk_dafs_designation_not_blank CHECK (btrim(designation) <> ''),
    CONSTRAINT chk_dafs_status
        CHECK (status IN ('pending', 'accepted', 'rejected_with_comments', 'rejected')),
    CONSTRAINT chk_dafs_attachments_array CHECK (jsonb_typeof(attachments) = 'array')
);

CREATE INDEX idx_dafs_org ON dafs (org_id);
