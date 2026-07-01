CREATE TYPE invoice_status AS ENUM ('DRAFT','SENT','PARTIALLY_PAID','PAID','CANCELLED');
CREATE TYPE invoice_type   AS ENUM ('STANDARD','DEPOSIT','BALANCE');

CREATE TABLE invoices (
  id UUID PRIMARY KEY,
  org_id UUID NOT NULL REFERENCES organizations(id),
  customer_id UUID NOT NULL REFERENCES customers(id),
  customer_context_id UUID NOT NULL REFERENCES customer_contexts(id),
  reference TEXT NULL,
  title TEXT NOT NULL,
  status invoice_status NOT NULL,
  invoice_type invoice_type NOT NULL DEFAULT 'STANDARD',
  source_quote_id UUID NULL REFERENCES quotes(id),
  parent_invoice_id UUID NULL REFERENCES invoices(id),
  deposit_basis TEXT NULL,
  deposit_value NUMERIC NULL,
  deposit_amount_cents INTEGER NULL,
  total_ht_cents INTEGER NOT NULL DEFAULT 0,
  total_vat_cents INTEGER NOT NULL DEFAULT 0,
  total_ttc_cents INTEGER NOT NULL DEFAULT 0,
  total_cents INTEGER NOT NULL DEFAULT 0,
  amount_paid_cents INTEGER NOT NULL DEFAULT 0,
  snapshot JSONB NULL,
  pdf_key TEXT NULL,
  issued_at TIMESTAMPTZ NULL,
  due_at TIMESTAMPTZ NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ NULL
);
CREATE UNIQUE INDEX uq_invoices_org_reference ON invoices(org_id, reference) WHERE reference IS NOT NULL;
CREATE INDEX idx_invoices_org ON invoices(org_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_invoices_source_quote ON invoices(source_quote_id);

CREATE TABLE invoice_lines (
  id UUID PRIMARY KEY,
  org_id UUID NOT NULL REFERENCES organizations(id),
  invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
  service_rate_id UUID NULL REFERENCES service_rates(id) ON DELETE SET NULL,
  label TEXT NOT NULL,
  quantity NUMERIC NOT NULL,
  unit TEXT NOT NULL,
  unit_price_cents INTEGER NOT NULL,
  vat_rate NUMERIC NOT NULL DEFAULT 20.00,
  notes TEXT NULL,
  photo_keys TEXT[] NOT NULL DEFAULT '{}',
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ NULL
);
CREATE INDEX idx_invoice_lines_invoice ON invoice_lines(invoice_id);

CREATE TABLE invoice_legal_mentions (
  id UUID PRIMARY KEY,
  org_id UUID NOT NULL REFERENCES organizations(id),
  invoice_id UUID NOT NULL REFERENCES invoices(id) ON DELETE CASCADE,
  template_id UUID NOT NULL REFERENCES legal_mention_templates(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (invoice_id, template_id)
);
CREATE INDEX idx_invoice_legal_mentions_invoice ON invoice_legal_mentions(invoice_id);
