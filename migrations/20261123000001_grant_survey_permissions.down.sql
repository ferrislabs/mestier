UPDATE roles SET permissions = permissions & ~100663296::bigint WHERE name = 'admin';
