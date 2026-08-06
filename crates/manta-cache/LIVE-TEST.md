# manta-cache — live test / demo runbook

A three-terminal walkthrough that runs the whole site-resolution chain
locally against one real site: `manta` CLI → `manta-cache-server` →
`manta-server` → CSM. Use it to demo the "no `--site` needed" flow or
to re-verify the integration end-to-end after changes.

Last verified 2026-07-19 against the CSCS **prealps** test site.

**Prerequisites**

- VPN access to the site's CSM endpoints (both `manta-server`'s
  backend calls and the cache's refresh die without it).
- A working classic setup: `manta-server` configured for the site in
  `server.toml`, and a `cli.toml` whose `manta_server_url` reaches it.
- A Keycloak account with roles on at least one HSM group at the site.

Throughout, `<site>` is the site name (e.g. `prealps`) and the config
directory is `~/.config/manta/` on Linux,
`~/Library/Application Support/local.cscs.manta/` on macOS. The CLI's
token cache lives at `~/.cache/manta/<site>_auth` on Linux,
`~/Library/Caches/local.cscs.manta/<site>_auth` on macOS.

## One-time configuration

### 1. `cache-server.toml` (next to `cli.toml`)

```toml
log = "info"

[server]
listen_address = "127.0.0.1"
allow_http = true          # local test; port defaults to 8081 without TLS
api_token = "local-test"   # enables POST /api/v1/refresh; drop this line
                           # (and cache_api_token below) for the minimal
                           # variant — lookups then need no bearer, but
                           # the management endpoints answer 403

[sites.<site>]
manta_server_url = "http://localhost:8080"   # same value as cli.toml's manta_server_url
# Point straight at the CLI's cached JWT: the file holds the raw token,
# and the cache re-reads it on every refresh — so whenever the CLI
# re-authenticates, the cache follows automatically.
token_file = "<absolute path to the CLI token cache>/<site>_auth"
```

> ⚠️ **The `token_file` above is a demo shortcut, not a deployment.**
> Pointing it at your own token cache makes this walkthrough a one-liner,
> but it refreshes the cache as *you*: the index inherits your Keycloak
> roles, and it stops working when your token expires or your account
> changes. Production uses one Keycloak **service account per site** —
> see [Production credentials](README.md#production-credentials).
>
> The server does not forbid this (that is what keeps this demo usable),
> but it will say so: expect a `⚠ WARNING: refreshing as personal
> account '<you>'` line in the startup summary, and the same text under
> `credential.warnings` in `GET /api/v1/dump`. Seeing that warning here
> is correct; seeing it in production is a bug report.

### 2. `cli.toml` additions

```toml
cache_url       = "http://localhost:8081"
cache_api_token = "local-test"    # only when the cache sets api_token
```

**Important:** make sure `site` is *not* set in `cli.toml` (comment it
out) and don't pass `--site` — otherwise resolution never triggers,
which is exactly the "explicit site wins" behaviour under test in
scenario 4 below.

## Startup order (three terminals)

The order matters: the cache refreshes from `manta-server` at boot, so
`manta-server` must be up first — and the CLI token file must be fresh.

```bash
# Terminal 1 — manta-server, as usual (needs VPN)
cd crates/manta-server && cargo run -- --allow-http

# Terminal 2 — refresh the site token first (the cache reads that
# file), then start the cache
cargo run -p manta-cli -- get groups --site <site>   # re-auth → rewrites <site>_auth
cargo run -p manta-cache-server
```

Expect in terminal 2: the startup summary (the credential is shown as
`token_file …`, never the value), then
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
#    "site '<site>' resolved via manta-cache (group '<name>')"
#    followed by the normal command output
cargo run -p manta-cli -- get group-nodes <a-real-group>

# 2. Xname resolution (take an xname from step 1's output)
cargo run -p manta-cli -- get nodes <xname>

# 3. Unknown group — hard, specific error; no manta-server call
cargo run -p manta-cli -- get group-nodes doesnotexist

# 4. Explicit site wins — no resolution notice appears
cargo run -p manta-cli -- get group-nodes <a-real-group> --site <site>

# 5. Degradation — Ctrl+C the cache in terminal 2, re-run step 1:
#    expect "warning: site resolution degraded — cache … unreachable"
#    followed by today's "No site selected" error
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
would answer — so it tells you what the CLI *would have* resolved,
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
  no group/xname target to resolve from. Only commands in the
  resolver's table resolve (`power … group|nodes`, `get nodes`,
  `get group-nodes`, `get sessions`, `apply boot group|nodes`,
  `apply boot-parameters`, `apply kernel-parameters`, `console node`
  — see `manta-cli`'s `common/site_resolution.rs::extract_target`).
  Hostlist bracket expressions and NIDs also fall back to requiring
  `--site` (the cache indexes plain xnames only).
- **Token expiry.** When the JWT expires, the cache's next refresh
  logs a 401 for the site and it drops from the index. Fix without a
  restart: run any CLI command with `--site <site>` (re-auth rewrites
  the token file), then the `POST /refresh` curl from step 6.
- **Cache started too early?** If `manta-server` wasn't up yet, the
  cache comes up with an empty index and a startup warning. Same fix:
  `POST /refresh` once `manta-server` is ready.
