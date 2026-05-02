use std::time::Duration;

use manta_backend_dispatcher::error::Error;

/// Timeout in seconds for the backend connectivity check.
const BACKEND_CONNECT_TIMEOUT_SECS: u64 = 3;

/// Verify that the backend API endpoint is reachable
/// (3-second connect timeout).
pub async fn check_network_connectivity_to_backend(
  shasta_base_url: &str,
) -> Result<(), Error> {
  let client = reqwest::Client::builder()
    .connect_timeout(Duration::from_secs(BACKEND_CONNECT_TIMEOUT_SECS))
    .build()?;

  tracing::info!("Validate CSM token against {}", shasta_base_url);

  client
    .get(shasta_base_url)
    .send()
    .await?
    .error_for_status()
    .map(|_| ())?;

  Ok(())
}
