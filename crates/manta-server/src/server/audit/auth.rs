//! The `/v2/auth/token` attempt event.
//!
//! `/v2/auth/*` is outside the [`super::audit_requests`] middleware —
//! there is no bearer token yet (the caller is asking for one) — so
//! the auth handler emits its own event, with the identity taken from
//! the submitted username. The password never reaches this module.

use axum::http::StatusCode;

use super::{
  AuditLog,
  event::{Actor, Client, EventKind, Outcome, Request, Verdict},
};

/// Emit an [`EventKind::AuthAttempt`] event for a `/v2/auth/token` call.
pub fn send_auth_audit(
  audit: &AuditLog,
  success: bool,
  username: &str,
  source_ip: &str,
  site: &str,
) {
  let mut event = audit.event(EventKind::AuthAttempt, "auth.token".into());
  event.actor = Actor {
    username: Some(username.to_string()),
    ..Actor::default()
  };
  event.client = Client {
    ip: Some(source_ip.to_string()),
    forwarded_for: None,
  };
  event.site = Some(site.to_string());
  event.request = Request {
    method: "POST".into(),
    route: "/v2/auth/token".into(),
    path: "/v2/auth/token".into(),
    query: None,
  };
  let status = if success {
    StatusCode::OK
  } else {
    StatusCode::UNAUTHORIZED
  };
  event.outcome = Outcome {
    result: Verdict::from_status(status),
    status: status.as_u16(),
    duration_ms: 0,
  };
  audit.emit(event);
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;

  use super::*;
  use crate::server::audit::CaptureSink;

  #[test]
  fn records_username_site_and_outcome() {
    let sink = Arc::new(CaptureSink::default());
    let log = AuditLog::with_sinks(vec![sink.clone()]);
    send_auth_audit(&log, false, "alice", "10.0.0.1", "alps");
    let events = sink.0.lock().unwrap();
    let e = &events[0];
    assert_eq!(e.kind, EventKind::AuthAttempt);
    assert_eq!(e.actor.username.as_deref(), Some("alice"));
    assert_eq!(e.client.ip.as_deref(), Some("10.0.0.1"));
    assert_eq!(e.site.as_deref(), Some("alps"));
    assert_eq!(e.outcome.result, Verdict::Denied);
  }
}
