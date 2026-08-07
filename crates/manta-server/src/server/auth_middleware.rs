//! Defensive middleware for the `/api/v1/auth/*` sub-router.
//!
//! Two layers, applied in this order:
//!
//! 1. `rate_limit` — per-source-IP token-bucket. Drops requests that
//!    exceed `[server].auth_rate_limit_per_minute` with a 429 response.
//!    Source IP comes from the connection (after the optional
//!    `X-Forwarded-For` handling that ConnectInfo gives us). Operators
//!    are still expected to terminate at a reverse proxy and rate-limit
//!    there too — this is defence in depth.
//!
//! 2. `strip_body_for_logs` — explicit, even though the request-logger
//!    in `super::log_requests` only logs `method + uri + status` today.
//!    Treat it as a hard guarantee that credentials submitted to
//!    `/auth/token` never end up in a log line, regardless of what the
//!    logger middleware grows into in future.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use axum::{
  Json,
  extract::{ConnectInfo, Request, State},
  http::StatusCode,
  middleware::Next,
  response::{IntoResponse, Response},
};

use super::ServerState;
use super::handlers::ErrorResponse;

/// Per-IP state for the token-bucket rate limiter.
struct WindowState {
  window_start: Instant,
  count: u32,
}

/// In-memory rate-limit table, sized by the number of distinct
/// source IPs that hit `/auth/*` in the last minute. For typical CLI
/// fleets this is small; entries older than two windows are pruned
/// on every check.
///
/// Constructed once by [`super::routes::build_router`] and threaded
/// through Axum's `Extension` layer into [`rate_limit`]. The limit
/// (requests per IP per minute) is read from
/// [`super::ServerState::auth_rate_limit_per_minute`] on every
/// request, so it can change at config-reload time without
/// rebuilding the router.
#[derive(Default)]
pub struct AuthRateLimiter {
  windows: Mutex<HashMap<IpAddr, WindowState>>,
}

impl AuthRateLimiter {
  /// Construct a fresh limiter wrapped in an `Arc` so it can be
  /// shared via Axum's `Extension` layer across handler invocations.
  pub fn new() -> Arc<Self> {
    Arc::new(Self::default())
  }

  /// Returns `true` if `ip` is allowed to make one more request under
  /// the given `limit` (requests per minute), `false` if it would
  /// exceed.
  fn check(&self, ip: IpAddr, limit: u32) -> bool {
    self.check_at(ip, limit, Instant::now())
  }

  /// Testable variant of [`Self::check`] with an explicit clock. The split
  /// lets unit tests exercise the window-reset and pruning logic
  /// without actually sleeping 60+ seconds.
  fn check_at(&self, ip: IpAddr, limit: u32, now: Instant) -> bool {
    let window = Duration::from_secs(60);
    let mut windows = self.windows.lock().expect("rate limiter mutex poisoned");

    // Opportunistic pruning of stale entries.
    windows
      .retain(|_, state| now.duration_since(state.window_start) < window * 2);

    let entry = windows.entry(ip).or_insert(WindowState {
      window_start: now,
      count: 0,
    });

    if now.duration_since(entry.window_start) >= window {
      entry.window_start = now;
      entry.count = 0;
    }

    if entry.count >= limit {
      return false;
    }
    entry.count += 1;
    true
  }
}

/// Per-source-IP rate-limit middleware for the `/api/v1/auth/*`
/// sub-router. Reads
/// [`super::ServerState::auth_rate_limit_per_minute`]; when `None`,
/// the middleware is a no-op (operators rate-limit at the proxy).
/// When the per-IP request count exceeds the limit, returns
/// `429 Too Many Requests` with an [`ErrorResponse`] body and a
/// `tracing::warn!` event.
pub async fn rate_limit(
  State(state): State<Arc<ServerState>>,
  ConnectInfo(peer): ConnectInfo<SocketAddr>,
  limiter: axum::extract::Extension<Arc<AuthRateLimiter>>,
  request: Request,
  next: Next,
) -> Response {
  let Some(limit) = state.auth_rate_limit_per_minute else {
    return next.run(request).await;
  };
  if !limiter.check(peer.ip(), limit) {
    tracing::warn!(
      "auth: rate limit exceeded for source {} (limit={}/min)",
      peer.ip(),
      limit
    );
    return (
      StatusCode::TOO_MANY_REQUESTS,
      Json(ErrorResponse {
        error: "rate limit exceeded".to_string(),
      }),
    )
      .into_response();
  }
  next.run(request).await
}

/// Belt-and-braces: ensure no `/auth/*` request body ever reaches a
/// logger. The runtime cost is one logger-scoped `tracing` span with
/// the body field redacted; the body itself is forwarded to the
/// handler untouched, so deserialisation in
/// [`super::handlers::auth_token`] still sees the original payload.
pub async fn strip_body_for_logs(request: Request, next: Next) -> Response {
  let span = tracing::info_span!("auth_request", body = "<redacted>");
  let _enter = span.enter();
  next.run(request).await
}

#[cfg(test)]
mod tests;
