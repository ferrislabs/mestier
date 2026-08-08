-- Destructive: this schema has no way to represent a free or a
-- soft-deleted seat, so reverting throws that information away rather
-- than hiding the loss.
--
-- Every member row with `user_id IS NULL` (a free seat) or a non-null
-- `deleted_at` (a former member) is deleted here, taking its
-- `member_roles` with it (`ON DELETE CASCADE`). Neither concept exists
-- pre-#180, where a member always wraps a user and is never soft-deleted
-- — resurrecting a soft-deleted row instead of dropping it would risk
-- colliding with an active row on `(organization_id, user_id)` once
-- `deleted_at` no longer distinguishes them, right below. Do not run this
-- against a database whose data matters.

DELETE FROM organization_members WHERE user_id IS NULL OR deleted_at IS NOT NULL;

ALTER TABLE organization_members
    DROP CONSTRAINT uq_members_id_organization_id;

DROP INDEX IF EXISTS idx_members_deleted_at;

DROP INDEX IF EXISTS uq_members_org_user_active;

ALTER TABLE organization_members
    ADD CONSTRAINT uq_members_org_user UNIQUE (organization_id, user_id);

ALTER TABLE organization_members
    DROP CONSTRAINT chk_members_last_name_not_blank,
    DROP CONSTRAINT chk_members_first_name_not_blank_when_present;

ALTER TABLE organization_members
    DROP COLUMN last_name,
    DROP COLUMN first_name,
    DROP COLUMN created_at,
    DROP COLUMN deleted_at;

ALTER TABLE organization_members
    ALTER COLUMN user_id SET NOT NULL,
    ALTER COLUMN joined_at SET NOT NULL,
    ALTER COLUMN joined_at SET DEFAULT now();
