-- Material assigned to work orders ("chantiers").
--
-- Physical deletion, on purpose: like `assignments`, a work-order/equipment
-- link carries no information worth keeping once removed (see ADR 0001).

CREATE TABLE work_order_equipment (
    id            UUID        PRIMARY KEY,
    org_id        UUID        NOT NULL REFERENCES organizations(id),
    work_order_id UUID        NOT NULL REFERENCES work_orders(id) ON DELETE CASCADE,
    equipment_id  UUID        NOT NULL REFERENCES equipment(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_work_order_equipment_work_order_equipment
        UNIQUE (work_order_id, equipment_id)
);

CREATE INDEX idx_work_order_equipment_org_id ON work_order_equipment(org_id);
CREATE INDEX idx_work_order_equipment_work_order_id ON work_order_equipment(work_order_id);
CREATE INDEX idx_work_order_equipment_equipment_id ON work_order_equipment(equipment_id);
