-- A payment recorded against an invoice (#320). A sub-resource of the
-- invoice aggregate, not a new aggregate: same device as `invoice_lines`
-- and, from #318, credit notes — one table, reached through
-- `InvoiceRepository`, no repository crate of its own.
--
-- Representation: positive amounts only, enforced here and again in the
-- domain. A refund is a credit note (#318), never a negative payment — the
-- two corrections live in different tables on purpose, so "how much was
-- actually paid" (this table) never has to net against "how much was
-- corrected" (credit notes) to mean what it says.
--
-- Soft-deleted with an audit trail, the precedent this table sets for the
-- codebase: a deletion always names who and when, together or not at all
-- (`chk_invoice_payments_deleted_state`), and the row survives deletion
-- rather than disappearing — the whole point of #320's payment history.
CREATE TABLE invoice_payments (
    id           UUID        PRIMARY KEY,
    org_id       UUID        NOT NULL REFERENCES organizations(id),
    invoice_id   UUID        NOT NULL,
    amount_cents INTEGER     NOT NULL,
    paid_on      DATE        NOT NULL,
    method       TEXT        NOT NULL,
    reference    TEXT        NULL,
    note         TEXT        NULL,
    recorded_by  UUID        NOT NULL REFERENCES users(id),
    deleted_by   UUID        NULL REFERENCES users(id),
    deleted_at   TIMESTAMPTZ NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_invoice_payments_amount_cents_positive
        CHECK (amount_cents > 0),
    CONSTRAINT chk_invoice_payments_method_not_blank
        CHECK (length(btrim(method)) > 0),
    CONSTRAINT chk_invoice_payments_reference_not_blank
        CHECK (reference IS NULL OR length(btrim(reference)) > 0),
    CONSTRAINT chk_invoice_payments_note_not_blank
        CHECK (note IS NULL OR length(btrim(note)) > 0),
    -- The audit trail #320 asks for: a deletion always names who and when,
    -- together or not at all.
    CONSTRAINT chk_invoice_payments_deleted_state
        CHECK ((deleted_at IS NULL) = (deleted_by IS NULL)),
    -- Same composite-FK device as `invoice_lines`/`fk_invoices_source_invoice`:
    -- makes a payment attached to another organization's invoice
    -- unrepresentable rather than merely forbidden by application code.
    CONSTRAINT fk_invoice_payments_invoice
        FOREIGN KEY (invoice_id, org_id) REFERENCES invoices(id, org_id)
);

CREATE INDEX idx_invoice_payments_org_id ON invoice_payments(org_id);
CREATE INDEX idx_invoice_payments_invoice_id ON invoice_payments(invoice_id);
CREATE INDEX idx_invoice_payments_deleted_at ON invoice_payments(deleted_at);
