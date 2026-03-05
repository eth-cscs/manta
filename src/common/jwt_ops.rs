use anyhow::Context;
use base64::prelude::*;
use serde_json::Value;

fn get_claims_from_jwt_token(token: &str) -> Result<Value, anyhow::Error> {
  // Handle both "Bearer <token>" and bare "<token>" formats
  let jwt_body = token.split(' ').nth(1).unwrap_or(token);

  let base64_claims = jwt_body.split('.').nth(1).context(
    "JWT token is malformed: expected \
     header.payload.signature format",
  )?;

  let claims_u8 = BASE64_URL_SAFE_NO_PAD
    .decode(base64_claims)
    .or_else(|_| BASE64_STANDARD.decode(base64_claims))
    .context("Could not get claims in JWT token")?;

  let claims_str = std::str::from_utf8(&claims_u8)
    .context("Could not convert JWT claims to string")?;

  serde_json::from_str::<Value>(claims_str)
    .context("Could not convert JWT claims to a JSON object")
}

pub fn get_name(token: &str) -> Result<String, anyhow::Error> {
  let jwt_claims = get_claims_from_jwt_token(token)?;

  let jwt_name = jwt_claims.get("name").and_then(Value::as_str);

  match jwt_name {
    Some(name) => Ok(name.to_string()),
    None => Ok("MISSING".to_string()),
  }
}

pub fn get_preferred_username(token: &str) -> Result<String, anyhow::Error> {
  let jwt_claims = get_claims_from_jwt_token(token)?;

  let jwt_preferred_username =
    jwt_claims.get("preferred_username").and_then(Value::as_str);

  match jwt_preferred_username {
    Some(name) => Ok(name.to_string()),
    None => Ok("MISSING".to_string()),
  }
}
