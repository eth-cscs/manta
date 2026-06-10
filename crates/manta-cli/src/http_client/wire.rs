//! URL-scheme rewriting + curl debug formatting with secrets redaction.
//!
//! Two unrelated concerns colocated because both touch the wire shape
//! of an outbound request without doing any IO themselves. Used by
//! the hand-rolled WebSocket / SSE paths (`console.rs`, `streaming.rs`)
//! and by the wrapper's debug logging in `client.rs`. The auto-
//! generated client handles its own URL construction and doesn't
//! call into either helper.

/// Convert an `http://` or `https://` base URL to the corresponding `ws://` / `wss://` URL.
pub fn ws_base_url(http_url: &str) -> String {
  if let Some(rest) = http_url.strip_prefix("https://") {
    format!("wss://{rest}")
  } else if let Some(rest) = http_url.strip_prefix("http://") {
    format!("ws://{rest}")
  } else {
    http_url.to_owned()
  }
}

/// Render `req` as a copy-pasteable `curl` invocation. Used by
/// `MantaClient::log_request_as_curl`; the secrets-redaction policy
/// lives here so it's consistent across every call site.
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

/// Walk `body` as JSON, replacing any `password` or `token` field value
/// with `<REDACTED>`. Falls back to the original string when the body
/// isn't parseable as JSON — non-JSON bodies are rare on this client
/// and never carry credentials.
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
