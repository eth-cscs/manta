use base64::decode;
use manta_backend_dispatcher::error::Error;
use serde_json::Value;

fn get_claims_from_jwt_token(token: &str) -> Result<Value, Error> {
  let base64_claims = token
    .split(' ')
    .nth(1)
    .unwrap_or(token)
    .split('.')
    .nth(1)
    .unwrap_or("JWT Token not valid");

  let claims_u8 = decode(base64_claims).map_err(|e| {
    Error::Message(format!(
      "ERROR - could not get claims in JWT token. Reason:\n{}",
      e
    ))
  })?;

  let claims_str = std::str::from_utf8(&claims_u8).map_err(|_| {
    Error::Message("ERROR - could not convert JWT claims to string".to_string())
  })?;

  serde_json::from_str::<Value>(claims_str).map_err(|_| {
    Error::Message(
      "ERROR - could not convert JWT claims to a JSON object".to_string(),
    )
  })
}

pub fn get_name(token: &str) -> Result<String, Error> {
  let jwt_claims = get_claims_from_jwt_token(token).unwrap();

  let jwt_name = jwt_claims["name"].as_str();

  match jwt_name {
    Some(name) => Ok(name.to_string()),
    None => Ok("MISSING".to_string()),
  }
}

pub fn get_preferred_username(token: &str) -> Result<String, Error> {
  let jwt_claims = get_claims_from_jwt_token(token).unwrap();

  let jwt_preferred_username = jwt_claims["preferred_username"].as_str();

  match jwt_preferred_username {
    Some(name) => Ok(name.to_string()),
    None => Ok("MISSING".to_string()),
  }
}
