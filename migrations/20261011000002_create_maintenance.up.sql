-- Maintenance contracts: a site entretenu for one season, its lines and the
-- interventions done on each line. See docs/chantiers/entretien.md.
--
-- The next due date is deliberately not stored (ADR 0006): it is computed
-- from the cadence, the season and the validated interventions.

CREATE TABLE maintenance_contracts (
    id                  UUID PRIMARY KEY,
    org_id              UUID NOT NULL REFERENCES organizations(id),
    customer_id         UUID NOT NULL REFERENCES customers(id),
    customer_context_id UUID NOT NULL REFERENCES customer_contexts(id),
    project_id          UUID NOT NULL,
    reference           TEXT NULL,
    season_year         SMALLINT NOT NULL,
    first_week          SMALLINT NOT NULL,
    last_week           SMALLINT NOT NULL,
    manager_member_id   UUID NULL,
    amount_cents        BIGINT NULL,
    quote_id            UUID NULL REFERENCES quotes(id),
    notes               TEXT NULL,
    status              TEXT NOT NULL DEFAULT 'ACTIVE',
    archived_at         TIMESTAMPTZ NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_maintenance_contracts_project
        FOREIGN KEY (project_id, org_id) REFERENCES projects(id, org_id),
    CONSTRAINT fk_maintenance_contracts_manager
        FOREIGN KEY (manager_member_id, org_id) REFERENCES organization_members(id, organization_id),
    -- A contract owns its project: the interventions' tasks live there, and
    -- the profitability report reads them from it (ADR 0002).
    CONSTRAINT uq_maintenance_contracts_project UNIQUE (project_id),
    CONSTRAINT uq_maintenance_contracts_id_org UNIQUE (id, org_id),
    CONSTRAINT chk_maintenance_contracts_reference_not_blank
        CHECK (reference IS NULL OR btrim(reference) <> ''),
    CONSTRAINT chk_maintenance_contracts_notes_not_blank
        CHECK (notes IS NULL OR btrim(notes) <> ''),
    CONSTRAINT chk_maintenance_contracts_season_year CHECK (season_year BETWEEN 2000 AND 2100),
    CONSTRAINT chk_maintenance_contracts_weeks
        CHECK (first_week BETWEEN 1 AND 53 AND last_week BETWEEN first_week AND 53),
    CONSTRAINT chk_maintenance_contracts_amount CHECK (amount_cents IS NULL OR amount_cents >= 0),
    CONSTRAINT chk_maintenance_contracts_status CHECK (status IN ('DRAFT', 'ACTIVE'))
);

CREATE INDEX idx_maintenance_contracts_org_season ON maintenance_contracts(org_id, season_year);

-- The cadence is JSONB: a value read and written whole with its line, never
-- queried from the inside.
CREATE TABLE maintenance_lines (
    id                       UUID PRIMARY KEY,
    org_id                   UUID NOT NULL REFERENCES organizations(id),
    contract_id              UUID NOT NULL,
    label                    TEXT NOT NULL,
    cadence                  JSONB NOT NULL,
    minutes_per_intervention INTEGER NULL,
    position                 INTEGER NOT NULL,
    created_at               TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at               TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_maintenance_lines_contract
        FOREIGN KEY (contract_id, org_id) REFERENCES maintenance_contracts(id, org_id) ON DELETE CASCADE,
    CONSTRAINT uq_maintenance_lines_id_org UNIQUE (id, org_id),
    CONSTRAINT chk_maintenance_lines_label_not_blank CHECK (btrim(label) <> ''),
    CONSTRAINT chk_maintenance_lines_cadence_object CHECK (jsonb_typeof(cadence) = 'object'),
    CONSTRAINT chk_maintenance_lines_minutes
        CHECK (minutes_per_intervention IS NULL OR minutes_per_intervention > 0)
);

CREATE INDEX idx_maintenance_lines_contract ON maintenance_lines(contract_id, position);

CREATE TABLE maintenance_interventions (
    id          UUID PRIMARY KEY,
    org_id      UUID NOT NULL REFERENCES organizations(id),
    line_id     UUID NOT NULL,
    done_on     DATE NOT NULL,
    status      TEXT NOT NULL,
    task_id     UUID NULL REFERENCES tasks(id) ON DELETE SET NULL,
    proposed_by UUID NOT NULL,
    resolved_by UUID NULL,
    resolved_at TIMESTAMPTZ NULL,
    note        TEXT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_maintenance_interventions_line
        FOREIGN KEY (line_id, org_id) REFERENCES maintenance_lines(id, org_id) ON DELETE CASCADE,
    CONSTRAINT fk_maintenance_interventions_proposed_by
        FOREIGN KEY (proposed_by, org_id) REFERENCES organization_members(id, organization_id),
    CONSTRAINT fk_maintenance_interventions_resolved_by
        FOREIGN KEY (resolved_by, org_id) REFERENCES organization_members(id, organization_id),
    CONSTRAINT chk_maintenance_interventions_status
        CHECK (status IN ('PROPOSED', 'VALIDATED', 'REJECTED')),
    -- A manager's direct entry is validated at once, so it carries a
    -- resolver too: `resolved_*` are null exactly while proposed.
    CONSTRAINT chk_maintenance_interventions_resolution CHECK (
        (status = 'PROPOSED' AND resolved_by IS NULL AND resolved_at IS NULL)
        OR
        (status <> 'PROPOSED' AND resolved_by IS NOT NULL AND resolved_at IS NOT NULL)
    ),
    CONSTRAINT chk_maintenance_interventions_note_not_blank
        CHECK (note IS NULL OR btrim(note) <> '')
);

CREATE INDEX idx_maintenance_interventions_line ON maintenance_interventions(line_id, done_on);
CREATE INDEX idx_maintenance_interventions_pending
    ON maintenance_interventions(org_id) WHERE status = 'PROPOSED';

ALTER TABLE tasks ADD COLUMN maintenance_line_id UUID NULL;

-- SET NULL on the line column only: nulling `org_id` with it would orphan
-- the task.
ALTER TABLE tasks
    ADD CONSTRAINT fk_tasks_maintenance_line
        FOREIGN KEY (maintenance_line_id, org_id) REFERENCES maintenance_lines(id, org_id)
        ON DELETE SET NULL (maintenance_line_id);

CREATE INDEX idx_tasks_maintenance_line ON tasks(maintenance_line_id)
    WHERE maintenance_line_id IS NOT NULL;
