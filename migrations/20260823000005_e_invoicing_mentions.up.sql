-- Fields the French e-invoicing reform requires, added before any invoice
-- is actually issued through the platform (issuing lands in #317) — a
-- migration over documents that become legally immutable the moment they
-- exist is worse than two nullable columns now. See #341.
--
-- Verified 2026-08-23 against economie.gouv.fr ("Tout savoir sur la
-- facturation electronique pour les entreprises"), urssaf.fr
-- ("Facturation electronique : obligatoire au 1er septembre 2026") and
-- corroborated by Pennylane's practitioner guide: from 1 September 2026,
-- an electronic invoice must additionally carry four mentions beyond what
-- #310/#284 already modelled: the SIREN of a VAT-subject customer, the
-- delivery address when it differs from the customer's, the nature of the
-- operation (goods / services / both), and an explicit statement of the
-- option for VAT on debits. The underlying PPF/PDP transmission mechanics
-- (and the exact enforcement date for TPE/PME emission, 2027) are NOT
-- re-verified here and are out of scope for this migration, which only
-- carries the mentions a document needs to display.
--
-- Nothing here is NOT NULL: existing organizations, customers and invoices
-- have none of this, and a migration that refuses to run is not an option.
-- Completeness for actual electronic transmission is asserted at the
-- boundary (`ElectronicInvoicingFacts::try_new`,
-- libs/core/src/domain/organization/legal_identity.rs), not by the column
-- definition — same device #310 used for `LegalIdentity`.

-- Property of the issuer's VAT regime: whether VAT is due on collection
-- (encaissements, the default) or on invoicing (debits). Always has a
-- value — it is a regime choice, not a fact that can be "missing" — so,
-- unlike the rest of this migration, this one column is NOT NULL with a
-- default rather than nullable.
ALTER TABLE organizations
    ADD COLUMN vat_on_debits BOOLEAN NOT NULL DEFAULT false;

-- The customer's own SIREN: at least one mention the reform requires is
-- about the customer, not just the issuer (required on the invoice when
-- the customer is itself VAT-subject — this codebase does not yet track a
-- customer's VAT status, so that gating is left to the caller for now,
-- noted as a follow-up in the commit).
ALTER TABLE customers
    ADD COLUMN registration_number TEXT NULL,
    ADD CONSTRAINT chk_customers_registration_number_not_blank
        CHECK (registration_number IS NULL OR length(btrim(registration_number)) > 0);

CREATE TYPE invoice_operation_nature AS ENUM (
    'GOODS',
    'SERVICES',
    'BOTH'
);

ALTER TABLE invoices
    ADD COLUMN operation_nature invoice_operation_nature NULL,
    ADD COLUMN delivery_address_line1       TEXT NULL,
    ADD COLUMN delivery_address_line2       TEXT NULL,
    ADD COLUMN delivery_address_postal_code TEXT NULL,
    ADD COLUMN delivery_address_city        TEXT NULL,
    ADD COLUMN delivery_address_country     TEXT NULL,
    ADD CONSTRAINT chk_invoices_delivery_address_line1_not_blank
        CHECK (delivery_address_line1 IS NULL OR length(btrim(delivery_address_line1)) > 0),
    ADD CONSTRAINT chk_invoices_delivery_address_line2_not_blank
        CHECK (delivery_address_line2 IS NULL OR length(btrim(delivery_address_line2)) > 0),
    ADD CONSTRAINT chk_invoices_delivery_address_postal_code_not_blank
        CHECK (delivery_address_postal_code IS NULL OR length(btrim(delivery_address_postal_code)) > 0),
    ADD CONSTRAINT chk_invoices_delivery_address_city_not_blank
        CHECK (delivery_address_city IS NULL OR length(btrim(delivery_address_city)) > 0),
    ADD CONSTRAINT chk_invoices_delivery_address_country_not_blank
        CHECK (delivery_address_country IS NULL OR length(btrim(delivery_address_country)) > 0),
    -- A partial delivery address is not a usable one: either every field
    -- required to print an address is present, or none are.
    ADD CONSTRAINT chk_invoices_delivery_address_all_or_nothing
        CHECK (
            (delivery_address_line1 IS NULL AND delivery_address_postal_code IS NULL
                AND delivery_address_city IS NULL AND delivery_address_country IS NULL)
            OR (delivery_address_line1 IS NOT NULL AND delivery_address_postal_code IS NOT NULL
                AND delivery_address_city IS NOT NULL AND delivery_address_country IS NOT NULL)
        );
