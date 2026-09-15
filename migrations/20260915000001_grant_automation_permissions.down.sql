-- Clears exactly the bits the up migration added, not the whole column: an
-- admin role may have gained other bits since (via `role.manage`), and a
-- revert must not discard those.
UPDATE roles SET permissions = permissions & ~1572864::bigint WHERE name = 'admin';
