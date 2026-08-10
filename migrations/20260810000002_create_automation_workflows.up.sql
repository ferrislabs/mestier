CREATE TABLE automation.workflow (
    id                  UUID        PRIMARY KEY,
    org_id              UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name                TEXT        NOT NULL,
    description         TEXT        NULL,
    enabled             BOOLEAN     NOT NULL DEFAULT true,
    -- NULL until the first version is saved; moved forward only by
    -- automation::workflow saving a new version (see the FK added below).
    current_version_id  UUID        NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_automation_workflow_name CHECK (length(trim(name)) > 0)
);

CREATE INDEX idx_automation_workflow_org
    ON automation.workflow (org_id);

CREATE TABLE automation.workflow_version (
    id          UUID        PRIMARY KEY,
    workflow_id UUID        NOT NULL REFERENCES automation.workflow(id) ON DELETE CASCADE,
    -- 1, 2, 3, ... within one workflow. Assigned by the repository under an
    -- advisory lock keyed by workflow_id, the same technique
    -- PgQuoteRepository::next_reference uses for its own per-organization
    -- counter, so two concurrent saves can never collide on this UNIQUE.
    version     INTEGER     NOT NULL,
    -- The whole graph, always read and written whole by the editor. Not
    -- normalized into connector/edge tables: no query ever asks "every
    -- Odoo connector in the system", and normalizing would turn one save
    -- into a multi-table reconciliation for no query anyone runs.
    graph       JSONB       NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    -- Whoever saved it, when a human did. NULL for a system-authored save
    -- (an import, say). SET NULL rather than CASCADE/RESTRICT: a deleted
    -- user must not retroactively invalidate a frozen, possibly still
    -- running (#200), version.
    created_by  UUID        NULL REFERENCES users(id) ON DELETE SET NULL,

    CONSTRAINT uq_automation_workflow_version UNIQUE (workflow_id, version),
    CONSTRAINT chk_automation_workflow_version CHECK (version > 0)
);

-- Makes "which workflows reference this credential_id" (the
-- delete_credential guard, #199) a practical query: a plain
-- `graph @> jsonb_build_object('connectors', jsonb_build_array(...))`
-- containment check, GIN-indexed, rather than a full scan decoding every
-- graph. The credential reference lives in JSONB with no FK to protect it
-- (a connector references a credential by UUID, inside `connectors[].
-- credential_id`), so this index is what makes the domain-level guard
-- affordable instead of merely correct.
CREATE INDEX idx_automation_workflow_version_graph
    ON automation.workflow_version USING GIN (graph);

CREATE INDEX idx_automation_workflow_version_workflow
    ON automation.workflow_version (workflow_id);

-- `workflow.current_version_id` and `workflow_version.workflow_id` point at
-- each other, so this FK cannot be declared inline on either CREATE TABLE
-- without the other table already existing. Added here, after both, as a
-- plain (non-deferred) constraint rather than DEFERRABLE INITIALLY
-- DEFERRED: `current_version_id` is NULL until a version already exists
-- (WorkflowRepository::insert_version always inserts the version row before
-- moving the pointer to it), so the constraint is never transiently
-- violated within a transaction — there is nothing here for deferring to
-- buy.
ALTER TABLE automation.workflow
    ADD CONSTRAINT fk_automation_workflow_current_version
        FOREIGN KEY (current_version_id) REFERENCES automation.workflow_version(id);
