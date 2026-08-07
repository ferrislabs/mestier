-- Task comments: the annotable half of the planning remodel's "a task
-- resembles a GitHub issue" framing — see the planning module design doc's
-- `task_comments` section and its "Décision" on the author.
--
-- The author is a `user`, not an `employee`: it's the person writing, and an
-- employee without an account cannot comment. This is the only place in the
-- planning module where identity matters more than worker role.
--
-- Deletion is logical (`deleted_at`), unlike `task_label_links`'/
-- `task_assignments`' physical deletion: a comment thread is a
-- conversation record, worth keeping even once retracted from view.

CREATE TABLE task_comments (
    id              UUID        PRIMARY KEY,
    org_id          UUID        NOT NULL REFERENCES organizations(id),
    task_id         UUID        NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    author_user_id  UUID        NOT NULL REFERENCES users(id),
    body            TEXT        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at      TIMESTAMPTZ NULL,

    CONSTRAINT chk_task_comments_body_not_blank
        CHECK (length(btrim(body)) > 0)
);

CREATE INDEX idx_task_comments_task_id_created_at ON task_comments(task_id, created_at);
