//! Tests for the `/api/v1/auth/*` defensive middleware.
//!
//! The rate-limiter is the main subject — its token-bucket algorithm
//! has time-based behaviours (window reset, stale-entry pruning) that
//! we exercise via the `check_at` clock-injection helper without
//! actually sleeping for 60+ seconds. The middleware wrapper itself
//! is one branch deep, so we cover its disabled-limit pass-through
//! at the integration level.
//!
//! `strip_body_for_logs` is not unit-tested here: the function only
//! opens a tracing span and forwards the request unchanged. Verifying
//! the span's `body = "<redacted>"` field would require a full
//! `tracing_subscriber` test harness, which is more setup than the
//! one-line implementation warrants. The body-redaction guarantee
//! is reviewable directly in the source.

use std::net::{IpAddr, Ipv4Addr};
use std::time::{Duration, Instant};

use super::AuthRateLimiter;

fn ip(last: u8) -> IpAddr {
  IpAddr::V4(Ipv4Addr::new(10, 0, 0, last))
}

#[test]
fn first_request_for_a_fresh_ip_is_allowed() {
  let limiter = AuthRateLimiter::default();
  assert!(limiter.check_at(ip(1), 60, Instant::now()));
}

#[test]
fn allows_exactly_limit_requests_then_rejects() {
  let limiter = AuthRateLimiter::default();
  let t0 = Instant::now();

  // The first `limit` calls all succeed.
  for i in 0..5 {
    assert!(
      limiter.check_at(ip(1), 5, t0),
      "request {} unexpectedly rejected within the limit",
      i + 1
    );
  }
  // The (limit + 1)-th call is rejected.
  assert!(!limiter.check_at(ip(1), 5, t0));
  // And keeps being rejected within the same window.
  assert!(!limiter.check_at(ip(1), 5, t0 + Duration::from_secs(30)));
}

#[test]
fn different_ips_have_independent_buckets() {
  let limiter = AuthRateLimiter::default();
  let t0 = Instant::now();

  // Exhaust IP A's quota.
  for _ in 0..3 {
    assert!(limiter.check_at(ip(1), 3, t0));
  }
  assert!(!limiter.check_at(ip(1), 3, t0), "IP A should be over limit");

  // IP B is unaffected.
  for _ in 0..3 {
    assert!(
      limiter.check_at(ip(2), 3, t0),
      "IP B's bucket was contaminated by IP A's"
    );
  }
}

#[test]
fn bucket_resets_after_60_seconds() {
  let limiter = AuthRateLimiter::default();
  let t0 = Instant::now();

  for _ in 0..2 {
    assert!(limiter.check_at(ip(1), 2, t0));
  }
  assert!(!limiter.check_at(ip(1), 2, t0));

  // Just before the window edge — still rejected.
  assert!(!limiter.check_at(ip(1), 2, t0 + Duration::from_secs(59)));

  // At the window edge — the bucket resets and a fresh quota starts.
  let t1 = t0 + Duration::from_secs(60);
  assert!(limiter.check_at(ip(1), 2, t1));
  assert!(limiter.check_at(ip(1), 2, t1));
  assert!(!limiter.check_at(ip(1), 2, t1));
}

#[test]
fn stale_entries_are_pruned_after_two_windows() {
  let limiter = AuthRateLimiter::default();
  let t0 = Instant::now();

  // Touch IP A at t0.
  assert!(limiter.check_at(ip(1), 1, t0));
  assert_eq!(limiter.windows.lock().unwrap().len(), 1);

  // Two windows later, a request from any IP triggers the prune.
  // IP A's stale entry should be gone, leaving only IP B's fresh one.
  let t_stale = t0 + Duration::from_secs(120);
  assert!(limiter.check_at(ip(2), 1, t_stale));
  let remaining: Vec<IpAddr> =
    limiter.windows.lock().unwrap().keys().copied().collect();
  assert_eq!(
    remaining,
    vec![ip(2)],
    "stale entry for IP A should be pruned"
  );
}

#[test]
fn limit_of_zero_rejects_everything() {
  let limiter = AuthRateLimiter::default();
  // Edge case: configuration of `0` means "block all auth attempts".
  // Documenting the behaviour even though no operator would set this.
  assert!(!limiter.check_at(ip(1), 0, Instant::now()));
}

// ---------------------------------------------------------------------------
// Middleware wrapper
//
// `rate_limit` itself is two `if` branches over `check`. The no-limit
// pass-through is the one branch worth covering at the wrapper level
// because it's what most production configs hit (`auth_rate_limit_per_minute`
// defaults to `None`). The rate-limited branch is covered indirectly
// by `AuthRateLimiter` tests above plus the existing route smoke
// tests in `crates/manta-server/tests/server_routes.rs`.
// ---------------------------------------------------------------------------

// (Currently no middleware-wrapper test — the wrapper plumbing
// requires building a full Axum app with `ConnectInfo<SocketAddr>`,
// which is heavier than the test value. Add one if the wrapper grows
// branches beyond the no-limit early-return.)
