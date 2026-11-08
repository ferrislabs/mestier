-- A category with rows here is private: only the roles and the members listed
-- can see the channels in it. A category with none is open to everyone, as
-- before, so "private" needs no column of its own.
CREATE TABLE chat.category_access (
    id          UUID        NOT NULL PRIMARY KEY,
    category_id UUID        NOT NULL REFERENCES chat.categories(id) ON DELETE CASCADE,
    org_id      UUID        NOT NULL REFERENCES organizations(id)   ON DELETE CASCADE,
    role_id     UUID        NULL REFERENCES roles(id)               ON DELETE CASCADE,
    user_id     UUID        NULL REFERENCES users(id)               ON DELETE CASCADE,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT chk_category_access_one_target
        CHECK ((role_id IS NOT NULL) <> (user_id IS NOT NULL))
);

CREATE UNIQUE INDEX uq_category_access_role ON chat.category_access(category_id, role_id)
    WHERE role_id IS NOT NULL;
CREATE UNIQUE INDEX uq_category_access_user ON chat.category_access(category_id, user_id)
    WHERE user_id IS NOT NULL;
CREATE INDEX idx_category_access_org ON chat.category_access(org_id);
