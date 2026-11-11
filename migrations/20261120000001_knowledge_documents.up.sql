ALTER TABLE knowledge_articles
    ADD COLUMN content         JSONB       NULL,
    ADD COLUMN revision        INTEGER     NOT NULL DEFAULT 0,
    ADD COLUMN lock_holder     UUID        NULL REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN lock_expires_at TIMESTAMPTZ NULL;

ALTER TABLE knowledge_articles DROP CONSTRAINT chk_knowledge_articles_kind;
ALTER TABLE knowledge_articles DROP CONSTRAINT chk_knowledge_articles_shape;

WITH lines AS (
    SELECT a.id, l.line, l.n
    FROM knowledge_articles a
    CROSS JOIN LATERAL regexp_split_to_table(a.body, E'\r?\n') WITH ORDINALITY AS l(line, n)
    WHERE a.kind = 'note' AND btrim(l.line, E' \t\r') <> ''
), pieces AS (
    SELECT lines.id, lines.n, g AS ord, substr(lines.line, g * 20000 + 1, 20000) AS text
    FROM lines
    CROSS JOIN LATERAL generate_series(0, (char_length(lines.line) - 1) / 20000) AS g
), numbered AS (
    SELECT id, text, row_number() OVER (PARTITION BY id ORDER BY n, ord) AS k
    FROM pieces
), built AS (
    SELECT id,
           jsonb_agg(
               jsonb_build_object(
                   'id', 'b' || k,
                   'type', 'paragraph',
                   'props', '{}'::jsonb,
                   'content', jsonb_build_array(
                       jsonb_build_object('type', 'text', 'text', text, 'styles', '{}'::jsonb)
                   ),
                   'children', '[]'::jsonb
               )
               ORDER BY k
           ) AS blocks,
           string_agg(text, E'\n\n' ORDER BY k) AS markdown
    FROM numbered
    GROUP BY id
)
UPDATE knowledge_articles a
SET kind = 'document',
    content = jsonb_build_object('version', 1, 'blocks', coalesce(built.blocks, '[]'::jsonb)),
    body = coalesce(built.markdown, '')
FROM (SELECT id FROM knowledge_articles WHERE kind = 'note') n
LEFT JOIN built ON built.id = n.id
WHERE a.id = n.id;

ALTER TABLE knowledge_articles ADD CONSTRAINT chk_knowledge_articles_kind
    CHECK (kind IN ('file', 'document'));

ALTER TABLE knowledge_articles ADD CONSTRAINT chk_knowledge_articles_revision
    CHECK (revision >= 0);

ALTER TABLE knowledge_articles ADD CONSTRAINT chk_knowledge_articles_shape CHECK (
    (kind = 'document' AND content IS NOT NULL AND jsonb_typeof(content) = 'object'
        AND body IS NOT NULL
        AND file_key IS NULL AND mime_type IS NULL AND size_bytes IS NULL)
    OR
    (kind = 'file' AND file_key IS NOT NULL AND mime_type IS NOT NULL AND size_bytes > 0
        AND body IS NULL AND content IS NULL
        AND lock_holder IS NULL AND lock_expires_at IS NULL)
);
