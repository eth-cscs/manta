use manta_backend_dispatcher::types::{Group, bss::BootParameters};

/// Get a vector of boot parameters that are restricted based on the groups available to the user.
pub fn get_restricted_boot_parameters(
  group_available_vec: &[Group],
  boot_parameter_vec: &[BootParameters],
) -> Vec<BootParameters> {
  let group_members: Vec<String> = group_available_vec
    .iter()
    .flat_map(|group| group.get_members())
    .collect();

  boot_parameter_vec
    .iter()
    .filter(|boot_param| {
      group_members
        .iter()
        .any(|gma| boot_param.hosts.contains(gma))
    })
    .cloned()
    .collect::<Vec<BootParameters>>()
}
