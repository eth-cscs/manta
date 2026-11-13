# manta-cache — live test / demo runbook

A three-terminal walkthrough that runs the whole site-resolution chain
locally against one real site: `manta` CLI → `manta-server` (which asks
`manta-cache-server` for the site) → CSM. Use it to demo the "no
`--site` needed" flow or to re-verify the integration end-to-end after
changes.

The earlier CLI-side flow was verified on 2026-07-19 against the CSCS
**prealps** test site. The server-side flow described here has not
been run live yet.

**Prerequisites**

- VPN access to the site's CSM endpoints (both `manta-server`'s
  backend calls and the cache's refresh die without it).
- A working classic setup: `manta-server` configured for the site in
  `server.toml`, and a `cli.toml` whose `manta_server_url` reaches it.
- A CSCS Keycloak account with roles on at least one HSM group at the
  site (and `[sites.<site>.token_exchange]` working in `server.toml`).
- A CSCS Keycloak **service account** (client id + secret) that the
  site's Keycloak accepts in the token exchange and maps to roles on
  at least one HSM group there. The cache refreshes as it.

Throughout, `<site>` is the site name (e.g. `prealps`) and the config
directory is `~/.config/manta/` on Linux,
`~/Library/Application Support/local.cscs.manta/` on macOS. The CLI's
token cache lives at `~/.cache/manta/token.json` on Linux,
`~/Library/Caches/local.cscs.manta/token.json` on macOS.

## One-time configuration

### 1. `cache-server.toml` (next to `cli.toml`)

```toml
log = "info"

[server]
listen_address = "127.0.0.1"
allow_http = true          # local test; port defaults to 8081 without TLS
api_token = "local-test"   # enables POST /api/v1/refresh; drop this line
                           # (and api_token_file below) for the minimal
                           # variant — lookups then need no bearer, but
                           # the management endpoints answer 403

[sites.<site>]
manta_server_url = "http://localhost:8080"   # same value as cli.toml's manta_server_url
```

No credential goes in this file: the cache takes the service account
from `MANTA_SA_CLIENT_ID` / `MANTA_SA_CLIENT_SECRET` (see "Startup
order") and gets a new token from the production CSCS realm for every
request. To test against another realm, add `[auth] issuer_url = "…"`.
See [Production credentials](README.md#production-credentials).

### 2. `server.toml` addition

```toml
[site_resolution]
url            = "http://localhost:8081"
api_token_file = "/tmp/manta-cache-api-token"   # holds "local-test"; only when the cache sets api_token
```

```bash
printf local-test > /tmp/manta-cache-api-token
```

**Important:** make sure `site` is *not* set in `cli.toml` (comment it
out) and don't pass `--site` — otherwise resolution never triggers,
which is exactly the "explicit site wins" behaviour under test in
scenario 4 below.

## Startup order (three terminals)

The order matters: the cache refreshes from `manta-server` at boot, so
`manta-server` must be up first. (`manta-server` itself does not
contact the cache until the first site-less request.)

```bash
# Terminal 1 — manta-server, as usual (needs VPN)
cd crates/manta-server && cargo run -- --allow-http

# Terminal 2 — the cache, as the service account
MANTA_SA_CLIENT_ID=<client id> MANTA_SA_CLIENT_SECRET=<secret> \
  cargo run -p manta-cache-server
```

Expect in terminal 2: the startup summary (the `[auth]` block shows the
service account's client id and the issuer, never the secret), then
`cross-site refresh finished sites=1 failures=0`, then
`server ready, accepting requests on http://127.0.0.1:8081`.

Sanity check:

```bash
curl -s -H "Authorization: Bearer local-test" http://127.0.0.1:8081/api/v1/sites
# → ["<site>"]
```

## Test walkthrough (terminal 3, no `--site` anywhere)

The numbering matches the [ROADMAP test
scenarios](ROADMAP.md#test-scenarios).

```bash
# 1. Group resolution — expect the stderr notice
#    "site '<site>' resolved by manta-server"
#    followed by the normal command output; manta-server logs
#    "site resolved via manta-cache"
cargo run -p manta-cli -- get group-nodes <a-real-group>

# 2. Xname resolution (take an xname from step 1's output)
cargo run -p manta-cli -- get nodes <xname>

# 3. Unknown group — HTTP 404 from manta-server: "The site-resolution
#    cache knows no group 'doesnotexist'"; no backend call
cargo run -p manta-cli -- get group-nodes doesnotexist

# 4. Explicit site wins — no resolution notice appears, and the cache
#    sees no lookup
cargo run -p manta-cli -- get group-nodes <a-real-group> --site <site>

# 5. Cache down — Ctrl+C the cache in terminal 2, re-run step 1:
#    expect HTTP 503 "The site could not be resolved: the cache is
#    unreachable (…). Pass --site <name>." — no fallback, no guess
cargo run -p manta-cli -- get group-nodes <a-real-group>

# 6. Management refresh (e.g. after changing group membership)
curl -s -X POST -H "Authorization: Bearer local-test" \
  http://127.0.0.1:8081/api/v1/refresh
# → {"sites":["<site>"],"failures":[]}

# 7. Inspect what is actually cached — the debugging dump
curl -s -H "Authorization: Bearer local-test" \
  http://127.0.0.1:8081/api/v1/dump | jq
```

`GET /dump` is the endpoint to reach for when a resolution surprises
you. It is management-gated (it is the only one serving group member
lists), and its `groups`/`xnames` are a bulk mirror of what the lookups
would answer — so it tells you what manta-server *would have* resolved,
without running a command. Useful slices:

```bash
DUMP='curl -s -H "Authorization: Bearer local-test" http://127.0.0.1:8081/api/v1/dump'

# Is the cache stale, and did every site refresh?
eval $DUMP | jq '.sites'          # age_seconds, last_error, in_index

# Why did this group resolve to that site?
eval $DUMP | jq '.groups["<a-real-group>"]'

# Anything contested across sites? (empty on a single-site setup)
eval $DUMP | jq '.conflicts'

# Diff two dumps: drop the fields that change on every call, or the
# diff is pure noise.
eval $DUMP | jq 'del(.generated_at, .sites[].age_seconds)' > /tmp/before
# … do something, refresh …
eval $DUMP | jq 'del(.generated_at, .sites[].age_seconds)' | diff /tmp/before -
```

(The cross-site split scenario needs a second configured site — see
the ROADMAP's second-fixture note.)

## Things to know while testing

- **`get groups` still needs `--site`.** A list-everything command has
  no group/xname target to resolve from. Only these commands resolve
  (`power … group|nodes`, `get nodes`, `get group-nodes`,
  `get sessions`, `apply boot group|nodes`, `apply boot-parameters`,
  `apply kernel-parameters`, `console node`); the endpoints behind them
  are listed in the root API.md § Site resolution. Hostlist bracket
  expressions and NIDs fail with 400 and need `--site` (the cache
  indexes plain xnames only).
- **A site fails with `no token for site …`.** The Keycloak refused
  the service account after startup (secret rotated, client disabled);
  nothing was sent to manta-server. Restart with the new credentials.
- **A site fails with `returned no groups`.** The token exchange worked
  but the site's Keycloak maps the service account to no HSM roles.
  Fix the role mapping at the site, then `POST /refresh`.
- **A site fails with HTTP 401 `token exchange … failed`.** manta-server
  could not exchange the service account's token at the site: the
  site's Keycloak does not accept that client in the exchange.
- **Cache started too early?** If `manta-server` wasn't up yet, the
  cache comes up with an empty index and a startup warning. Same fix:
  `POST /refresh` once `manta-server` is ready.
