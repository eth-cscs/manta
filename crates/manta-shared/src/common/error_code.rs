//! Stable, machine-readable error codes (the catalog behind issue #64).
//!
//! Every failure manta reports — in the HTTP error body, in server
//! logs, and in CLI output — carries one code from this catalog next
//! to its human-readable message. The code identifies the failure
//! *class* (one code per class, not per call site); the message
//! carries the case-specific detail.
//!
//! # Stability contract
//!
//! Codes are a public interface, referenced by scripts, docs, and
//! support channels (`ERRORS.md` is the user-facing reference):
//!
//! - **Append-only.** A code is never renamed, reused, or removed;
//!   an obsolete code is deprecated in `ERRORS.md`, not deleted.
//! - **Messages are not stable** — only the code is. Reword messages
//!   freely; never repurpose a code.
//! - The wire string is an **explicit literal** in the catalog below,
//!   never derived from the Rust variant name, so a refactor cannot
//!   silently change a published code. `catalog_is_pinned` locks
//!   every string.
//!
//! # Where codes are assigned
//!
//! At the layer chokepoints, not at raise sites:
//!
//! - [`MantaError::error_code`](crate::common::error::MantaError::error_code)
//!   for this crate's own error type;
//! - `manta-server`'s `wire_conv::backend_error_code` for the foreign
//!   `BackendError`, stamped into the wire body by `to_handler_error`;
//! - the CLI's HTTP-client error shaping for CLI-local failures
//!   (transport, auth-server markers).

/// Defines [`ErrorCode`], its wire strings, and [`ErrorCode::ALL`]
/// from a single table so the three can never drift. Each row is
/// `#[doc] Variant => "MANTA_WIRE_STRING";`.
macro_rules! error_code_catalog {
  (
    $(
      $(#[$doc:meta])+
      $variant:ident => $code:literal;
    )+
  ) => {
    /// A stable, machine-readable code identifying a failure class.
    ///
    /// See the [module docs](self) for the stability contract. The
    /// wire representation is [`as_str`](ErrorCode::as_str), e.g.
    /// `"MANTA_SESSION_NOT_FOUND"`; that string — not the Rust
    /// variant name — is the public interface.
    ///
    /// ```
    /// use manta_shared::common::error_code::ErrorCode;
    ///
    /// assert_eq!(
    ///   ErrorCode::SessionNotFound.as_str(),
    ///   "MANTA_SESSION_NOT_FOUND",
    /// );
    /// assert!(ErrorCode::ALL.contains(&ErrorCode::SessionNotFound));
    /// ```
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum ErrorCode {
      $( $(#[$doc])+ $variant, )+
    }

    impl ErrorCode {
      /// The stable wire string for this code (`MANTA_*`). This exact
      /// string appears in the HTTP error body's `code` field, in
      /// server logs, and in CLI output.
      #[must_use]
      pub const fn as_str(self) -> &'static str {
        match self { $( ErrorCode::$variant => $code, )+ }
      }

      /// Every code in the catalog, in catalog order. Used by the
      /// docs-sync and uniqueness tests; handy for future tooling
      /// (e.g. a command listing all codes).
      pub const ALL: &'static [ErrorCode] = &[ $( ErrorCode::$variant, )+ ];
    }
  };
}

error_code_catalog! {
  // --- Invalid user input (typically HTTP 400) ---------------------
  /// Request rejected by validation; the message names the offending
  /// input.
  BadRequest => "MANTA_BAD_REQUEST";
  /// A user-supplied pattern (hardware pattern, hostlist expression,
  /// glob) didn't parse.
  InvalidPattern => "MANTA_INVALID_PATTERN";
  /// A node id (xname) is syntactically invalid or unknown in the
  /// request context.
  InvalidNodeId => "MANTA_INVALID_NODE_ID";
  /// A datetime argument (e.g. a `--since`/`--until` filter) didn't
  /// parse.
  InvalidDatetime => "MANTA_INVALID_DATETIME";
  /// The operation isn't supported by the site's backend type.
  UnsupportedBackend => "MANTA_UNSUPPORTED_BACKEND";

  // --- Authentication (typically HTTP 401) -------------------------
  /// No authentication token was supplied, or none is cached for the
  /// site.
  AuthTokenNotFound => "MANTA_AUTH_TOKEN_NOT_FOUND";
  /// The JWT was structurally invalid (wrong number of dots,
  /// undecodable claims, non-UTF-8 payload).
  JwtMalformed => "MANTA_JWT_MALFORMED";
  /// Username/password rejected during the `/auth/token` exchange
  /// (the detail intentionally stays server-side).
  InvalidCredentials => "MANTA_INVALID_CREDENTIALS";
  /// The token is valid but carries a role that forbids the operation
  /// (e.g. the read-only role on a mutating endpoint; HTTP 403).
  Forbidden => "MANTA_FORBIDDEN";
  /// Too many authentication attempts from this source (HTTP 429).
  RateLimited => "MANTA_RATE_LIMITED";

  // --- Resource state (typically HTTP 404 / 409 / 422) -------------
  /// Generic resource lookup failed; the message names the resource.
  NotFound => "MANTA_NOT_FOUND";
  /// A CFS session was not found.
  SessionNotFound => "MANTA_SESSION_NOT_FOUND";
  /// A CFS configuration was not found.
  ConfigurationNotFound => "MANTA_CONFIGURATION_NOT_FOUND";
  /// The requested site isn't hosted by the manta-server instance
  /// (`X-Manta-Site` mismatch), or the server has no such site.
  SiteNotFound => "MANTA_SITE_NOT_FOUND";
  /// The operation conflicts with current resource state.
  Conflict => "MANTA_CONFLICT";
  /// A configuration with that name already exists.
  ConfigurationAlreadyExists => "MANTA_CONFIGURATION_ALREADY_EXISTS";
  /// Not enough capacity/resources to satisfy the request.
  InsufficientResources => "MANTA_INSUFFICIENT_RESOURCES";
  /// An optional server feature (vault, k8s) isn't configured for
  /// this site (typically HTTP 501).
  NotConfigured => "MANTA_NOT_CONFIGURED";

  // --- Upstream backend (CSM / OCHAMI) -----------------------------
  /// The backend returned an HTTP error; the upstream status and body
  /// are in the message (and the HTTP status is propagated verbatim).
  BackendHttpError => "MANTA_BACKEND_HTTP_ERROR";
  /// The manta-server → backend call timed out (typically HTTP 504).
  BackendTimeout => "MANTA_BACKEND_TIMEOUT";
  /// manta-server could not establish a TCP/TLS connection to the
  /// backend.
  BackendConnectFailed => "MANTA_BACKEND_CONNECT_FAILED";
  /// Other outbound HTTP failure (DNS, body stream, protocol).
  NetworkError => "MANTA_NETWORK_ERROR";

  // --- Internal (typically HTTP 500) -------------------------------
  /// A required field was absent in backend data or configuration.
  MissingField => "MANTA_MISSING_FIELD";
  /// Filesystem I/O failure.
  IoError => "MANTA_IO_ERROR";
  /// JSON (de)serialisation failure.
  SerdeError => "MANTA_SERDE_ERROR";
  /// YAML parse / serialise failure.
  YamlError => "MANTA_YAML_ERROR";
  /// TOML parse / edit / serialise failure.
  TomlError => "MANTA_TOML_ERROR";
  /// Configuration file loading / schema failure.
  ConfigError => "MANTA_CONFIG_ERROR";
  /// Kafka producer construction or delivery failed.
  KafkaError => "MANTA_KAFKA_ERROR";
  /// Kubernetes API interaction failed.
  K8sError => "MANTA_K8S_ERROR";
  /// Node-console session failure.
  ConsoleError => "MANTA_CONSOLE_ERROR";
  /// Jinja2 / template rendering failed (SAT files).
  TemplateError => "MANTA_TEMPLATE_ERROR";
  /// A configured hook failed.
  HookError => "MANTA_HOOK_ERROR";
  /// A local git operation failed.
  GitError => "MANTA_GIT_ERROR";
  /// Catch-all internal error; prefer a specific code when adding new
  /// failure modes.
  Internal => "MANTA_INTERNAL";

  // --- CLI ↔ manta-server transport (CLI-local, no HTTP status) ----
  /// The CLI could not connect to manta-server (refused, unreachable,
  /// or connect timeout).
  ServerUnreachable => "MANTA_SERVER_UNREACHABLE";
  /// The CLI-side request timeout fired before manta-server responded.
  ServerTimeout => "MANTA_SERVER_TIMEOUT";
  /// Other CLI ↔ manta-server transport failure.
  TransportError => "MANTA_TRANSPORT_ERROR";
}

impl std::fmt::Display for ErrorCode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str(self.as_str())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn codes_are_unique() {
    let mut codes: Vec<&str> =
      ErrorCode::ALL.iter().map(|c| c.as_str()).collect();
    codes.sort_unstable();
    let before = codes.len();
    codes.dedup();
    assert_eq!(before, codes.len(), "duplicate wire string in catalog");
  }

  #[test]
  fn codes_follow_the_naming_convention() {
    for code in ErrorCode::ALL {
      let s = code.as_str();
      assert!(s.starts_with("MANTA_"), "{s}: missing MANTA_ prefix");
      assert!(
        s.chars()
          .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_'),
        "{s}: not SCREAMING_SNAKE_CASE"
      );
    }
  }

  // The stability pin: every published wire string, in catalog order.
  // Deliberately duplicates the catalog literals — changing a code
  // requires touching both places, which is the point (codes are
  // append-only; see the module docs). Extend the list when appending
  // a code; never edit an existing entry.
  #[test]
  fn catalog_is_pinned() {
    let expected = [
      "MANTA_BAD_REQUEST",
      "MANTA_INVALID_PATTERN",
      "MANTA_INVALID_NODE_ID",
      "MANTA_INVALID_DATETIME",
      "MANTA_UNSUPPORTED_BACKEND",
      "MANTA_AUTH_TOKEN_NOT_FOUND",
      "MANTA_JWT_MALFORMED",
      "MANTA_INVALID_CREDENTIALS",
      "MANTA_FORBIDDEN",
      "MANTA_RATE_LIMITED",
      "MANTA_NOT_FOUND",
      "MANTA_SESSION_NOT_FOUND",
      "MANTA_CONFIGURATION_NOT_FOUND",
      "MANTA_SITE_NOT_FOUND",
      "MANTA_CONFLICT",
      "MANTA_CONFIGURATION_ALREADY_EXISTS",
      "MANTA_INSUFFICIENT_RESOURCES",
      "MANTA_NOT_CONFIGURED",
      "MANTA_BACKEND_HTTP_ERROR",
      "MANTA_BACKEND_TIMEOUT",
      "MANTA_BACKEND_CONNECT_FAILED",
      "MANTA_NETWORK_ERROR",
      "MANTA_MISSING_FIELD",
      "MANTA_IO_ERROR",
      "MANTA_SERDE_ERROR",
      "MANTA_YAML_ERROR",
      "MANTA_TOML_ERROR",
      "MANTA_CONFIG_ERROR",
      "MANTA_KAFKA_ERROR",
      "MANTA_K8S_ERROR",
      "MANTA_CONSOLE_ERROR",
      "MANTA_TEMPLATE_ERROR",
      "MANTA_HOOK_ERROR",
      "MANTA_GIT_ERROR",
      "MANTA_INTERNAL",
      "MANTA_SERVER_UNREACHABLE",
      "MANTA_SERVER_TIMEOUT",
      "MANTA_TRANSPORT_ERROR",
    ];
    let actual: Vec<&str> = ErrorCode::ALL.iter().map(|c| c.as_str()).collect();
    assert_eq!(actual, expected);
  }

  #[test]
  fn display_matches_as_str() {
    assert_eq!(
      ErrorCode::BackendTimeout.to_string(),
      "MANTA_BACKEND_TIMEOUT"
    );
  }

  // Two-way sync between the catalog and the user-facing reference.
  // Forward: every code has a row in ERRORS.md. Reverse: every
  // `MANTA_*` token in ERRORS.md is a real catalog code, so the doc
  // can't keep a stale row for a code that no longer exists (which
  // the append-only policy forbids anyway) or a typo'd one.
  #[test]
  fn every_code_is_documented() {
    let doc = std::fs::read_to_string(concat!(
      env!("CARGO_MANIFEST_DIR"),
      "/../../ERRORS.md"
    ))
    .expect("ERRORS.md must exist at the repo root");

    for code in ErrorCode::ALL {
      assert!(
        doc.contains(&format!("`{}`", code.as_str())),
        "{} is in the catalog but has no row in ERRORS.md",
        code.as_str()
      );
    }

    let catalog: std::collections::HashSet<&str> =
      ErrorCode::ALL.iter().map(|c| c.as_str()).collect();
    let bytes = doc.as_bytes();
    let mut i = 0;
    while let Some(pos) = doc[i..].find("MANTA_") {
      let start = i + pos;
      let mut end = start;
      while end < bytes.len()
        && (bytes[end].is_ascii_uppercase()
          || bytes[end].is_ascii_digit()
          || bytes[end] == b'_')
      {
        end += 1;
      }
      let token = &doc[start..end];
      // Skip only true wildcard prose references ("MANTA_* code",
      // "a MANTA_BACKEND_* gateway error"): a trailing underscore
      // whose very next character is `*`. A bare trailing underscore
      // without the `*` (truncated code, lowercase typo like
      // `MANTA_BACKEND_timeout`) must still be flagged.
      let is_wildcard =
        token.ends_with('_') && bytes.get(end).is_some_and(|b| *b == b'*');
      if !is_wildcard {
        assert!(
          catalog.contains(token),
          "ERRORS.md mentions {token}, which is not in the catalog"
        );
      }
      i = end;
    }
  }
}
