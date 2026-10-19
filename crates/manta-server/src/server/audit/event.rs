//! The audit event wire shape (`schema_version` 1).
//!
//! One [`AuditEvent`] is emitted per audited request, after the
//! response is known, so every event carries both *who asked for
//! what* and *how it ended*. The JSON produced by `serde` is the
//! contract with whoever consumes the Kafka topic: add fields freely,
//! but rename or remove one only together with a `SCHEMA_VERSION`
//! bump. A field with no value (`None`, an empty map) is left out of
//! the JSON rather than written as `null` / `{}`.
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
  #[serde(skip_serializing_if = "Option::is_none")]
  pub username: Option<String>,
  /// `name` claim — the display name.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub name: Option<String>,
  /// `sub` claim — the stable IdP subject id.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub subject: Option<String>,
}

/// Where the request came from.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Client {
  /// TCP peer address of the connection (a reverse proxy, if any).
  #[serde(skip_serializing_if = "Option::is_none")]
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
  #[serde(skip_serializing_if = "Option::is_none")]
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
  /// One human-readable line saying who did what, where, and how it
  /// ended — see [`AuditEvent::describe`]. Derived from the other
  /// fields when the event is emitted; for reading, not for parsing.
  pub summary: String,
  /// Who did it.
  pub actor: Actor,
  /// From where.
  pub client: Client,
  /// Target site (`X-Manta-Site`), when supplied.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub site: Option<String>,
  /// The request as received.
  pub request: Request,
  /// How it ended.
  pub outcome: Outcome,
  /// `true` when the caller asked for a dry run (nothing changed),
  /// `false` for a real run of an operation that has a dry-run mode;
  /// absent when the operation has none or the handler never ran.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub dry_run: Option<bool>,
  /// Objects the operation acts on, keyed by object kind (`group`,
  /// `node`, `hosts`, `image`, …). Values are as the caller supplied
  /// them (a hosts expression is not expanded; see [`Self::nodes`]).
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  pub targets: BTreeMap<String, Vec<String>>,
  /// Concrete node xnames the server resolved the request to, when it
  /// expanded a hosts expression or a group itself (power, boot,
  /// kernel parameters, runtime configuration, migrate). Power, boot
  /// and kernel parameters record it before the operation runs, so it
  /// is present on dry runs and failures too; runtime configuration
  /// and migrate resolve and apply in one service call, so there it
  /// is recorded only once that call succeeds.
  #[serde(skip_serializing_if = "Vec::is_empty")]
  pub nodes: Vec<String>,
  /// What the operation created or started, by kind (`transition`,
  /// `session`, `configuration`, `image`, `hostname`) — the ids to
  /// look the operation up by in the backend. Only on success, never
  /// on a dry run.
  #[serde(skip_serializing_if = "BTreeMap::is_empty")]
  pub produced: BTreeMap<String, String>,
  /// Further operation parameters a handler chose to record (e.g.
  /// `force`, the power action, the kernel parameters).
  #[serde(skip_serializing_if = "serde_json::Map::is_empty")]
  pub details: serde_json::Map<String, serde_json::Value>,
  /// Which server emitted it.
  pub emitter: Emitter,
}

/// Most values of one target kind spelled out in a summary.
const SUMMARY_MAX_VALUES: usize = 5;

impl AuditEvent {
  /// One line for humans, built from the structured fields:
  ///
  /// ```text
  /// rjanalik @ alps: power off nid001288 (1 node, force) → success, transition 4c1e…
  /// alice @ alps: image delete a b (dry run) → success
  /// anonymous: DELETE /v2/groups/zinal → denied (401)
  /// ```
  ///
  /// Control characters are escaped (`\n` → `\\n`): the line is
  /// logged as plain text and its values are caller-controlled.
  ///
  /// Every route gets one with no per-handler code: the verb is the
  /// handler's `action` with its separators turned into spaces, or
  /// method and path when no handler named it.
  pub fn describe(&self) -> String {
    use std::fmt::Write as _;

    let who = self
      .actor
      .username
      .as_deref()
      .or(self.actor.subject.as_deref())
      .unwrap_or("anonymous");
    let mut line = match &self.site {
      Some(site) => format!("{who} @ {site}:"),
      None => format!("{who}:"),
    };

    if self.action.contains(' ') {
      // Fallback `"<METHOD> <route>"`: show the concrete path instead.
      let _ = write!(line, " {} {}", self.request.method, self.request.path);
    } else {
      let _ = write!(line, " {}", self.action.replace(['.', '_'], " "));
      for values in self.targets.values() {
        for v in values.iter().take(SUMMARY_MAX_VALUES) {
          let _ = write!(line, " {v}");
        }
        if values.len() > SUMMARY_MAX_VALUES {
          let _ =
            write!(line, " (+{} more)", values.len() - SUMMARY_MAX_VALUES);
        }
      }
    }

    let mut notes = Vec::new();
    match self.nodes.len() {
      0 => {}
      1 => notes.push("1 node".to_string()),
      n => notes.push(format!("{n} nodes")),
    }
    if self.dry_run == Some(true) {
      notes.push("dry run".to_string());
    }
    if self.details.get("force") == Some(&serde_json::Value::Bool(true)) {
      notes.push("force".to_string());
    }
    if !notes.is_empty() {
      let _ = write!(line, " ({})", notes.join(", "));
    }

    let result = serde_json::to_value(self.outcome.result)
      .ok()
      .and_then(|v| v.as_str().map(str::to_string))
      .unwrap_or_default()
      .replace('_', " ");
    let _ = write!(line, " → {result}");
    if self.outcome.result != Verdict::Success {
      let _ = write!(line, " ({})", self.outcome.status);
    }
    for (kind, id) in &self.produced {
      let _ = write!(line, ", {kind} {id}");
    }
    // Most of the line is caller-controlled (unverified token claims,
    // path segments, body strings), and the summary is a plain-text
    // log message: escape control characters so a `\n` cannot forge
    // a second log line.
    line
      .chars()
      .flat_map(|c| {
        let escaped: Vec<char> = if c.is_control() {
          c.escape_default().collect()
        } else {
          vec![c]
        };
        escaped
      })
      .collect()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn event(action: &str) -> AuditEvent {
    AuditEvent {
      schema_version: SCHEMA_VERSION,
      event_id: "e".into(),
      timestamp: "t".into(),
      kind: EventKind::ApiRequest,
      action: action.into(),
      summary: String::new(),
      actor: Actor {
        username: Some("rjanalik".into()),
        ..Actor::default()
      },
      client: Client::default(),
      site: Some("alps".into()),
      request: Request {
        method: "POST".into(),
        route: "/v2/power".into(),
        path: "/v2/power".into(),
        query: None,
      },
      outcome: Outcome {
        result: Verdict::Success,
        status: 200,
        duration_ms: 1,
      },
      dry_run: None,
      targets: BTreeMap::new(),
      nodes: Vec::new(),
      produced: BTreeMap::new(),
      details: serde_json::Map::new(),
      emitter: Emitter {
        service: "manta-server",
        version: "0",
        host: None,
      },
    }
  }

  #[test]
  fn describe_names_who_what_where_and_how_it_ended() {
    let mut e = event("power.off");
    e.targets.insert("hosts".into(), vec!["nid001288".into()]);
    e.nodes = vec!["x1000c0s0b0n0".into()];
    e.details.insert("force".into(), serde_json::json!(true));
    e.produced.insert("transition".into(), "abc-123".into());
    assert_eq!(
      e.describe(),
      "rjanalik @ alps: power off nid001288 (1 node, force) → success, \
       transition abc-123"
    );
  }

  #[test]
  fn describe_marks_dry_runs_and_failures() {
    let mut e = event("boot_parameters.delete");
    e.dry_run = Some(true);
    e.outcome = Outcome {
      result: Verdict::TimedOut,
      status: 408,
      duration_ms: 1,
    };
    assert_eq!(
      e.describe(),
      "rjanalik @ alps: boot parameters delete (dry run) → timed out (408)"
    );
  }

  #[test]
  fn describe_escapes_control_characters() {
    let mut e = event("group.delete");
    e.actor.username = Some("x\nINFO forged".into());
    e.targets
      .insert("group".into(), vec!["g\r\u{1b}[31m".into()]);
    let line = e.describe();
    assert!(!line.chars().any(char::is_control), "{line:?}");
    assert_eq!(
      line,
      "x\\nINFO forged @ alps: group delete g\\r\\u{1b}[31m → success"
    );
  }

  #[test]
  fn describe_truncates_long_target_lists() {
    let mut e = event("image.delete");
    e.targets
      .insert("image".into(), (1..=7).map(|i| format!("i{i}")).collect());
    assert_eq!(
      e.describe(),
      "rjanalik @ alps: image delete i1 i2 i3 i4 i5 (+2 more) → success"
    );
  }

  #[test]
  fn describe_falls_back_to_method_and_path() {
    let mut e = event("DELETE /v2/groups/{label}");
    e.actor = Actor::default();
    e.site = None;
    e.request.method = "DELETE".into();
    e.request.path = "/v2/groups/zinal".into();
    e.outcome = Outcome {
      result: Verdict::Denied,
      status: 401,
      duration_ms: 1,
    };
    assert_eq!(
      e.describe(),
      "anonymous: DELETE /v2/groups/zinal → denied (401)"
    );
  }

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
