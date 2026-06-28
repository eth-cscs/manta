//! Yes/no confirmation prompt with an `--assume-yes` bypass.
//!
//! Used by destructive verbs (`delete`, `migrate`, `upgrade`, …) to
//! gate the side effect on operator confirmation. The `assume_yes`
//! parameter is wired from each verb's `-y/--assume-yes` flag so
//! scripted invocations skip the prompt.

use dialoguer::{Confirm, theme::ColorfulTheme};

/// Prompt for confirmation; returns `true` immediately if
/// `assume_yes` is set. Returns `false` when the prompt itself
/// fails (no TTY, user aborts with Ctrl-C / Ctrl-D) so callers
/// can treat a failed prompt the same as an explicit "no".
pub fn confirm(message: &str, assume_yes: bool) -> bool {
  if assume_yes {
    return true;
  }

  Confirm::with_theme(&ColorfulTheme::default())
    .with_prompt(message)
    .interact()
    .unwrap_or(false)
}
