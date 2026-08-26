-- A received supplier invoice: somebody else's document, not a mirror of
-- the ones this product issues. It is immutable because somebody else
-- issued it, it carries their identity rather than ours, and its lines are
-- theirs — sharing `invoices` would put a direction-and-status rule on
-- every query, exactly the kind of rule that gets honoured in one code
-- path. See #334, #336.
--
-- `status`: a parsed (or manually entered) document is a proposal until a
-- human confirms it. Nothing here is a cost until somebody said so — #338
-- is what actually allocates a confirmed invoice's cost to a project.
CREATE TYPE supplier_invoice_status AS ENUM (
    'RECEIVED',
    'CONFIRMED',
    'REJECTED'
);

-- An organization buys from the same merchants repeatedly, so recognising
-- one earns its own table rather than denormalised text on every invoice.
-- Deliberately thin: this issue only needs the row to exist for
-- `supplier_invoices.supplier_id` to point at, nothing here manages
-- suppliers yet.
CREATE TABLE suppliers (
    id                   UUID        PRIMARY KEY,
    org_id               UUID        NOT NULL REFERENCES organizations(id),
    name                 TEXT        NOT NULL,
    registration_number  TEXT        NULL,
    vat_number           TEXT        NULL,
    deleted_at           TIMESTAMPTZ NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_suppliers_name_not_blank
        CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_suppliers_registration_number_not_blank
        CHECK (registration_number IS NULL OR length(btrim(registration_number)) > 0),
    CONSTRAINT chk_suppliers_vat_number_not_blank
        CHECK (vat_number IS NULL OR length(btrim(vat_number)) > 0),
    -- Target of `supplier_invoices`' composite FK below — same device as
    -- `invoices.uq_invoices_id_org`: a supplier attached to another
    -- organization's invoice becomes unrepresentable, not merely forbidden.
    CONSTRAINT uq_suppliers_id_org UNIQUE (id, org_id)
);

CREATE INDEX idx_suppliers_org_id ON suppliers(org_id);
CREATE INDEX idx_suppliers_deleted_at ON suppliers(deleted_at);

CREATE TABLE supplier_invoices (
    id                            UUID                    PRIMARY KEY,
    org_id                        UUID                    NOT NULL REFERENCES organizations(id),
    -- Nullable on purpose: an invoice must be storable before its supplier
    -- is recognised, or the first import of an unknown merchant fails. The
    -- identity fields below are what the document itself says, independent
    -- of whether it is ever linked to a row here.
    supplier_id                   UUID                    NULL,
    supplier_name                 TEXT                    NOT NULL,
    supplier_registration_number  TEXT                    NULL,
    supplier_vat_number           TEXT                    NULL,
    -- As issued by them, never allocated by us — contrast `invoices.number`.
    number                        TEXT                    NOT NULL,
    issued_on                     DATE                    NOT NULL,
    due_on                        DATE                    NULL,
    received_at                   TIMESTAMPTZ             NOT NULL DEFAULT now(),
    -- How this row came to exist. TEXT + CHECK, not a native enum: unlike
    -- `status` this is expected to grow (a PDP transport is a real future
    -- value) and a CHECK is one migration to widen, a native enum two.
    source                        TEXT                    NOT NULL,
    status                        supplier_invoice_status NOT NULL DEFAULT 'RECEIVED',
    currency                      TEXT                    NOT NULL DEFAULT 'EUR',
    -- Our metadata about the document, not part of it — what a
    -- `SupplierInvoiceReview` (libs/core/src/domain/supplier_invoice/mod.rs)
    -- is allowed to touch, alongside `status`.
    notes                         TEXT                    NULL,
    net_cents                     INTEGER                 NOT NULL DEFAULT 0,
    vat_breakdown                 JSONB                   NOT NULL DEFAULT '[]',
    gross_cents                   INTEGER                 NOT NULL DEFAULT 0,
    deleted_at                    TIMESTAMPTZ             NULL,
    created_at                    TIMESTAMPTZ             NOT NULL DEFAULT now(),
    updated_at                    TIMESTAMPTZ             NOT NULL DEFAULT now(),

    CONSTRAINT chk_supplier_invoices_supplier_name_not_blank
        CHECK (length(btrim(supplier_name)) > 0),
    CONSTRAINT chk_supplier_invoices_supplier_registration_number_not_blank
        CHECK (supplier_registration_number IS NULL OR length(btrim(supplier_registration_number)) > 0),
    CONSTRAINT chk_supplier_invoices_supplier_vat_number_not_blank
        CHECK (supplier_vat_number IS NULL OR length(btrim(supplier_vat_number)) > 0),
    CONSTRAINT chk_supplier_invoices_number_not_blank
        CHECK (length(btrim(number)) > 0),
    CONSTRAINT chk_supplier_invoices_source_not_blank
        CHECK (length(btrim(source)) > 0),
    CONSTRAINT chk_supplier_invoices_currency_iso
        CHECK (currency ~ '^[A-Z]{3}$'),
    CONSTRAINT chk_supplier_invoices_notes_not_blank_when_present
        CHECK (notes IS NULL OR length(btrim(notes)) > 0),
    CONSTRAINT chk_supplier_invoices_net_cents_non_negative
        CHECK (net_cents >= 0),
    CONSTRAINT chk_supplier_invoices_gross_cents_non_negative
        CHECK (gross_cents >= 0),
    -- Target of `supplier_invoice_lines`' composite FK below — same device
    -- as `invoices.uq_invoices_id_org`.
    CONSTRAINT uq_supplier_invoices_id_org UNIQUE (id, org_id),
    CONSTRAINT fk_supplier_invoices_supplier
        FOREIGN KEY (supplier_id, org_id) REFERENCES suppliers(id, org_id)
);

CREATE INDEX idx_supplier_invoices_org_id ON supplier_invoices(org_id);
CREATE INDEX idx_supplier_invoices_supplier_id ON supplier_invoices(supplier_id);
CREATE INDEX idx_supplier_invoices_status ON supplier_invoices(status);
CREATE INDEX idx_supplier_invoices_received_at ON supplier_invoices(received_at);
CREATE INDEX idx_supplier_invoices_deleted_at ON supplier_invoices(deleted_at);

CREATE TABLE supplier_invoice_lines (
    id                     UUID        PRIMARY KEY,
    org_id                 UUID        NOT NULL,
    supplier_invoice_id    UUID        NOT NULL,
    -- Copied text, never a foreign key to anything of ours: this is their
    -- line, not a projection of one of our catalogue items.
    label                  TEXT        NOT NULL,
    quantity               NUMERIC     NOT NULL,
    -- Free text as printed (`m³`, `kg`, `u`, ...) — we do not control their
    -- unit vocabulary the way `ServiceRateUnit` controls ours.
    unit                   TEXT        NULL,
    unit_price_cents       INTEGER     NOT NULL,
    -- Stored exactly as printed, not derived from quantity * unit_price:
    -- their own rounding is part of the document, and the three numbers
    -- disagreeing by a cent is normal, not a bug to silently correct.
    line_total_cents       INTEGER     NOT NULL,
    vat_rate_basis_points  INTEGER     NULL,
    position               INTEGER     NOT NULL,
    deleted_at             TIMESTAMPTZ NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_supplier_invoice_lines_label_not_blank
        CHECK (length(btrim(label)) > 0),
    -- Not `> 0`: a credit/rebate line from the supplier is a real,
    -- legitimate line on a received document, not a state we get to refuse
    -- the way `invoice_lines` refuses one on a document we author
    -- ourselves. Only an exactly-zero quantity is meaningless.
    CONSTRAINT chk_supplier_invoice_lines_quantity_not_zero
        CHECK (quantity != 0),
    CONSTRAINT chk_supplier_invoice_lines_unit_not_blank
        CHECK (unit IS NULL OR length(btrim(unit)) > 0),
    CONSTRAINT chk_supplier_invoice_lines_vat_rate_basis_points_range
        CHECK (vat_rate_basis_points IS NULL OR vat_rate_basis_points BETWEEN 0 AND 10000),
    CONSTRAINT chk_supplier_invoice_lines_position_non_negative
        CHECK (position >= 0),
    CONSTRAINT fk_supplier_invoice_lines_invoice
        FOREIGN KEY (supplier_invoice_id, org_id) REFERENCES supplier_invoices(id, org_id) ON DELETE CASCADE
);

CREATE INDEX idx_supplier_invoice_lines_org_id ON supplier_invoice_lines(org_id);
CREATE INDEX idx_supplier_invoice_lines_supplier_invoice_id ON supplier_invoice_lines(supplier_invoice_id);
CREATE INDEX idx_supplier_invoice_lines_deleted_at ON supplier_invoice_lines(deleted_at);
