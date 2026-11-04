//! TEMPORARY: reuse the Hydra token for a CSCS token on OpenCHAMI sites.
//!
//! Hydra's JWT bearer grant accepts an assertion's `jti` only once until
//! the assertion expires, but the CLI sends one CSCS token on several
//! requests (at least the session's group lookup and the command itself),
//! and the server exchanges on every request. So the first exchange's
//! result is kept here for the later requests.
//!
//! Remove this module by reverting its commit once OpenCHAMI moves to
//! TokenSmith, whose exchange has no such limit.
//!
//! The cache is in memory, so after a server restart a CSCS token that
//! was already exchanged is refused (`401`) until the CLI refreshes it.
//! It also assumes one server instance.

use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use manta_backend_dispatcher::error::Error;
use sha2::{Digest, Sha256};
use tokio::sync::OnceCell;

use crate::server::common::jwt_ops;

/// SHA-256 over the site name and the whole CSCS token. Never the `jti`:
/// signatures are not checked here, so a forged token could copy someone
/// else's `jti` and get their Hydra token.
type Key = [u8; 32];

struct Entry {
  token: String,
  /// Epoch seconds; the earlier of the two tokens' `exp`, so an expired
  /// CSCS token is never served.
  expires_at: i64,
}

/// Exchanged tokens by CSCS token, for OpenCHAMI sites.
#[derive(Default)]
pub struct HydraTokenCache {
  entries: Mutex<HashMap<Key, Arc<OnceCell<Entry>>>>,
}

impl HydraTokenCache {
  /// The cached token for `subject_token` at `site`, or the result of
  /// `exchange`, cached on success. Concurrent callers with the same key
  /// run `exchange` once, since Hydra would refuse the second.
  ///
  /// # Errors
  ///
  /// Whatever `exchange` returns. A failure is not cached.
  pub async fn get_or_exchange<F, Fut>(
    &self,
    site: &str,
    subject_token: &str,
    exchange: F,
  ) -> Result<String, Error>
  where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<String, Error>>,
  {
    let cell = self.cell(key(site, subject_token));
    let entry = cell
      .get_or_try_init(|| async {
        let token = exchange().await?;
        let expires_at = expiry(subject_token).min(expiry(&token));
        Ok::<_, Error>(Entry { token, expires_at })
      })
      .await?;
    Ok(entry.token.clone())
  }

  /// The cell for `key`, after dropping expired entries and cells left
  /// empty by a failed exchange (any token string can create one).
  fn cell(&self, key: Key) -> Arc<OnceCell<Entry>> {
    let now = chrono::Utc::now().timestamp();
    let mut entries = self.entries.lock().unwrap_or_else(|e| e.into_inner());
    entries.retain(|_, cell| match cell.get() {
      Some(entry) => entry.expires_at > now,
      None => Arc::strong_count(cell) > 1,
    });
    Arc::clone(entries.entry(key).or_default())
  }
}

fn key(site: &str, subject_token: &str) -> Key {
  Sha256::new()
    .chain_update(site)
    .chain_update([0])
    .chain_update(subject_token)
    .finalize()
    .into()
}

/// The token's `exp`, or 0 (already expired, so not reused) when it has
/// none or cannot be decoded.
fn expiry(token: &str) -> i64 {
  jwt_ops::get_expiration(token).ok().flatten().unwrap_or(0)
}

#[cfg(test)]
mod tests {
  use std::sync::atomic::{AtomicUsize, Ordering};

  use base64::prelude::*;

  use super::*;

  const SITE: &str = "basodino";

  fn now() -> i64 {
    chrono::Utc::now().timestamp()
  }

  /// A token expiring at `exp`; `nonce` makes tokens distinct.
  fn jwt(exp: i64, nonce: &str) -> String {
    let claims = serde_json::json!({ "exp": exp, "jti": nonce }).to_string();
    let payload = BASE64_URL_SAFE_NO_PAD.encode(claims);
    format!("e30.{payload}.sig")
  }

  /// `get_or_exchange` with an exchange that returns `hydra_token` and
  /// counts its calls.
  async fn exchange(
    cache: &HydraTokenCache,
    site: &str,
    subject: &str,
    hydra_token: &str,
    calls: &AtomicUsize,
  ) -> Result<String, Error> {
    cache
      .get_or_exchange(site, subject, || async {
        calls.fetch_add(1, Ordering::SeqCst);
        Ok(hydra_token.to_string())
      })
      .await
  }

  #[tokio::test]
  async fn the_same_token_is_exchanged_once() {
    let cache = HydraTokenCache::default();
    let calls = AtomicUsize::new(0);
    let subject = jwt(now() + 300, "subject");
    let hydra = jwt(now() + 3600, "hydra");

    for _ in 0..3 {
      let token = exchange(&cache, SITE, &subject, &hydra, &calls).await;
      assert_eq!(token.unwrap(), hydra);
    }

    assert_eq!(calls.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn concurrent_requests_share_one_exchange() {
    let cache = HydraTokenCache::default();
    let calls = AtomicUsize::new(0);
    let subject = jwt(now() + 300, "subject");
    let hydra = jwt(now() + 3600, "hydra");
    let slow_exchange = || async {
      calls.fetch_add(1, Ordering::SeqCst);
      tokio::time::sleep(std::time::Duration::from_millis(20)).await;
      Ok(hydra.clone())
    };

    let (first, second) = tokio::join!(
      cache.get_or_exchange(SITE, &subject, slow_exchange),
      cache.get_or_exchange(SITE, &subject, slow_exchange),
    );

    assert_eq!(first.unwrap(), hydra);
    assert_eq!(second.unwrap(), hydra);
    assert_eq!(calls.load(Ordering::SeqCst), 1);
  }

  #[tokio::test]
  async fn sites_and_tokens_do_not_share_entries() {
    let cache = HydraTokenCache::default();
    let calls = AtomicUsize::new(0);
    let first = jwt(now() + 300, "first");
    let second = jwt(now() + 300, "second");
    let hydra = jwt(now() + 3600, "hydra");

    exchange(&cache, "a", &first, &hydra, &calls).await.unwrap();
    exchange(&cache, "b", &first, &hydra, &calls).await.unwrap();
    exchange(&cache, "a", &second, &hydra, &calls)
      .await
      .unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 3);
  }

  /// Same `jti`, different token: a forged copy must not get the entry.
  #[tokio::test]
  async fn a_token_copying_a_jti_does_not_hit_the_entry() {
    let cache = HydraTokenCache::default();
    let calls = AtomicUsize::new(0);
    let exp = now() + 300;
    let genuine = jwt(exp, "same-jti");
    let forged = format!("{}forged", jwt(exp, "same-jti"));
    let hydra = jwt(now() + 3600, "hydra");

    exchange(&cache, SITE, &genuine, &hydra, &calls)
      .await
      .unwrap();
    exchange(&cache, SITE, &forged, "refused-by-hydra", &calls)
      .await
      .unwrap();

    assert_eq!(calls.load(Ordering::SeqCst), 2);
  }

  #[tokio::test]
  async fn a_failure_is_not_cached_and_leaves_no_entry() {
    let cache = HydraTokenCache::default();
    let calls = AtomicUsize::new(0);
    let subject = jwt(now() + 300, "subject");
    let hydra = jwt(now() + 3600, "hydra");

    let failed = cache
      .get_or_exchange(SITE, "garbage", || async {
        Err(Error::Unauthorized("rejected".to_string()))
      })
      .await;
    assert!(matches!(failed, Err(Error::Unauthorized(_))));

    let token = exchange(&cache, SITE, &subject, &hydra, &calls).await;
    assert_eq!(token.unwrap(), hydra);
    assert_eq!(cache.entries.lock().unwrap().len(), 1);
  }

  #[tokio::test]
  async fn an_expired_cscs_token_is_not_served_from_the_cache() {
    let cache = HydraTokenCache::default();
    let calls = AtomicUsize::new(0);
    let subject = jwt(now() - 1, "subject");
    let hydra = jwt(now() + 3600, "hydra");

    for _ in 0..2 {
      exchange(&cache, SITE, &subject, &hydra, &calls)
        .await
        .unwrap();
    }

    assert_eq!(calls.load(Ordering::SeqCst), 2);
  }

  #[tokio::test]
  async fn an_expired_hydra_token_is_not_served_from_the_cache() {
    let cache = HydraTokenCache::default();
    let calls = AtomicUsize::new(0);
    let subject = jwt(now() + 300, "subject");
    let hydra = jwt(now() - 1, "hydra");

    for _ in 0..2 {
      exchange(&cache, SITE, &subject, &hydra, &calls)
        .await
        .unwrap();
    }

    assert_eq!(calls.load(Ordering::SeqCst), 2);
  }

  #[tokio::test]
  async fn expired_entries_are_dropped() {
    let cache = HydraTokenCache::default();
    let calls = AtomicUsize::new(0);
    let hydra = jwt(now() + 3600, "hydra");

    exchange(&cache, SITE, &jwt(now() - 1, "old"), &hydra, &calls)
      .await
      .unwrap();
    exchange(&cache, SITE, &jwt(now() + 300, "new"), &hydra, &calls)
      .await
      .unwrap();

    assert_eq!(cache.entries.lock().unwrap().len(), 1);
  }

  #[tokio::test]
  async fn a_token_without_exp_is_not_reused() {
    let cache = HydraTokenCache::default();
    let calls = AtomicUsize::new(0);
    let hydra = jwt(now() + 3600, "hydra");

    for _ in 0..2 {
      exchange(&cache, SITE, "not-a-jwt", &hydra, &calls)
        .await
        .unwrap();
    }

    assert_eq!(calls.load(Ordering::SeqCst), 2);
  }
}
