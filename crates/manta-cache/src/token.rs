//! Where [`crate::refresh`] gets the bearer token of each request.

/// Error a [`TokenSource`] returns when it cannot supply a token.
pub type TokenError = Box<dyn std::error::Error + Send + Sync>;

/// Supplies the bearer token for each request to manta-server.
///
/// [`crate::refresh`] and [`crate::fetch_snapshots`] call
/// [`token`](Self::token) right before **every** request — one per
/// site per refresh — and never reuse a token. manta-server exchanges
/// the token on every request, and an OpenCHAMI site's Hydra accepts
/// each token (its `jti`) only once, so a reused token fails there.
///
/// One source serves every site: a CSCS Keycloak token is valid for
/// all of them. Where the token comes from (a service account's client
/// credentials grant, in `manta-cache-server`) is the caller's concern;
/// this crate holds no OIDC code.
pub trait TokenSource: Sync {
  /// A new token to send now.
  ///
  /// # Errors
  ///
  /// Whatever stops the source from issuing a token (the Keycloak
  /// unreachable, the credentials refused). The request that needed it
  /// is not sent; its site fails with [`crate::CacheError::Token`].
  fn token(
    &self,
  ) -> impl std::future::Future<Output = Result<String, TokenError>> + Send;
}
