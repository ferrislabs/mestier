"""End-to-end check of the MCP chain against a running local stack.

FerrisKey with MCP support (CIMD, DCR, RFC 8414, RFC 8707, RFC 9207), Cerbos,
the gateway, the MCP server and the Mestier API. See docs/chantiers/mcp.md,
W6, for the realm setup. Every endpoint can be overridden by environment.

    MCP_E2E_DATABASE_URL=... FERRISKEY_ENV_FILE=.../ferriskey/.env python3 apps/mcp-gateway/e2e/local_stack.py
"""
import base64, hashlib, http.cookiejar, json, os, secrets, subprocess, sys, urllib.parse, urllib.request, urllib.error, uuid

FK = os.environ.get("MCP_E2E_FERRISKEY", "http://localhost:3333")
REALM = FK + "/realms/" + os.environ.get("MCP_E2E_REALM", "mestier")
GW = os.environ.get("MCP_E2E_GATEWAY", "http://localhost:8090/mcp")
API = os.environ.get("MCP_E2E_API", "http://localhost:3456")
DB = os.environ["MCP_E2E_DATABASE_URL"]
REDIRECT = "http://127.0.0.1:9999/cb"
results = []


def check(name, ok, detail=""):
    results.append((name, ok))
    print(("PASS " if ok else "FAIL ") + name + (f"  [{detail}]" if detail and not ok else ""))


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *a, **k):
        return None


def request(method, url, body=None, headers=None, form=False, opener=None):
    data = None
    headers = dict(headers or {})
    if body is not None:
        if form:
            data = urllib.parse.urlencode(body).encode()
            headers["Content-Type"] = "application/x-www-form-urlencoded"
        else:
            data = json.dumps(body).encode()
            headers["Content-Type"] = "application/json"
    req = urllib.request.Request(url, data=data, method=method, headers=headers)
    try:
        resp = (opener or urllib.request.build_opener(NoRedirect)).open(req, timeout=15)
        status, hdrs, raw = resp.status, resp.headers, resp.read()
    except urllib.error.HTTPError as e:
        status, hdrs, raw = e.code, e.headers, e.read()
    try:
        parsed = json.loads(raw) if raw else None
    except ValueError:
        parsed = raw.decode(errors="replace")
    return status, hdrs, parsed


def claims(token):
    payload = token.split(".")[1]
    return json.loads(base64.urlsafe_b64decode(payload + "=" * (-len(payload) % 4)))


def sql(statement):
    out = subprocess.run(["psql", DB, "-tAc", statement], capture_output=True, text=True)
    if out.returncode:
        raise SystemExit("sql failed: " + out.stderr)
    return out.stdout.strip()


# --- admin: a test user ------------------------------------------------------
env = dict(l.split("=", 1) for l in open(os.environ["FERRISKEY_ENV_FILE"]).read().splitlines() if "=" in l and not l.startswith("#"))
_, _, tok = request("POST", FK + "/realms/master/protocol/openid-connect/token",
                    {"grant_type": "password", "client_id": "admin-cli",
                     "username": env["ADMIN_USERNAME"], "password": env["ADMIN_PASSWORD"]}, form=True)
ADMIN = {"Authorization": "Bearer " + tok["access_token"]}
username = "mcp-e2e-" + secrets.token_hex(3)
password = "E2e-" + secrets.token_urlsafe(12)
st, _, user = request("POST", REALM + "/users", {"username": username, "email": username + "@example.com",
                       "firstname": "MCP", "lastname": "E2E", "email_verified": True}, ADMIN)
user = user.get("data", user) if isinstance(user, dict) else user
st2, _, _ = request("PUT", f"{REALM}/users/{user['id']}/reset-password",
                    {"credential_type": "password", "temporary": False, "value": password}, ADMIN)
check("test user created", st in (200, 201) and st2 in (200, 204), f"{st} {st2}")

# --- @spec-mcp-1 -------------------------------------------------------------
st, hdrs, _ = request("POST", GW, {"jsonrpc": "2.0", "id": 1, "method": "tools/list"})
www = hdrs.get("WWW-Authenticate", "")
check("@spec-mcp-1 401 points to resource metadata", st == 401 and "resource_metadata=" in www, f"{st} {www}")
_, _, prm = request("GET", urllib.parse.urljoin(GW, "/.well-known/oauth-protected-resource"))
check("@spec-mcp-1 metadata names the realm and the resource",
      prm["authorization_servers"] == [REALM] and prm["resource"] == GW, str(prm))


# --- @spec-mcp-2 DCR + code + PKCE + resource + iss --------------------------
def sign_in(scope):
    st, _, reg = request("POST", REALM + "/clients/register", {
        "client_name": "MCP e2e", "redirect_uris": [REDIRECT], "token_endpoint_auth_method": "none",
        "grant_types": ["authorization_code", "refresh_token"], "response_types": ["code"], "scope": scope})
    if st != 201:
        raise SystemExit(f"DCR failed {st} {reg}")
    client_id = reg["client_id"]
    verifier = secrets.token_urlsafe(48)
    challenge = base64.urlsafe_b64encode(hashlib.sha256(verifier.encode()).digest()).rstrip(b"=").decode()
    jar = http.cookiejar.CookieJar()
    opener = urllib.request.build_opener(NoRedirect, urllib.request.HTTPCookieProcessor(jar))
    q = urllib.parse.urlencode({"response_type": "code", "client_id": client_id, "redirect_uri": REDIRECT,
                                "scope": scope, "state": "st-1", "code_challenge": challenge,
                                "code_challenge_method": "S256", "resource": GW})
    st, _, _ = request("GET", f"{REALM}/protocol/openid-connect/auth?{q}", opener=opener)
    st, _, auth = request("POST", f"{REALM}/login-actions/authenticate?client_id={urllib.parse.quote(client_id)}",
                          {"username": username, "password": password}, opener=opener)
    url = auth["url"]
    params = dict(urllib.parse.parse_qsl(urllib.parse.urlsplit(url).query))
    if "consent_token" in params:
        st, _, decision = request("POST", f"{REALM}/auth/consent",
                                  {"consent_token": params["consent_token"], "approved_scopes": []}, opener=opener)
        url = decision["redirect_url"]
        params = dict(urllib.parse.parse_qsl(urllib.parse.urlsplit(url).query))
    st, _, token = request("POST", f"{REALM}/protocol/openid-connect/token", {
        "grant_type": "authorization_code", "client_id": client_id, "code": params.get("code", ""),
        "redirect_uri": REDIRECT, "code_verifier": verifier, "resource": GW}, form=True)
    return client_id, url, params, st, token


client_id, url, params, st, token = sign_in("openid mestier.read mestier.doe.write mestier.maintenance.write")
check("@spec-mcp-2 DCR client registered and redirected to the client", url.startswith(REDIRECT), url[:120])
check("@spec-mcp-2 redirect carries iss", params.get("iss") == REALM, str(params.get("iss")))
check("@spec-mcp-2 code exchanged with PKCE", st == 200 and "access_token" in token, f"{st} {token}")
access = token["access_token"]
c = claims(access)
aud = c["aud"] if isinstance(c["aud"], list) else [c["aud"]]
check("@spec-mcp-2 token audience is the gateway resource", GW in aud, str(c.get("aud")))
check("token carries the Mestier scopes",
      all(scope in c.get("scope", "").split() for scope in ("mestier.read", "mestier.doe.write", "mestier.maintenance.write")),
      c.get("scope"))

# --- @spec-mcp-3 -------------------------------------------------------------
st, _, _ = request("GET", API + "/api/v1/users/@me/organizations", headers={"Authorization": "Bearer " + access})
check("@spec-mcp-3 the API refuses the client token", st == 401, str(st))

# --- MCP calls through the gateway --------------------------------------------
MCP_HEADERS = {"Authorization": "Bearer " + access, "Accept": "application/json, text/event-stream",
               "MCP-Protocol-Version": "2025-11-25"}


def mcp(method, params=None, headers=MCP_HEADERS):
    st, _, body = request("POST", GW, {"jsonrpc": "2.0", "id": secrets.randbelow(10**6), "method": method,
                                       "params": params or {}}, headers)
    if isinstance(body, str) and body.startswith("event:"):
        body = json.loads([l[5:] for l in body.splitlines() if l.startswith("data:")][-1])
    return st, body


def tool(name, args=None, headers=MCP_HEADERS):
    return mcp("tools/call", {"name": name, "arguments": args or {}}, headers)


st, init = mcp("initialize", {"protocolVersion": "2025-11-25", "capabilities": {},
                              "clientInfo": {"name": "w6", "version": "0"}})
check("initialize through the gateway", st == 200 and "result" in (init or {}), f"{st} {init}")
st, tl = mcp("tools/list")
names = sorted(t["name"] for t in tl.get("result", {}).get("tools", [])) if isinstance(tl, dict) else []
check("tools/list returns the 29 tools",
      len(names) == 29 and "add_doe_file" in names and "create_maintenance_contract_draft" in names, f"{st} {len(names)}")

# First delegated call creates the Mestier user (the API registers users on first sight).
st, r = tool("list_organizations")
check("@spec-mcp-4 list_organizations runs end to end", st == 200 and "result" in r, f"{st} {r}")
user_id = sql(f"SELECT id FROM users WHERE sub = '{c['sub']}'")
check("the API saw the user's own identity (sub)", bool(user_id), c["sub"])

open_org, closed_org = str(uuid.uuid4()), str(uuid.uuid4())
for org, name, enabled in [(open_org, "MCP E2E ouverte", "true"), (closed_org, "MCP E2E fermée", "false")]:
    sql(f"INSERT INTO organizations (id, name, slug, owner_id, mcp_enabled) VALUES ('{org}', '{name}', 'e2e-{org[:8]}', '{user_id}', {enabled})")
    sql(f"INSERT INTO organization_members (id, organization_id, user_id, last_name) VALUES ('{uuid.uuid4()}', '{org}', '{user_id}', 'E2E')")
sql(f"INSERT INTO projects (id, org_id, name) VALUES ('{uuid.uuid4()}', '{open_org}', 'Avenue du 8 mai')")
sql(f"INSERT INTO projects (id, org_id, name) VALUES ('{uuid.uuid4()}', '{closed_org}', 'Projet caché')")

st, r = tool("list_organizations")
text = json.dumps(r)
check("@spec-mcp-12 list_organizations shows the open org only (act claim honoured)",
      f"e2e-{open_org[:8]}" in text and f"e2e-{closed_org[:8]}" not in text, text[:300])
st, r = tool("list_projects", {"organization": f"e2e-{open_org[:8]}"})
check("@spec-mcp-4 list_projects with the user's identity", "Avenue du 8 mai" in json.dumps(r, ensure_ascii=False), json.dumps(r)[:300])
st, r = tool("list_projects", {"organization": f"e2e-{closed_org[:8]}"})
check("@spec-mcp-5/12 closed org gives an error and no data",
      r.get("result", {}).get("isError") and "Projet caché" not in json.dumps(r, ensure_ascii=False), json.dumps(r, ensure_ascii=False)[:300])

st, r = tool("delete_project", {"organization": f"e2e-{open_org[:8]}"})
check("@spec-mcp-8 unknown tool refused", (r or {}).get("error", {}).get("code") == -32002, f"{st} {r}")

st, r = tool("create_doe", {"organization": f"e2e-{open_org[:8]}", "project_id": str(uuid.uuid4())})
body = json.dumps(r, ensure_ascii=False)
check("@spec-mcp-9 write allowed by policy, refused by the API (no planning.manage)",
      r.get("result", {}).get("isError") is True, body[:300])

# --- @spec-mcp-6 read-only token cannot write ---------------------------------
_, _, _, st, ro = sign_in("openid mestier.read")
ro_headers = dict(MCP_HEADERS, Authorization="Bearer " + ro["access_token"])
st, r = tool("create_doe", {"organization": f"e2e-{open_org[:8]}", "project_id": str(uuid.uuid4())}, ro_headers)
check("@spec-mcp-6 read-only token: write denied by policy", (r or {}).get("error", {}).get("code") == -32001, f"{st} {r}")

# --- Entretien: the maintenance write scope -----------------------------------
# The draft is created for a customer and a site that do not exist: the point is
# which layer answers. Scope missing: the policy (-32001). Scope held: the
# policy lets it through and the API answers (a tool error, not a policy error).
draft = {"organization": f"e2e-{open_org[:8]}", "customer_id": str(uuid.uuid4()), "customer_context_id": str(uuid.uuid4()),
         "season": {"year": 2026, "first_week": 12, "last_week": 48},
         "lines": [{"label": "Tonte", "cadence": {"kind": "ROLLING", "every_days": 14}}]}
st, r = tool("create_maintenance_contract_draft", draft, ro_headers)
check("maintenance: a token without mestier.maintenance.write is denied by policy",
      (r or {}).get("error", {}).get("code") == -32001, f"{st} {r}")
st, r = tool("create_maintenance_contract_draft", draft)
check("maintenance: with the scope the policy lets the draft through (the API judges)",
      "error" not in (r or {}) and r.get("result", {}).get("isError") is True, json.dumps(r, ensure_ascii=False)[:300])
st, r = tool("list_maintenance_due", {"organization": f"e2e-{open_org[:8]}", "week": "2026-W43"}, ro_headers)
check("maintenance: reading the recap needs only mestier.read",
      "error" not in (r or {}), f"{st} {r}")

# --- cleanup Mestier rows ------------------------------------------------------
for org in (open_org, closed_org):
    sql(f"DELETE FROM projects WHERE org_id = '{org}'")
    sql(f"DELETE FROM organization_members WHERE organization_id = '{org}'")
    sql(f"DELETE FROM organizations WHERE id = '{org}'")

failed = [n for n, ok in results if not ok]
print(f"\n{len(results) - len(failed)}/{len(results)} passed")
sys.exit(1 if failed else 0)
