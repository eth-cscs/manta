/// Dispatches a method call to the underlying backend variant.
///
/// Both `CSM` and `OCHAMI` variants always delegate to the
/// same method with identical arguments, so this macro
/// eliminates the repetitive `match self` boilerplate.
///
/// # Usage
/// ```ignore
/// // async method:
/// dispatch!(self, method_name, arg1, arg2)
/// // sync method:
/// dispatch!(sync self, method_name, arg1)
/// ```
macro_rules! dispatch {
  // async (default): adds `.await` after the call
  ($self:ident, $method:ident $(, $arg:expr)*) => {
    match $self {
      CSM(b) => b.$method($($arg),*).await,
      OCHAMI(b) => b.$method($($arg),*).await,
    }
  };
  // sync: no `.await`
  (sync $self:ident, $method:ident $(, $arg:expr)*) => {
    match $self {
      CSM(b) => b.$method($($arg),*),
      OCHAMI(b) => b.$method($($arg),*),
    }
  };
}

mod apply_hardware_cluster_pin;
mod apply_session;
mod authentication;
mod boot_parameters;
mod cfs;
mod cluster_session;
mod cluster_template;
mod component;
mod console;
mod delete_configurations_and_related;
mod group;
mod hardware_inventory;
mod ims;
mod migrate_backup;
mod migrate_restore;
mod pcs;
mod redfish_endpoint;
mod sat;
