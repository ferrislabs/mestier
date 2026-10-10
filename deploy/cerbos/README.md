# Cerbos, interim PDP of the MCP gateway

Pinned image: `ghcr.io/cerbos/cerbos:0.56.0` (see ADR 0005).

## Run

```bash
docker compose up -d cerbos
```

Published on `127.0.0.1:3592` only. Policies are watched: edits apply without restart.
The image's binary lives at `/cerbos`, so never mount a directory there.

## Test the policies

```bash
docker run --rm -v "$PWD/deploy/cerbos:/conf" ghcr.io/cerbos/cerbos:0.56.0 \
  compile --tests=/conf/policies/tests /conf/policies
```

## AuthZEN endpoint

AuthZEN 1.0 is served by default on the HTTP port, no flag needed.

- From the host: `POST http://127.0.0.1:3592/access/v1/evaluation`
- Inside the compose network: `POST http://cerbos:3592/access/v1/evaluation`

Response: `{"decision":true}` or `{"decision":false}`.

## Request mapping (observed on 0.56.0)

| AuthZEN | Cerbos |
|---|---|
| `subject.type` | principal role (the policy requires `user`) |
| `subject.id` | principal id |
| `subject.properties.*` | principal `attr` (`P.attr.scopes`, `P.attr.client_id`) |
| `action.name` | action |
| `resource.type` | resource kind (`mcp_tool`) |
| `resource.id` | resource id |
| `resource.properties.*` | resource `attr` (`R.attr.access`, `R.attr.domain`, `R.attr.organization`) |

`subject.properties.roles` is NOT read as roles; it is only an ordinary attribute. The role comes from `subject.type`, so the gateway must send `"type": "user"`.

## Policy v1

`policies/mcp_tool.yaml`: allow `access == read` with scope `mestier.read`; allow `access == write` and `domain == doe` with scope `mestier.doe.write`; everything else is denied by default. Mestier business roles are not modelled here.
