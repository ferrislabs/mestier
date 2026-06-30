CREATE TABLE quote_legal_mentions (
  id UUID PRIMARY KEY,
  org_id UUID NOT NULL REFERENCES organizations(id),
  quote_id UUID NOT NULL REFERENCES quotes(id) ON DELETE CASCADE,
  template_id UUID NOT NULL REFERENCES legal_mention_templates(id),
  created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
  UNIQUE (quote_id, template_id)
);
CREATE INDEX idx_quote_legal_mentions_quote ON quote_legal_mentions(quote_id);
