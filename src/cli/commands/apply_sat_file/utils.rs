use manta_backend_dispatcher::error::Error;
use image::Image;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use self::sessiontemplate::SessionTemplate;

#[derive(Deserialize, Serialize, Debug)]
/// Top-level representation of a SAT (System Admin Toolkit)
/// YAML file containing configurations, images, and session
/// templates.
pub struct SatFile {
  #[serde(skip_serializing_if = "Option::is_none")]
  pub configurations: Option<Vec<configuration::Configuration>>,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub images: Option<Vec<image::Image>>,
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
    pub name: String,
    pub image: Image,
    pub configuration: String,
    pub bos_parameters: BosParamters,
  }

  /// How the IMS image is referenced — by name or by UUID.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImsDetails {
    Name { name: String },
    Id { id: String },
  }

  /// Image reference within a session template — either an
  /// IMS image or a cross-reference to another SAT image.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Image {
    Ims { ims: ImsDetails },
    ImageRef { image_ref: String },
  }

  /// BOS boot parameters containing a map of named boot sets.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct BosParamters {
    pub boot_sets: HashMap<String, BootSet>,
  }

  /// A single boot set defining the kernel, network, and node
  /// targeting for a BOS session template.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct BootSet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<Arch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kernel_parameters: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_list: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_roles_group: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_groups: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootfs_provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rootfs_provider_passthrough: Option<String>,
  }

  /// Processor architecture for a boot set.
  #[derive(Deserialize, Serialize, Debug, Display)]
  #[allow(clippy::upper_case_acronyms)]
  pub enum Arch {
    X86,
    ARM,
    Other,
    Unknown,
  }
}

/// struct to represent the `images` section in SAT file
pub mod image {
  use serde::{Deserialize, Serialize};

  /// Processor architecture for an IMS image build.
  #[derive(Deserialize, Serialize, Debug)]
  pub enum Arch {
    #[serde(rename(serialize = "aarch64", deserialize = "aarch64"))]
    Aarch64,
    #[serde(rename(serialize = "x86_64", deserialize = "x86_64"))]
    X86_64,
  }

  /// Legacy IMS image reference with a recipe flag.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImageIms {
    NameIsRecipe { name: String, is_recipe: bool },
    IdIsRecipe { id: String, is_recipe: bool },
  }

  /// Base IMS image reference used in newer SAT file format.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImageBaseIms {
    NameType { name: String, r#type: String },
    IdType { id: String, r#type: String },
    BackwardCompatible { is_recipe: Option<bool>, id: String },
  }

  /// Criteria for filtering product catalog images.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Filter {
    Prefix { prefix: String },
    Wildcard { wildcard: String },
    Arch { arch: Arch },
  }

  /// A product catalog entry used as an image source.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct Product {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    r#type: String,
    filter: Filter,
  }

  /// Source for a base image — IMS, product catalog, or
  /// cross-reference to another SAT image.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Base {
    Ims { ims: ImageBaseIms },
    Product { product: Product },
    ImageRef { image_ref: String },
  }

  /// Wrapper for backward compatibility between the older
  /// `ims` key and the newer `base` key in SAT image entries.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum BaseOrIms {
    Base { base: Base },
    Ims { ims: ImageIms },
  }

  /// An image definition in the SAT file `images` section.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct Image {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ref_name: Option<String>,
    #[serde(flatten)]
    pub base_or_ims: BaseOrIms,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration_group_names: Option<Vec<String>>,
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
    ProductVersionBranch {
      name: String,
      version: Option<String>,
      branch: String,
    },
    ProductVersionCommit {
      name: String,
      version: Option<String>,
      commit: String,
    },
    ProductVersion {
      name: String,
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
    GitCommit { url: String, commit: String },
    GitBranch { url: String, branch: String },
    GitTag { url: String, tag: String },
  }

  /// Extra CFS layer parameters (e.g., requiring DKMS).
  #[derive(Deserialize, Serialize, Debug)]
  pub struct SpecialParameters {
    pub ims_require_dkms: bool,
  }

  /// A CFS configuration layer sourced from a Git repo.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct LayerGit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook: Option<String>, // This field is optional but with default value. Therefore we won't
    pub git: Git,
    pub special_parameters: Option<SpecialParameters>,
  }

  /// A CFS configuration layer sourced from a product catalog.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct LayerProduct {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub playbook: Option<String>, // This field is optional but with default value. Therefore we won't
    pub product: Product,
  }

  /// A CFS configuration layer — either Git-based or
  /// product-catalog-based.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Layer {
    LayerGit(LayerGit),
    LayerProduct(LayerProduct),
  }

  /// An Ansible inventory source for a CFS configuration,
  /// pinned by commit or branch.
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Inventory {
    InventoryCommit {
      #[serde(skip_serializing_if = "Option::is_none")]
      name: Option<String>,
      url: String,
      commit: String,
    },
    InventoryBranch {
      #[serde(skip_serializing_if = "Option::is_none")]
      name: Option<String>,
      url: String,
      branch: String,
    },
  }

  /// A CFS configuration definition in the SAT file
  /// `configurations` section.
  #[derive(Deserialize, Serialize, Debug)]
  pub struct Configuration {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub layers: Vec<Layer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_inventory: Option<Inventory>,
  }
}

// Removed unused module sat_file_image_old which contained Ims and Product structs

/// Merge 2 yamls, 'b' values will overwrite 'a' values
/// eg:
/// having a:
///
/// ```
/// key_1
///   key_1_1: value_1_1
///   key_1_2: value_1_2
/// key_2: value_2
/// key_3: value_3
/// ```
/// and b:
/// ```
/// key_1
///   key_1_1: new_value_1_1
///   key_1_2: value_1_2
///   key_1_3: new_value_1_3
/// key_2: new_value_2
/// key_4: new_value_4
/// ```
/// would convert a into:
/// ```
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
/// ```
/// key_1.key_2.key_3=1
/// ````
/// would result in a serde_yaml::Value equivalent to:
/// ```
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
        return Err(Error::TemplateError("Unexpected structure encountered".to_string()));
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
      .map_err(|e| Error::TemplateError(format!("Failed to build jinja2 syntax config: {e}")))?,
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
      .map_err(|e| Error::TemplateError(format!("Error parsing values file to YAML: {e}")))?;
    serde_yaml::from_str(&values_file_rendered)
      ?
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
        merge_yaml(values_file_yaml.clone(), cli_var_context_yaml).ok_or_else(|| {
          Error::TemplateError(
            "Failed to merge CLI variable values into \
             SAT file YAML"
              .to_string(),
          )
        })?;
    }
  }

  // render sat template file
  tracing::info!("Expand variables in 'SAT file'");
  let sat_file_rendered = env
    .render_str(sat_file_content, values_file_yaml)
    .map_err(|e| Error::TemplateError(format!("Failed to render SAT file template: {e}")))?;

  // Disable debug
  env.set_debug(false);

  Ok(serde_yaml::from_str(&sat_file_rendered)?)
}

#[cfg(test)]
mod tests {
  use super::*;
  use serde_yaml::Value;

  // ── merge_yaml ──

  #[test]
  fn merge_yaml_scalars_overwrite() {
    let base = Value::String("old".into());
    let merge = Value::String("new".into());
    let result = merge_yaml(base, merge).unwrap();
    assert_eq!(result, Value::String("new".into()));
  }

  #[test]
  fn merge_yaml_maps_deep_merge() {
    let base: Value = serde_yaml::from_str("a:\n  b: 1\n  c: 2").unwrap();
    let merge: Value = serde_yaml::from_str("a:\n  b: 99\n  d: 3").unwrap();
    let result = merge_yaml(base, merge).unwrap();
    // b is overwritten, c is preserved, d is added
    let a = result.get("a").unwrap();
    assert_eq!(a.get("b").unwrap().as_u64(), Some(99));
    assert_eq!(a.get("c").unwrap().as_u64(), Some(2));
    assert_eq!(a.get("d").unwrap().as_u64(), Some(3));
  }

  #[test]
  fn merge_yaml_sequences_concatenated() {
    let base: Value = serde_yaml::from_str("[1, 2]").unwrap();
    let merge: Value = serde_yaml::from_str("[3, 4]").unwrap();
    let result = merge_yaml(base, merge).unwrap();
    let seq = result.as_sequence().unwrap();
    assert_eq!(seq.len(), 4);
  }

  #[test]
  fn merge_yaml_adds_new_top_level_keys() {
    let base: Value = serde_yaml::from_str("x: 1").unwrap();
    let merge: Value = serde_yaml::from_str("y: 2").unwrap();
    let result = merge_yaml(base, merge).unwrap();
    assert_eq!(result.get("x").unwrap().as_u64(), Some(1));
    assert_eq!(result.get("y").unwrap().as_u64(), Some(2));
  }

  // ── dot_notation_to_yaml ──

  #[test]
  fn dot_notation_single_key() {
    let result = dot_notation_to_yaml("key=value").unwrap();
    assert_eq!(result.get("key").unwrap().as_str(), Some("value"));
  }

  #[test]
  fn dot_notation_nested_keys() {
    let result = dot_notation_to_yaml("a.b.c=hello").unwrap();
    let a = result.get("a").unwrap();
    let b = a.get("b").unwrap();
    assert_eq!(b.get("c").unwrap().as_str(), Some("hello"));
  }

  #[test]
  fn dot_notation_strips_quotes_from_value() {
    let result = dot_notation_to_yaml("key=\"quoted\"").unwrap();
    assert_eq!(result.get("key").unwrap().as_str(), Some("quoted"));
  }

  #[test]
  fn dot_notation_invalid_format_no_equals() {
    assert!(dot_notation_to_yaml("no_equals_sign").is_err());
  }

  #[test]
  fn dot_notation_multiple_equals_rejected() {
    assert!(dot_notation_to_yaml("a=b=c").is_err());
  }

  // ── SatFile::filter ──

  #[test]
  fn filter_image_only_removes_session_templates() {
    let yaml = r#"
configurations:
  - name: cfg-used
    layers:
      - name: layer1
        git:
          url: https://example.com/repo.git
          branch: main
  - name: cfg-unused
    layers:
      - name: layer2
        git:
          url: https://example.com/repo.git
          branch: main
images:
  - name: img1
    ims:
      is_recipe: false
      id: abc-123
    configuration: cfg-used
session_templates:
  - name: st1
    image:
      image_ref: img1
    configuration: cfg-used
    bos_parameters:
      boot_sets:
        compute:
          node_groups:
            - group1
"#;
    let mut sat: SatFile = serde_yaml::from_str(yaml).unwrap();
    sat.filter(true, false).unwrap();
    assert!(sat.session_templates.is_none());
    // Only cfg-used is kept
    let configs = sat.configurations.unwrap();
    assert_eq!(configs.len(), 1);
    assert_eq!(configs[0].name, "cfg-used");
  }

  #[test]
  fn filter_session_template_only_removes_unused_images() {
    let yaml = r#"
configurations:
  - name: cfg-st
    layers:
      - name: layer1
        git:
          url: https://example.com/repo.git
          branch: main
  - name: cfg-img-only
    layers:
      - name: layer2
        git:
          url: https://example.com/repo.git
          branch: main
images:
  - name: used-image
    ims:
      is_recipe: false
      id: abc-123
    configuration: cfg-img-only
  - name: unused-image
    ims:
      is_recipe: false
      id: def-456
session_templates:
  - name: st1
    image:
      image_ref: used-image
    configuration: cfg-st
    bos_parameters:
      boot_sets:
        compute:
          node_groups:
            - group1
"#;
    let mut sat: SatFile = serde_yaml::from_str(yaml).unwrap();
    sat.filter(false, true).unwrap();
    // Only used-image should remain
    let images = sat.images.unwrap();
    assert_eq!(images.len(), 1);
    assert_eq!(images[0].name, "used-image");
  }

  #[test]
  fn filter_neither_flag_is_noop() {
    let yaml = r#"
configurations:
  - name: cfg1
    layers:
      - name: layer1
        git:
          url: https://example.com/repo.git
          branch: main
images:
  - name: img1
    ims:
      is_recipe: false
      id: abc-123
session_templates:
  - name: st1
    image:
      image_ref: img1
    configuration: cfg1
    bos_parameters:
      boot_sets:
        compute:
          node_groups:
            - group1
"#;
    let mut sat: SatFile = serde_yaml::from_str(yaml).unwrap();
    sat.filter(false, false).unwrap();
    assert!(sat.images.is_some());
    assert!(sat.session_templates.is_some());
    assert!(sat.configurations.is_some());
  }
}
