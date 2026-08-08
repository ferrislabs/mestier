-- Turns `organization_members` into a standalone seat: `user_id` becomes
-- optional (occupied vs free), the seat gains its own name
-- (`last_name`/`first_name` — see `format_person_name`), and it becomes
-- soft-deletable so a departure no longer destroys anything a future
-- attachment (roles today, HR history from #182 on) needs to survive.
--
-- `joined_at` drops its `NOT NULL DEFAULT now()`: a seat that has never
-- been occupied has no join date to lie about.

ALTER TABLE organization_members
    ALTER COLUMN user_id DROP NOT NULL,
    ALTER COLUMN joined_at DROP NOT NULL,
    ALTER COLUMN joined_at DROP DEFAULT;

ALTER TABLE organization_members
    ADD COLUMN last_name TEXT,
    ADD COLUMN first_name TEXT,
    ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ADD COLUMN deleted_at TIMESTAMPTZ NULL;

-- Backfill: no data in production yet (see issue #180), still name every
-- existing row from its user so `last_name SET NOT NULL` below never
-- trips. `display_name` is `users`' `NOT NULL` column (the Rust field is
-- `name` — see `libs/core/src/infrastructure/user/postgres/model.rs`);
-- `username`/`email` are belt-and-suspenders fallbacks that should never
-- actually fire. `created_at` is backfilled from `joined_at`, which is
-- `NOT NULL` on every pre-existing row (only rows inserted after this
-- migration can have it unset).
UPDATE organization_members m
    SET last_name = COALESCE(u.display_name, u.username, u.email),
        created_at = m.joined_at
    FROM users u
    WHERE u.id = m.user_id;

ALTER TABLE organization_members
    ALTER COLUMN last_name SET NOT NULL;

ALTER TABLE organization_members
    ADD CONSTRAINT chk_members_last_name_not_blank
        CHECK (length(btrim(last_name)) > 0),
    ADD CONSTRAINT chk_members_first_name_not_blank_when_present
        CHECK (first_name IS NULL OR length(btrim(first_name)) > 0);

-- A free seat (`user_id IS NULL`) never conflicts with anything, and a
-- soft-deleted one no longer holds its old `(organization_id, user_id)`
-- pair — mirrors `uq_employees_org_user_active`.
ALTER TABLE organization_members
    DROP CONSTRAINT uq_members_org_user;

CREATE UNIQUE INDEX uq_members_org_user_active
    ON organization_members(organization_id, user_id)
    WHERE user_id IS NOT NULL AND deleted_at IS NULL;

-- Target of the composite FK `employees(member_id, organization_id)` that
-- #182 will add — redundant with the primary key alone, but a foreign key
-- must reference a unique constraint on exactly the columns it composes.
ALTER TABLE organization_members
    ADD CONSTRAINT uq_members_id_organization_id UNIQUE (id, organization_id);

CREATE INDEX idx_members_deleted_at ON organization_members(deleted_at);
