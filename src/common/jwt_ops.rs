use base64::prelude::*;
use manta_backend_dispatcher::error::Error;
use serde_json::Value;

fn get_claims_from_jwt_token(token: &str) -> Result<Value, Error> {
  // Handle both "Bearer <token>" and bare "<token>" formats
  let jwt_body = token.split(' ').nth(1).unwrap_or(token);

  let base64_claims = jwt_body.split('.').nth(1).ok_or_else(|| {
    Error::Message(
      "JWT token is malformed: expected \
       header.payload.signature format"
        .to_string(),
    )
  })?;

  let claims_u8 = BASE64_URL_SAFE_NO_PAD
    .decode(base64_claims)
    .or_else(|_| BASE64_STANDARD.decode(base64_claims))
    .map_err(|e| {
      Error::Message(format!("Could not get claims in JWT token: {}", e))
    })?;

  let claims_str = std::str::from_utf8(&claims_u8).map_err(|e| {
    Error::Message(format!("Could not convert JWT claims to string: {}", e))
  })?;

  Ok(serde_json::from_str::<Value>(claims_str)?)
}

/// Extract the `name` claim from a JWT token.
///
/// Returns `"MISSING"` if the claim is absent.
pub fn get_name(token: &str) -> Result<String, Error> {
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
pub fn get_preferred_username(token: &str) -> Result<String, Error> {
  let jwt_claims = get_claims_from_jwt_token(token)?;

  let jwt_preferred_username =
    jwt_claims.get("preferred_username").and_then(Value::as_str);

  match jwt_preferred_username {
    Some(name) => Ok(name.to_string()),
    None => Ok("MISSING".to_string()),
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  /// Build a fake JWT with the given JSON payload.
  fn make_jwt(payload: &serde_json::Value) -> String {
    let header = BASE64_URL_SAFE_NO_PAD.encode(r#"{"alg":"none","typ":"JWT"}"#);
    let body = BASE64_URL_SAFE_NO_PAD.encode(payload.to_string());
    format!("{}.{}.sig", header, body)
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
    let bearer_token = format!("Bearer {}", token);
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
    let token = format!("{}.{}.sig", header, body);
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
    let token = format!("header.{}.sig", body);
    assert!(get_claims_from_jwt_token(&token).is_err());
  }

  #[test]
  fn jwt_with_valid_base64_but_invalid_utf8() {
    // Raw bytes that aren't valid UTF-8
    let body = BASE64_URL_SAFE_NO_PAD.encode(&[0xFF, 0xFE, 0xFD]);
    let token = format!("header.{}.sig", body);
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
    let bad_bearer = format!("Bearer  {}", token);
    // nth(1) returns empty string, which has no dots -> error
    assert!(get_name(&bad_bearer).is_err());
  }
}
