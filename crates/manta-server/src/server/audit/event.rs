//! The audit event wire shape (`schema_version` 1).
//!
//! One [`AuditEvent`] is emitted per audited request, after the
//! response is known, so every event carries both *who asked for
//! what* and *how it ended*. The JSON produced by `serde` is the
//! contract with whoever consumes the Kafka topic: add fields freely,
//! but rename or remove one only together with a `SCHEMA_VERSION`
//! bump.
//!
//! Nothing secret ever lands here by construction: there is no field
//! for a token, a password or a raw request body. Handlers describe
//! their operation through [`super::AuditNote`] with values they pick
//! explicitly.

use std::collections::BTreeMap;

use axum::http::StatusCode;
use serde::Serialize;

/// Version of the event shape. Bump on any rename/removal.
pub const SCHEMA_VERSION: u32 = 1;

/// What kind of activity the event records.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EventKind {
  /// A mutating `/v2/*` API call (any method other than GET/HEAD/OPTIONS).
  ApiRequest,
  /// An interactive console attach (node or CFS session) over WebSocket.
  ConsoleSession,
  /// A `/v2/auth/token` credential exchange.
  AuthAttempt,
}

/// How the request ended, derived from the response status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
  /// 1xx–3xx: the operation went through (101 = console attached).
  Success,
  /// 401/403: the caller was not allowed to do it.
  Denied,
  /// 408: the server gave up on the request (`TimeoutLayer`).
  TimedOut,
  /// Any other 4xx: the request was refused as invalid.
  Rejected,
  /// 5xx: the server or a backend failed while doing it.
  Failed,
}

impl Verdict {
  /// Classify a response status.
  pub fn from_status(status: StatusCode) -> Self {
    match status.as_u16() {
      401 | 403 => Self::Denied,
      408 => Self::TimedOut,
      400..=499 => Self::Rejected,
      500.. => Self::Failed,
      _ => Self::Success,
    }
  }
}

/// Who performed the operation, as claimed by the inbound bearer token.
///
/// The token signature is not verified locally (see
/// `manta_shared::common::jwt_ops`); the backend verifies it on use.
/// The identity is therefore authoritative when `outcome.result` is
/// `success`, and a *claim* otherwise.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Actor {
  /// `preferred_username` claim — the login name.
  pub username: Option<String>,
  /// `name` claim — the display name.
  pub name: Option<String>,
  /// `sub` claim — the stable IdP subject id.
  pub subject: Option<String>,
}

/// Where the request came from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Client {
  /// TCP peer address of the connection (a reverse proxy, if any).
  pub ip: Option<String>,
  /// Raw `X-Forwarded-For` header, recorded as-is and **not**
  /// trusted: any client can set it.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub forwarded_for: Option<String>,
}

/// The HTTP request as received.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Request {
  /// HTTP method, e.g. `DELETE`.
  pub method: String,
  /// Route template, e.g. `/v2/groups/{label}`.
  pub route: String,
  /// Concrete path, e.g. `/v2/groups/zinal`.
  pub path: String,
  /// Raw query string, when present.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub query: Option<String>,
}

/// How the request ended.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Outcome {
  /// Coarse classification of [`Self::status`].
  pub result: Verdict,
  /// HTTP response status code.
  pub status: u16,
  /// Wall time from the request reaching the audit layer to the
  /// response leaving the handler. For a console session, the time to
  /// attach (the session itself outlives the response).
  pub duration_ms: u64,
}

/// Which manta-server instance emitted the event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Emitter {
  /// Always `manta-server`.
  pub service: &'static str,
  /// `CARGO_PKG_VERSION` of the running binary.
  pub version: &'static str,
  /// Host (pod) name, when it can be determined.
  pub host: Option<String>,
}

/// One audit record. See the module docs for the stability contract.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AuditEvent {
  /// Always [`SCHEMA_VERSION`].
  pub schema_version: u32,
  /// Random v4 UUID; also the Kafka message key.
  pub event_id: String,
  /// RFC 3339 UTC timestamp (millisecond precision) of emission.
  pub timestamp: String,
  /// What kind of activity this is.
  pub kind: EventKind,
  /// Stable operation name chosen by the handler, e.g.
  /// `group.delete`; `"<METHOD> <route>"` when the handler did not
  /// name it (e.g. the request was rejected before the handler ran).
  pub action: String,
  /// Who did it.
  pub actor: Actor,
  /// From where.
  pub client: Client,
  /// Target site (`X-Manta-Site`), when supplied.
  pub site: Option<String>,
  /// The request as received.
  pub request: Request,
  /// How it ended.
  pub outcome: Outcome,
  /// `true` when the caller asked for a dry run (nothing changed);
  /// `None` when the operation has no dry-run mode or the handler
  /// never ran.
  pub dry_run: Option<bool>,
  /// Objects the operation acts on, keyed by object kind (`group`,
  /// `node`, `hosts`, `image`, …). Values are as the caller supplied
  /// them (a hosts expression is not expanded).
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  pub targets: BTreeMap<String, Vec<String>>,
  /// Further operation parameters a handler chose to record (e.g.
  /// `force`, the power action, the kernel parameters).
  #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
  pub details: serde_json::Map<String, serde_json::Value>,
  /// Which server emitted it.
  pub emitter: Emitter,
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn result_classifies_statuses() {
    let r = |s: u16| Verdict::from_status(StatusCode::from_u16(s).unwrap());
    assert_eq!(r(101), Verdict::Success);
    assert_eq!(r(200), Verdict::Success);
    assert_eq!(r(204), Verdict::Success);
    assert_eq!(r(400), Verdict::Rejected);
    assert_eq!(r(401), Verdict::Denied);
    assert_eq!(r(403), Verdict::Denied);
    assert_eq!(r(404), Verdict::Rejected);
    assert_eq!(r(408), Verdict::TimedOut);
    assert_eq!(r(409), Verdict::Rejected);
    assert_eq!(r(500), Verdict::Failed);
    assert_eq!(r(502), Verdict::Failed);
  }

  #[test]
  fn result_serialises_snake_case() {
    assert_eq!(
      serde_json::to_value(Verdict::TimedOut).unwrap(),
      serde_json::json!("timed_out")
    );
    assert_eq!(
      serde_json::to_value(EventKind::ConsoleSession).unwrap(),
      serde_json::json!("console_session")
    );
  }
}
