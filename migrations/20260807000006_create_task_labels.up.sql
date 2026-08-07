-- Task labels: the tags that absorb what used to be a task's "nature"
-- (meeting, trip, training...). Not a column, not an enum — see the
-- planning module design doc. "Réunion", "Déplacement" and "Formation" are
-- seeded transactionally by `OrganizationService::create_organization`,
-- alongside the default roles; nothing here distinguishes a preset from a
-- label created by hand, and the user may rename or delete either.

CREATE TABLE task_labels (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id),
    name        TEXT        NOT NULL,
    color       TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_task_labels_org_id_name UNIQUE (org_id, name),
    CONSTRAINT chk_task_labels_name_not_blank
        CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_task_labels_color_format
        CHECK (color ~ '^#[0-9A-Fa-f]{6}$')
);

CREATE INDEX idx_task_labels_org_id ON task_labels(org_id);

-- Physical deletion, on purpose: a link carries no information worth
-- keeping once removed — mirrors `task_assignments`. `ON DELETE CASCADE` on
-- `label_id` is exactly what makes "delete a label" mean "remove it from
-- every task that carried it," for free.
CREATE TABLE task_label_links (
    id          UUID        PRIMARY KEY,
    task_id     UUID        NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    label_id    UUID        NOT NULL REFERENCES task_labels(id) ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_task_label_links_task_id_label_id UNIQUE (task_id, label_id)
);

CREATE INDEX idx_task_label_links_task_id ON task_label_links(task_id);
CREATE INDEX idx_task_label_links_label_id ON task_label_links(label_id);
