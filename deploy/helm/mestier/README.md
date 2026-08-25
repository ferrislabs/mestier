# mestier (Helm chart)

Packages the two images built by the root `Dockerfile` (targets `api` and
`webapp`) as a Deployment + Service each, with an optional Ingress or Gateway
API `HTTPRoute` in front.

This chart does **not** provision Postgres, Redis, RustFS or the observability
stack — those are external dependencies (see the root `docker-compose.yml`
for what a local stack looks like). Point `api.env.DATABASE_HOST`,
`api.env.RATE_LIMIT_REDIS_URL` and `api.env.FILE_STORAGE_ENDPOINT` at wherever
you run them in your cluster.

## Install

```bash
helm upgrade --install mestier deploy/helm/mestier \
  --set api.image.tag=<version> \
  --set webapp.image.tag=<version> \
  --set api.secret.DATABASE_PASSWORD=<...> \
  --set api.secret.AUTH_CLIENT_SECRET=<...> \
  --set api.secret.FILE_STORAGE_ACCESS_KEY_ID=<...> \
  --set api.secret.FILE_STORAGE_SECRET_ACCESS_KEY=<...>
```

Never commit real secrets into a values file. Either pass them with `--set`
from a secret manager at deploy time, or set `api.secret.existingSecret` to
the name of a Secret you create out-of-band with the same keys as
`templates/api/secret.yaml`.

## Exposing the app: Ingress vs. Gateway API

The two are independent toggles — enable whichever your cluster runs, or
both, or neither (then reach the Services in-cluster only).

### Ingress

```yaml
ingress:
  enabled: true
  className: nginx
  webapp:
    host: app.mestier.example.com
  api:
    host: api.mestier.example.com
  tls:
    - secretName: mestier-tls
      hosts: [app.mestier.example.com, api.mestier.example.com]
```

### Gateway API

Requires a Gateway (and its GatewayClass) already provisioned in the
cluster — this chart only creates the `HTTPRoute`s that attach to it via
`gatewayApi.parentRefs`.

```yaml
gatewayApi:
  enabled: true
  parentRefs:
    - name: my-gateway
      namespace: gateway-system
  webapp:
    hostnames: [app.mestier.example.com]
  api:
    hostnames: [api.mestier.example.com]
```

## Key values

See `values.yaml` for the full, commented list. Highlights:

| Key | Purpose |
|---|---|
| `api.env` | Non-sensitive API config (mirrors `libs/args`), rendered into a ConfigMap |
| `api.secret` | Sensitive API config (DB password, auth secret, storage keys), rendered into a Secret unless `existingSecret` is set |
| `api.service.internalPort` | Health (`/health`) and metrics port, probed by the Deployment, not exposed via ingress/gateway |
| `webapp.env` | `API_URL` / `ISSUER_URL`, injected at container start into `config.json` (see `apps/webapp/docker-entrypoint.sh`) |
| `api.autoscaling` / `webapp.autoscaling` | Optional HPA per component |

TLS termination inside the API pod (`SERVER_TLS_CERT`/`SERVER_TLS_KEY`) is
out of scope for this chart — terminate TLS at the Ingress or Gateway.

## Verify

```bash
helm lint deploy/helm/mestier
helm template deploy/helm/mestier
helm template deploy/helm/mestier --set ingress.enabled=true
helm template deploy/helm/mestier --set gatewayApi.enabled=true
```
