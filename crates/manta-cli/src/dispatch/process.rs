//! Root CLI dispatcher: matches the parsed top-level verb and calls
//! the per-verb handler in this module's siblings.
//!
//! For every command:
//!
//! 1. Read-only gate — refuses backend-mutating verbs when `cli.toml`
//!    has `read_only = true`. Runs before any token acquisition so no
//!    HTTP call leaves the process on the refusal path.
//! 2. (Authenticated verbs only) Token cascade
//!    ([`get_api_token`]) — one round-trip if the cache is hot;
//!    interactive prompt if cold.
//! 3. (Authenticated verbs only) [`SessionContext::build`] — one
//!    `GET /groups/available` round-trip plus the JWT claims; cached
//!    on `AppContext` for the duration of the command.
//!
//! Server-side read-only enforcement is independent, driven by the
//! `manta-read-only` realm role on the bearer token
//! (`manta-server`'s `server::auth_middleware::read_only_guard`).
//! The CLI gate is fast local feedback, not the security boundary.

use crate::common::app_context::AppContext;
use crate::common::authentication::get_api_token;
use crate::common::session::SessionContext;
use crate::http_client::MantaClient;
use anyhow::{Error, bail};
use clap::ArgMatches;

use crate::dispatch::{
  add, apply, backup, config, console, delete, gen_autocomplete, gen_man, get,
  log, migrate, power, restore, run, upgrade,
};

/// Verbs that operate purely on the local config file (or local
/// machine state) and never call `get_api_token`. Derived empirically
/// from `grep -rl get_api_token crates/manta-cli/src/dispatch` — any
/// handler that does NOT appear in that grep result belongs here.
///
/// Today the no-auth set is: `gen-autocomplete`, `gen-man`, `upgrade`,
/// `config set site`, `config set log`, `config set read-only`,
/// `config unset hsm`, `config unset auth`,
/// `config unset read-only`. Every other verb (including `config show`
/// when a site is selected, and `config set hsm`) needs a token and
/// therefore a `SessionContext`.
fn verb_skips_session(cli_root: &ArgMatches) -> bool {
  match cli_root.subcommand() {
    Some(("gen-autocomplete", _)) | Some(("gen-man", _)) => true,
    Some(("upgrade", _)) => true,
    Some(("config", config_m)) => match config_m.subcommand() {
      Some(("set", set_m)) => matches!(
        set_m.subcommand(),
        Some(("site", _)) | Some(("log", _)) | Some(("read-only", _))
      ),
      Some(("unset", unset_m)) => matches!(
        unset_m.subcommand(),
        Some(("hsm", _)) | Some(("auth", _)) | Some(("read-only", _))
      ),
      _ => false,
    },
    _ => false,
  }
}

/// Parse CLI arguments and dispatch to the appropriate
/// subcommand handler.
///
/// # Errors
///
/// Returns an error when the auth cascade fails, the SessionContext
/// build fails, the read-only gate refuses a mutating verb, no
/// subcommand is provided, the subcommand name is unknown, or the
/// chosen verb's handler returns an error.
pub async fn process_cli(
  cli_root: &ArgMatches,
  mut ctx: AppContext<'_>,
) -> Result<(), Error> {
  // Step 1: read-only gate. Fires purely off `cli.toml`, before any
  // HTTP call — a `read_only = true` operator running `manta apply`
  // is refused without the token cascade ever running.
  crate::common::read_only::read_only_gate(cli_root, ctx.read_only)?;

  let needs_session = !verb_skips_session(cli_root) && ctx.site_name.is_some();

  // Step 2: resolve the token (only for authenticated verbs).
  let token_opt: Option<String> = if needs_session {
    Some(get_api_token(&ctx).await?)
  } else {
    None
  };

  // Step 3: full SessionContext build (groups fetch + JWT claims).
  // Stash the token on `ctx.token` so downstream handlers that call
  // `get_api_token` hit the short-circuit in `common::authentication`
  // instead of re-walking the cache.
  if let Some(token) = token_opt {
    let client = MantaClient::from_app_ctx(&ctx, Some(&token))?;
    ctx.session = Some(SessionContext::build(&client, &token).await?);
    ctx.token = Some(token);
  }

  match cli_root.subcommand() {
    Some(("config", m)) => config::handle_config(m, &ctx).await?,
    Some(("power", m)) => power::handle_power(m, &ctx).await?,
    Some(("add", m)) => add::handle_add(m, &ctx).await?,
    Some(("get", m)) => get::handle_get(m, &ctx).await?,
    Some(("apply", m)) => apply::handle_apply(m, &ctx).await?,
    Some(("log", m)) => log::handle_log(m, &ctx).await?,
    Some(("console", m)) => console::handle_console(m, &ctx).await?,
    Some(("migrate", m)) => migrate::handle_migrate(m, &ctx).await?,
    Some(("backup", m)) => backup::handle_backup(m, &ctx).await?,
    Some(("restore", m)) => restore::handle_restore(m, &ctx).await?,
    Some(("run", m)) => run::handle_run(m, &ctx).await?,
    Some(("delete", m)) => delete::handle_delete(m, &ctx).await?,
    Some(("gen-autocomplete", m)) => {
      gen_autocomplete::handle_gen_autocomplete(m, &ctx).await?;
    }
    Some(("gen-man", m)) => gen_man::handle_gen_man(m, &ctx).await?,
    Some(("upgrade", m)) => upgrade::handle_upgrade(m, &ctx).await?,
    Some((other, _)) => bail!("Unknown command: {other}"),
    None => bail!("No command provided"),
  }
  Ok(())
}
