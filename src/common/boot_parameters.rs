use manta_backend_dispatcher::types::{BootParameters, Group};

/// Get a vector of boot parameters that are restricted based on the groups available to the user.
pub fn get_restricted_boot_parameters(
  group_available_vec: &[Group],
  boot_parameter_vec: &[BootParameters],
) -> Vec<BootParameters> {
  let mut group_member_available_vec = group_available_vec
    .iter()
    .flat_map(|group| group.get_members());

  boot_parameter_vec
    .into_iter()
    .filter(|&boot_param| {
      group_member_available_vec.any(|gma| !boot_param.hosts.contains(&gma))
    })
    .cloned()
    .collect::<Vec<BootParameters>>()
}
