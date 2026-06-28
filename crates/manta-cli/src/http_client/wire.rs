//! URL-scheme rewriting + curl debug formatting with secrets redaction.
//!
//! Two unrelated concerns colocated because both touch the wire shape
//! of an outbound request without doing any IO themselves. Used by
//! the hand-rolled WebSocket / SSE paths (`console.rs`, `streaming.rs`)
//! and by the wrapper's debug logging in `client.rs`. The auto-
//! generated client handles its own URL construction and doesn't
//! call into either helper.
//!
//! Redaction policy:
//!
//! - The `Authorization` header value is replaced with
//!   `Bearer <REDACTED>` (or just `<REDACTED>` when not a bearer
//!   token).
//! - Request body fields named `password` or `token` (at any depth)
//!   are replaced with `<REDACTED>`. Bodies that aren't valid JSON
//!   pass through untouched — non-JSON bodies on this client are
//!   rare and never carry credentials.

/// Convert an `http://` or `https://` base URL to the corresponding
/// `ws://` / `wss://` URL.
///
/// Path and query are preserved. Any input that doesn't start with a
/// recognised HTTP scheme is returned unchanged — defensive behaviour
/// for a URL the caller never expected to be HTTP in the first place.
pub fn ws_base_url(http_url: &str) -> String {
  if let Some(rest) = http_url.strip_prefix("https://") {
    format!("wss://{rest}")
  } else if let Some(rest) = http_url.strip_prefix("http://") {
    format!("ws://{rest}")
  } else {
    http_url.to_owned()
  }
}

/// Render `req` as a copy-pasteable `curl` invocation, with header
/// and body secrets redacted (see the module-level redaction
/// policy).
///
/// Always passes `-k` (insecure TLS) since the development manta
/// servers commonly use self-signed certs; the rendered command is
/// for debugging convenience, not security. Used by
/// `MantaClient::log_request_as_curl` to emit a single redacted
/// curl line per outbound request when tracing is enabled.
pub fn format_request_as_curl(req: &reqwest::Request) -> String {
  let mut out = format!("  curl -k -X {} '{}'", req.method(), req.url());
  for (name, value) in req.headers() {
    let raw = value.to_str().unwrap_or("<binary>");
    let rendered = if name == reqwest::header::AUTHORIZATION {
      if raw.starts_with("Bearer ") {
        "Bearer <REDACTED>".to_string()
      } else {
        "<REDACTED>".to_string()
      }
    } else {
      raw.to_string()
    };
    out.push_str(&format!(" \\\n    -H '{name}: {rendered}'"));
  }
  if let Some(body_bytes) = req.body().and_then(reqwest::Body::as_bytes) {
    let body_str = std::str::from_utf8(body_bytes).unwrap_or("<binary>");
    let redacted = redact_json_secrets(body_str);
    out.push_str(&format!(" \\\n    --data-raw '{redacted}'"));
  }
  out
}

/// Walk `body` as JSON, replacing any `password` or `token` field
/// value with the literal string `<REDACTED>`. Recurses into nested
/// objects and arrays. Falls back to the original string when the
/// body isn't parseable as JSON — non-JSON bodies are rare on this
/// client and never carry credentials.
pub fn redact_json_secrets(body: &str) -> String {
  let Ok(mut value) = serde_json::from_str::<serde_json::Value>(body) else {
    return body.to_string();
  };
  redact_value(&mut value);
  serde_json::to_string(&value).unwrap_or_else(|_| body.to_string())
}

fn redact_value(v: &mut serde_json::Value) {
  match v {
    serde_json::Value::Object(map) => {
      for (k, val) in map.iter_mut() {
        if matches!(k.as_str(), "password" | "token") {
          *val = serde_json::Value::String("<REDACTED>".to_string());
        } else {
          redact_value(val);
        }
      }
    }
    serde_json::Value::Array(arr) => {
      for item in arr {
        redact_value(item);
      }
    }
    _ => {}
  }
}
