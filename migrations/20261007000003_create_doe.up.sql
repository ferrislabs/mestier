-- DOE (dossier des ouvrages exécutés): the file handed to the customer at
-- reception, listing what was installed. See docs/chantiers/doe.md.
--
-- Parties and sections are JSONB on purpose. A DOE is a document copied from
-- the catalogue at one point in time, never queried from the inside, and
-- the editor saves it whole. Five normalized tables would buy joins nobody
-- runs.

CREATE TABLE doe_models (
    id          UUID PRIMARY KEY,
    org_id      UUID NOT NULL REFERENCES organizations(id),
    name        TEXT NOT NULL,
    description TEXT NULL,
    sections    JSONB NOT NULL DEFAULT '[]',
    archived_at TIMESTAMPTZ NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_doe_models_name_not_blank CHECK (btrim(name) <> ''),
    CONSTRAINT chk_doe_models_description_not_blank
        CHECK (description IS NULL OR btrim(description) <> ''),
    CONSTRAINT chk_doe_models_sections_array CHECK (jsonb_typeof(sections) = 'array')
);

CREATE INDEX idx_doe_models_org_id ON doe_models(org_id);

CREATE TABLE doe_settings (
    org_id     UUID PRIMARY KEY REFERENCES organizations(id),
    accent     TEXT NOT NULL,
    logo_key   TEXT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_doe_settings_accent_hex CHECK (accent ~ '^#[0-9a-f]{6}$'),
    CONSTRAINT chk_doe_settings_logo_key_not_blank
        CHECK (logo_key IS NULL OR btrim(logo_key) <> '')
);

CREATE TABLE does (
    id         UUID PRIMARY KEY,
    org_id     UUID NOT NULL REFERENCES organizations(id),
    project_id UUID NOT NULL,
    title      TEXT NOT NULL,
    place      TEXT NULL,
    lot        TEXT NULL,
    issued_on  DATE NOT NULL,
    author     TEXT NULL,
    revision   TEXT NOT NULL,
    parties    JSONB NOT NULL DEFAULT '[]',
    sections   JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Composite FK: a DOE on another organization's project is unrepresentable.
    CONSTRAINT fk_does_project FOREIGN KEY (project_id, org_id) REFERENCES projects(id, org_id),
    -- One DOE per project. Revisions are a field, not rows.
    CONSTRAINT uq_does_project UNIQUE (project_id),
    CONSTRAINT chk_does_title_not_blank CHECK (btrim(title) <> ''),
    CONSTRAINT chk_does_revision_length CHECK (char_length(btrim(revision)) BETWEEN 1 AND 8),
    CONSTRAINT chk_does_parties_array CHECK (jsonb_typeof(parties) = 'array'),
    CONSTRAINT chk_does_sections_array CHECK (jsonb_typeof(sections) = 'array')
);

CREATE INDEX idx_does_org_id ON does(org_id);
