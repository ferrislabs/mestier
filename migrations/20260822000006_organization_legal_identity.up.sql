-- Legal identity an organization needs before it can issue a valid
-- document (quote, invoice). Nothing here is NOT NULL: existing
-- organizations have none of it, and a migration that refuses to run is
-- not an option. Completeness is asserted at the boundary, when a
-- document is issued, by `LegalIdentity::try_from_organization`
-- (libs/core/src/domain/organization/legal_identity.rs) — not by the
-- column definition. See #310.
ALTER TABLE organizations
    ADD COLUMN legal_name          TEXT    NULL,
    ADD COLUMN legal_form          TEXT    NULL,
    ADD COLUMN registration_number TEXT    NULL,
    -- VAT status is an enum, not a nullable number: "subject" carries a
    -- VAT number, "not_subject" carries the legal basis for the
    -- exemption. Modelling exemption as a missing number would make it
    -- indistinguishable from "nobody filled this in yet".
    ADD COLUMN vat_status          TEXT    NULL,
    ADD COLUMN vat_number          TEXT    NULL,
    ADD COLUMN vat_exemption_basis TEXT    NULL,
    ADD COLUMN share_capital_cents BIGINT  NULL,
    ADD COLUMN address_line1       TEXT    NULL,
    ADD COLUMN address_line2       TEXT    NULL,
    ADD COLUMN address_postal_code TEXT    NULL,
    ADD COLUMN address_city        TEXT    NULL,
    ADD COLUMN address_country     TEXT    NULL,
    ADD COLUMN contact_email       TEXT    NULL,
    ADD COLUMN contact_phone       TEXT    NULL,
    ADD COLUMN insurance_mention   TEXT    NULL;

ALTER TABLE organizations
    ADD CONSTRAINT chk_organizations_legal_name_not_blank
        CHECK (legal_name IS NULL OR length(btrim(legal_name)) > 0),
    ADD CONSTRAINT chk_organizations_legal_form_not_blank
        CHECK (legal_form IS NULL OR length(btrim(legal_form)) > 0),
    ADD CONSTRAINT chk_organizations_registration_number_not_blank
        CHECK (registration_number IS NULL OR length(btrim(registration_number)) > 0),
    ADD CONSTRAINT chk_organizations_vat_status
        CHECK (vat_status IS NULL OR vat_status IN ('subject', 'not_subject')),
    ADD CONSTRAINT chk_organizations_vat_number_when_subject
        CHECK (
            vat_status IS DISTINCT FROM 'subject'
            OR (vat_number IS NOT NULL AND length(btrim(vat_number)) > 0)
        ),
    ADD CONSTRAINT chk_organizations_vat_exemption_basis_when_not_subject
        CHECK (
            vat_status IS DISTINCT FROM 'not_subject'
            OR (vat_exemption_basis IS NOT NULL AND length(btrim(vat_exemption_basis)) > 0)
        ),
    ADD CONSTRAINT chk_organizations_share_capital_cents_non_negative
        CHECK (share_capital_cents IS NULL OR share_capital_cents >= 0),
    ADD CONSTRAINT chk_organizations_address_line1_not_blank
        CHECK (address_line1 IS NULL OR length(btrim(address_line1)) > 0),
    ADD CONSTRAINT chk_organizations_address_line2_not_blank
        CHECK (address_line2 IS NULL OR length(btrim(address_line2)) > 0),
    ADD CONSTRAINT chk_organizations_address_postal_code_not_blank
        CHECK (address_postal_code IS NULL OR length(btrim(address_postal_code)) > 0),
    ADD CONSTRAINT chk_organizations_address_city_not_blank
        CHECK (address_city IS NULL OR length(btrim(address_city)) > 0),
    ADD CONSTRAINT chk_organizations_address_country_not_blank
        CHECK (address_country IS NULL OR length(btrim(address_country)) > 0),
    ADD CONSTRAINT chk_organizations_contact_email_not_blank
        CHECK (contact_email IS NULL OR length(btrim(contact_email)) > 0),
    ADD CONSTRAINT chk_organizations_contact_phone_not_blank
        CHECK (contact_phone IS NULL OR length(btrim(contact_phone)) > 0),
    ADD CONSTRAINT chk_organizations_insurance_mention_not_blank
        CHECK (insurance_mention IS NULL OR length(btrim(insurance_mention)) > 0);
