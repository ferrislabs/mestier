-- Clears exactly the two bits, not the column: an admin role may have gained
-- others since, and a revert must not discard those.
UPDATE roles SET permissions = permissions & ~1572864::bigint WHERE name = 'admin';
