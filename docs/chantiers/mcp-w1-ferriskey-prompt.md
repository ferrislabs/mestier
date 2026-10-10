# Prompt for the FerrisKey agent: OAuth for MCP clients (workstream W1)

Copy everything below the line into the agent working in the FerrisKey repository.

---

## Mission

Make FerrisKey usable as the authorization server of an MCP server, so that a user only pastes the MCP server URL into ChatGPT (or any MCP client) and logs in with their FerrisKey account. Concretely: add OAuth Client ID Metadata Documents (CIMD), Dynamic Client Registration (RFC 7591), authorization server metadata (RFC 8414), the `iss` authorization response parameter (RFC 9207), and resource indicators (RFC 8707) in the authorization code flow.

## Context

- Repository: `/opt/ferrislabs/ferriskey/ferriskey`, branch from `main`. Read `CLAUDE.md`, `CONTRIBUTING.md` and `DEVELOPMENT.md` first and follow them (architecture, naming, tests, commit and PR rules). Work on a branch, open a PR, never merge.
- Consumer: Mestier (ERP, `/opt/ferrislabs/mestier`) is adding an MCP gateway (OAuth resource server) in front of an MCP server. Its spec is `/opt/ferrislabs/mestier/docs/chantiers/mcp.md` and its architecture is ADR `/opt/ferrislabs/mestier/docs/adr/0004-mcp-gateway-pep-token-exchange.md`. Read both; this workstream is "W1" there.
- The MCP authorization spec (2025-11-25 and 2026-07-28 revisions) requires: the MCP server publishes RFC 9728 protected resource metadata pointing at the authorization server; the client discovers the authorization server through RFC 8414 (or OpenID discovery); the client identifies itself with CIMD (preferred) or registers through DCR; authorization code + PKCE (S256); `resource` parameter (RFC 8707) so the access token audience is the MCP server; the authorization server returns `iss` in the authorization response (RFC 9207) and the client validates it.
- ChatGPT prefers CIMD when the authorization server metadata advertises it, otherwise uses DCR. Without either it cannot complete OAuth. With CIMD it authenticates at the token endpoint with `none` (public client + PKCE) or `private_key_jwt`.
- Already present in FerrisKey (verified on `main`): PKCE, OpenID discovery (`libs/ferriskey-api-authentication/src/handlers/openid_configuration.rs`), consent (`libs/ferriskey-consent`, `libs/ferriskey-api-consent`), token exchange RFC 8693 with `audience`, `resource`, `scope`, `actor_token` and per-client policies (`core/src/domain/authentication/token_exchange/`). Not found: CIMD, DCR. Not verified: RFC 8414 endpoint, `resource` in the authorization code flow, `iss` in the authorization response. Start by checking these three and report what exists.

## Requirements

1. **Authorization server metadata (RFC 8414)** at `/.well-known/oauth-authorization-server/realms/{realm}` (path-insertion form) and the same fields added to the realm's `openid-configuration`: `issuer`, `authorization_endpoint`, `token_endpoint`, `jwks_uri`, `registration_endpoint` (when DCR is enabled on the realm), `scopes_supported`, `response_types_supported: ["code"]`, `grant_types_supported` (including `authorization_code`, `refresh_token`, `urn:ietf:params:oauth:grant-type:token-exchange`), `code_challenge_methods_supported: ["S256"]`, `token_endpoint_auth_methods_supported` (at least `none`, `client_secret_basic`, `client_secret_post`; `private_key_jwt` if supported), `client_id_metadata_document_supported: true` (when CIMD is enabled), `authorization_response_iss_parameter_supported: true`.
2. **RFC 9207**: every authorization response redirect (success and error) carries `iss` equal to the realm issuer.
3. **RFC 8707 in the code flow**: accept `resource` on the authorization request and on the token request (authorization_code and refresh_token grants). The access token `aud` is the requested resource. Reject a resource the realm does not know with `invalid_target`. A realm keeps a list of allowed resource URIs (admin-managed). A refresh keeps the resource of the original grant unless a narrower one is requested.
4. **CIMD** (draft-ietf-oauth-client-id-metadata-document), realm-level toggle, off by default:
   - When `client_id` is an `https` URL, fetch the metadata document at authorization time.
   - Fetch safety: HTTPS only, no redirects followed, refuse private, loopback and link-local IPs after DNS resolution (SSRF), timeout about 5 s, size cap about 5 KB, JSON only.
   - Validate: the document's `client_id` equals the URL exactly; `redirect_uris` present and the request's `redirect_uri` matches one exactly; `token_endpoint_auth_method` is `none` (PKCE mandatory) or `private_key_jwt` (with `jwks_uri` or `jwks`). If you skip `private_key_jwt`, say so.
   - Cache documents honoring HTTP caching headers, capped at 24 h, with a short cache on failure.
   - Show `client_name` and the client URL host on the consent screen; a CIMD client always gets consent (never skipped).
   - Optional realm allowlist of client URL hosts.
5. **DCR (RFC 7591)**, realm-level toggle, off by default:
   - Anonymous `POST` on `registration_endpoint`, rate limited per IP.
   - Accept public clients (`token_endpoint_auth_method: none`, PKCE required) and confidential ones (`client_secret_basic` or `client_secret_post`).
   - `redirect_uris` must be `https`, or `http` on loopback only.
   - Registered clients are flagged as dynamic: limited to the realm's allowed scopes, no token exchange, no service account, always consent.
   - Admin can list and delete them. RFC 7592 management endpoints are out of scope unless trivial.
   - Respond per RFC 7591 §3.2 (`client_id`, `client_id_issued_at`, `client_secret` when confidential, echoed metadata).
6. **Admin UI and API**: realm toggles for CIMD and DCR, the allowed resource list, the CIMD host allowlist, and dynamic clients marked as such in the client list.

## Out of scope

- The MCP gateway, the MCP server and the PDP (they live in Mestier).
- Changing token exchange semantics (already used as is by the gateway).

## Acceptance

Each line becomes a test (follow the repository's test conventions; API tests live in `api/tests/`):

1. The RFC 8414 document lists the fields above; `registration_endpoint` and `client_id_metadata_document_supported` appear only when their toggle is on.
2. Authorization code + PKCE with a CIMD `client_id` served by a test HTTP server succeeds; the redirect carries `code`, `state` and `iss`; the token's `aud` is the requested `resource`.
3. CIMD refusals: document `client_id` mismatch, `redirect_uri` not listed, HTTP URL, private IP target, oversized document, timeout.
4. DCR: a public client registers, then completes code + PKCE; a registration with an `http` non-loopback redirect is refused; a dynamic client cannot use token exchange.
5. `resource` not in the realm's allowed list gives `invalid_target`; refresh keeps the original audience.
6. Error redirects carry `iss` too.
7. With both toggles off, behavior is unchanged (existing test suite green).

## Verification

Run the repository's own checks (fmt, clippy with warnings denied, the full test suite including API tests, front checks if the UI changes) and report the actual results.

## Report back (this is what the Mestier side needs)

- The PR URL and branch.
- The exact discovery URLs for a realm (RFC 8414 and OpenID), and a sample of each document.
- How to enable CIMD, DCR and allowed resources on a realm (admin API calls or UI steps), so the Mestier `mestier` realm can be configured.
- What was not implemented (`private_key_jwt`, RFC 7592…) and any deviation from this prompt.
