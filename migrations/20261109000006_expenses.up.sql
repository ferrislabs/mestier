CREATE TABLE expense_categories (
    id         UUID        PRIMARY KEY,
    org_id     UUID        NOT NULL REFERENCES organizations(id),
    name       TEXT        NOT NULL,
    hue        INTEGER     NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_expense_categories_name_not_blank CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_expense_categories_hue_range CHECK (hue >= 0 AND hue <= 359),
    CONSTRAINT uq_expense_categories_id_org UNIQUE (id, org_id)
);

CREATE UNIQUE INDEX uq_expense_categories_org_name ON expense_categories(org_id, lower(name));

CREATE TABLE expenses (
    id                    UUID        PRIMARY KEY,
    org_id                UUID        NOT NULL REFERENCES organizations(id),
    date                  DATE        NOT NULL,
    label                 TEXT        NOT NULL,
    supplier_id           UUID        NULL,
    category_id           UUID        NOT NULL,
    reference             TEXT        NULL,
    amount_ht_cents       INTEGER     NOT NULL,
    vat_cents             INTEGER     NOT NULL DEFAULT 0,
    due_date              DATE        NULL,
    status                TEXT        NOT NULL DEFAULT 'TO_PAY',
    paid_on               DATE        NULL,
    method                TEXT        NULL,
    advanced_by_member_id UUID        NULL,
    reimbursed_on         DATE        NULL,
    customer_id           UUID        NULL,
    proof_file_key        TEXT        NULL,
    note                  TEXT        NULL,
    created_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at            TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_expenses_label_not_blank CHECK (length(btrim(label)) > 0),
    CONSTRAINT chk_expenses_amount_ht_positive CHECK (amount_ht_cents > 0),
    CONSTRAINT chk_expenses_vat_non_negative CHECK (vat_cents >= 0),
    CONSTRAINT chk_expenses_due_not_before_date CHECK (due_date IS NULL OR due_date >= date),
    CONSTRAINT chk_expenses_status_known CHECK (status IN ('TO_PAY', 'PAID')),
    CONSTRAINT chk_expenses_method_known
        CHECK (method IS NULL OR method IN ('CARD', 'TRANSFER', 'CHECK', 'CASH', 'DIRECT_DEBIT')),
    CONSTRAINT chk_expenses_paid_state
        CHECK ((status = 'PAID') = (paid_on IS NOT NULL AND method IS NOT NULL)
            AND (status = 'PAID' OR (paid_on IS NULL AND method IS NULL))),
    CONSTRAINT chk_expenses_advanced_is_paid
        CHECK (advanced_by_member_id IS NULL OR (status = 'PAID' AND due_date IS NULL)),
    CONSTRAINT chk_expenses_reimbursed_needs_advance
        CHECK (reimbursed_on IS NULL OR advanced_by_member_id IS NOT NULL),
    CONSTRAINT chk_expenses_text_not_blank
        CHECK ((reference IS NULL OR length(btrim(reference)) > 0)
            AND (note IS NULL OR length(btrim(note)) > 0)
            AND (proof_file_key IS NULL OR length(btrim(proof_file_key)) > 0)),
    CONSTRAINT fk_expenses_supplier
        FOREIGN KEY (supplier_id, org_id) REFERENCES suppliers(id, org_id),
    CONSTRAINT fk_expenses_category
        FOREIGN KEY (category_id, org_id) REFERENCES expense_categories(id, org_id),
    CONSTRAINT fk_expenses_customer
        FOREIGN KEY (customer_id, org_id) REFERENCES customers(id, org_id),
    CONSTRAINT fk_expenses_advanced_by
        FOREIGN KEY (advanced_by_member_id, org_id) REFERENCES organization_members(id, organization_id)
);

CREATE INDEX idx_expenses_org_date ON expenses(org_id, date DESC);
CREATE INDEX idx_expenses_category_id ON expenses(category_id);
CREATE INDEX idx_expenses_supplier_id ON expenses(supplier_id) WHERE supplier_id IS NOT NULL;

CREATE TABLE unallocated_payments (
    id           UUID        PRIMARY KEY,
    org_id       UUID        NOT NULL REFERENCES organizations(id),
    customer_id  UUID        NULL,
    amount_cents INTEGER     NOT NULL,
    paid_on      DATE        NOT NULL,
    method       TEXT        NOT NULL,
    reference    TEXT        NULL,
    note         TEXT        NULL,
    recorded_by  UUID        NOT NULL REFERENCES users(id),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_unallocated_payments_amount_positive CHECK (amount_cents > 0),
    CONSTRAINT chk_unallocated_payments_method_known
        CHECK (method IN ('CARD', 'TRANSFER', 'CHECK', 'CASH', 'DIRECT_DEBIT')),
    CONSTRAINT chk_unallocated_payments_text_not_blank
        CHECK ((reference IS NULL OR length(btrim(reference)) > 0)
            AND (note IS NULL OR length(btrim(note)) > 0)),
    CONSTRAINT fk_unallocated_payments_customer
        FOREIGN KEY (customer_id, org_id) REFERENCES customers(id, org_id)
);

CREATE INDEX idx_unallocated_payments_org_paid_on ON unallocated_payments(org_id, paid_on DESC);
