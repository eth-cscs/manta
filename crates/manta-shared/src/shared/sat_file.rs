//! Deserialization types for HPE Cray SAT (System Admin Toolkit) YAML files.

use crate::common::error::MantaError as Error;
use image::Image;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use self::sessiontemplate::SessionTemplate;

/// Top-level representation of a SAT YAML file.
#[derive(Deserialize, Serialize, Debug)]
pub struct SatFile {
  /// CFS configurations to create or update.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configurations: Option<Vec<configuration::Configuration>>,
  /// IMS images to build.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub images: Option<Vec<image::Image>>,
  /// BOS session templates to apply.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub session_templates: Option<Vec<sessiontemplate::SessionTemplate>>,
}

impl SatFile {
  /// Filter either images or session_templates section according to user request
  pub fn filter(
    &mut self,
    image_only: bool,
    session_template_only: bool,
  ) -> Result<(), Error> {
    // Clean SAT template file if user only wan'ts to process the 'images' section. In this case,
    // we will remove 'session_templates' section from SAT fiel and also the entries in
    // 'configurations' section not used
    if image_only {
      let image_vec_opt: Option<&[Image]> = self.images.as_deref();

      let configuration_name_image_vec: Vec<String> = match image_vec_opt {
        Some(image_vec) => image_vec
          .iter()
          .filter_map(|sat_template_image| {
            sat_template_image.configuration.clone()
          })
          .collect(),
        None => {
          return Err(Error::MissingField(
            "'images' section missing in SAT file".to_string(),
          ));
        }
      };

      // Remove configurations not used by any image
      if let Some(configurations) = self.configurations.as_mut() {
        configurations.retain(|configuration| {
          configuration_name_image_vec.contains(&configuration.name)
        });
      }

      // Remove section "session_templates"
      self.session_templates = None;
    }

    // Clean SAT template file if user only wan'ts to process the 'session_template' section. In this case,
    // we will remove 'images' section from SAT fiel and also the entries in
    // 'configurations' section not used
    if session_template_only {
      let sessiontemplate_vec_opt: Option<&[SessionTemplate]> =
        self.session_templates.as_deref();

      let image_name_sessiontemplate_vec: Vec<String> = self
        .session_templates
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|sessiontemplate| match &sessiontemplate.image {
          sessiontemplate::Image::ImageRef { image_ref: name } => Some(name),
          sessiontemplate::Image::Ims { ims } => match ims {
            sessiontemplate::ImsDetails::Name { name } => Some(name),
            sessiontemplate::ImsDetails::Id { .. } => None,
          },
        })
        .cloned()
        .collect();

      // Remove images not used by any sessiontemplate
      if let Some(images) = self.images.as_mut() {
        images
          .retain(|image| image_name_sessiontemplate_vec.contains(&image.name));
      }

      if self.images.as_ref().is_some_and(|images| images.is_empty()) {
        self.images = None;
      }

      // Get configuration names from session templates
      let configuration_name_sessiontemplate_vec: Vec<String> =
        match sessiontemplate_vec_opt {
          Some(sessiontemplate_vec) => sessiontemplate_vec
            .iter()
            .map(|sat_sessiontemplate| {
              sat_sessiontemplate.configuration.clone()
            })
            .collect(),
          None => {
            return Err(Error::MissingField(
              "'session_templates' section not defined \
               in SAT file"
                .to_string(),
            ));
          }
        };

      // Get configuration names from images used by the session templates
      let configuration_name_image_vec: Vec<String> = self
        .images
        .as_deref()
        .unwrap_or_default()
        .iter()
        .filter_map(|image| image.configuration.as_ref().cloned())
        .collect();

      // Merge configuration names from images and session templates
      let configuration_to_keep_vec = [
        configuration_name_image_vec,
        configuration_name_sessiontemplate_vec,
      ]
      .concat();

      // Remove configurations not used by any sessiontemplate or image used by the
      // sessiontemplate

      if let Some(configurations) = self.configurations.as_mut() {
        configurations.retain(|configuration| {
          configuration_to_keep_vec.contains(&configuration.name)
        });
      }
    }

    Ok(())
  }
}

/// struct to represent the `session_templates` section in SAT file
pub mod sessiontemplate {
  use std::collections::HashMap;
  use strum_macros::Display;

  use serde::{Deserialize, Serialize};

  /// A BOS session template linking an image, configuration,
  /// and boot parameters for a set of nodes.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct SessionTemplate {
    /// Unique name for the session template.
    pub name: String,
    /// IMS image (or cross-reference to a SAT `images` entry) to boot.
    pub image: Image,
    /// CFS configuration applied to the booted nodes.
    pub configuration: String,
    /// Per-boot-set kernel / rootfs / node-targeting parameters.
    pub bos_parameters: BosParamters,
  }

  /// How the IMS image is referenced — by name or by UUID.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImsDetails {
    /// Reference by human-readable IMS image name.
    Name {
      /// IMS image name.
      name: String,
    },
    /// Reference by IMS image UUID.
    Id {
      /// IMS image UUID.
      id: String,
    },
  }

  /// Image reference within a session template — either an
  /// IMS image or a cross-reference to another SAT image.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Image {
    /// Directly identifies an IMS image via name or UUID.
    Ims {
      /// IMS image identifier (name or UUID).
      ims: ImsDetails,
    },
    /// Cross-references the `name` of another image in the SAT `images` section.
    ImageRef {
      /// `name` of a sibling image defined in the SAT file's
      /// `images` section.
      image_ref: String,
    },
  }

  /// BOS boot parameters containing a map of named boot sets.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct BosParamters {
    /// Named boot sets keyed by their BOS identifier.
    pub boot_sets: HashMap<String, BootSet>,
  }

  /// A single boot set defining the kernel, network, and node
  /// targeting for a BOS session template.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct BootSet {
    /// Processor architecture; defaults to whatever the linked image
    /// reports if omitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<Arch>,
    /// Kernel command-line parameters passed to the booted nodes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_parameters: Option<String>,
    /// Network name to boot over (e.g. `nmn`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    /// Explicit list of node xnames to target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_list: Option<Vec<String>>,
    /// HSM roles (e.g. `Compute`, `Application`) to target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_roles_group: Option<Vec<String>>,
    /// HSM group names to target.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_groups: Option<Vec<String>>,
    /// Root-filesystem provider, e.g. `sbps` or `ais`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootfs_provider: Option<String>,
    /// Extra arguments forwarded to the rootfs provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootfs_provider_passthrough: Option<String>,
  }

  /// Processor architecture for a boot set.
  #[derive(Deserialize, Serialize, Debug, Display)]
  #[allow(clippy::upper_case_acronyms)]
  pub enum Arch {
    /// x86-64 nodes.
    X86,
    /// AArch64 / ARM nodes.
    ARM,
    /// Any other architecture.
    Other,
    /// Architecture could not be determined.
    Unknown,
  }
}

/// struct to represent the `images` section in SAT file
pub mod image {
  use serde::{Deserialize, Serialize};

  /// Processor architecture for an IMS image build.
  #[derive(Deserialize, Serialize, Debug)]
  pub enum Arch {
    /// 64-bit ARM (serialized as `"aarch64"`).
    #[serde(rename(serialize = "aarch64", deserialize = "aarch64"))]
    Aarch64,
    /// x86-64 (serialized as `"x86_64"`).
    #[serde(rename(serialize = "x86_64", deserialize = "x86_64"))]
    X86_64,
  }

  /// Legacy IMS image reference with a recipe flag (older SAT format).
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImageIms {
    /// Image identified by name; `is_recipe` indicates whether it is an IMS recipe.
    NameIsRecipe {
      /// IMS image name.
      name: String,
      /// `true` if `name` refers to an IMS recipe rather than a built image.
      is_recipe: bool,
    },
    /// Image identified by UUID; `is_recipe` indicates whether it is an IMS recipe.
    IdIsRecipe {
      /// IMS image UUID.
      id: String,
      /// `true` if `id` refers to an IMS recipe rather than a built image.
      is_recipe: bool,
    },
  }

  /// Base IMS image reference used in newer SAT file format.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImageBaseIms {
    /// Image identified by name and type string.
    NameType {
      /// IMS image name.
      name: String,
      /// IMS object type, e.g. `"image"` or `"recipe"`.
      r#type: String,
    },
    /// Image identified by UUID and type string.
    IdType {
      /// IMS image UUID.
      id: String,
      /// IMS object type, e.g. `"image"` or `"recipe"`.
      r#type: String,
    },
    /// Older format with UUID and optional `is_recipe` flag.
    BackwardCompatible {
      /// `true` if `id` is a recipe; `None` defaults to image.
      is_recipe: Option<bool>,
      /// IMS image UUID.
      id: String,
    },
  }

  /// Criteria for filtering product catalog images.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Filter {
    /// Match images whose name starts with `prefix`.
    Prefix {
      /// Required image-name prefix.
      prefix: String,
    },
    /// Match images whose name matches the `wildcard` glob.
    Wildcard {
      /// Glob pattern applied to image names.
      wildcard: String,
    },
    /// Match images built for the given architecture.
    Arch {
      /// Architecture filter.
      arch: Arch,
    },
  }

  /// A product catalog entry used as an image source.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct Product {
    /// Product name (e.g. `cos`, `slingshot-host-software`).
    name: String,
    /// Optional product version pin.
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    /// Product type, e.g. `"image"` or `"recipe"`.
    r#type: String,
    /// Filter applied to the product's image list.
    filter: Filter,
  }

  /// Source for a base image — IMS, product catalog, or cross-reference.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Base {
    /// Directly references an IMS image by name, UUID, or type.
    Ims {
      /// IMS image reference (name, UUID, or legacy `is_recipe`).
      ims: ImageBaseIms,
    },
    /// Pulls the latest matching image from the product catalog.
    Product {
      /// Product entry to query the catalog for.
      product: Product,
    },
    /// Cross-references the `name` of another image in the SAT `images` section.
    ImageRef {
      /// `name` of a sibling image defined in the SAT file's
      /// `images` section.
      image_ref: String,
    },
  }

  /// Wrapper bridging the older `ims` key and the newer `base` key in SAT image entries.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum BaseOrIms {
    /// Newer format using the `base` key.
    Base {
      /// Newer-format base reference.
      base: Base,
    },
    /// Legacy format using the `ims` key with a recipe flag.
    Ims {
      /// Legacy IMS reference carrying the `is_recipe` flag.
      ims: ImageIms,
    },
  }

  /// An image definition in the SAT file `images` section.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct Image {
    /// Unique name for this image; used as the cross-reference target for `image_ref`.
    pub name: String,
    /// Optional alias used to reference this image from session templates.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_name: Option<String>,
    /// Base image source (IMS or product catalog), in legacy or current format.
    #[serde(flatten)]
    pub base_or_ims: BaseOrIms,
    /// CFS configuration to apply when building the image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<String>,
    /// HSM group names passed as Ansible group vars during the image build.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_group_names: Option<Vec<String>>,
    /// Free-form human description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
  }
}

/// struct to represent the `configurations` section in SAT file
pub mod configuration {
  use serde::{Deserialize, Serialize};

  /// A product reference within a CFS configuration layer.
  /// Variants capture different ways to pin a version/branch.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)]
  // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  #[allow(clippy::enum_variant_names)]
  pub enum Product {
    /// Product pinned to a specific branch (and optionally a version).
    ProductVersionBranch {
      /// Product name (e.g. `cos`).
      name: String,
      /// Optional product version pin.
      version: Option<String>,
      /// Git branch HEAD to track.
      branch: String,
    },
    /// Product pinned to a specific commit (and optionally a version).
    ProductVersionCommit {
      /// Product name.
      name: String,
      /// Optional product version pin.
      version: Option<String>,
      /// Exact commit SHA to pin to.
      commit: String,
    },
    /// Product pinned by exact version string.
    ProductVersion {
      /// Product name.
      name: String,
      /// Exact product version.
      version: String,
    },
  }

  /// A Git repository reference within a CFS configuration
  /// layer, pinned by commit, branch, or tag.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)]
  // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  #[allow(clippy::enum_variant_names)]
  pub enum Git {
    /// Layer pinned to an exact commit SHA.
    GitCommit {
      /// Git repository URL.
      url: String,
      /// Exact commit SHA.
      commit: String,
    },
    /// Layer pinned to a branch HEAD.
    GitBranch {
      /// Git repository URL.
      url: String,
      /// Branch name whose HEAD is tracked.
      branch: String,
    },
    /// Layer pinned to a tag.
    GitTag {
      /// Git repository URL.
      url: String,
      /// Tag name.
      tag: String,
    },
  }

  /// Extra CFS layer parameters (e.g., requiring DKMS).
  #[derive(Deserialize, Serialize, Debug)]
  pub struct SpecialParameters {
    /// When `true`, the resulting image build must include DKMS.
    pub ims_require_dkms: bool,
  }

  /// A CFS configuration layer sourced from a Git repo.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct LayerGit {
    /// Optional human-friendly name for this layer; defaults to a
    /// CFS-generated identifier when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional Ansible playbook filename within the layer's repo;
    /// CFS uses `site.yml` when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook: Option<String>, // This field is optional but with default value. Therefore we won't
    /// Git pin (commit / branch / tag).
    pub git: Git,
    /// Layer-specific knobs such as DKMS requirements.
    pub special_parameters: Option<SpecialParameters>,
  }

  /// A CFS configuration layer sourced from a product catalog.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct LayerProduct {
    /// Optional human-friendly name for this layer; defaults to a
    /// CFS-generated identifier when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Optional Ansible playbook filename within the product's repo.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook: Option<String>, // This field is optional but with default value. Therefore we won't
    /// Product reference (name + version pin).
    pub product: Product,
  }

  /// A CFS configuration layer — either Git-based or
  /// product-catalog-based.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Layer {
    /// CFS configuration layer sourced from a Git repository.
    LayerGit(LayerGit),
    /// CFS configuration layer sourced from a product catalog entry.
    LayerProduct(LayerProduct),
  }

  /// An Ansible inventory source for a CFS configuration,
  /// pinned by commit or branch.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Inventory {
    /// Inventory repository pinned to a specific commit SHA.
    InventoryCommit {
      /// Optional human-friendly name for the inventory source.
      #[serde(skip_serializing_if = "Option::is_none")]
      name: Option<String>,
      /// Git repository URL.
      url: String,
      /// Exact commit SHA.
      commit: String,
    },
    /// Inventory repository pinned to a branch HEAD.
    InventoryBranch {
      /// Optional human-friendly name for the inventory source.
      #[serde(skip_serializing_if = "Option::is_none")]
      name: Option<String>,
      /// Git repository URL.
      url: String,
      /// Branch name whose HEAD is tracked.
      branch: String,
    },
  }

  /// A CFS configuration definition in the SAT file
  /// `configurations` section.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct Configuration {
    /// Configuration name; must be unique within the SAT file.
    pub name: String,
    /// Free-form human description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Ordered list of CFS layers applied to nodes using this
    /// configuration.
    pub layers: Vec<Layer>,
    /// Optional Ansible inventory source for the configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_inventory: Option<Inventory>,
  }
}

// Removed unused module sat_file_image_old which contained Ims and Product structs

/// Merge 2 yamls, 'b' values will overwrite 'a' values
/// eg:
/// having a:
///
/// ```text
/// key_1
///   key_1_1: value_1_1
///   key_1_2: value_1_2
/// key_2: value_2
/// key_3: value_3
/// ```
/// and b:
/// ```text
/// key_1
///   key_1_1: new_value_1_1
///   key_1_2: value_1_2
///   key_1_3: new_value_1_3
/// key_2: new_value_2
/// key_4: new_value_4
/// ```
/// would convert a into:
/// ```text
/// key_1
///   key_1_1: new_value_1_1
///   key_1_3: new_value_1_3
/// key_2: new_value_2
/// key_3: value_3
/// key_4: new_value_4
/// ```
fn merge_yaml(base: Value, merge: Value) -> Option<Value> {
  match (base, merge) {
    (Value::Mapping(mut base_map), Value::Mapping(merge_map)) => {
      for (key, value) in merge_map {
        if let Some(base_value) = base_map.get_mut(&key) {
          *base_value = merge_yaml(base_value.clone(), value)?;
        } else {
          base_map.insert(key, value);
        }
      }
      Some(Value::Mapping(base_map))
    }
    (Value::Sequence(mut base_seq), Value::Sequence(merge_seq)) => {
      base_seq.extend(merge_seq);
      Some(Value::Sequence(base_seq))
    }
    (_, merge) => Some(merge),
  }
}

/// Convert a String dot notation expression into a serde_yaml::Value.
/// eg:
/// dot notation input like:
/// ```text
/// key_1.key_2.key_3=1
/// ```
/// would result in a serde_yaml::Value equivalent to:
/// ```text
/// key_1
///   key_2
///     key_3: 1
/// ```
fn dot_notation_to_yaml(
  dot_notation: &str,
) -> Result<serde_yaml::Value, Error> {
  let parts: Vec<&str> = dot_notation.split('=').collect();
  if parts.len() != 2 {
    return Err(Error::InvalidPattern("Invalid format".to_string()));
  }

  let keys: Vec<&str> = parts[0].trim().split('.').collect();
  let value_str = parts[1].trim().trim_matches('"'); // Remove leading and trailing quotes
  let value: Value = Value::String(value_str.to_string());

  let mut root = Value::Mapping(Mapping::new());
  let mut current_level = &mut root;

  for (i, &key) in keys.iter().enumerate() {
    if i == keys.len() - 1 {
      // Last key, assign the value
      if let Value::Mapping(map) = current_level {
        map.insert(Value::String(key.to_string()), value.clone());
      }
    } else {
      // Not the last key, create or use existing map
      let next_level = if let Value::Mapping(map) = current_level {
        if map.contains_key(Value::String(key.to_string())) {
          // Use existing map
          map.get_mut(Value::String(key.to_string())).ok_or_else(|| {
            Error::TemplateError(
              "Failed to get mutable reference to \
               existing YAML map entry"
                .to_string(),
            )
          })?
        } else {
          // Create new map and insert
          map.insert(
            Value::String(key.to_string()),
            Value::Mapping(Mapping::new()),
          );
          map.get_mut(Value::String(key.to_string())).ok_or_else(|| {
            Error::TemplateError(
              "Failed to get mutable reference to \
               newly inserted YAML map entry"
                .to_string(),
            )
          })?
        }
      } else {
        return Err(Error::TemplateError(
          "Unexpected structure encountered".to_string(),
        ));
      };
      current_level = next_level;
    }
  }

  Ok(root)
}

/// Render a SAT file as a Jinja2 template, optionally
/// merging a values file and CLI-provided overrides in dot
/// notation.
pub fn render_jinja2_sat_file_yaml(
  sat_file_content: &str,
  values_file_content_opt: Option<&str>,
  value_cli_vec_opt: Option<&[String]>,
) -> Result<Value, Error> {
  let mut env = minijinja::Environment::new();
  // Set/enable debug in order to force minijinja to print debug error messages which are more
  // descriptive. Eg https://github.com/mitsuhiko/minijinja/blob/main/examples/error/src/main.rs#L4-L5
  env.set_debug(true);
  // Set lines starting with `#` as comments
  env.set_syntax(
    minijinja::syntax::SyntaxConfig::builder()
      .line_comment_prefix("#")
      .build()
      .map_err(|e| {
        Error::TemplateError(format!(
          "Failed to build jinja2 syntax config: {e}"
        ))
      })?,
  );
  // Set 'String' as undefined behaviour meaning, missing values won't pass the template
  // rendering
  env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);

  // Render session values file
  let mut values_file_yaml: Value = if let Some(values_file_content) =
    values_file_content_opt
  {
    tracing::info!(
      "'Session vars' file provided. Going to process SAT file as a jinja template."
    );
    tracing::info!("Expand variables in 'session vars' file");
    // Read sesson vars file and parse it to YAML
    let values_file_yaml: Value = serde_yaml::from_str(values_file_content)?;
    // Render session vars file with itself (copying ansible behaviour where the ansible vars
    // file is also a jinja template and combine both vars and values in it)
    let values_file_rendered = env
      .render_str(values_file_content, values_file_yaml)
      .map_err(|e| {
        Error::TemplateError(format!("Error parsing values file to YAML: {e}"))
      })?;
    serde_yaml::from_str(&values_file_rendered)?
  } else {
    serde_yaml::from_str(sat_file_content)?
  };

  // Convert variable values sent by cli argument from dot notation to yaml format
  tracing::debug!(
    "Convert variable values sent by cli argument from dot notation to yaml format"
  );
  if let Some(value_option_vec) = value_cli_vec_opt {
    for value_option in value_option_vec {
      let cli_var_context_yaml = dot_notation_to_yaml(value_option)?;

      values_file_yaml =
        merge_yaml(values_file_yaml.clone(), cli_var_context_yaml).ok_or_else(
          || {
            Error::TemplateError(
              "Failed to merge CLI variable values into \
             SAT file YAML"
                .to_string(),
            )
          },
        )?;
    }
  }

  // render sat template file
  tracing::info!("Expand variables in 'SAT file'");
  let sat_file_rendered = env
    .render_str(sat_file_content, values_file_yaml)
    .map_err(|e| {
      Error::TemplateError(format!("Failed to render SAT file template: {e}"))
    })?;

  // Disable debug
  env.set_debug(false);

  Ok(serde_yaml::from_str(&sat_file_rendered)?)
}

#[cfg(test)]
mod tests;
