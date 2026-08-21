-- Projects: the thing a cost is reported against.
--
-- A "chantier" was never an entity. It was a root `tasks` row that happened to
-- carry `customer_id`, and profitability grouped on
-- `COALESCE(parent_task_id, id)`, which only works because the domain caps the
-- hierarchy at two levels (`validate_parent_depth`). Work with no customer had
-- nowhere to go at all: a recurring internal meeting or an admin task could
-- only join a subject by becoming somebody's subtask, and never two subjects
-- at once.
--
-- A project exists with or without a customer. That is the whole point, and it
-- is what makes an internal cost centre expressible. See
-- docs/adr/0002-planned-cost-model.md.
--
-- One state column, not two. `archived_at` carries the entire lifecycle:
-- archiving hides a project from pickers and lists, and never hides its cost
-- history, because past cost stays true whether or not the work continues. A
-- separate `status` enum alongside it would let the two disagree.
--
-- `tasks.customer_id` and `tasks.quote_id` deliberately stay where they are.
-- Nothing reads the project's copy yet, and one migration should not have to be
-- right about the new shape and the removal of the old one at the same time.
-- Dropping them is a follow-up, once every reader has moved.

CREATE TABLE projects (
    id                  UUID PRIMARY KEY,
    org_id              UUID NOT NULL REFERENCES organizations(id),
    name                TEXT NOT NULL,
    customer_id         UUID NULL REFERENCES customers(id),
    customer_context_id UUID NULL REFERENCES customer_contexts(id),
    quote_id            UUID NULL REFERENCES quotes(id),
    archived_at         TIMESTAMPTZ NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_projects_name_not_blank
        CHECK (btrim(name) <> ''),
    -- Same relaxed pairing as `tasks` since 20260815000002: a context needs a
    -- customer, a customer does not need a context.
    CONSTRAINT chk_projects_context_requires_customer
        CHECK (customer_context_id IS NULL OR customer_id IS NOT NULL),
    -- The target of the composite foreign key below. Same device as
    -- `organization_members(id, organization_id)`: it makes a task attached to
    -- another organization's project unrepresentable rather than merely
    -- forbidden by application code.
    CONSTRAINT uq_projects_id_org UNIQUE (id, org_id)
);

CREATE INDEX idx_projects_org_id ON projects(org_id);
CREATE INDEX idx_projects_customer_id ON projects(customer_id);
CREATE INDEX idx_projects_archived_at ON projects(archived_at);

-- No depth constraint: a subtask attaches straight to a project without going
-- through its parent, which is exactly what the two-level cap used to forbid.
ALTER TABLE tasks ADD COLUMN project_id UUID NULL;

ALTER TABLE tasks
    ADD CONSTRAINT fk_tasks_project
        FOREIGN KEY (project_id, org_id) REFERENCES projects(id, org_id);

CREATE INDEX idx_tasks_project_id ON tasks(project_id);

-- Backfill: every root task carrying a customer becomes a project, and its
-- subtasks join it.
--
-- The project reuses the root task's own UUID. That is deliberate and specific
-- to this backfill: it makes the mapping from task to project deterministic
-- without a temporary column, keeps the v7 ordering the application generates
-- elsewhere, and lets the revert be a plain drop. Projects created after this
-- migration get their own fresh id like any other row.
INSERT INTO projects (id, org_id, name, customer_id, customer_context_id, quote_id, created_at, updated_at)
SELECT t.id, t.org_id, t.title, t.customer_id, t.customer_context_id, t.quote_id, t.created_at, t.updated_at
FROM tasks t
WHERE t.parent_task_id IS NULL
  AND t.customer_id IS NOT NULL
  AND t.deleted_at IS NULL;

UPDATE tasks t
SET project_id = COALESCE(t.parent_task_id, t.id)
WHERE EXISTS (
    SELECT 1 FROM projects p WHERE p.id = COALESCE(t.parent_task_id, t.id)
);
