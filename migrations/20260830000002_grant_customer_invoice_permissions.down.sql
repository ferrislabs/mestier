-- Clears exactly the bits the up migration added, not the whole column: an
-- admin or member role may have gained other bits since (via `role.manage`),
-- and a revert must not discard those.
UPDATE roles SET permissions = permissions & ~229376::bigint WHERE name = 'admin' AND is_seeded = true;
UPDATE roles SET permissions = permissions & ~98304::bigint WHERE name = 'member' AND is_seeded = true;
