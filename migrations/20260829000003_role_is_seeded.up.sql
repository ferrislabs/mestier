-- #283/#308: distinguish an organization's seeded owner/admin/member roles
-- from a custom one by more than their name. The backfill migration
-- (20260829000002) already flagged the gap this closes: "roles carries no
-- separate is-default flag, and a custom role a user renamed to admin or
-- member is indistinguishable from the seeded one either way." #308 adds
-- role deletion, and a name-only match would let an organization rename
-- `owner` away, then delete what is now an ordinary-looking role and lock
-- itself out. A seeded role's name is fixed from here on; its permissions
-- stay editable.
ALTER TABLE roles ADD COLUMN is_seeded BOOLEAN NOT NULL DEFAULT false;

UPDATE roles SET is_seeded = true WHERE name IN ('owner', 'admin', 'member');
