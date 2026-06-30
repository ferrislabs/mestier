CREATE TABLE billing_settings (
  org_id UUID PRIMARY KEY REFERENCES organizations(id),
  payment_terms_days INTEGER NOT NULL DEFAULT 30,
  late_penalty_rate NUMERIC NOT NULL DEFAULT 0,
  recovery_indemnity_cents INTEGER NOT NULL DEFAULT 4000,
  default_deposit_basis TEXT NULL,
  default_deposit_value NUMERIC NULL,
  default_vat_rate NUMERIC NOT NULL DEFAULT 20.00,
  iban TEXT, bic TEXT, siret TEXT, rcs TEXT, ape TEXT, vat_intracom TEXT, footer TEXT,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
