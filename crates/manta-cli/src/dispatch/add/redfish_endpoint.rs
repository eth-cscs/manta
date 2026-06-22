//! Implements the `manta add redfish-endpoint` command.

use anyhow::Error;

use crate::common::app_context::AppContext;
use crate::http_client::{MantaClient, OpenApiResultExt};
use crate::openapi_client::types::UpdateRedfishEndpointParams;
use crate::output::action_result;

pub struct ExecParams<'a> {
  pub id: &'a str,
  pub name: Option<&'a str>,
  pub hostname: Option<&'a str>,
  pub domain: Option<&'a str>,
  pub fqdn: Option<&'a str>,
  pub enabled: bool,
  pub user: Option<&'a str>,
  pub password: Option<&'a str>,
  pub use_ssdp: bool,
  pub mac_required: bool,
  pub mac_addr: Option<&'a str>,
  pub ip_address: Option<&'a str>,
  pub rediscover_on_update: bool,
  pub template_id: Option<&'a str>,
  pub output: Option<&'a str>,
  /// When true, surface the `UpdateRedfishEndpointParams` JSON
  /// payload via `action_result::preview_request` and return
  /// without calling the server. The server has no `dry_run`
  /// support on this endpoint; the preview is purely client-side.
  pub dry_run: bool,
}

/// CLI adapter for `manta add redfish-endpoint`.
pub async fn exec(
  ctx: &AppContext<'_>,
  token: &str,
  p: ExecParams<'_>,
) -> Result<(), Error> {
  let client = MantaClient::from_app_ctx(ctx, Some(token))?;

  let req = UpdateRedfishEndpointParams {
    id: p.id.to_string(),
    name: p.name.map(str::to_string),
    hostname: p.hostname.map(str::to_string),
    domain: p.domain.map(str::to_string),
    fqdn: p.fqdn.map(str::to_string),
    enabled: p.enabled,
    user: p.user.map(str::to_string),
    password: p.password.map(str::to_string),
    use_ssdp: p.use_ssdp,
    mac_required: p.mac_required,
    mac_addr: p.mac_addr.map(str::to_string),
    ip_address: p.ip_address.map(str::to_string),
    rediscover_on_update: p.rediscover_on_update,
    template_id: p.template_id.map(str::to_string),
  };

  if p.dry_run {
    return action_result::preview_request(
      "POST",
      "/redfish-endpoints",
      &req,
      p.output,
    );
  }

  client
    .openapi
    .add_redfish_endpoint(client.site_name(), &req)
    .await
    .into_anyhow()?;

  action_result::print(
    &format!("Redfish endpoint for node '{}' added", p.id),
    p.output,
  )?;

  Ok(())
}

#[cfg(test)]
mod tests {
  /// `--dry-run` parses on `manta add redfish-endpoints` (long flag).
  #[test]
  fn accepts_dry_run() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "add",
      "redfish-endpoints",
      "--id",
      "x1000c0r0s0b0",
      "--dry-run",
    ]);
    assert!(
      result.is_ok(),
      "expected --dry-run to parse on `add redfish-endpoints`: {result:?}"
    );
  }

  /// `-d` short alias also parses (freed by the `-d` -> `-D` swap on
  /// `--domain`).
  #[test]
  fn accepts_dry_run_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "add",
      "redfish-endpoints",
      "--id",
      "x1000c0r0s0b0",
      "-d",
    ]);
    assert!(
      result.is_ok(),
      "expected -d short alias to parse: {result:?}"
    );
  }

  /// `-D` short alias for `--domain` still parses after the
  /// `-d` -> `-D` swap (pins down the breaking-change surface).
  #[test]
  fn accepts_domain_short_alias() {
    let result = crate::build::build_cli().try_get_matches_from([
      "manta",
      "add",
      "redfish-endpoints",
      "--id",
      "x1000c0r0s0b0",
      "-D",
      "example.com",
    ]);
    assert!(
      result.is_ok(),
      "expected -D short alias for --domain to parse: {result:?}"
    );
  }
}
