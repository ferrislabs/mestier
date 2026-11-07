ALTER TABLE customer_contacts
    ADD CONSTRAINT uq_customer_contacts_id_customer UNIQUE (id, customer_id),
    ADD COLUMN mobile TEXT NULL,
    ADD CONSTRAINT chk_customer_contacts_mobile_not_blank_when_present
        CHECK (mobile IS NULL OR length(btrim(mobile)) > 0);

ALTER TABLE customer_contexts
    ADD CONSTRAINT uq_customer_contexts_id_customer UNIQUE (id, customer_id),
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'SITE',
    ADD COLUMN country TEXT NOT NULL DEFAULT 'FR',
    ADD COLUMN address_line2 TEXT NULL,
    ADD CONSTRAINT chk_customer_contexts_kind
        CHECK (kind IN ('BILLING', 'SITE', 'OTHER')),
    ADD CONSTRAINT chk_customer_contexts_country_not_blank
        CHECK (length(btrim(country)) > 0),
    ADD CONSTRAINT chk_customer_contexts_address_line2_not_blank_when_present
        CHECK (address_line2 IS NULL OR length(btrim(address_line2)) > 0);

ALTER TABLE customers
    ADD COLUMN kind TEXT NOT NULL DEFAULT 'PROFESSIONAL',
    ADD COLUMN website TEXT NULL,
    ADD COLUMN vat_number TEXT NULL,
    ADD COLUMN owner_member_id UUID NULL,
    ADD COLUMN lead_source TEXT NULL,
    ADD COLUMN notes TEXT NULL,
    ADD COLUMN payment_terms_days INTEGER NULL,
    ADD COLUMN billing_contact_id UUID NULL,
    ADD COLUMN billing_address_id UUID NULL,
    ADD COLUMN converted_at TIMESTAMPTZ NULL,
    ADD COLUMN archived_at TIMESTAMPTZ NULL,
    ADD CONSTRAINT chk_customers_kind
        CHECK (kind IN ('INDIVIDUAL', 'PROFESSIONAL')),
    ADD CONSTRAINT chk_customers_website_not_blank_when_present
        CHECK (website IS NULL OR length(btrim(website)) > 0),
    ADD CONSTRAINT chk_customers_vat_number_not_blank_when_present
        CHECK (vat_number IS NULL OR length(btrim(vat_number)) > 0),
    ADD CONSTRAINT chk_customers_notes_not_blank_when_present
        CHECK (notes IS NULL OR length(btrim(notes)) > 0),
    ADD CONSTRAINT chk_customers_lead_source
        CHECK (lead_source IS NULL OR lead_source IN (
            'bouche_a_oreille', 'site_web', 'recommandation', 'salon', 'architecte', 'autre'
        )),
    ADD CONSTRAINT chk_customers_payment_terms_days
        CHECK (payment_terms_days IS NULL OR payment_terms_days BETWEEN 0 AND 60),
    ADD CONSTRAINT fk_customers_owner_member
        FOREIGN KEY (owner_member_id, org_id) REFERENCES organization_members(id, organization_id),
    ADD CONSTRAINT fk_customers_billing_contact
        FOREIGN KEY (billing_contact_id, id) REFERENCES customer_contacts(id, customer_id),
    ADD CONSTRAINT fk_customers_billing_address
        FOREIGN KEY (billing_address_id, id) REFERENCES customer_contexts(id, customer_id);

UPDATE customers SET converted_at = created_at WHERE status = 'CLIENT';
UPDATE customers SET archived_at = updated_at WHERE status = 'ARCHIVED';

CREATE INDEX idx_customers_owner_member_id ON customers(owner_member_id) WHERE owner_member_id IS NOT NULL;
