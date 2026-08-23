## Ferris Labs Web App

### Regenerating the API client

```bash
pnpm gen:api
```

Reads the OpenAPI document from `http://localhost:3456/api-docs/openapi.json` by
default — override with `API_URL`. That server must be `apps/api` built from
**your own branch**, not a dev server left running from another checkout: a
stale server produces a client silently missing the routes you just added, with
no error.
