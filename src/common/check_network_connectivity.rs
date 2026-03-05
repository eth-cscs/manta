use std::time::Duration;

use anyhow::{Context, Result};

/// Verify that the backend API endpoint is reachable
/// (3-second connect timeout).
pub async fn check_network_connectivity_to_backend(
  shasta_base_url: &str,
) -> Result<()> {
  let client_builder =
    reqwest::Client::builder().connect_timeout(Duration::new(3, 0));

  // Build client
  let client = client_builder
    .build()
    .context("Failed to build HTTP client")?;

  let api_url = shasta_base_url;

  log::info!("Validate CSM token against {}", api_url);

  client
    .get(api_url)
    .send()
    .await?
    .error_for_status()
    .map(|_| ())?;

  Ok(())
}
