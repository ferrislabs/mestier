-- Work orders ("chantiers") and their assignments to employees.
--
-- An all-day work order is stored as the [00:00, 24:00) window of the
-- organization's timezone; `all_day` tells the renderer to draw it as a
-- banner instead of a timed segment (see ADR 0001 and the planning module
-- design doc).

CREATE TYPE work_order_status AS ENUM (
    'PLANNED',
    'IN_PROGRESS',
    'DONE',
    'CANCELLED'
);

CREATE TABLE work_orders (
    id                   UUID               PRIMARY KEY,
    org_id               UUID               NOT NULL REFERENCES organizations(id),
    customer_id          UUID               NOT NULL REFERENCES customers(id),
    customer_context_id  UUID               NOT NULL REFERENCES customer_contexts(id),
    quote_id             UUID               NULL REFERENCES quotes(id),
    starts_at            TIMESTAMPTZ        NOT NULL,
    ends_at              TIMESTAMPTZ        NOT NULL,
    all_day              BOOLEAN            NOT NULL DEFAULT false,
    status               work_order_status  NOT NULL,
    title                TEXT               NULL,
    note                 TEXT               NULL,
    created_at           TIMESTAMPTZ        NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ        NOT NULL DEFAULT now(),
    deleted_at           TIMESTAMPTZ        NULL,

    CONSTRAINT chk_work_orders_ends_at_after_starts_at
        CHECK (ends_at > starts_at),
    CONSTRAINT chk_work_orders_title_not_blank_when_present
        CHECK (title IS NULL OR length(btrim(title)) > 0),
    CONSTRAINT chk_work_orders_note_not_blank_when_present
        CHECK (note IS NULL OR length(btrim(note)) > 0)
);

CREATE INDEX idx_work_orders_org_id_starts_at ON work_orders(org_id, starts_at);
CREATE INDEX idx_work_orders_customer_id ON work_orders(customer_id);
CREATE INDEX idx_work_orders_customer_context_id ON work_orders(customer_context_id);
CREATE INDEX idx_work_orders_quote_id ON work_orders(quote_id);
CREATE INDEX idx_work_orders_deleted_at ON work_orders(deleted_at);

-- Physical deletion, on purpose: an assignment carries no information worth
-- keeping once removed, unlike work orders and absences which are
-- soft-deleted for audit and history.
CREATE TABLE assignments (
    id            UUID        PRIMARY KEY,
    org_id        UUID        NOT NULL REFERENCES organizations(id),
    work_order_id UUID        NOT NULL REFERENCES work_orders(id) ON DELETE CASCADE,
    employee_id   UUID        NOT NULL REFERENCES employees(id) ON DELETE CASCADE,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT uq_assignments_work_order_employee UNIQUE (work_order_id, employee_id)
);

CREATE INDEX idx_assignments_org_id ON assignments(org_id);
CREATE INDEX idx_assignments_work_order_id ON assignments(work_order_id);
CREATE INDEX idx_assignments_employee_id ON assignments(employee_id);
