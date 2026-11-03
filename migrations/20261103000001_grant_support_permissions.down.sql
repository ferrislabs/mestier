UPDATE roles SET permissions = permissions & ~6291456::bigint WHERE name = 'admin';
