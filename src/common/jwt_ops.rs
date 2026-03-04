use anyhow::Context;
use base64::decode;
use serde_json::Value;

fn get_claims_from_jwt_token(token: &str) -> Result<Value, anyhow::Error> {
  let base64_claims = token
    .split(' ')
    .nth(1)
    .unwrap_or(token)
    .split('.')
    .nth(1)
    .unwrap_or("JWT Token not valid");

  let claims_u8 =
    decode(base64_claims).context("Could not get claims in JWT token")?;

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
