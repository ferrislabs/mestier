CREATE TABLE news (
    id           UUID        PRIMARY KEY,
    title        TEXT        NOT NULL,
    image_key    TEXT        NULL,
    items        JSONB       NOT NULL,
    cta          JSONB       NULL,
    published_at TIMESTAMPTZ NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_news_title_not_blank CHECK (length(btrim(title)) > 0),
    CONSTRAINT chk_news_items_array CHECK (jsonb_typeof(items) = 'array')
);

CREATE INDEX idx_news_published_at ON news (published_at);

CREATE TABLE news_views (
    news_id UUID        NOT NULL REFERENCES news(id) ON DELETE CASCADE,
    user_id UUID        NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    seen_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (news_id, user_id)
);

CREATE INDEX idx_news_views_user ON news_views (user_id);
