ALTER TABLE survey.response ADD CONSTRAINT uq_response_org_id UNIQUE (org_id, id);

CREATE TABLE survey.response_photo (
    org_id      UUID        NOT NULL,
    response_id UUID        NOT NULL,
    key         TEXT        NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (response_id, key),
    CONSTRAINT fk_response_photo_response
        FOREIGN KEY (org_id, response_id) REFERENCES survey.response (org_id, id)
        ON DELETE CASCADE,
    CONSTRAINT chk_response_photo_key CHECK (key <> '')
);

CREATE INDEX idx_response_abandoned
    ON survey.response (updated_at)
    WHERE status = 'IN_PROGRESS';
