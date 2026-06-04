//! Yes/no confirmation prompt with an `--assume-yes` bypass.

use dialoguer::{Confirm, theme::ColorfulTheme};

/// Prompt for confirmation; returns `true` immediately if `assume_yes` is set.
pub fn confirm(message: &str, assume_yes: bool) -> bool {
  if assume_yes {
    return true;
  }

  Confirm::with_theme(&ColorfulTheme::default())
    .with_prompt(message)
    .interact()
    .unwrap_or(false)
}
