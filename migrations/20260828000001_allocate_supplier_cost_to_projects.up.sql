-- Attributing a confirmed supplier invoice's lines to the projects they were
-- bought for, so a supplier's bill folds into what a project actually cost
-- — see #334, #338.
--
-- A line may be split across several projects, or left partly (or wholly)
-- unallocated: overhead the business absorbs generally is a legitimate
-- outcome, not a gap to fill. What must never happen is a line's allocations
-- adding up to more than what was printed on it. A per-row CHECK cannot see
-- that: it only ever sees the row being written, never the *sum* across a
-- line's other allocations, so the bound is enforced below by a trigger
-- instead, which can read that sum on every insert or update.
--
-- `uq_supplier_invoice_lines_id_org` did not exist before this issue: no
-- earlier table needed a composite foreign key onto a supplier invoice
-- line. Added here, on the existing table, rather than growing the frozen
-- #336 migration — same device as every other `(id, org_id)` unique already
-- in this schema (`uq_supplier_invoices_id_org`, `uq_projects_id_org`, ...).
ALTER TABLE supplier_invoice_lines
    ADD CONSTRAINT uq_supplier_invoice_lines_id_org UNIQUE (id, org_id);

CREATE TABLE supplier_invoice_line_allocations (
    id                        UUID        PRIMARY KEY,
    org_id                    UUID        NOT NULL REFERENCES organizations(id),
    supplier_invoice_line_id  UUID        NOT NULL,
    project_id                UUID        NOT NULL,
    -- Net of VAT, a slice of the line's own `line_total_cents` — never the
    -- gross figure. Whether the real cost is this net amount or the grossed-
    -- up one depends on the organization's own VAT status, and that decision
    -- is made once, at report time, in `profitability::service` — not here,
    -- and not per allocation.
    --
    -- Same sign as the line's own `line_total_cents`: a credit/rebate line
    -- can only ever be allocated as a credit, never flipped into a positive
    -- cost by the act of allocating it.
    amount_cents              INTEGER     NOT NULL,
    created_at                TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at                TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_supplier_invoice_line_allocations_amount_not_zero
        CHECK (amount_cents <> 0),
    -- Target of the composite FK below is `uq_supplier_invoice_lines_id_org`,
    -- added just above: an allocation attached to another organization's
    -- line becomes unrepresentable rather than merely forbidden. Cascades
    -- with the line: the line is what the amount is a slice of, and it has
    -- no meaning once that line is gone.
    CONSTRAINT fk_supplier_invoice_line_allocations_line
        FOREIGN KEY (supplier_invoice_line_id, org_id)
        REFERENCES supplier_invoice_lines(id, org_id) ON DELETE CASCADE,
    -- Same composite-FK device against `uq_projects_id_org`. No cascade here:
    -- a project is not expected to disappear the way a line's own invoice
    -- might be corrected, and losing the allocation silently on a project
    -- delete would quietly change a past invoice's own bookkeeping.
    CONSTRAINT fk_supplier_invoice_line_allocations_project
        FOREIGN KEY (project_id, org_id)
        REFERENCES projects(id, org_id)
);

CREATE INDEX idx_supplier_invoice_line_allocations_org_id
    ON supplier_invoice_line_allocations(org_id);
CREATE INDEX idx_supplier_invoice_line_allocations_line_id
    ON supplier_invoice_line_allocations(supplier_invoice_line_id);
CREATE INDEX idx_supplier_invoice_line_allocations_project_id
    ON supplier_invoice_line_allocations(project_id);

-- The one rule a row-level CHECK cannot enforce: the sum of a line's
-- allocations must never overshoot what the line itself is worth. Compares
-- magnitude and sign together, so a credit line cannot be "allocated" past
-- zero into a positive cost the same way a normal line cannot be
-- over-allocated into more expense than it printed.
CREATE FUNCTION supplier_invoice_line_allocations_enforce_line_total()
RETURNS TRIGGER AS $$
DECLARE
    v_line_total INTEGER;
    v_allocated  BIGINT;
BEGIN
    SELECT line_total_cents INTO v_line_total
    FROM supplier_invoice_lines
    WHERE id = NEW.supplier_invoice_line_id;

    SELECT COALESCE(SUM(amount_cents), 0) INTO v_allocated
    FROM supplier_invoice_line_allocations
    WHERE supplier_invoice_line_id = NEW.supplier_invoice_line_id;

    IF v_line_total >= 0 AND (v_allocated < 0 OR v_allocated > v_line_total) THEN
        RAISE EXCEPTION
            'supplier invoice line % allocations total % cents, which is outside [0, %]',
            NEW.supplier_invoice_line_id, v_allocated, v_line_total
            USING ERRCODE = '23514';
    ELSIF v_line_total < 0 AND (v_allocated > 0 OR v_allocated < v_line_total) THEN
        RAISE EXCEPTION
            'supplier invoice line % allocations total % cents, which is outside [%, 0]',
            NEW.supplier_invoice_line_id, v_allocated, v_line_total
            USING ERRCODE = '23514';
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_supplier_invoice_line_allocations_enforce_line_total
    AFTER INSERT OR UPDATE ON supplier_invoice_line_allocations
    FOR EACH ROW
    EXECUTE FUNCTION supplier_invoice_line_allocations_enforce_line_total();
