ALTER TABLE quotes   ADD COLUMN emitter_context_id UUID NULL REFERENCES organization_contexts(id);
ALTER TABLE invoices ADD COLUMN emitter_context_id UUID NULL REFERENCES organization_contexts(id);
