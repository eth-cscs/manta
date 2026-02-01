use dialoguer::{theme::ColorfulTheme, Confirm};

/// Prompts the user for confirmation unless `assume_yes` is true.
///
/// # Arguments
/// * `message` - The message to display in the prompt.
/// * `assume_yes` - If true, the prompt is skipped and the function returns `true`.
///
/// # Returns
/// `true` if the user confirmed or `assume_yes` was set, `false` otherwise.
pub fn confirm(message: &str, assume_yes: bool) -> bool {
  if assume_yes {
    return true;
  }

  Confirm::with_theme(&ColorfulTheme::default())
    .with_prompt(message)
    .interact()
    .unwrap_or(false)
}
