//! OIDC login against the CSCS Keycloak.
//!
//! manta-cli obtains its bearer token directly from the CSCS Keycloak —
//! `manta-server` takes no part in authentication and simply forwards
//! the token to the site backends. One token serves every site.
//!
//! Three flows, all against the public client [`OidcSettings::client_id`]:
//!
//! - **Browser** ([`login`], default): authorization code + PKCE (S256),
//!   redirected to a loopback listener on
//!   `http://localhost:`[`REDIRECT_PORT`]. The ID token's nonce and
//!   `at_hash` are verified against the realm's JWKS.
//! - **Device code** ([`login`] when headless): prints a verification
//!   URL + user code and polls the token endpoint. Used when
//!   `oidc_headless = true`, inside an SSH session, without a graphical
//!   display (outside macOS), or when no browser can be opened.
//! - **Refresh** ([`refresh`]): exchanges a cached refresh token for a
//!   new access token without user interaction.
//!
//! The resulting [`TokenStore`] is cached as JSON by
//! [`crate::common::authentication`].

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, anyhow, bail};
use openidconnect::core::{CoreClient, CoreProviderMetadata, CoreResponseType};
use openidconnect::{
  AccessTokenHash, AuthenticationFlow, AuthorizationCode, ClientId, CsrfToken,
  HttpRequest, HttpResponse, IssuerUrl, Nonce, OAuth2TokenResponse,
  PkceCodeChallenge, RedirectUrl, Scope, TokenResponse as _,
};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;

/// Default issuer URL of the CSCS Keycloak realm. Overridable with
/// `oidc_issuer_url` in `cli.toml`.
pub const DEFAULT_OIDC_ISSUER_URL: &str =
  "https://auth.cscs.ch/auth/realms/cscs";

/// Default OIDC client id. Overridable with `oidc_client_id` in
/// `cli.toml`.
///
/// The same client id is used on the production and TDS realms.
pub const DEFAULT_OIDC_CLIENT_ID: &str = "manta-cli";

/// Loopback port the browser flow listens on. Must match the redirect
/// URI registered on the Keycloak client (`http://localhost:<port>`).
///
/// The TDS registration accepts `http://localhost:8765`; loopback IP
/// redirects (`http://127.0.0.1:<port>`) are rejected.
pub const REDIRECT_PORT: u16 = 8765;

/// How long the browser flow waits for the redirect before giving up.
const BROWSER_LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

/// Upper bound on the device-code wait, whatever `expires_in` the
/// Keycloak sends (its default device-code lifespan is 10 minutes).
const DEVICE_LOGIN_MAX_WAIT: Duration = Duration::from_secs(30 * 60);

/// Longest request line the redirect listener reads; the real redirect
/// is well under 1 KiB.
const MAX_REQUEST_LINE: u64 = 8 * 1024;

/// Per-request timeout for every call to the Keycloak.
const HTTP_TIMEOUT: Duration = Duration::from_secs(30);

/// A cached token is treated as expired this long before its real
/// expiry, so it does not lapse between the check and its use.
const EXPIRY_GRACE_SECS: i64 = 10;

/// Page shown in the browser once the redirect has been received.
const SUCCESS_HTML: &str = "<!DOCTYPE html><html><head><title>manta</title></head>\
<body><h2>Authentication successful</h2>\
<p>You can close this window and return to the terminal.</p></body></html>";

/// Where and how to authenticate. Built from `cli.toml` (with the
/// defaults above) by [`crate::common::authentication`].
#[derive(Debug, Clone, Copy)]
pub struct OidcSettings<'a> {
  /// Keycloak realm issuer URL (discovery lives under
  /// `<issuer>/.well-known/openid-configuration`).
  pub issuer_url: &'a str,
  /// Public client id registered for manta in the realm.
  pub client_id: &'a str,
  /// Force the device-code flow instead of opening a browser.
  pub headless: bool,
}

/// Tokens returned by the Keycloak, persisted in the token cache file.
#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct TokenStore {
  /// Bearer token sent to manta-server.
  pub access_token: String,
  /// Refresh token, used by [`refresh`] once the access token expires.
  pub refresh_token: Option<String>,
  /// ID token from the login; kept for completeness, not sent anywhere.
  pub id_token: Option<String>,
  /// Access-token expiry as seconds since the Unix epoch. `None` means
  /// unknown, which [`TokenStore::is_expired`] treats as expired.
  pub expires_at: Option<i64>,
}

impl std::fmt::Debug for TokenStore {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("TokenStore")
      .field("access_token", &"<REDACTED>")
      .field(
        "refresh_token",
        &self.refresh_token.as_ref().map(|_| "<REDACTED>"),
      )
      .field("id_token", &self.id_token.as_ref().map(|_| "<REDACTED>"))
      .field("expires_at", &self.expires_at)
      .finish()
  }
}

impl TokenStore {
  /// `true` once the access token is within [`EXPIRY_GRACE_SECS`] of
  /// its expiry (or its expiry is unknown).
  pub fn is_expired(&self) -> bool {
    self.is_expired_at(now_secs())
  }

  fn is_expired_at(&self, now: i64) -> bool {
    self
      .expires_at
      .is_none_or(|exp| now.saturating_add(EXPIRY_GRACE_SECS) >= exp)
  }

  /// Read a token store from `path`.
  ///
  /// # Errors
  ///
  /// The file cannot be read or is not a valid token store.
  pub fn load(path: &Path) -> Result<Self> {
    let content = std::fs::read_to_string(path).with_context(|| {
      format!("could not read token cache '{}'", path.display())
    })?;
    serde_json::from_str(&content)
      .with_context(|| format!("token cache '{}' is corrupt", path.display()))
  }

  /// Write the token store to `path` with `0600` permissions, creating
  /// the parent directory (`0700`) when missing.
  ///
  /// The JSON goes to a fresh `0600` temp file that is then renamed over
  /// `path`, so a concurrent reader or an interrupted write never sees
  /// a torn cache, and an existing cache with looser permissions ends
  /// up `0600` too.
  ///
  /// # Errors
  ///
  /// The directory or file cannot be created or written.
  pub fn save(&self, path: &Path) -> Result<()> {
    use std::io::Write;
    use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};

    if let Some(dir) = path.parent() {
      std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(dir)
        .with_context(|| format!("could not create '{}'", dir.display()))?;
    }
    let json = serde_json::to_string_pretty(self)?;
    let mut tmp_name = std::ffi::OsString::from(".");
    tmp_name.push(path.file_name().unwrap_or_default());
    tmp_name.push(format!(".{}.tmp", std::process::id()));
    let tmp = path.with_file_name(tmp_name);

    // A leftover from a crashed run with the same pid would make
    // `create_new` fail.
    let _ = std::fs::remove_file(&tmp);
    let write = || -> std::io::Result<()> {
      let mut file = std::fs::File::options()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&tmp)?;
      file.write_all(json.as_bytes())?;
      file.sync_all()?;
      std::fs::rename(&tmp, path)
    };
    write()
      .inspect_err(|_| {
        let _ = std::fs::remove_file(&tmp);
      })
      .with_context(|| {
        format!("could not write token cache '{}'", path.display())
      })
  }

  fn from_token_response(resp: RawTokenResponse) -> Self {
    Self {
      access_token: resp.access_token,
      refresh_token: resp.refresh_token,
      id_token: resp.id_token,
      expires_at: resp.expires_in.map(expires_at),
    }
  }
}

/// Interactive login: the browser flow, or the device-code flow when
/// headless (see the module docs for when that applies). A browser
/// that cannot be opened also falls back to the device-code flow.
///
/// # Errors
///
/// Discovery or any Keycloak call fails, the user denies access, or
/// the login times out.
pub async fn login(settings: &OidcSettings<'_>) -> Result<TokenStore> {
  if settings.headless || browser_unreachable(|v| std::env::var_os(v).is_some())
  {
    tracing::debug!("headless login: using the device-code flow");
    return login_via_device_code(settings).await;
  }
  match login_via_browser(settings).await {
    Err(e) if e.is::<BrowserUnavailable>() => {
      tracing::debug!("{e}; falling back to the device-code flow");
      login_via_device_code(settings).await
    }
    other => other,
  }
}

/// Exchange `refresh_token` for a fresh token set.
///
/// # Errors
///
/// Discovery fails, or the Keycloak rejects the refresh token (it
/// expired, was revoked, or the session ended).
pub async fn refresh(
  settings: &OidcSettings<'_>,
  refresh_token: &str,
) -> Result<TokenStore> {
  let http = http_client()?;
  let discovery = discover(&http, settings.issuer_url).await?;
  let resp = http
    .post(&discovery.token_endpoint)
    .form(&[
      ("grant_type", "refresh_token"),
      ("client_id", settings.client_id),
      ("refresh_token", refresh_token),
    ])
    .send()
    .await
    .context("failed to send refresh-token request")?;
  let status = resp.status();
  let body = resp.bytes().await?;
  if !status.is_success() {
    bail!("token refresh rejected: {}", oauth_error_message(&body));
  }
  let token: RawTokenResponse = serde_json::from_slice(&body)
    .context("failed to parse refresh-token response")?;
  let mut store = TokenStore::from_token_response(token);
  // Keycloak may omit the refresh token when it is not rotated; keep
  // the one we have.
  if store.refresh_token.is_none() {
    store.refresh_token = Some(refresh_token.to_string());
  }
  Ok(store)
}

/// The browser could not be opened; [`login`] falls back to the
/// device-code flow.
#[derive(Debug)]
struct BrowserUnavailable(String);

impl std::fmt::Display for BrowserUnavailable {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "could not open a browser: {}", self.0)
  }
}

impl std::error::Error for BrowserUnavailable {}

async fn login_via_browser(settings: &OidcSettings<'_>) -> Result<TokenStore> {
  let http = http_client()?;
  let oidc_http = OidcHttp(http.clone());
  let issuer_url = IssuerUrl::new(settings.issuer_url.to_string())
    .context("invalid oidc_issuer_url")?;
  let provider_metadata =
    CoreProviderMetadata::discover_async(issuer_url, &oidc_http)
      .await
      .context("OIDC discovery failed")?;
  let client = CoreClient::from_provider_metadata(
    provider_metadata,
    ClientId::new(settings.client_id.to_string()),
    None,
  )
  .set_redirect_uri(RedirectUrl::new(format!(
    "http://localhost:{REDIRECT_PORT}"
  ))?);

  let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
  let (auth_url, csrf_token, nonce) = client
    .authorize_url(
      AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
      CsrfToken::new_random,
      Nonce::new_random,
    )
    .add_scope(Scope::new("openid".to_string()))
    .set_pkce_challenge(pkce_challenge)
    .url();

  // Bind before opening the browser so the redirect cannot race us.
  let listener = TcpListener::bind(("127.0.0.1", REDIRECT_PORT))
    .await
    .with_context(|| {
      format!(
        "could not listen on localhost:{REDIRECT_PORT} for the login redirect"
      )
    })?;

  webbrowser::open(auth_url.as_str())
    .map_err(|e| BrowserUnavailable(e.to_string()))?;
  eprintln!(
    "Opened a browser to log in to the CSCS Keycloak. If it did not appear, open:\n  {auth_url}"
  );

  let code = tokio::time::timeout(
    BROWSER_LOGIN_TIMEOUT,
    wait_for_redirect(&listener, csrf_token.secret()),
  )
  .await
  .map_err(|_| anyhow!("timed out waiting for the browser login"))??;

  let token_response = client
    .exchange_code(AuthorizationCode::new(code))?
    .set_pkce_verifier(pkce_verifier)
    .request_async(&oidc_http)
    .await
    .context("authorization-code exchange failed")?;

  // Nonce check (replay protection) and access-token hash check (the
  // access token was not substituted for another one).
  let id_token = token_response
    .id_token()
    .ok_or_else(|| anyhow!("Keycloak did not return an ID token"))?;
  let verifier = client.id_token_verifier();
  let claims = id_token
    .claims(&verifier, &nonce)
    .context("ID token verification failed")?;
  if let Some(expected) = claims.access_token_hash() {
    let actual = AccessTokenHash::from_token(
      token_response.access_token(),
      id_token.signing_alg()?,
      id_token.signing_key(&verifier)?,
    )?;
    if actual != *expected {
      bail!("access token does not match the ID token's at_hash");
    }
  }

  Ok(TokenStore {
    access_token: token_response.access_token().secret().clone(),
    refresh_token: token_response.refresh_token().map(|t| t.secret().clone()),
    id_token: Some(id_token.to_string()),
    expires_at: token_response.expires_in().map(|d| expires_at(d.as_secs())),
  })
}

/// Accept connections on `listener` until one carries the OIDC
/// redirect, answer it, and return the authorization code. Any other
/// request (e.g. a browser's `/favicon.ico`, or one without our
/// `state`) gets a `404` and does not end the login; neither does a
/// connection that fails or sends an unreadable request line.
async fn wait_for_redirect(
  listener: &TcpListener,
  expected_state: &str,
) -> Result<String> {
  loop {
    let (mut stream, _) = listener.accept().await?;
    let mut request_line = String::new();
    let read = BufReader::new((&mut stream).take(MAX_REQUEST_LINE))
      .read_line(&mut request_line)
      .await;
    if let Err(e) = read {
      tracing::debug!("ignoring unreadable request on the redirect port: {e}");
      continue;
    }

    let outcome = parse_redirect(&request_line, expected_state);
    let (status, content_type, body) = match &outcome {
      Some(Ok(_)) => ("200 OK", "text/html", SUCCESS_HTML.to_string()),
      Some(Err(e)) => (
        "400 Bad Request",
        "text/plain; charset=utf-8",
        format!("Authentication failed: {e}"),
      ),
      None => ("404 Not Found", "text/plain; charset=utf-8", String::new()),
    };
    let response = format!(
      "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
      body.len()
    );
    // Best effort: the code is already in hand even if the reply fails.
    let _ = stream.write_all(response.as_bytes()).await;

    if let Some(result) = outcome {
      return result;
    }
  }
}

/// Parse the request line of a redirect (`GET /?state=..&code=.. HTTP/1.1`).
/// `None` when it is not our redirect: no `state`, or a different one.
/// Otherwise the code, or the error the Keycloak returned (which, per
/// RFC 6749 §4.1.2.1, also carries our `state`). Checking `state`
/// first means no other local client can end the login.
fn parse_redirect(
  request_line: &str,
  expected_state: &str,
) -> Option<Result<String>> {
  let target = request_line.split_whitespace().nth(1)?;
  let url =
    openidconnect::url::Url::parse(&format!("http://localhost{target}"))
      .ok()?;
  let param = |name: &str| {
    url
      .query_pairs()
      .find(|(k, _)| k == name)
      .map(|(_, v)| v.into_owned())
  };

  if param("state")? != expected_state {
    return None;
  }
  if let Some(error) = param("error") {
    let description = param("error_description").unwrap_or(error);
    return Some(Err(anyhow!("Keycloak returned an error: {description}")));
  }
  Some(param("code").ok_or_else(|| anyhow!("login redirect carried no code")))
}

async fn login_via_device_code(
  settings: &OidcSettings<'_>,
) -> Result<TokenStore> {
  let http = http_client()?;
  let discovery = discover(&http, settings.issuer_url).await?;
  let device_endpoint =
    discovery.device_authorization_endpoint.ok_or_else(|| {
      anyhow!(
        "the Keycloak at {} does not offer the device authorization flow",
        settings.issuer_url
      )
    })?;

  let resp = http
    .post(&device_endpoint)
    .form(&[("client_id", settings.client_id), ("scope", "openid")])
    .send()
    .await
    .context("failed to request a device code")?;
  let status = resp.status();
  let body = resp.bytes().await?;
  if !status.is_success() {
    bail!(
      "device authorization rejected: {}",
      oauth_error_message(&body)
    );
  }
  let device: DeviceAuthResponse = serde_json::from_slice(&body)
    .context("failed to parse the device authorization response")?;

  eprintln!();
  if let Some(uri) = &device.verification_uri_complete {
    eprintln!("To log in to the CSCS Keycloak, open this URL in a browser:");
    eprintln!("  {uri}");
  } else {
    eprintln!("To log in to the CSCS Keycloak, open this URL in a browser:");
    eprintln!("  {}", device.verification_uri);
    eprintln!("and enter the code: {}", device.user_code);
  }
  eprintln!();
  eprintln!("Waiting for authentication...");

  let mut interval = Duration::from_secs(device.interval.unwrap_or(5));
  let deadline =
    tokio::time::Instant::now() + device_login_window(device.expires_in);

  loop {
    // Never sleep past the deadline, however long the server's interval.
    let remaining =
      deadline.saturating_duration_since(tokio::time::Instant::now());
    tokio::time::sleep(interval.min(remaining)).await;
    if tokio::time::Instant::now() >= deadline {
      bail!("device code expired before the login completed");
    }

    let resp = http
      .post(&discovery.token_endpoint)
      .form(&[
        ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
        ("client_id", settings.client_id),
        ("device_code", device.device_code.as_str()),
      ])
      .send()
      .await
      .context("failed to poll the token endpoint")?;
    let status = resp.status();
    let body = resp.bytes().await?;

    if status.is_success() {
      let token: RawTokenResponse = serde_json::from_slice(&body)
        .context("failed to parse the token response")?;
      eprintln!("Authentication successful.");
      return Ok(TokenStore::from_token_response(token));
    }

    match device_poll_step(&body)? {
      PollStep::Continue => {}
      PollStep::SlowDown => {
        interval = interval.saturating_add(Duration::from_secs(5));
      }
    }
  }
}

/// How long to wait for a device-code login: the server's
/// `expires_in`, capped at [`DEVICE_LOGIN_MAX_WAIT`].
fn device_login_window(expires_in_secs: u64) -> Duration {
  Duration::from_secs(expires_in_secs).min(DEVICE_LOGIN_MAX_WAIT)
}

/// What to do after a non-success poll of the token endpoint.
#[derive(Debug, PartialEq, Eq)]
enum PollStep {
  Continue,
  SlowDown,
}

/// Classify a device-flow token-endpoint error body (RFC 8628 §3.5).
fn device_poll_step(body: &[u8]) -> Result<PollStep> {
  let err: OAuthError = serde_json::from_slice(body).with_context(|| {
    format!(
      "unexpected token endpoint response: {}",
      String::from_utf8_lossy(body)
    )
  })?;
  match err.error.as_str() {
    "authorization_pending" => Ok(PollStep::Continue),
    "slow_down" => Ok(PollStep::SlowDown),
    "expired_token" => bail!("device code expired; please try again"),
    "access_denied" => bail!("login was denied"),
    _ => bail!(
      "authentication error: {}",
      err.error_description.unwrap_or(err.error)
    ),
  }
}

/// `true` when a browser started on this host could not reach the
/// user: inside an SSH session, or (outside macOS, whose `open` needs
/// no display) without `DISPLAY`/`WAYLAND_DISPLAY`. The display check
/// catches `ssh host` + `sudo -i`, which drops the `SSH_*` variables,
/// on a server where `webbrowser` would otherwise "succeed" in starting
/// an invisible browser. `is_set` reports whether an env var is set.
fn browser_unreachable(is_set: impl Fn(&str) -> bool) -> bool {
  let in_ssh = is_set("SSH_CONNECTION") || is_set("SSH_TTY");
  let no_display = cfg!(not(target_os = "macos"))
    && !is_set("DISPLAY")
    && !is_set("WAYLAND_DISPLAY");
  in_ssh || no_display
}

pub(crate) fn now_secs() -> i64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// Absolute expiry for a Keycloak-supplied `expires_in`, saturating
/// instead of overflowing on absurd values.
fn expires_at(expires_in_secs: u64) -> i64 {
  now_secs().saturating_add(i64::try_from(expires_in_secs).unwrap_or(i64::MAX))
}

fn http_client() -> Result<reqwest::Client> {
  reqwest::Client::builder()
    // OIDC endpoints never need redirects; following one could leak
    // a code or token to an unexpected host.
    .redirect(reqwest::redirect::Policy::none())
    .timeout(HTTP_TIMEOUT)
    .build()
    .context("failed to build the Keycloak HTTP client")
}

/// The endpoints manta needs from the realm's discovery document.
#[derive(Deserialize, Debug)]
struct Discovery {
  token_endpoint: String,
  device_authorization_endpoint: Option<String>,
}

async fn discover(
  http: &reqwest::Client,
  issuer_url: &str,
) -> Result<Discovery> {
  let url = format!(
    "{}/.well-known/openid-configuration",
    issuer_url.trim_end_matches('/')
  );
  http
    .get(&url)
    .send()
    .await
    .and_then(reqwest::Response::error_for_status)
    .with_context(|| {
      format!("failed to fetch OIDC discovery document '{url}'")
    })?
    .json()
    .await
    .context("failed to parse the OIDC discovery document")
}

#[derive(Deserialize, Debug)]
struct RawTokenResponse {
  access_token: String,
  refresh_token: Option<String>,
  id_token: Option<String>,
  expires_in: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct DeviceAuthResponse {
  device_code: String,
  user_code: String,
  verification_uri: String,
  verification_uri_complete: Option<String>,
  expires_in: u64,
  interval: Option<u64>,
}

#[derive(Deserialize, Debug)]
struct OAuthError {
  error: String,
  error_description: Option<String>,
}

/// Best-effort human message out of an OAuth error body.
fn oauth_error_message(body: &[u8]) -> String {
  serde_json::from_slice::<OAuthError>(body).map_or_else(
    |_| String::from_utf8_lossy(body).into_owned(),
    |e| e.error_description.unwrap_or(e.error),
  )
}

/// Adapter letting `openidconnect` issue requests through the CLI's
/// reqwest 0.13 (openidconnect's own reqwest integration is built on
/// 0.12).
struct OidcHttp(reqwest::Client);

impl<'c> openidconnect::AsyncHttpClient<'c> for OidcHttp {
  type Error = std::io::Error;
  type Future = std::pin::Pin<
    Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + Send + 'c>,
  >;

  fn call(&'c self, request: HttpRequest) -> Self::Future {
    Box::pin(async move {
      let (parts, body) = request.into_parts();
      let resp = self
        .0
        .request(parts.method, parts.uri.to_string())
        .headers(parts.headers)
        .body(body)
        .send()
        .await
        .map_err(std::io::Error::other)?;
      let status = resp.status();
      let headers = resp.headers().clone();
      let body = resp.bytes().await.map_err(std::io::Error::other)?;
      let mut response = HttpResponse::new(body.to_vec());
      *response.status_mut() = status;
      *response.headers_mut() = headers;
      Ok(response)
    })
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::os::unix::fs::PermissionsExt;

  fn store(expires_at: Option<i64>) -> TokenStore {
    TokenStore {
      access_token: "access".to_string(),
      refresh_token: Some("refresh".to_string()),
      id_token: None,
      expires_at,
    }
  }

  #[test]
  fn expiry_honours_grace_period() {
    let now = 1_000_000;
    assert!(!store(Some(now + EXPIRY_GRACE_SECS + 1)).is_expired_at(now));
    assert!(store(Some(now + EXPIRY_GRACE_SECS)).is_expired_at(now));
    assert!(store(Some(now - 1)).is_expired_at(now));
  }

  #[test]
  fn unknown_expiry_counts_as_expired() {
    assert!(store(None).is_expired_at(0));
  }

  #[test]
  fn huge_expires_in_saturates_instead_of_overflowing() {
    assert_eq!(expires_at(u64::MAX), i64::MAX);
    assert!(!store(Some(expires_at(u64::MAX))).is_expired_at(now_secs()));
  }

  #[test]
  fn expiry_check_near_i64_max_does_not_overflow() {
    assert!(store(Some(i64::MAX)).is_expired_at(i64::MAX));
  }

  #[test]
  fn huge_expires_in_from_token_endpoint_saturates() {
    let token: RawTokenResponse = serde_json::from_str(
      r#"{"access_token":"a","expires_in":18446744073709551615}"#,
    )
    .unwrap();
    let store = TokenStore::from_token_response(token);
    assert_eq!(store.expires_at, Some(i64::MAX));
  }

  #[test]
  fn negative_expires_in_from_token_endpoint_is_rejected() {
    assert!(
      serde_json::from_str::<RawTokenResponse>(
        r#"{"access_token":"a","expires_in":-5}"#
      )
      .is_err()
    );
  }

  #[test]
  fn browser_unreachable_detection() {
    let env = |vars: &'static [&'static str]| move |v: &str| vars.contains(&v);
    assert!(browser_unreachable(env(&["SSH_TTY", "DISPLAY"])));
    assert!(browser_unreachable(env(&["SSH_CONNECTION", "DISPLAY"])));
    assert!(!browser_unreachable(env(&["DISPLAY"])));
    assert!(!browser_unreachable(env(&["WAYLAND_DISPLAY"])));
    // No SSH variables and no display (e.g. after `sudo -i`).
    assert_eq!(browser_unreachable(env(&[])), !cfg!(target_os = "macos"));
  }

  #[test]
  fn save_then_load_roundtrips_with_private_permissions() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("token.json");
    let original = store(Some(42));

    original.save(&path).unwrap();
    assert_eq!(TokenStore::load(&path).unwrap(), original);

    let file_mode =
      std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(file_mode, 0o600);
    let dir_mode = std::fs::metadata(path.parent().unwrap())
      .unwrap()
      .permissions()
      .mode()
      & 0o777;
    assert_eq!(dir_mode, 0o700);
  }

  #[test]
  fn save_tightens_a_looser_existing_cache_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("token.json");
    std::fs::write(&path, "{}").unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
      .unwrap();

    store(Some(7)).save(&path).unwrap();

    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    assert_eq!(TokenStore::load(&path).unwrap().expires_at, Some(7));
    let entries: Vec<_> = std::fs::read_dir(dir.path())
      .unwrap()
      .map(|e| e.unwrap().file_name())
      .collect();
    assert_eq!(entries, ["token.json"]);
  }

  #[test]
  fn save_overwrites_existing_cache() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("token.json");
    store(Some(1)).save(&path).unwrap();
    store(Some(2)).save(&path).unwrap();
    assert_eq!(TokenStore::load(&path).unwrap().expires_at, Some(2));
  }

  #[test]
  fn debug_output_redacts_tokens() {
    let rendered = format!("{:?}", store(Some(1)));
    assert!(!rendered.contains("access\""), "got: {rendered}");
    assert!(!rendered.contains("\"refresh\""), "got: {rendered}");
    assert!(rendered.contains("<REDACTED>"));
  }

  #[test]
  fn redirect_with_matching_state_yields_code() {
    let line = "GET /?state=abc&session_state=x&code=the-code HTTP/1.1\r\n";
    assert_eq!(parse_redirect(line, "abc").unwrap().unwrap(), "the-code");
  }

  #[test]
  fn redirect_with_wrong_state_is_ignored() {
    let line = "GET /?state=evil&code=the-code HTTP/1.1\r\n";
    assert!(parse_redirect(line, "abc").is_none());
  }

  #[test]
  fn error_without_our_state_is_ignored() {
    let line = "GET /?error=access_denied HTTP/1.1\r\n";
    assert!(parse_redirect(line, "abc").is_none());
    let line = "GET /?state=evil&error=access_denied HTTP/1.1\r\n";
    assert!(parse_redirect(line, "abc").is_none());
  }

  #[test]
  fn redirect_error_is_surfaced() {
    let line = "GET /?state=abc&error=access_denied&error_description=User%20said%20no HTTP/1.1\r\n";
    let err = parse_redirect(line, "abc").unwrap().unwrap_err();
    assert!(err.to_string().contains("User said no"), "got: {err}");
  }

  #[test]
  fn unrelated_request_is_not_the_redirect() {
    assert!(parse_redirect("GET /favicon.ico HTTP/1.1\r\n", "abc").is_none());
  }

  /// Send `request` on a fresh connection and return the response's
  /// status line.
  async fn send_to(addr: std::net::SocketAddr, request: &[u8]) -> String {
    let mut stream = tokio::net::TcpStream::connect(addr).await.unwrap();
    stream.write_all(request).await.unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).await.unwrap();
    response.lines().next().unwrap_or_default().to_string()
  }

  #[tokio::test]
  async fn redirect_listener_survives_stray_requests() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let client = tokio::spawn(async move {
      let not_found = "HTTP/1.1 404 Not Found";
      assert_eq!(
        send_to(addr, b"GET /favicon.ico HTTP/1.1\r\n").await,
        not_found
      );
      assert_eq!(
        send_to(addr, b"GET /?error=x HTTP/1.1\r\n").await,
        not_found
      );
      assert_eq!(
        send_to(addr, b"GET /?state=evil&code=c HTTP/1.1\r\n").await,
        not_found
      );
      // Not UTF-8: the read fails and the connection is dropped.
      let mut bad = tokio::net::TcpStream::connect(addr).await.unwrap();
      bad.write_all(b"\xff\xfe\r\n").await.unwrap();
      drop(bad);
      // No newline and kept open: only the length cap ends the read.
      let mut long = tokio::net::TcpStream::connect(addr).await.unwrap();
      long.write_all(&[b'a'; 16 * 1024]).await.unwrap();
      let status =
        send_to(addr, b"GET /?state=abc&code=the-code HTTP/1.1\r\n").await;
      drop(long);
      status
    });

    let code = tokio::time::timeout(
      Duration::from_secs(10),
      wait_for_redirect(&listener, "abc"),
    )
    .await
    .expect("redirect listener hung")
    .unwrap();
    assert_eq!(code, "the-code");
    assert_eq!(client.await.unwrap(), "HTTP/1.1 200 OK");
  }

  #[test]
  fn device_poll_steps() {
    assert_eq!(
      device_poll_step(br#"{"error":"authorization_pending"}"#).unwrap(),
      PollStep::Continue
    );
    assert_eq!(
      device_poll_step(br#"{"error":"slow_down"}"#).unwrap(),
      PollStep::SlowDown
    );
    for (body, needle) in [
      (&br#"{"error":"expired_token"}"#[..], "expired"),
      (&br#"{"error":"access_denied"}"#[..], "denied"),
      (
        &br#"{"error":"invalid_client","error_description":"bad client"}"#[..],
        "bad client",
      ),
    ] {
      let err = device_poll_step(body).unwrap_err().to_string();
      assert!(err.contains(needle), "got: {err}");
    }
  }

  // ---- flows against a stub Keycloak ----

  use std::sync::Arc;
  use std::sync::atomic::{AtomicUsize, Ordering};
  use tokio::io::AsyncReadExt;

  /// Minimal Keycloak stand-in on an ephemeral port. `handler` gets
  /// `(base_url, path, request_body)` and returns `(status, json)`.
  /// Returns the base URL, which doubles as the issuer URL.
  async fn stub_keycloak<F>(handler: F) -> String
  where
    F: Fn(&str, &str, &str) -> (u16, String) + Send + Sync + 'static,
  {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let handler = Arc::new(handler);
    let served_base = base.clone();
    tokio::spawn(async move {
      while let Ok((stream, _)) = listener.accept().await {
        let handler = Arc::clone(&handler);
        let base = served_base.clone();
        tokio::spawn(async move {
          let mut reader = BufReader::new(stream);
          let mut line = String::new();
          reader.read_line(&mut line).await.unwrap();
          let path = line.split_whitespace().nth(1).unwrap_or("").to_string();
          let mut len = 0;
          loop {
            let mut header = String::new();
            reader.read_line(&mut header).await.unwrap();
            if header == "\r\n" || header.is_empty() {
              break;
            }
            if let Some(v) =
              header.to_ascii_lowercase().strip_prefix("content-length:")
            {
              len = v.trim().parse().unwrap();
            }
          }
          let mut body = vec![0; len];
          reader.read_exact(&mut body).await.unwrap();
          let (status, json) =
            handler(&base, &path, &String::from_utf8_lossy(&body));
          let out = format!(
            "HTTP/1.1 {status} X\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{json}",
            json.len()
          );
          reader.into_inner().write_all(out.as_bytes()).await.unwrap();
        });
      }
    });
    base
  }

  fn discovery_json(base: &str, with_device: bool) -> String {
    let device = if with_device {
      format!(r#","device_authorization_endpoint":"{base}/device""#)
    } else {
      String::new()
    };
    format!(r#"{{"token_endpoint":"{base}/token"{device}}}"#)
  }

  fn stub_settings(issuer_url: &str) -> OidcSettings<'_> {
    OidcSettings {
      issuer_url,
      client_id: "test-client",
      headless: true,
    }
  }

  #[tokio::test]
  async fn refresh_exchanges_the_refresh_token() {
    let issuer = stub_keycloak(|base, path, body| match path {
      "/.well-known/openid-configuration" => (200, discovery_json(base, false)),
      "/token"
        if body.contains("grant_type=refresh_token")
          && body.contains("refresh_token=old-refresh")
          && body.contains("client_id=test-client") =>
      {
        (
          200,
          r#"{"access_token":"new-access","refresh_token":"new-refresh","expires_in":300}"#
            .to_string(),
        )
      }
      _ => (404, "{}".to_string()),
    })
    .await;

    let store = refresh(&stub_settings(&issuer), "old-refresh")
      .await
      .unwrap();
    assert_eq!(store.access_token, "new-access");
    assert_eq!(store.refresh_token.as_deref(), Some("new-refresh"));
    assert!(!store.is_expired());
  }

  #[tokio::test]
  async fn refresh_keeps_refresh_token_when_not_rotated() {
    let issuer = stub_keycloak(|base, path, _| match path {
      "/.well-known/openid-configuration" => (200, discovery_json(base, false)),
      "/token" => (
        200,
        r#"{"access_token":"new-access","expires_in":300}"#.to_string(),
      ),
      _ => (404, "{}".to_string()),
    })
    .await;

    let store = refresh(&stub_settings(&issuer), "old-refresh")
      .await
      .unwrap();
    assert_eq!(store.refresh_token.as_deref(), Some("old-refresh"));
  }

  #[tokio::test]
  async fn refresh_rejection_is_an_error() {
    let issuer = stub_keycloak(|base, path, _| match path {
      "/.well-known/openid-configuration" => (200, discovery_json(base, false)),
      "/token" => (
        400,
        r#"{"error":"invalid_grant","error_description":"Session not active"}"#
          .to_string(),
      ),
      _ => (404, "{}".to_string()),
    })
    .await;

    let err = refresh(&stub_settings(&issuer), "old-refresh")
      .await
      .unwrap_err()
      .to_string();
    assert!(err.contains("Session not active"), "got: {err}");
  }

  #[tokio::test]
  async fn headless_login_polls_device_flow_until_authorized() {
    let polls = Arc::new(AtomicUsize::new(0));
    let seen = Arc::clone(&polls);
    let issuer = stub_keycloak(move |base, path, body| match path {
      "/.well-known/openid-configuration" => (200, discovery_json(base, true)),
      "/device" => (
        200,
        r#"{"device_code":"dev-code","user_code":"ABCD-EFGH","verification_uri":"https://kc/device","expires_in":60,"interval":0}"#
          .to_string(),
      ),
      "/token" if body.contains("device_code=dev-code") => {
        if seen.fetch_add(1, Ordering::SeqCst) == 0 {
          (400, r#"{"error":"authorization_pending"}"#.to_string())
        } else {
          (
            200,
            r#"{"access_token":"device-access","refresh_token":"device-refresh","expires_in":300}"#
              .to_string(),
          )
        }
      }
      _ => (404, "{}".to_string()),
    })
    .await;

    let store = login(&stub_settings(&issuer)).await.unwrap();
    assert_eq!(store.access_token, "device-access");
    assert_eq!(store.refresh_token.as_deref(), Some("device-refresh"));
    assert_eq!(polls.load(Ordering::SeqCst), 2);
  }

  #[test]
  fn device_login_window_is_capped() {
    assert_eq!(device_login_window(60), Duration::from_secs(60));
    assert_eq!(device_login_window(u64::MAX), DEVICE_LOGIN_MAX_WAIT);
  }

  #[tokio::test]
  async fn device_flow_huge_interval_still_gives_up_at_the_deadline() {
    let issuer = stub_keycloak(|base, path, _| match path {
      "/.well-known/openid-configuration" => (200, discovery_json(base, true)),
      "/device" => (
        200,
        r#"{"device_code":"dev-code","user_code":"ABCD-EFGH","verification_uri":"https://kc/device","expires_in":1,"interval":18446744073709551615}"#
          .to_string(),
      ),
      "/token" => (400, r#"{"error":"authorization_pending"}"#.to_string()),
      _ => (404, "{}".to_string()),
    })
    .await;

    let err = tokio::time::timeout(
      Duration::from_secs(10),
      login(&stub_settings(&issuer)),
    )
    .await
    .expect("device login ignored its deadline")
    .unwrap_err()
    .to_string();
    assert!(err.contains("device code expired"), "got: {err}");
  }

  #[tokio::test]
  async fn device_flow_without_device_endpoint_is_an_error() {
    let issuer = stub_keycloak(|base, path, _| match path {
      "/.well-known/openid-configuration" => (200, discovery_json(base, false)),
      _ => (404, "{}".to_string()),
    })
    .await;

    let err = login(&stub_settings(&issuer))
      .await
      .unwrap_err()
      .to_string();
    assert!(err.contains("device authorization"), "got: {err}");
  }
}
