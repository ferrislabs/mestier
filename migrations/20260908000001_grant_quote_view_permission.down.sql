-- Clears exactly the bit the up migration added, not the whole column.
UPDATE roles SET permissions = permissions & ~262144::bigint WHERE name = 'admin' AND is_seeded = true;
UPDATE roles SET permissions = permissions & ~262144::bigint WHERE name = 'member' AND is_seeded = true;
