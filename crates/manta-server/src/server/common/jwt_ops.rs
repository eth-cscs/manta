//! JWT claim extractors used by the audit and authorization paths.
//!
//! Decodes a bearer token (with or without the `Bearer ` prefix),
//! tolerates both URL-safe and standard Base64 encodings, and
//! returns named claims as `String`. All failures map to
//! [`MantaError::JwtMalformed`] with a structured message; the
//! HTTP layer maps that to a 401.
//!
//! # Security caveat
//!
//! These helpers **do not verify the JWT signature**. Claims are
//! extracted on trust. The signature is verified upstream by the
//! backend (CSM / OpenCHAMI) on every call that uses the token, so a
//! forged token with `pa_admin` in `realm_access.roles` will still be
//! rejected at the first backend round-trip — but the in-process
//! `is_user_admin` short-circuit means any code path that returns
//! before the backend call is reached (e.g. a future cached path or
//! a handler that only checks the local roles) would skip every
//! group-access check.
//!
//! TODO: verify the signature locally against the per-site Keycloak
//! JWKS, cached in `ServerState` with refresh on `kid` miss. Tracked
//! as a follow-up because it requires JWKS fetching, key rotation,
//! and a per-site cache. For now treat `is_user_admin` as advisory:
//! never grant a privilege based on it alone without a follow-up
//! call that hits the backend.

use base64::prelude::*;
use serde_json::Value;

use manta_shared::common::error::MantaError;

use crate::service::authorization::PA_ADMIN;

fn get_claims_from_jwt_token(token: &str) -> Result<Value, MantaError> {
  // Handle both "Bearer <token>" and bare "<token>" formats
  let jwt_body = token.split(' ').nth(1).unwrap_or(token);

  let base64_claims = jwt_body.split('.').nth(1).ok_or_else(|| {
    MantaError::JwtMalformed(
      "expected header.payload.signature format".to_string(),
    )
  })?;

  let claims_u8 = BASE64_URL_SAFE_NO_PAD
    .decode(base64_claims)
    .or_else(|_| BASE64_STANDARD.decode(base64_claims))
    .map_err(|e| {
      MantaError::JwtMalformed(format!("could not decode claims: {e}"))
    })?;

  let claims_str = std::str::from_utf8(&claims_u8).map_err(|e| {
    MantaError::JwtMalformed(format!("claims are not valid UTF-8: {e}"))
  })?;

  Ok(serde_json::from_str::<Value>(claims_str)?)
}

/// Extract the `name` claim from a JWT token.
///
/// Returns `"MISSING"` if the claim is absent.
pub fn get_name(token: &str) -> Result<String, MantaError> {
  let jwt_claims = get_claims_from_jwt_token(token)?;

  let jwt_name = jwt_claims.get("name").and_then(Value::as_str);

  match jwt_name {
    Some(name) => Ok(name.to_string()),
    None => Ok("MISSING".to_string()),
  }
}

/// Extract the `preferred_username` claim from a JWT token.
///
/// Returns `"MISSING"` if the claim is absent.
pub fn get_preferred_username(token: &str) -> Result<String, MantaError> {
  let jwt_claims = get_claims_from_jwt_token(token)?;

  let jwt_preferred_username =
    jwt_claims.get("preferred_username").and_then(Value::as_str);

  match jwt_preferred_username {
    Some(name) => Ok(name.to_string()),
    None => Ok("MISSING".to_string()),
  }
}

/// Returns the list of available HSM groups in JWT user token. The list is filtered and system HSM
/// groups (eg alps, alpsm, alpse, etc)
pub fn get_roles(token: &str) -> Result<Vec<String>, MantaError> {
  // If JWT does not have `/realm_access/roles` claim, then we will assume, user is admin
  Ok(
    get_claims_from_jwt_token(token)?
      .pointer("/realm_access/roles")
      .unwrap_or(&serde_json::json!([]))
      .as_array()
      .cloned()
      .unwrap_or_default()
      .iter()
      .filter_map(|role_value| role_value.as_str().map(str::to_string))
      .collect(),
  )
}

/// This function will return true if the user is an admin, otherwise false
pub fn is_user_admin(token: &str) -> bool {
  let roles_rslt = get_roles(token);

  roles_rslt.is_ok_and(|roles| roles.contains(&PA_ADMIN.to_string()))
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Build a fake JWT with the given JSON payload.
  fn make_jwt(payload: &serde_json::Value) -> String {
    let header = BASE64_URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
    let body = BASE64_URL_SAFE_NO_PAD.encode(payload.to_string());
    format!("{header}.{body}.sig")
  }

  // ---- get_name ----

  #[test]
  fn get_name_present() {
    let token = make_jwt(&serde_json::json!({
      "name": "Alice Smith",
      "preferred_username": "alice"
    }));
    assert_eq!(get_name(&token).unwrap(), "Alice Smith");
  }

  #[test]
  fn get_name_missing_returns_missing() {
    let token = make_jwt(&serde_json::json!({
      "preferred_username": "alice"
    }));
    assert_eq!(get_name(&token).unwrap(), "MISSING");
  }

  #[test]
  fn get_name_with_bearer_prefix() {
    let token = make_jwt(&serde_json::json!({
      "name": "Bob Jones"
    }));
    let bearer_token = format!("Bearer {token}");
    assert_eq!(get_name(&bearer_token).unwrap(), "Bob Jones");
  }

  // ---- get_preferred_username ----

  #[test]
  fn get_preferred_username_present() {
    let token = make_jwt(&serde_json::json!({
      "name": "Alice",
      "preferred_username": "alice123"
    }));
    assert_eq!(get_preferred_username(&token).unwrap(), "alice123");
  }

  #[test]
  fn get_preferred_username_missing_returns_missing() {
    let token = make_jwt(&serde_json::json!({"name": "Alice"}));
    assert_eq!(get_preferred_username(&token).unwrap(), "MISSING");
  }

  // ---- get_claims_from_jwt_token ----

  #[test]
  fn malformed_jwt_no_dots() {
    assert!(get_claims_from_jwt_token("nodots").is_err());
  }

  #[test]
  fn malformed_jwt_invalid_base64() {
    assert!(get_claims_from_jwt_token("header.!!!invalid.sig").is_err());
  }

  #[test]
  fn jwt_with_standard_base64_padding() {
    // Some JWTs use standard base64 with padding
    let payload = serde_json::json!({"name": "Test"});
    let header = BASE64_STANDARD.encode(r#"{"alg":"none"}"#);
    let body = BASE64_STANDARD.encode(payload.to_string());
    let token = format!("{header}.{body}.sig");
    assert_eq!(get_name(&token).unwrap(), "Test");
  }

  #[test]
  fn empty_token_string_is_err() {
    assert!(get_claims_from_jwt_token("").is_err());
  }

  #[test]
  fn jwt_with_valid_base64_but_invalid_json() {
    // base64 of "not json at all"
    let body = BASE64_URL_SAFE_NO_PAD.encode("not json at all");
    let token = format!("header.{body}.sig");
    assert!(get_claims_from_jwt_token(&token).is_err());
  }

  #[test]
  fn jwt_with_valid_base64_but_invalid_utf8() {
    // Raw bytes that aren't valid UTF-8
    let body = BASE64_URL_SAFE_NO_PAD.encode([0xFF, 0xFE, 0xFD]);
    let token = format!("header.{body}.sig");
    assert!(get_claims_from_jwt_token(&token).is_err());
  }

  #[test]
  fn get_name_with_empty_string_name() {
    let token = make_jwt(&serde_json::json!({"name": ""}));
    assert_eq!(get_name(&token).unwrap(), "");
  }

  #[test]
  fn bearer_prefix_with_extra_spaces() {
    // "Bearer  token" - the split(' ').nth(1) would get empty string
    let token = make_jwt(&serde_json::json!({"name": "Test"}));
    let bad_bearer = format!("Bearer  {token}");
    // nth(1) returns empty string, which has no dots -> error
    assert!(get_name(&bad_bearer).is_err());
  }
}
