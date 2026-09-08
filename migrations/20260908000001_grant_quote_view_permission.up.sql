-- #396: backfill VIEW_QUOTES onto every existing organization's default
-- `admin` and `member` roles. `owner` needs no backfill: it is seeded (and
-- stays seeded) as Permissions::ALL, which already contains every bit.
-- Matched by `is_seeded`, not name alone, so a renamed custom role never
-- collides with the seeded one (#308).
--
-- Bit value must stay in lockstep with domain::role::default_admin_business_permissions /
-- default_member_business_permissions:
--   VIEW_QUOTES (262144), granted to both admin and member.
UPDATE roles SET permissions = permissions | 262144::bigint WHERE name = 'admin' AND is_seeded = true;
UPDATE roles SET permissions = permissions | 262144::bigint WHERE name = 'member' AND is_seeded = true;
