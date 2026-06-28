//! [`CfsTrait`] impl for [`StaticBackendDispatcher`].
//!
//! Forwards to the Configuration Framework Service (CFS) v3 API
//! (`/apis/cfs/v3/{healthz,sessions,configurations,components}`) and
//! its companion Kubernetes log-tailing path on CSM. The Ochami
//! backend has an empty `impl CfsTrait for Ochami {}`, so every
//! method on this trait goes through the trait default and returns
//! [`Error::Message`] (`"... command not implemented for this
//! backend"`) when the dispatcher is the Ochami variant.

use super::*;

impl CfsTrait for StaticBackendDispatcher {
  /// Associated buffered reader type returned by the session-log
  /// streaming methods. Pin-boxed and `Send` so it can cross await
  /// points and be fed straight into the HTTP layer.
  type T = Pin<Box<dyn AsyncBufRead + Send>>;

  /// `GET /cfs/healthz` — liveness probe.
  async fn get_cfs_health(&self) -> Result<(), Error> {
    dispatch!(self, get_cfs_health)
  }

  /// Tail the Ansible-container log of `cfs_session_name`. Opens a
  /// `kubectl logs --follow` style stream against the session pod in
  /// the CSM `services` namespace via the supplied `k8s` context.
  async fn get_session_logs_stream(
    &self,
    token: &str,
    site_name: &str,
    cfs_session_name: &str,
    timestamps: bool,
    k8s: &K8sDetails,
  ) -> Result<Pin<Box<dyn AsyncBufRead + Send>>, Error> {
    dispatch!(
      self,
      get_session_logs_stream,
      token,
      site_name,
      cfs_session_name,
      timestamps,
      k8s
    )
  }

  /// Resolve `xname` to its most recent CFS session and tail that
  /// session's pod log. Convenience for "what is happening on this
  /// node right now"; otherwise identical to
  /// [`get_session_logs_stream`](Self::get_session_logs_stream).
  async fn get_session_logs_stream_by_xname(
    &self,
    auth_token: &str,
    site_name: &str,
    xname: &str,
    timestamps: bool,
    k8s: &K8sDetails,
  ) -> Result<Pin<Box<dyn AsyncBufRead + Send>>, Error> {
    dispatch!(
      self,
      get_session_logs_stream_by_xname,
      auth_token,
      site_name,
      xname,
      timestamps,
      k8s
    )
  }

  /// `POST /cfs/v3/sessions` — submit `session` for execution.
  /// Returns the backend's representation of the created session.
  async fn post_session(
    &self,
    token: &str,
    session: &CfsSessionPostRequest,
  ) -> Result<CfsSessionGetResponse, Error> {
    dispatch!(self, post_session, token, session)
  }

  /// `GET /cfs/v3/sessions` — server-side filtered list. Filters
  /// match CFS v3 query parameters verbatim (`name`, `limit`,
  /// `after_id`, `min_age`, `max_age`, `status`, `name_contains`,
  /// `succeeded`, `tags`).
  async fn get_sessions(
    &self,
    auth_token: &str,
    session_name_opt: Option<&String>,
    limit_opt: Option<u8>,
    after_id_opt: Option<String>,
    min_age_opt: Option<String>,
    max_age_opt: Option<String>,
    status_opt: Option<String>,
    name_contains_opt: Option<String>,
    is_succeded_opt: Option<bool>,
    tags_opt: Option<String>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    dispatch!(
      self,
      get_sessions,
      auth_token,
      session_name_opt,
      limit_opt,
      after_id_opt,
      min_age_opt,
      max_age_opt,
      status_opt,
      name_contains_opt,
      is_succeded_opt,
      tags_opt
    )
  }

  /// Fetch sessions and apply client-side group/xname filtering that
  /// CFS doesn't natively support. The backend issues the broadest
  /// query CFS allows, then narrows the result set in-process to
  /// `group_name_vec` / `xname_vec` / `type_opt`.
  async fn get_and_filter_sessions(
    &self,
    token: &str,
    group_name_vec: Vec<String>,
    xname_vec: Vec<&str>,
    min_age_opt: Option<&String>,
    max_age_opt: Option<&String>,
    type_opt: Option<&String>,
    status_opt: Option<&String>,
    cfs_session_name_opt: Option<&String>,
    limit_number_opt: Option<&u8>,
    is_succeded_opt: Option<bool>,
  ) -> Result<Vec<CfsSessionGetResponse>, Error> {
    dispatch!(
      self,
      get_and_filter_sessions,
      token,
      group_name_vec,
      xname_vec,
      min_age_opt,
      max_age_opt,
      type_opt,
      status_opt,
      cfs_session_name_opt,
      limit_number_opt,
      is_succeded_opt
    )
  }

  /// Cancel an in-flight session (PATCH `status=cancelled`) and then
  /// delete it along with derived BSS/CFS-component state, gated by
  /// `group_available_vec` (RBAC) and `dry_run`. The backend
  /// validates `cfs_session` against the supplied components and
  /// bootparameters before mutating anything.
  async fn delete_and_cancel_session(
    &self,
    token: &str,
    group_available_vec: &[Group],
    cfs_session: &CfsSessionGetResponse,
    cfs_component_vec: &[CfsComponent],
    bss_bootparameter_vec: &[BootParameters],
    dry_run: bool,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      delete_and_cancel_session,
      token,
      group_available_vec,
      cfs_session,
      cfs_component_vec,
      bss_bootparameter_vec,
      dry_run
    )
  }

  /// Build (but do not POST) a `CfsConfigurationRequest` from Gitea
  /// repos pinned to specific commit ids. Used by callers that want
  /// to inspect or further edit the configuration before applying it.
  async fn create_configuration_from_repos(
    &self,
    gitea_token: &str,
    gitea_base_url: &str,
    repo_name_vec: &[&str],
    local_git_commit_vec: &[&str],
    playbook_file_name_opt: Option<&str>,
  ) -> Result<CfsConfigurationRequest, Error> {
    dispatch!(
      self,
      create_configuration_from_repos,
      gitea_token,
      gitea_base_url,
      repo_name_vec,
      local_git_commit_vec,
      playbook_file_name_opt
    )
  }

  /// `GET /cfs/v3/configurations` — single configuration when
  /// `cfs_configuration_name_opt` is `Some`, otherwise the full list.
  async fn get_configuration(
    &self,
    auth_token: &str,
    cfs_configuration_name_opt: Option<&String>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    dispatch!(
      self,
      get_configuration,
      auth_token,
      cfs_configuration_name_opt
    )
  }

  /// List configurations, optionally narrowed by exact name, regex,
  /// the groups the caller can see, a date window, and a result
  /// count cap. Name/group filtering happens server-side where CFS
  /// supports it and client-side otherwise.
  async fn get_and_filter_configuration(
    &self,
    auth_token: &str,
    configuration_name: Option<&str>,
    configuration_name_pattern: Option<&str>,
    group_name_vec: &[String],
    since_opt: Option<NaiveDateTime>,
    until_opt: Option<NaiveDateTime>,
    limit_number_opt: Option<&u8>,
  ) -> Result<Vec<CfsConfigurationResponse>, Error> {
    dispatch!(
      self,
      get_and_filter_configuration,
      auth_token,
      configuration_name,
      configuration_name_pattern,
      group_name_vec,
      since_opt,
      until_opt,
      limit_number_opt
    )
  }

  /// Resolve a single configuration `Layer` (Gitea repo + commit or
  /// branch + playbook) to its enriched `LayerDetails` — adds the
  /// most recent commit metadata, the resolved commit when only a
  /// branch was given, and any tag the commit carries.
  async fn get_configuration_layer_details(
    &self,
    gitea_base_url: &str,
    gitea_token: &str,
    layer: Layer,
    site_name: &str,
  ) -> Result<LayerDetails, Error> {
    dispatch!(
      self,
      get_configuration_layer_details,
      gitea_base_url,
      gitea_token,
      layer,
      site_name
    )
  }

  /// `PATCH /cfs/v3/components` — set
  /// `desired_configuration` on each `xnames[]` entry and toggle the
  /// component `enabled` flag. This is the per-node "runtime
  /// reconfigure" knob (next CFS pass picks the change up).
  async fn update_runtime_configuration(
    &self,
    auth_token: &str,
    xnames: &[String],
    desired_configuration: &str,
    enabled: bool,
  ) -> Result<(), Error> {
    dispatch!(
      self,
      update_runtime_configuration,
      auth_token,
      xnames,
      desired_configuration,
      enabled
    )
  }

  /// `PUT /cfs/v3/configurations/{name}` — create or replace
  /// `configuration_name`. When `overwrite` is false and the
  /// configuration already exists, the backend returns
  /// [`Error::ConfigurationAlreadyExistsError`].
  async fn put_configuration(
    &self,
    token: &str,
    configuration: &CfsConfigurationRequest,
    configuration_name: &str,
    overwrite: bool,
  ) -> Result<CfsConfigurationResponse, Error> {
    dispatch!(
      self,
      put_configuration,
      token,
      configuration,
      configuration_name,
      overwrite
    )
  }

  /// Find every artifact that references `configuration_name`:
  /// sessions that ran against it, BOS templates that pin it, and
  /// IMS images stamped with it. Each tuple slot is `None` when the
  /// corresponding lookup returned an empty result so callers can
  /// distinguish "no derivatives" from "lookup not performed".
  async fn get_derivatives(
    &self,
    auth_token: &str,
    configuration_name: &str,
  ) -> Result<
    (
      Option<Vec<CfsSessionGetResponse>>,
      Option<Vec<BosSessionTemplate>>,
      Option<Vec<Image>>,
    ),
    Error,
  > {
    dispatch!(self, get_derivatives, auth_token, configuration_name)
  }

  /// `GET /cfs/v3/components` — per-xname CFS state filtered by
  /// `configuration_name`, an `ids=` xname list, and/or `status`
  /// (e.g. `pending`, `failed`, `configured`).
  async fn get_cfs_components(
    &self,
    token: &str,
    configuration_name: Option<&str>,
    components_ids: Option<&str>,
    status: Option<&str>,
  ) -> Result<Vec<CfsComponent>, Error> {
    dispatch!(
      self,
      get_cfs_components,
      token,
      configuration_name,
      components_ids,
      status
    )
  }
}
