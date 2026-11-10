-- 25165824 = VIEW_KNOWLEDGE (8388608) | MANAGE_KNOWLEDGE (16777216), and must
-- stay in lockstep with domain::role::default_admin_business_permissions.
-- `member` is absent on purpose, and `owner` is seeded as Permissions::ALL.
UPDATE roles SET permissions = permissions | 25165824::bigint WHERE name = 'admin';
