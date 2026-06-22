-- migrations/20260620000001_create_discord_tables.up.sql

-- ── Enum types ────────────────────────────────────────────────────────────────

CREATE TYPE channel_type AS ENUM ('TEXT', 'THREAD');
CREATE TYPE author_type  AS ENUM ('USER', 'WEBHOOK', 'SYSTEM');
CREATE TYPE presence_status AS ENUM ('ONLINE', 'OFFLINE', 'DND');

-- ── categories ────────────────────────────────────────────────────────────────

CREATE TABLE categories (
    id              UUID        PRIMARY KEY,
    org_id          UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            TEXT        NOT NULL,
    position        INTEGER     NOT NULL DEFAULT 0,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_categories_name_not_blank
        CHECK (length(btrim(name)) > 0)
);

CREATE INDEX idx_categories_org_id   ON categories(org_id);
CREATE INDEX idx_categories_position ON categories(org_id, position);

-- ── channels ──────────────────────────────────────────────────────────────────

CREATE TABLE channels (
    id                  UUID         PRIMARY KEY,
    org_id              UUID         NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    channel_type        channel_type NOT NULL,
    name                TEXT         NOT NULL,
    topic               TEXT         NULL,
    position            INTEGER      NOT NULL DEFAULT 0,
    category_id         UUID         NULL REFERENCES categories(id) ON DELETE SET NULL,
    parent_id           UUID         NULL REFERENCES channels(id)   ON DELETE CASCADE,
    origin_message_id   UUID         NULL,  -- FK added after messages table; see CHECK below
    archived            BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at          TIMESTAMPTZ  NOT NULL DEFAULT now(),

    CONSTRAINT chk_channels_name_not_blank
        CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_channels_topic_not_blank_when_present
        CHECK (topic IS NULL OR length(btrim(topic)) > 0),
    CONSTRAINT chk_channels_thread_requires_parent
        CHECK (channel_type <> 'THREAD' OR parent_id IS NOT NULL),
    CONSTRAINT chk_channels_text_no_parent
        CHECK (channel_type <> 'TEXT' OR parent_id IS NULL),
    CONSTRAINT chk_channels_category_text_only
        CHECK (category_id IS NULL OR channel_type = 'TEXT')
);

CREATE INDEX idx_channels_org_id      ON channels(org_id);
CREATE INDEX idx_channels_category_id ON channels(category_id);
CREATE INDEX idx_channels_parent_id   ON channels(parent_id);
CREATE INDEX idx_channels_type        ON channels(org_id, channel_type);

-- ── messages ──────────────────────────────────────────────────────────────────

CREATE TABLE messages (
    id                  UUID        PRIMARY KEY,
    org_id              UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    channel_id          UUID        NOT NULL REFERENCES channels(id)      ON DELETE CASCADE,
    author_type         author_type NOT NULL,
    author_user_id      UUID        NULL REFERENCES users(id)             ON DELETE SET NULL,
    author_webhook_id   UUID        NULL,  -- FK added after webhooks table; see below
    content             TEXT        NOT NULL DEFAULT '',
    components          JSONB       NULL,
    mention_user_ids    UUID[]      NOT NULL DEFAULT '{}',
    mention_role_ids    UUID[]      NOT NULL DEFAULT '{}',
    mention_channel_ids UUID[]      NOT NULL DEFAULT '{}',
    mention_everyone    BOOLEAN     NOT NULL DEFAULT FALSE,
    edited_at           TIMESTAMPTZ NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_messages_user_author
        CHECK (author_type <> 'USER'    OR (author_user_id IS NOT NULL AND author_webhook_id IS NULL AND components IS NULL)),
    CONSTRAINT chk_messages_webhook_author
        CHECK (author_type <> 'WEBHOOK' OR (author_webhook_id IS NOT NULL AND author_user_id IS NULL)),
    CONSTRAINT chk_messages_system_author
        CHECK (author_type <> 'SYSTEM'  OR (author_user_id IS NULL AND author_webhook_id IS NULL))
);

CREATE INDEX idx_messages_channel_id      ON messages(channel_id);
CREATE INDEX idx_messages_org_id          ON messages(org_id);
CREATE INDEX idx_messages_author_user_id  ON messages(author_user_id) WHERE author_user_id IS NOT NULL;
CREATE INDEX idx_messages_created_at      ON messages(channel_id, id);   -- uuid v7 → time-ordered cursor

-- ── webhooks ──────────────────────────────────────────────────────────────────

CREATE TABLE webhooks (
    id          UUID        PRIMARY KEY,
    org_id      UUID        NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    channel_id  UUID        NOT NULL REFERENCES channels(id)      ON DELETE CASCADE,
    name        TEXT        NOT NULL,
    avatar_url  TEXT        NULL,
    token       TEXT        NOT NULL,
    created_by  UUID        NOT NULL REFERENCES users(id)         ON DELETE RESTRICT,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT chk_webhooks_name_not_blank  CHECK (length(btrim(name)) > 0),
    CONSTRAINT chk_webhooks_token_not_blank CHECK (length(btrim(token)) > 0)
);

CREATE INDEX idx_webhooks_channel_id ON webhooks(channel_id);
CREATE INDEX idx_webhooks_org_id     ON webhooks(org_id);

-- Now that webhooks exists, add the FK from messages to webhooks.
ALTER TABLE messages
    ADD CONSTRAINT fk_messages_author_webhook
        FOREIGN KEY (author_webhook_id) REFERENCES webhooks(id) ON DELETE SET NULL;

-- Now that messages exists, add the FK from channels to messages for origin_message_id.
ALTER TABLE channels
    ADD CONSTRAINT fk_channels_origin_message
        FOREIGN KEY (origin_message_id) REFERENCES messages(id) ON DELETE SET NULL;

-- ── message_reactions ─────────────────────────────────────────────────────────

CREATE TABLE message_reactions (
    message_id  UUID        NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    emoji       TEXT        NOT NULL,
    user_id     UUID        NOT NULL REFERENCES users(id)    ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),

    CONSTRAINT pk_message_reactions PRIMARY KEY (message_id, emoji, user_id),
    CONSTRAINT chk_message_reactions_emoji_not_blank CHECK (length(btrim(emoji)) > 0)
);

CREATE INDEX idx_message_reactions_message_id ON message_reactions(message_id);

-- ── member_presence ───────────────────────────────────────────────────────────

CREATE TABLE member_presence (
    org_id      UUID            NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id     UUID            NOT NULL REFERENCES users(id)         ON DELETE CASCADE,
    status      presence_status NOT NULL DEFAULT 'ONLINE',
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT now(),

    CONSTRAINT pk_member_presence PRIMARY KEY (org_id, user_id)
);

CREATE INDEX idx_member_presence_org_id ON member_presence(org_id);
