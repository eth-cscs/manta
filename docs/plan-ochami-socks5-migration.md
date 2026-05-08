# Plan: ochami-rs SOCKS5 env-var → explicit parameter migration

## Context

csm-rs was fully migrated in a previous session: every http_client function
now receives `socks5_proxy: Option<&str>` as an explicit parameter. ochami-rs
was not touched. It still reads `std::env::var("SOCKS5")` inside every
function body — 93 reads across 15 files.

This plan mirrors what was done in csm-rs, applied to ochami-rs.

---

## Why this matters

- `std::env::var` is not safe to call from a multi-threaded process while any
  other thread may call `std::env::set_var` (Rust 1.66+).
- The global env-var prevents per-site proxy configuration in a multi-site
  server future.
- Explicit parameters make the dependency visible at the call site and
  testable.

---

## Relationship to the BackendClient constructor plan

`docs/plan-http-client-constructor.md` describes folding `(base_url,
root_cert, socks5_proxy)` into a single `BackendClient` struct. If that plan
is executed first, the socks5 parameter never needs to be threaded through as
a standalone step — the constructor absorbs it. If that plan has not yet been
done, execute this plan first; the BackendClient plan then applies cleanly to
ochami-rs in the same form it applies to csm-rs.

---

## The replacement

### Two env-var patterns present in ochami-rs

**Pattern A** — imperative `let client;` with `if`:

```rust
let client;
let client_builder = reqwest::Client::builder()...;
if std::env::var("SOCKS5").is_ok() {
    let socks5proxy = reqwest::Proxy::all(std::env::var("SOCKS5").unwrap())?;
    client = client_builder.proxy(socks5proxy).build()?;
} else {
    client = client_builder.build()?;
}
```

**Pattern B** — `let client = if let Ok(socks5_env) = ...`:

```rust
let client = if let Ok(socks5_env) = std::env::var("SOCKS5") {
    let socks5proxy = reqwest::Proxy::all(socks5_env)?;
    client_builder.proxy(socks5proxy).build()?
} else {
    client_builder.build()?
};
```

Both are replaced by the same expression (identical to csm-rs):

```rust
let client = match socks5_proxy {
    Some(proxy) => client_builder.proxy(reqwest::Proxy::all(proxy)?).build()?,
    None => client_builder.build()?,
};
```

---

## Parameter placement rule

`socks5_proxy: Option<&str>` is inserted **immediately after `root_cert:
&[u8]`** (or `shasta_root_cert: &[u8]` in PCS files), regardless of what
parameters come before or after root_cert.

```rust
// Before (bss pattern)
pub async fn get(
    base_url: &str,
    auth_token: &str,
    root_cert: &[u8],
    xnames_opt: &Option<Vec<String>>,
) -> ...

// After
pub async fn get(
    base_url: &str,
    auth_token: &str,
    root_cert: &[u8],
    socks5_proxy: Option<&str>,
    xnames_opt: &Option<Vec<String>>,
) -> ...
```

```rust
// Before (pcs pattern — no token)
pub async fn get(
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
) -> ...

// After
pub async fn get(
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,
) -> ...
```

**Note**: ochami-rs has pre-existing parameter order inconsistencies across
files (some functions have `auth_token` before `base_url`, or `root_cert`
before `auth_token`). Do NOT fix these as part of this migration — it is
out of scope, increases noise, and risks breaking unrelated call sites.
The rule "socks5_proxy goes after root_cert" is applied mechanically.

---

## Wrapper functions

Several files have thin delegating functions. They must accept `socks5_proxy`
and pass it through:

```rust
// Before
pub async fn get_all(
    base_url: &str,
    auth_token: &str,
    root_cert: &[u8],
) -> Result<Vec<Group>, Error> {
    get(base_url, auth_token, root_cert, None, None).await
}

// After
pub async fn get_all(
    base_url: &str,
    auth_token: &str,
    root_cert: &[u8],
    socks5_proxy: Option<&str>,
) -> Result<Vec<Group>, Error> {
    get(base_url, auth_token, root_cert, socks5_proxy, None, None).await
}
```

---

## Files and scope

### 15 http_client files (env-var reads to remove)

| File | Functions | Env-var reads |
|------|-----------|---------------|
| `bss/http_client.rs` | 5 (get_all, get, post, put, patch, delete) | 6 |
| `pcs/power_cap/http_client.rs` | 4 (get, get_task_id, post_snapshot, patch) | 6 |
| `pcs/power_status/http_client.rs` | 1 (post) | 2 |
| `pcs/transitions/http_client.rs` | 5 (get, get_by_id, post, post_block, wait_to_complete) | 5 |
| `hsm/group/http_client.rs` | 9 (get_all, get, get_one, get_labels, get_members, post, post_member, delete_one, delete_member) | 8 |
| `hsm/component/http_client.rs` | 10 (get_all, get_all_nodes, get, get_one, post, post_query, post_bynid_query, put, delete_one, delete) | 8 |
| `hsm/state/components/http_client.rs` | 10 | 10 |
| `hsm/defaults/node_map/http_client.rs` | 6 (get, get_one, post, put, delete_all, +1) | 6 |
| `hsm/node_map/http_client.rs` | 7 | 6 |
| `hsm/inventory/ethernet_interfaces/http_client.rs` | 9 | 9 |
| `hsm/inventory/hardware/http_client.rs` | 6 | 6 |
| `hsm/inventory/hardware_by_fru/http_client.rs` | 4 | 4 |
| `hsm/inventory/redfish_endpoint/http_client.rs` | 8 | 7 |
| `hsm/memberships/http_client.rs` | 2 | 2 |
| `hsm/partition/http_client.rs` | 6 | 8 |
| **Total** | **~95 functions** | **93 reads** |

### `hsm/group/utils.rs` — 7 call sites, 5 functions

This file calls `http_client::*` functions and has its own public signatures
that external callers use. It needs `socks5_proxy: Option<&str>` in its own
signatures AND passed through to every inner call:

```rust
// Before
pub async fn add_member(
    base_url: &str,
    root_cert: &[u8],
    auth_token: &str,
    ...
) {
    crate::hsm::group::http_client::get_one(base_url, auth_token, root_cert, ...).await
    crate::hsm::group::http_client::post_member(auth_token, base_url, root_cert, ...).await
}

// After
pub async fn add_member(
    base_url: &str,
    root_cert: &[u8],
    socks5_proxy: Option<&str>,    // ← after root_cert
    auth_token: &str,
    ...
) {
    crate::hsm::group::http_client::get_one(base_url, auth_token, root_cert, socks5_proxy, ...).await
    crate::hsm::group::http_client::post_member(auth_token, base_url, root_cert, socks5_proxy, ...).await
}
```

Note: some of these utils functions use `shasta_` prefixes on some parameters
(mixed naming, pre-existing issue). Apply the rule mechanically.

### `backend_connector.rs` — 41 call sites

Every `impl SomeTrait for Ochami` method that calls an http_client function
must pass `self.socks5_proxy.as_deref()` immediately after `&self.root_cert`.

```rust
// Before
hsm::group::http_client::get(
    &self.base_url,
    auth_token,
    &self.root_cert,
    label_vec_opt,
    tag_vec_opt,
)

// After
hsm::group::http_client::get(
    &self.base_url,
    auth_token,
    &self.root_cert,
    self.socks5_proxy.as_deref(),
    label_vec_opt,
    tag_vec_opt,
)
```

---

## Special case: `pcs/transitions/http_client.rs` — `wait_to_complete`

`wait_to_complete` calls `get_by_id` in a polling loop. After adding
`socks5_proxy` to `get_by_id`, `wait_to_complete` must accept and forward it:

```rust
pub async fn wait_to_complete(
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    socks5_proxy: Option<&str>,   // ← new
    shasta_token: &str,
    transition_id: &str,
) -> ... {
    loop {
        let transition = get_by_id(
            shasta_base_url,
            shasta_root_cert,
            socks5_proxy,           // ← forwarded
            shasta_token,
            transition_id,
        ).await?;
        ...
    }
}
```

---

## What does NOT change

- The `Ochami` struct already has `socks5_proxy: Option<String>` (added when
  `Ochami::new` was updated). No struct changes needed.
- `StaticBackendDispatcher::new` already passes `socks5_proxy` to
  `Ochami::new`. No change needed there.
- The `manta` crate does not call ochami-rs http_client functions directly
  (all ochami-rs access is via the dispatcher). No manta changes.

---

## Execution order

```
Step 1   bss/http_client.rs
         5 function signatures, 6 env-var reads → socks5_proxy param

Step 2   pcs/power_cap/http_client.rs
         4 function signatures, 6 env-var reads

Step 3   pcs/power_status/http_client.rs
         1 function signature, 2 env-var reads

Step 4   pcs/transitions/http_client.rs
         5 function signatures, 5 env-var reads
         (wait_to_complete calls get_by_id — update both, pass socks5_proxy)

Step 5   hsm/group/http_client.rs
         9 function signatures, 8 env-var reads

Step 6   hsm/component/http_client.rs
         10 function signatures, 8 env-var reads

Step 7   hsm/state/components/http_client.rs
         10 function signatures, 10 env-var reads

Step 8   hsm/defaults/node_map/http_client.rs
         6 function signatures, 6 env-var reads

Step 9   hsm/node_map/http_client.rs
         7 function signatures, 6 env-var reads

Step 10  hsm/inventory/ethernet_interfaces/http_client.rs
         9 function signatures, 9 env-var reads

Step 11  hsm/inventory/hardware/http_client.rs
         6 function signatures, 6 env-var reads

Step 12  hsm/inventory/hardware_by_fru/http_client.rs
         4 function signatures, 4 env-var reads

Step 13  hsm/inventory/redfish_endpoint/http_client.rs
         8 function signatures, 7 env-var reads

Step 14  hsm/memberships/http_client.rs
         2 function signatures, 2 env-var reads

Step 15  hsm/partition/http_client.rs
         6 function signatures, 8 env-var reads

         → cargo check ochami-rs now reports E0061 for every call site that
           is missing the new argument. This drives the remaining steps.

Step 16  hsm/group/utils.rs
         5 function signatures gain socks5_proxy
         7 inner call sites updated

Step 17  backend_connector.rs
         41 call sites gain self.socks5_proxy.as_deref()

Step 18  cargo check ochami-rs: 0 errors
         cargo check manta:     0 errors (no manta changes expected)
```

After Step 15, the compiler reports every missed call site as E0061. Steps 16
and 17 are driven to completion by those errors — no missed site is possible.
