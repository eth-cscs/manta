//! Audit trail of every mutating operation, produced to Kafka.
//!
//! manta-server is the single chokepoint for every backend call, so it
//! is where auditing lives — the CLI emits nothing. The pieces:
//!
//! - [`audit_requests`] — router middleware that turns every
//!   mutating `/v2/*` request (any method except GET/HEAD/OPTIONS) and
//!   every console attach into one [`event::AuditEvent`], emitted
//!   after the response is known. New routes are covered without any
//!   per-handler code.
//! - [`AuditNote`] — extractor through which a handler adds what only
//!   it knows: a stable `action` name, the targets, the dry-run flag.
//! - [`AuditLog`] — fans each event out to its [`AuditSink`]s: always
//!   a `tracing` line on the [`AUDIT_TARGET`] target, plus Kafka
//!   ([`kafka::KafkaSink`]) when `[auditor.kafka]` is configured.
//! - [`send_auth_audit`] — the `/v2/auth/token` attempt event.
//!
//! Auditing is fail-open: a broker outage is logged and never fails or
//! slows the request.

mod auth;
pub mod event;
pub mod kafka;
mod middleware;

use std::{sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};

pub use auth::send_auth_audit;
pub use middleware::{AuditNote, audit_requests};

use event::{AuditEvent, Emitter, EventKind};

/// `tracing` target of the always-on log copy of every audit event.
pub const AUDIT_TARGET: &str = "manta::audit";

/// `[auditor]` section of `server.toml`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Auditor {
  /// Kafka destination for audit events.
  pub kafka: kafka::Kafka,
}

/// A destination for audit events.
///
/// `emit` is called on the request path and must not block: queue the
/// event and return. Failures are the sink's to log; they never reach
/// the request.
pub trait AuditSink: Send + Sync {
  /// Hand one event to the destination.
  fn emit(&self, event: &AuditEvent);
  /// Drain anything still queued, waiting at most `timeout`. Called
  /// once at shutdown, off the async runtime (it may block).
  fn flush(&self, _timeout: Duration) {}
}

/// Logs each event as one JSON line at INFO on [`AUDIT_TARGET`], so
/// the container log keeps a copy even when Kafka is down or not
/// configured.
struct TracingSink;

impl AuditSink for TracingSink {
  fn emit(&self, event: &AuditEvent) {
    match serde_json::to_string(event) {
      Ok(json) => tracing::info!(target: AUDIT_TARGET, "{json}"),
      Err(e) => tracing::warn!(
        target: AUDIT_TARGET,
        event_id = %event.event_id,
        "audit event not serialisable: {e}"
      ),
    }
  }
}

/// The server's audit trail: builds events and fans them out to sinks.
/// Cheap to clone.
#[derive(Clone)]
pub struct AuditLog {
  sinks: Vec<Arc<dyn AuditSink>>,
  emitter: Emitter,
}

impl Default for AuditLog {
  /// Log-only audit trail (no Kafka).
  fn default() -> Self {
    Self::with_sinks(vec![Arc::new(TracingSink)])
  }
}

impl AuditLog {
  /// The production trail: the log copy plus `kafka`, when configured.
  pub fn new(kafka: Option<kafka::KafkaSink>) -> Self {
    let mut sinks: Vec<Arc<dyn AuditSink>> = vec![Arc::new(TracingSink)];
    if let Some(k) = kafka {
      sinks.push(Arc::new(k));
    }
    Self::with_sinks(sinks)
  }

  /// A trail with exactly these sinks (tests plug a capturing sink).
  pub fn with_sinks(sinks: Vec<Arc<dyn AuditSink>>) -> Self {
    Self {
      sinks,
      emitter: Emitter {
        service: "manta-server",
        version: env!("CARGO_PKG_VERSION"),
        host: host_name(),
      },
    }
  }

  /// A fresh event of `kind` with its id, timestamp and emitter
  /// filled in; the caller sets the rest.
  pub fn event(&self, kind: EventKind, action: String) -> AuditEvent {
    AuditEvent {
      schema_version: event::SCHEMA_VERSION,
      event_id: uuid::Uuid::new_v4().to_string(),
      timestamp: chrono::Utc::now()
        .to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
      kind,
      action,
      actor: Default::default(),
      client: Default::default(),
      site: None,
      request: Default::default(),
      outcome: event::Outcome {
        result: event::Verdict::Success,
        status: 0,
        duration_ms: 0,
      },
      dry_run: None,
      targets: Default::default(),
      details: Default::default(),
      emitter: self.emitter.clone(),
    }
  }

  /// Hand `event` to every sink. Never blocks, never fails.
  pub fn emit(&self, event: &AuditEvent) {
    for sink in &self.sinks {
      sink.emit(event);
    }
  }

  /// Drain every sink, each waiting at most `timeout`. Blocking.
  pub fn flush(&self, timeout: Duration) {
    for sink in &self.sinks {
      sink.flush(timeout);
    }
  }
}

/// Host (in k8s: pod) name — `$HOSTNAME`, else `/etc/hostname`.
fn host_name() -> Option<String> {
  std::env::var("HOSTNAME")
    .ok()
    .or_else(|| std::fs::read_to_string("/etc/hostname").ok())
    .map(|h| h.trim().to_string())
    .filter(|h| !h.is_empty())
}

/// Test helper: a sink that keeps every event it is handed.
#[cfg(test)]
#[derive(Default)]
pub(crate) struct CaptureSink(pub std::sync::Mutex<Vec<AuditEvent>>);

#[cfg(test)]
impl AuditSink for CaptureSink {
  fn emit(&self, event: &AuditEvent) {
    self.0.lock().unwrap().push(event.clone());
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn event_prefills_identity_fields() {
    let log = AuditLog::default();
    let a = log.event(EventKind::ApiRequest, "x.y".into());
    let b = log.event(EventKind::ApiRequest, "x.y".into());
    assert_eq!(a.schema_version, event::SCHEMA_VERSION);
    assert_ne!(a.event_id, b.event_id, "event ids must be unique");
    assert!(a.timestamp.ends_with('Z'), "UTC timestamp: {}", a.timestamp);
    assert_eq!(a.emitter.service, "manta-server");
    assert_eq!(a.emitter.version, env!("CARGO_PKG_VERSION"));
  }

  #[test]
  fn emit_fans_out_to_every_sink() {
    let first = Arc::new(CaptureSink::default());
    let second = Arc::new(CaptureSink::default());
    let log = AuditLog::with_sinks(vec![first.clone(), second.clone()]);
    log.emit(&log.event(EventKind::ApiRequest, "x.y".into()));
    assert_eq!(first.0.lock().unwrap().len(), 1);
    assert_eq!(second.0.lock().unwrap().len(), 1);
  }

  #[test]
  fn serialised_event_has_no_credential_fields() {
    let log = AuditLog::default();
    let json =
      serde_json::to_string(&log.event(EventKind::ApiRequest, "x.y".into()))
        .unwrap();
    for forbidden in ["token", "password", "authorization", "secret"] {
      assert!(!json.contains(forbidden), "`{forbidden}` in {json}");
    }
  }
}
