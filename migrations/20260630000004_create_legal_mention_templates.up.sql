CREATE TABLE legal_mention_templates (
  id UUID PRIMARY KEY,
  org_id UUID NOT NULL REFERENCES organizations(id),
  name TEXT NOT NULL,
  body TEXT NOT NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  deleted_at TIMESTAMPTZ NULL
);
CREATE INDEX idx_legal_mention_templates_org ON legal_mention_templates(org_id) WHERE deleted_at IS NULL;
