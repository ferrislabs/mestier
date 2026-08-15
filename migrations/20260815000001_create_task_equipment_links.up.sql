-- Links equipment (hourly-rated tooling/vehicles, already a standalone
-- referential) to tasks, the missing half of M6's profitability input:
-- Σ(temps × €/h matériel) alongside the employee cost already covered by
-- task_assignments. Mirrors task_label_links exactly.
--
-- Physical deletion, on purpose: a link carries no information worth
-- keeping once removed. `ON DELETE CASCADE` on both sides is what makes
-- "delete a task" and "delete an equipment record" both mean "remove the
-- link," for free.
CREATE TABLE task_equipment_links (
    id            UUID        PRIMARY KEY,
    task_id       UUID        NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    equipment_id  UUID        NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_task_equipment_links_task_id_equipment_id UNIQUE (task_id, equipment_id)
);

CREATE INDEX idx_task_equipment_links_task_id ON task_equipment_links(task_id);
CREATE INDEX idx_task_equipment_links_equipment_id ON task_equipment_links(equipment_id);
