-- A credit note is not a fifth table: it is an invoice with
-- `kind = 'CREDIT_NOTE'` that references the invoice it corrects (#318).
-- An issued invoice is never edited (see #316's header comment) — the only
-- way to correct one is to issue a document against it, and that document
-- takes its number from the same `invoice_number_counters` sequence #317
-- already wired up. Representation: positive amounts throughout, `kind` is
-- what marks a document as a correction, never a negative `net_cents` — see
-- the domain-side ruling in `InvoiceService::issue_credit_note`.
--
-- `source_invoice_id` is nullable because only a `CREDIT_NOTE` ever sets it;
-- every other kind stays NULL. Same composite-FK device the table already
-- uses for `project_id` (`fk_invoices_project`, #316's migration), applied
-- here to `invoices` itself: self-referential, scoped by `org_id` so a
-- credit note can never point at another organization's invoice.
ALTER TABLE invoices
    ADD COLUMN source_invoice_id UUID NULL,
    ADD CONSTRAINT fk_invoices_source_invoice
        FOREIGN KEY (source_invoice_id, org_id) REFERENCES invoices(id, org_id);

CREATE INDEX idx_invoices_source_invoice_id ON invoices(source_invoice_id);
