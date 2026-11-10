UPDATE roles SET permissions = permissions & ~25165824::bigint WHERE name = 'admin';
