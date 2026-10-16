//! The audit middleware and the [`AuditNote`] extractor.

use std::{
  collections::BTreeMap,
  convert::Infallible,
  net::SocketAddr,
  sync::{Arc, Mutex},
  time::Instant,
};

use axum::{
  extract::{
    ConnectInfo, FromRequestParts, MatchedPath, OriginalUri, Request, State,
  },
  http::{HeaderMap, Method, StatusCode, header, request::Parts},
  middleware::Next,
  response::Response,
};
use manta_shared::common::jwt_ops;
use serde::Serialize;

use super::event::{self, Actor, Client, EventKind, Verdict};
use crate::server::ServerState;

/// What a handler adds to its request's audit event.
#[derive(Default)]
struct Note {
  action: Option<&'static str>,
  dry_run: Option<bool>,
  targets: BTreeMap<String, Vec<String>>,
  nodes: Vec<String>,
  produced: BTreeMap<String, String>,
  details: serde_json::Map<String, serde_json::Value>,
}

/// Handler-side handle on the current request's audit event.
///
/// Extract it next to `RequestCtx` and describe the operation **before**
/// the first fallible call, so the description is kept on the error
/// path too:
///
/// ```ignore
/// note.action("group.delete").target("group", &label).detail("force", q.force);
/// ```
///
/// Outside an audited request (GET routes, unit tests) the handle is
/// inert and every call is a no-op. Never put a credential in it.
#[derive(Clone, Default)]
pub struct AuditNote(Option<Arc<Mutex<Note>>>);

impl AuditNote {
  fn with(&self, f: impl FnOnce(&mut Note)) -> &Self {
    if let Some(note) = &self.0 {
      // A poisoned lock means a handler panicked mid-note; the
      // half-written note is still the best record we have.
      f(&mut note.lock().unwrap_or_else(|p| p.into_inner()));
    }
    self
  }

  /// Stable operation name, `<resource>.<verb>` (e.g. `group.delete`).
  pub fn action(&self, action: &'static str) -> &Self {
    self.with(|n| n.action = Some(action))
  }

  /// Whether the caller asked for a dry run.
  pub fn dry_run(&self, dry_run: bool) -> &Self {
    self.with(|n| n.dry_run = Some(dry_run))
  }

  /// Add one object of `kind` (`group`, `node`, `hosts`, …) that the
  /// operation acts on.
  pub fn target(&self, kind: &str, value: impl Into<String>) -> &Self {
    self.with(|n| {
      n.targets
        .entry(kind.to_string())
        .or_default()
        .push(value.into());
    })
  }

  /// Add several objects of `kind`.
  pub fn targets<I, S>(&self, kind: &str, values: I) -> &Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    self.with(|n| {
      n.targets
        .entry(kind.to_string())
        .or_default()
        .extend(values.into_iter().map(Into::into));
    })
  }

  /// Record the node xnames the server resolved the targets to.
  pub fn nodes<I, S>(&self, values: I) -> &Self
  where
    I: IntoIterator<Item = S>,
    S: Into<String>,
  {
    self.with(|n| n.nodes.extend(values.into_iter().map(Into::into)))
  }

  /// Record the id of something the operation created or started
  /// (`transition`, `session`, …). Call it only once that happened.
  pub fn produced(&self, kind: &str, id: impl Into<String>) -> &Self {
    self.with(|n| {
      n.produced.insert(kind.to_string(), id.into());
    })
  }

  /// Record a further operation parameter. Silently skipped if it
  /// does not serialise.
  pub fn detail(&self, key: &str, value: impl Serialize) -> &Self {
    self.with(|n| {
      if let Ok(v) = serde_json::to_value(value) {
        n.details.insert(key.to_string(), v);
      }
    })
  }
}

impl<S: Send + Sync> FromRequestParts<S> for AuditNote {
  type Rejection = Infallible;

  async fn from_request_parts(
    parts: &mut Parts,
    _state: &S,
  ) -> Result<Self, Self::Rejection> {
    Ok(
      parts
        .extensions
        .get::<AuditNote>()
        .cloned()
        .unwrap_or_default(),
    )
  }
}

/// What the middleware audits: mutating methods, plus console attaches
/// (a GET upgrade, but an interactive root shell on a node).
fn audited_kind(method: &Method, route: &str) -> Option<EventKind> {
  if route.ends_with("/console") {
    Some(EventKind::ConsoleSession)
  } else if matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) {
    None
  } else {
    Some(EventKind::ApiRequest)
  }
}

/// Identity claims of the inbound bearer token. Reads the
/// `Authorization` header itself rather than any processed token, so
/// the event names the caller as they authenticated.
fn actor(headers: &HeaderMap) -> Actor {
  let Some(token) = headers
    .get(header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
  else {
    return Actor::default();
  };
  let claim = |name| jwt_ops::get_string_claim(token, name).ok().flatten();
  Actor {
    username: claim("preferred_username"),
    name: claim("name"),
    subject: claim("sub"),
  }
}

fn header_str(headers: &HeaderMap, name: &str) -> Option<String> {
  headers
    .get(name)
    .and_then(|v| v.to_str().ok())
    .map(str::to_string)
}

/// Router middleware: emit one [`event::AuditEvent`] per audited
/// request once its response is known. Layered outside the request
/// timeout, so timed-out requests are recorded too. Requests that
/// matched no route (404) or no method (405) are not audited — nothing
/// was attempted.
pub async fn audit_requests(
  State(state): State<Arc<ServerState>>,
  mut request: Request,
  next: Next,
) -> Response {
  let Some(route) = request
    .extensions()
    .get::<MatchedPath>()
    .map(|p| p.as_str().to_string())
  else {
    return next.run(request).await;
  };
  let Some(kind) = audited_kind(request.method(), &route) else {
    return next.run(request).await;
  };

  let note = AuditNote(Some(Arc::default()));
  request.extensions_mut().insert(note.clone());

  let headers = request.headers();
  let actor = actor(headers);
  let site = header_str(headers, "X-Manta-Site");
  let client = Client {
    ip: request
      .extensions()
      .get::<ConnectInfo<SocketAddr>>()
      .map(|ConnectInfo(peer)| peer.ip().to_string()),
    forwarded_for: header_str(headers, "X-Forwarded-For"),
  };
  // The nested `/v2` router sees its URI with the prefix stripped;
  // record the path as the client sent it.
  let uri = request
    .extensions()
    .get::<OriginalUri>()
    .map_or_else(|| request.uri().clone(), |OriginalUri(u)| u.clone());
  let req = event::Request {
    method: request.method().to_string(),
    route,
    path: uri.path().to_string(),
    query: uri.query().map(str::to_string),
  };

  let started = Instant::now();
  let response = next.run(request).await;
  let status = response.status();
  if status == StatusCode::METHOD_NOT_ALLOWED {
    return response;
  }

  // Take rather than unwrap the Arc: a clone may outlive the response
  // (e.g. moved into a WebSocket upgrade callback).
  let note = note
    .0
    .map(|n| std::mem::take(&mut *n.lock().unwrap_or_else(|p| p.into_inner())))
    .unwrap_or_default();
  let action = note
    .action
    .map_or_else(|| format!("{} {}", req.method, req.route), str::to_string);
  let mut event = state.audit.event(kind, action);
  event.actor = actor;
  event.client = client;
  event.site = site;
  event.request = req;
  event.outcome = event::Outcome {
    result: Verdict::from_status(status),
    status: status.as_u16(),
    duration_ms: u64::try_from(started.elapsed().as_millis())
      .unwrap_or(u64::MAX),
  };
  event.dry_run = note.dry_run;
  event.targets = note.targets;
  event.nodes = note.nodes;
  event.produced = note.produced;
  event.details = note.details;
  state.audit.emit(event);

  response
}

#[cfg(test)]
mod tests {
  use axum::http::HeaderValue;
  use base64::prelude::*;

  use super::*;

  #[test]
  fn audits_mutations_and_consoles_only() {
    let r = "/v2/groups/{label}";
    assert_eq!(audited_kind(&Method::GET, r), None);
    assert_eq!(audited_kind(&Method::HEAD, r), None);
    assert_eq!(audited_kind(&Method::OPTIONS, r), None);
    for m in [Method::POST, Method::PUT, Method::PATCH, Method::DELETE] {
      assert_eq!(audited_kind(&m, r), Some(EventKind::ApiRequest), "{m}");
    }
    assert_eq!(
      audited_kind(&Method::GET, "/v2/nodes/{xname}/console"),
      Some(EventKind::ConsoleSession)
    );
  }

  #[test]
  fn actor_reads_claims_from_bearer_header() {
    let claims = serde_json::json!({
      "preferred_username": "alice", "name": "Alice A", "sub": "u-1"
    });
    let token =
      format!("h.{}.s", BASE64_URL_SAFE_NO_PAD.encode(claims.to_string()));
    let mut headers = HeaderMap::new();
    headers.insert(
      header::AUTHORIZATION,
      HeaderValue::from_str(&format!("Bearer {token}")).unwrap(),
    );
    assert_eq!(
      actor(&headers),
      Actor {
        username: Some("alice".into()),
        name: Some("Alice A".into()),
        subject: Some("u-1".into()),
      }
    );
  }

  #[test]
  fn actor_is_empty_without_or_with_garbage_token() {
    assert_eq!(actor(&HeaderMap::new()), Actor::default());
    let mut headers = HeaderMap::new();
    headers.insert(header::AUTHORIZATION, HeaderValue::from_static("Bearer x"));
    assert_eq!(actor(&headers), Actor::default());
  }

  #[test]
  fn inert_note_ignores_calls() {
    let note = AuditNote::default();
    note.action("x.y").target("group", "g").dry_run(true);
    assert!(note.0.is_none());
  }

  #[test]
  fn note_accumulates_targets_and_details() {
    let note = AuditNote(Some(Arc::default()));
    note
      .action("group.delete")
      .target("group", "g1")
      .targets("node", ["x1", "x2"])
      .dry_run(false)
      .detail("force", true)
      .nodes(["x1000c0s0b0n0", "x1000c0s0b0n1"])
      .produced("transition", "t-1");
    let n = note.0.unwrap();
    let n = n.lock().unwrap();
    assert_eq!(n.action, Some("group.delete"));
    assert_eq!(n.targets["group"], vec!["g1"]);
    assert_eq!(n.targets["node"], vec!["x1", "x2"]);
    assert_eq!(n.dry_run, Some(false));
    assert_eq!(n.details["force"], serde_json::json!(true));
    assert_eq!(n.nodes, vec!["x1000c0s0b0n0", "x1000c0s0b0n1"]);
    assert_eq!(n.produced["transition"], "t-1");
  }
}
