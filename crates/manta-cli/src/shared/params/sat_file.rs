//! Parameters for `POST /sat-file`.

/// Parameters for applying a SAT file.
pub struct ApplySatFileParams<'a> {
  pub sat_file_content: &'a str,
  pub values: Option<&'a serde_json::Value>,
  pub values_file_content: Option<&'a str>,
  pub ansible_verbosity: Option<u8>,
  pub ansible_passthrough: Option<&'a str>,
  pub reboot: bool,
  pub watch_logs: bool,
  pub timestamps: bool,
  pub image_only: bool,
  pub session_template_only: bool,
  pub overwrite: bool,
  pub dry_run: bool,
}
