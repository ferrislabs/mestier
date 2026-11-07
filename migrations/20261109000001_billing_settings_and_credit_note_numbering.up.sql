CREATE TABLE organization_billing_settings (
    org_id               UUID        PRIMARY KEY REFERENCES organizations(id),
    payment_terms_days   INTEGER     NOT NULL DEFAULT 30,
    quote_validity_days  INTEGER     NOT NULL DEFAULT 30,
    default_vat_rate_bp  INTEGER     NOT NULL DEFAULT 2000,
    late_penalty         TEXT        NOT NULL DEFAULT 'trois fois le taux d''intérêt légal',
    recovery_fee_cents   INTEGER     NOT NULL DEFAULT 4000,
    invoice_footer       TEXT        NOT NULL DEFAULT '',
    number_padding       INTEGER     NOT NULL DEFAULT 4,
    iban                 TEXT        NULL,
    bic                  TEXT        NULL,
    created_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at           TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_billing_settings_payment_terms_days
        CHECK (payment_terms_days BETWEEN 0 AND 60),
    CONSTRAINT chk_billing_settings_quote_validity_days
        CHECK (quote_validity_days >= 1),
    CONSTRAINT chk_billing_settings_default_vat_rate_bp
        CHECK (default_vat_rate_bp IN (0, 550, 1000, 2000)),
    CONSTRAINT chk_billing_settings_late_penalty_not_blank
        CHECK (length(btrim(late_penalty)) > 0),
    CONSTRAINT chk_billing_settings_recovery_fee_cents
        CHECK (recovery_fee_cents >= 0),
    CONSTRAINT chk_billing_settings_number_padding
        CHECK (number_padding BETWEEN 1 AND 6),
    CONSTRAINT chk_billing_settings_iban_not_blank
        CHECK (iban IS NULL OR length(btrim(iban)) > 0),
    CONSTRAINT chk_billing_settings_bic_not_blank
        CHECK (bic IS NULL OR length(btrim(bic)) > 0)
);

CREATE TABLE credit_note_number_counters (
    organization_id UUID        NOT NULL REFERENCES organizations(id),
    year            INTEGER     NOT NULL,
    last_number     INTEGER     NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    PRIMARY KEY (organization_id, year),
    CONSTRAINT chk_credit_note_number_counters_last_number_positive
        CHECK (last_number > 0)
);
