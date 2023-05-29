use std::error::Error;

use base64::decode;
use serde_json::Value;

pub fn get_claims_from_jwt_token(token: &str) -> Result<Value, Box<dyn Error>> {
    let base64_claims = token
        .split(' ')
        .nth(1)
        .unwrap_or(token)
        .split('.')
        .nth(1)
        .unwrap_or("JWT Token not valid");

    let claims_u8 = decode(base64_claims).unwrap();

    Ok(serde_json::from_str::<Value>(std::str::from_utf8(&claims_u8).unwrap()).unwrap())
}
