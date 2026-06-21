ALTER TABLE quotes
    ADD COLUMN reference TEXT NULL,
    ADD COLUMN title TEXT NULL;

WITH numbered_quotes AS (
    SELECT
        id,
        EXTRACT(YEAR FROM created_at)::INTEGER AS quote_year,
        ROW_NUMBER() OVER (
            PARTITION BY org_id, EXTRACT(YEAR FROM created_at)::INTEGER
            ORDER BY created_at ASC, id ASC
        ) AS quote_rank
    FROM quotes
)
UPDATE quotes q
SET
    reference = format(
        'DEV-%s-%s',
        numbered_quotes.quote_year,
        lpad(numbered_quotes.quote_rank::TEXT, 4, '0')
    ),
    title = 'Devis sans objet'
FROM numbered_quotes
WHERE q.id = numbered_quotes.id;

ALTER TABLE quotes
    ALTER COLUMN reference SET NOT NULL,
    ALTER COLUMN title SET NOT NULL,
    ADD CONSTRAINT chk_quotes_reference_not_blank CHECK (length(btrim(reference)) > 0),
    ADD CONSTRAINT chk_quotes_title_not_blank CHECK (length(btrim(title)) > 0),
    ADD CONSTRAINT uq_quotes_org_reference UNIQUE (org_id, reference);

CREATE INDEX idx_quotes_reference ON quotes(reference);
