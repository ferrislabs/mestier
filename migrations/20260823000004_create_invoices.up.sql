-- An invoice: a document frozen the moment it is issued. Not a quote in
-- another status — a quote is edited until accepted, an invoice is immutable
-- from the moment it exists as anything other than a draft, and one table
-- would need a mutability rule that depends on status, exactly the kind of
-- rule that gets forgotten in one code path. See #316.
--
-- The lines are a snapshot: `invoice_lines` does not reference `quote_lines`.
-- Editing a quote after invoicing must not rewrite an issued invoice, the
-- same failure the cost-history epic (#274) fixed one table over for
-- employee cost. `label` is copied text, never a foreign key.
--
-- The issuer's identity is snapshotted too, in `issuer_identity`, as a JSONB
-- blob rather than denormalised columns: its shape mirrors
-- `LegalIdentity` (libs/core/src/domain/organization/legal_identity.rs),
-- which already changes as one unit (see #310, and #341 landing right after
-- this migration to extend it further) — a JSONB blob keeps invoices from
-- being schema-coupled to every future field `LegalIdentity` gains. `NULL`
-- until the invoice is issued.
--
-- `kind` exists from this migration on, even though only `STANDARD` is
-- reachable through this issue's own service: #317 (deposit / final /
-- deduction) and #318 (credit notes) both need it and neither carries its
-- own migration, so it is cheaper to model the whole shape now than to
-- shoehorn an ALTER TABLE into an issue that isn't supposed to touch schema.
-- Same reasoning for `invoice_number_counters` and `invoice_number_prefix`:
-- #317 is the issue that actually allocates a number, using the locking
-- pattern #313 established for quotes, but the table and the column have to
-- exist before that issue can avoid a migration of its own.
CREATE TYPE invoice_status AS ENUM (
    'DRAFT',
    'ISSUED',
    'PAID',
    'PARTIALLY_PAID',
    'CANCELLED'
);

CREATE TYPE invoice_kind AS ENUM (
    'STANDARD',
    'DEPOSIT',
    'FINAL',
    'CREDIT_NOTE'
);

CREATE TABLE invoices (
    id                   UUID           PRIMARY KEY,
    org_id               UUID           NOT NULL REFERENCES organizations(id),
    -- Allocated only when the invoice first leaves Draft. Gapless and unique
    -- per organization once allocated, and never reallocated afterwards.
    number               TEXT           NULL,
    kind                 invoice_kind   NOT NULL DEFAULT 'STANDARD',
    -- Nullable: a project with no quote can still be invoiced (#317), and an
    -- invoice created outside a project's billing flow has nowhere to point.
    project_id           UUID           NULL,
    customer_id          UUID           NOT NULL REFERENCES customers(id),
    customer_context_id  UUID           NOT NULL REFERENCES customer_contexts(id),
    status               invoice_status NOT NULL DEFAULT 'DRAFT',
    issued_at            TIMESTAMPTZ    NULL,
    due_at               TIMESTAMPTZ    NULL,
    notes                TEXT           NULL,
    net_cents            INTEGER        NOT NULL DEFAULT 0,
    vat_breakdown         JSONB          NOT NULL DEFAULT '[]',
    gross_cents          INTEGER        NOT NULL DEFAULT 0,
    -- The issuer's `LegalIdentity` as it was the instant the invoice was
    -- issued. `NULL` on a draft.
    issuer_identity       JSONB          NULL,
    deleted_at           TIMESTAMPTZ    NULL,
    created_at           TIMESTAMPTZ    NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ    NOT NULL DEFAULT now(),

    CONSTRAINT chk_invoices_number_not_blank_when_present
        CHECK (number IS NULL OR length(btrim(number)) > 0),
    CONSTRAINT chk_invoices_notes_not_blank_when_present
        CHECK (notes IS NULL OR length(btrim(notes)) > 0),
    CONSTRAINT chk_invoices_net_cents_non_negative
        CHECK (net_cents >= 0),
    CONSTRAINT chk_invoices_gross_cents_non_negative
        CHECK (gross_cents >= 0),
    -- A draft has no number and no frozen issuer identity; anything past
    -- draft must have both. Belt on top of the type-level guarantee
    -- (`DraftInvoice`/`Invoice` in the domain): the database refuses the row
    -- even if a future code path forgets the rule the type enforces today.
    CONSTRAINT chk_invoices_issued_state
        CHECK (
            (status = 'DRAFT' AND number IS NULL AND issued_at IS NULL AND issuer_identity IS NULL)
            OR (status != 'DRAFT' AND number IS NOT NULL AND issued_at IS NOT NULL AND issuer_identity IS NOT NULL)
        ),
    -- The target of the composite foreign keys below (`invoice_lines`,
    -- and future `invoice_payments`/credit notes): same device as
    -- `projects.uq_projects_id_org`, it makes an invoice line attached to
    -- another organization's invoice unrepresentable rather than merely
    -- forbidden by application code.
    CONSTRAINT uq_invoices_id_org UNIQUE (id, org_id),
    CONSTRAINT uq_invoices_org_number UNIQUE (org_id, number),
    CONSTRAINT fk_invoices_project
        FOREIGN KEY (project_id, org_id) REFERENCES projects(id, org_id)
);

CREATE INDEX idx_invoices_org_id ON invoices(org_id);
CREATE INDEX idx_invoices_customer_id ON invoices(customer_id);
CREATE INDEX idx_invoices_project_id ON invoices(project_id);
CREATE INDEX idx_invoices_status ON invoices(status);
CREATE INDEX idx_invoices_due_at ON invoices(due_at);
CREATE INDEX idx_invoices_deleted_at ON invoices(deleted_at);

CREATE TABLE invoice_lines (
    id                     UUID        PRIMARY KEY,
    org_id                 UUID        NOT NULL REFERENCES organizations(id),
    invoice_id             UUID        NOT NULL,
    -- Copied text, never a foreign key to `quote_lines`: see the header
    -- comment. A quote line edited or deleted after invoicing must not
    -- change, or lose, what an issued invoice already says.
    label                  TEXT        NOT NULL,
    quantity               NUMERIC     NOT NULL,
    unit_price_cents       INTEGER     NOT NULL,
    vat_rate_basis_points  INTEGER     NULL,
    position               INTEGER     NOT NULL,
    deleted_at             TIMESTAMPTZ NULL,
    created_at             TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at             TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_invoice_lines_label_not_blank
        CHECK (length(btrim(label)) > 0),
    CONSTRAINT chk_invoice_lines_quantity_positive
        CHECK (quantity > 0),
    CONSTRAINT chk_invoice_lines_unit_price_cents_non_negative
        CHECK (unit_price_cents >= 0),
    CONSTRAINT chk_invoice_lines_vat_rate_basis_points_range
        CHECK (vat_rate_basis_points IS NULL OR vat_rate_basis_points BETWEEN 0 AND 10000),
    CONSTRAINT chk_invoice_lines_position_non_negative
        CHECK (position >= 0),
    CONSTRAINT fk_invoice_lines_invoice
        FOREIGN KEY (invoice_id, org_id) REFERENCES invoices(id, org_id) ON DELETE CASCADE
);

CREATE INDEX idx_invoice_lines_org_id ON invoice_lines(org_id);
CREATE INDEX idx_invoice_lines_invoice_id ON invoice_lines(invoice_id);
CREATE INDEX idx_invoice_lines_deleted_at ON invoice_lines(deleted_at);

-- Numbering: identical device to `quote_number_counters` (see #313 and the
-- `quote_numbering` migration) — one row per (organization, year), locked by
-- the upsert that reads it. Invoice numbering is stricter than quote
-- numbering (a gap in an invoice sequence is a real problem, not just an
-- inconvenience), which is exactly why it reuses a pattern already proven
-- safe under contention rather than inventing a second one.
CREATE TABLE invoice_number_counters (
    organization_id UUID        NOT NULL REFERENCES organizations(id),
    year            INTEGER     NOT NULL,
    next_number     INTEGER     NOT NULL DEFAULT 1,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (organization_id, year),
    CONSTRAINT chk_invoice_number_counters_next_number_positive
        CHECK (next_number > 0)
);

ALTER TABLE organizations
    ADD COLUMN invoice_number_prefix TEXT NOT NULL DEFAULT 'FAC',
    ADD CONSTRAINT chk_organizations_invoice_number_prefix_not_blank
        CHECK (length(btrim(invoice_number_prefix)) > 0);
