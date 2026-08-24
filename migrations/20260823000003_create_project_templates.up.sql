-- Project templates: an ordered set of task shapes, not a project to be
-- copied.
--
-- An organization that does the same kind of job twenty times a year
-- rebuilds the same task list twenty times, and the twentieth is where a
-- forgotten task becomes an understated project. A template names the shape
-- of the work — titles, relative offsets, expenses, a hierarchy — and
-- instantiation turns that shape into real tasks under a real project, on a
-- date the caller picks.
--
-- No assignees here, on purpose: who does the job changes every time, and a
-- template that guesses is a template people fight.

CREATE TABLE project_templates (
    id          UUID PRIMARY KEY,
    org_id      UUID NOT NULL REFERENCES organizations(id),
    name        TEXT NOT NULL,
    description TEXT NULL,
    archived_at TIMESTAMPTZ NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_project_templates_name_not_blank
        CHECK (btrim(name) <> ''),
    CONSTRAINT chk_project_templates_description_not_blank
        CHECK (description IS NULL OR btrim(description) <> ''),
    -- Same device as `uq_projects_id_org`: the target of the composite FK
    -- below, so a task shape attached to another organization's template is
    -- unrepresentable rather than merely forbidden by application code.
    CONSTRAINT uq_project_templates_id_org UNIQUE (id, org_id)
);

CREATE INDEX idx_project_templates_org_id ON project_templates(org_id);
CREATE INDEX idx_project_templates_archived_at ON project_templates(archived_at);

-- One row per task shape. `day_offset` is relative to whatever date
-- instantiation is given — never an absolute date, so instantiating asks for
-- exactly one thing. `starts_minute`/`ends_minute` are minutes since local
-- midnight, mirroring `work_slots`, and are both absent on an all-day shape:
-- an all-day task stays all-day at instantiation, so `expand_work_slots`
-- costs it from the assignee's slots rather than a guessed amplitude.
--
-- `parent_index` gives a two-level hierarchy, the same cap the `tasks`
-- domain enforces on `parent_task_id` (`validate_parent_depth`) — but a
-- template task has no id yet to reference, so this points at the
-- `position` of another row of the same template instead. Referential
-- integrity (the index names a real, root-level row) is a domain
-- invariant — `ProjectTemplateService` always replaces every task of a
-- template together in one transaction, so it is never at risk of drifting
-- the way a standalone foreign key would need runtime data to check anyway.
--
-- Physically replaced wholesale on every edit (delete then insert), like
-- `task_assignments` — a task shape carries no history worth keeping past
-- that replacement, so there is no `updated_at` to maintain.
CREATE TABLE project_template_tasks (
    id                  UUID PRIMARY KEY,
    org_id              UUID NOT NULL REFERENCES organizations(id),
    template_id         UUID NOT NULL,
    title               TEXT NOT NULL,
    description         TEXT NULL,
    day_offset          INTEGER NOT NULL,
    starts_minute       SMALLINT NULL,
    ends_minute         SMALLINT NULL,
    all_day             BOOLEAN NOT NULL DEFAULT false,
    blocks_availability BOOLEAN NOT NULL DEFAULT true,
    expenses_cents      INTEGER NOT NULL DEFAULT 0,
    expenses_label      TEXT NULL,
    parent_index        INTEGER NULL,
    position            INTEGER NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_project_template_tasks_template
        FOREIGN KEY (template_id, org_id) REFERENCES project_templates(id, org_id) ON DELETE CASCADE,
    CONSTRAINT chk_project_template_tasks_title_not_blank
        CHECK (btrim(title) <> ''),
    -- Same equivalence `tasks` already enforces: an amount with no reason
    -- cannot be audited three months later.
    CONSTRAINT chk_project_template_tasks_expenses_not_negative
        CHECK (expenses_cents >= 0),
    CONSTRAINT chk_project_template_tasks_expenses_label_required
        CHECK ((expenses_cents = 0) = (expenses_label IS NULL)),
    CONSTRAINT chk_project_template_tasks_expenses_label_not_blank
        CHECK (expenses_label IS NULL OR btrim(expenses_label) <> ''),
    CONSTRAINT chk_project_template_tasks_minutes_both_or_neither
        CHECK ((starts_minute IS NULL) = (ends_minute IS NULL)),
    CONSTRAINT chk_project_template_tasks_minutes_range
        CHECK (
            (starts_minute IS NULL OR (starts_minute >= 0 AND starts_minute < 1440))
            AND (ends_minute IS NULL OR (ends_minute > 0 AND ends_minute <= 1440))
        ),
    CONSTRAINT chk_project_template_tasks_minutes_order
        CHECK (starts_minute IS NULL OR ends_minute > starts_minute),
    CONSTRAINT chk_project_template_tasks_parent_not_self
        CHECK (parent_index IS NULL OR parent_index <> position),
    CONSTRAINT uq_project_template_tasks_template_position UNIQUE (template_id, position)
);

CREATE INDEX idx_project_template_tasks_template_id ON project_template_tasks(template_id);
CREATE INDEX idx_project_template_tasks_org_id ON project_template_tasks(org_id);
