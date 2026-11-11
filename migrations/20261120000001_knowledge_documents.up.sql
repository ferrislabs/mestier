ALTER TABLE knowledge_articles
    ADD COLUMN content         JSONB       NULL,
    ADD COLUMN revision        INTEGER     NOT NULL DEFAULT 0,
    ADD COLUMN lock_holder     UUID        NULL REFERENCES users(id) ON DELETE SET NULL,
    ADD COLUMN lock_expires_at TIMESTAMPTZ NULL;

ALTER TABLE knowledge_articles DROP CONSTRAINT chk_knowledge_articles_kind;
ALTER TABLE knowledge_articles DROP CONSTRAINT chk_knowledge_articles_shape;

WITH lines AS (
    SELECT a.id, l.line, l.n,
           sum((char_length(l.line) - 1) / 20000 + 1) OVER (PARTITION BY a.id) > 5000 AS merged
    FROM knowledge_articles a
    CROSS JOIN LATERAL regexp_split_to_table(a.body, E'\r?\n') WITH ORDINALITY AS l(line, n)
    WHERE a.kind = 'note' AND btrim(l.line, E' \t\r') <> ''
), pieces AS (
    SELECT lines.id, lines.n, lines.merged, g AS ord,
           substr(lines.line, g * sz.size + 1, sz.size) AS text
    FROM lines
    CROSS JOIN LATERAL (SELECT CASE WHEN lines.merged THEN 10000 ELSE 20000 END AS size) sz
    CROSS JOIN LATERAL generate_series(0, (char_length(lines.line) - 1) / sz.size) AS g
), offsets AS (
    SELECT id, n, ord, text,
           CASE WHEN merged
                THEN (sum(char_length(text) + 1) OVER (PARTITION BY id ORDER BY n, ord) - char_length(text) - 1) / 10000
                ELSE n * 1000000 + ord
           END AS bucket
    FROM pieces
), grouped AS (
    SELECT id,
           min(n * 1000000 + ord) AS position,
           string_agg(text, E'\n' ORDER BY n, ord) AS text
    FROM offsets
    GROUP BY id, bucket
), numbered AS (
    SELECT id, text, row_number() OVER (PARTITION BY id ORDER BY position) AS k
    FROM grouped
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
