-- 1572864 = VIEW_AUTOMATION (524288) | MANAGE_AUTOMATION (1048576), and must
-- stay in lockstep with domain::role::default_admin_business_permissions.
-- `member` is absent on purpose, and `owner` needs no backfill: it is seeded
-- as Permissions::ALL.
UPDATE roles SET permissions = permissions | 1572864::bigint WHERE name = 'admin';
