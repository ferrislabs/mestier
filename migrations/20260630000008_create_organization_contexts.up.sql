CREATE TABLE organization_contexts (
	id UUID PRIMARY KEY,
	org_id UUID NOT NULL REFERENCES organizations(id),
	label TEXT NOT NULL,
	address_line TEXT NULL,
	postal_code TEXT NULL,
	city TEXT NULL,
	country TEXT NULL,
	siret TEXT NULL,
	rcs TEXT NULL,
	ape TEXT NULL,
	vat_intracom TEXT NULL,
	iban TEXT NULL,
	bic TEXT NULL,
	created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
	updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
	deleted_at TIMESTAMPTZ NULL
);
CREATE INDEX idx_organization_contexts_org ON organization_contexts(org_id) WHERE deleted_at IS NULL;
