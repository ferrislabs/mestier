ALTER TABLE knowledge_articles DROP CONSTRAINT chk_knowledge_articles_kind;
ALTER TABLE knowledge_articles DROP CONSTRAINT chk_knowledge_articles_shape;
ALTER TABLE knowledge_articles DROP CONSTRAINT chk_knowledge_articles_revision;

UPDATE knowledge_articles a
SET body = coalesce((
    SELECT string_agg(
               (SELECT coalesce(string_agg(inline ->> 'text', '' ORDER BY inline_n), '')
                FROM jsonb_array_elements(block -> 'content') WITH ORDINALITY AS i(inline, inline_n)),
               E'\n' ORDER BY block_n)
    FROM jsonb_array_elements(a.content -> 'blocks') WITH ORDINALITY AS b(block, block_n)
), '')
WHERE a.kind = 'document'
  AND NOT EXISTS (
      SELECT 1
      FROM jsonb_array_elements(a.content -> 'blocks') AS b(block)
      WHERE block ->> 'type' <> 'paragraph'
         OR jsonb_array_length(block -> 'children') > 0
         OR EXISTS (
             SELECT 1 FROM jsonb_array_elements(block -> 'content') AS i(inline)
             WHERE inline ->> 'type' <> 'text'
         )
  );

UPDATE knowledge_articles
SET body = title
WHERE kind = 'document' AND btrim(body) = '';

UPDATE knowledge_articles SET kind = 'note' WHERE kind = 'document';

ALTER TABLE knowledge_articles
    DROP COLUMN content,
    DROP COLUMN revision,
    DROP COLUMN lock_holder,
    DROP COLUMN lock_expires_at;

ALTER TABLE knowledge_articles ADD CONSTRAINT chk_knowledge_articles_kind
    CHECK (kind IN ('file', 'note'));

ALTER TABLE knowledge_articles ADD CONSTRAINT chk_knowledge_articles_shape CHECK (
    (kind = 'note' AND body IS NOT NULL AND length(btrim(body)) > 0
        AND file_key IS NULL AND mime_type IS NULL AND size_bytes IS NULL)
    OR
    (kind = 'file' AND file_key IS NOT NULL AND mime_type IS NOT NULL AND size_bytes > 0
        AND body IS NULL)
);
