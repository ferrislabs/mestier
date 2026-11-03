CREATE TABLE support_ticket_entries (
    id               UUID PRIMARY KEY,
    org_id           UUID NOT NULL REFERENCES organizations(id),
    ticket_id        UUID NOT NULL,
    kind             TEXT NOT NULL,
    author_member_id UUID NULL,
    body             TEXT NOT NULL,
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_support_ticket_entries_ticket
        FOREIGN KEY (ticket_id, org_id) REFERENCES support_tickets(id, org_id) ON DELETE CASCADE,
    CONSTRAINT fk_support_ticket_entries_author
        FOREIGN KEY (author_member_id, org_id)
        REFERENCES organization_members(id, organization_id)
        ON DELETE SET NULL (author_member_id),
    CONSTRAINT uq_support_ticket_entries_id_org UNIQUE (id, org_id),
    CONSTRAINT chk_support_ticket_entries_kind
        CHECK (kind IN ('client_message', 'reply', 'internal_note', 'draft')),
    CONSTRAINT chk_support_ticket_entries_body_not_blank CHECK (btrim(body) <> ''),
    CONSTRAINT chk_support_ticket_entries_client_has_no_author
        CHECK (kind <> 'client_message' OR author_member_id IS NULL)
);

CREATE INDEX idx_support_ticket_entries_ticket
    ON support_ticket_entries(org_id, ticket_id, created_at);

CREATE TABLE support_ticket_attachments (
    id         UUID PRIMARY KEY,
    org_id     UUID NOT NULL REFERENCES organizations(id),
    ticket_id  UUID NOT NULL,
    entry_id   UUID NOT NULL,
    file_key   TEXT NOT NULL,
    file_name  TEXT NOT NULL,
    mime_type  TEXT NOT NULL,
    size_bytes BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT fk_support_ticket_attachments_entry
        FOREIGN KEY (entry_id, org_id) REFERENCES support_ticket_entries(id, org_id) ON DELETE CASCADE,
    CONSTRAINT fk_support_ticket_attachments_ticket
        FOREIGN KEY (ticket_id, org_id) REFERENCES support_tickets(id, org_id) ON DELETE CASCADE,
    CONSTRAINT uq_support_ticket_attachments_file_key UNIQUE (file_key),
    CONSTRAINT chk_support_ticket_attachments_size CHECK (size_bytes >= 0),
    CONSTRAINT chk_support_ticket_attachments_name_not_blank CHECK (btrim(file_name) <> '')
);

CREATE INDEX idx_support_ticket_attachments_entry ON support_ticket_attachments(org_id, entry_id);
