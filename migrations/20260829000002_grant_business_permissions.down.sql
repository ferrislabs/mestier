-- Clears exactly the bits the up migration added, not the whole column:
-- an admin or member role may have gained other bits since (via
-- `role.manage`), and a revert must not discard those.
UPDATE roles SET permissions = permissions & ~31104::bigint WHERE name = 'admin';
UPDATE roles SET permissions = permissions & ~384::bigint WHERE name = 'member';
