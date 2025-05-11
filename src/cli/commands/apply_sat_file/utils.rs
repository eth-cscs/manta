use image::Image;
use manta_backend_dispatcher::error::Error;
use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};

use self::sessiontemplate::SessionTemplate;

#[derive(Deserialize, Serialize, Debug)]
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
  pub fn filter(&mut self, image_only: bool, session_template_only: bool) {
    // Clean SAT template file if user only wan'ts to process the 'images' section. In this case,
    // we will remove 'session_templates' section from SAT fiel and also the entries in
    // 'configurations' section not used
    if image_only {
      let image_vec_opt: Option<&Vec<Image>> = self.images.as_ref();

      let configuration_name_image_vec: Vec<String> = match image_vec_opt {
        Some(image_vec) => image_vec
          .iter()
          .filter_map(|sat_template_image| {
            sat_template_image.configuration.clone()
          })
          .collect(),
        None => {
          eprintln!("ERROR - 'images' section missing in SAT file");
          std::process::exit(1);
        }
      };

      // Remove configurations not used by any image
      self
        .configurations
        .as_mut()
        .unwrap_or(&mut Vec::new())
        .retain(|configuration| {
          configuration_name_image_vec.contains(&configuration.name)
        });

      // Remove section "session_templates"
      self.session_templates = None;
    }

    // Clean SAT template file if user only wan'ts to process the 'session_template' section. In this case,
    // we will remove 'images' section from SAT fiel and also the entries in
    // 'configurations' section not used
    if session_template_only {
      let sessiontemplate_vec_opt: Option<&Vec<SessionTemplate>> =
        self.session_templates.as_ref();

      let image_name_sessiontemplate_vec: Vec<String> = self
        .session_templates
        .as_ref()
        .unwrap_or(&Vec::new())
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
      self
        .images
        .as_mut()
        .unwrap_or(&mut Vec::new())
        .retain(|image| image_name_sessiontemplate_vec.contains(&image.name));

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
            eprintln!(
              "ERROR - 'session_templates' section not defined in SAT file"
            );
            std::process::exit(1);
          }
        };

      // Get configuration names from images used by the session templates
      let configuration_name_image_vec: Vec<String> = self
        .images
        .as_ref()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|image| image.configuration.as_ref().unwrap().clone())
        .collect();

      // Merge configuration names from images and session templates
      let configuration_to_keep_vec = [
        configuration_name_image_vec,
        configuration_name_sessiontemplate_vec,
      ]
      .concat();

      // Remove configurations not used by any sessiontemplate or image used by the
      // sessiontemplate

      if self.configurations.is_some() {
        self
          .configurations
          .as_mut()
          .unwrap_or(&mut Vec::new())
          .retain(|configuration| {
            configuration_to_keep_vec.contains(&configuration.name)
          })
      } else {
        self.configurations = None;
      }
    }
  }
}

/// struct to represent the `session_templates` section in SAT file
pub mod sessiontemplate {
  use std::collections::HashMap;
  use strum_macros::Display;

  use serde::{Deserialize, Serialize};

  #[derive(Deserialize, Serialize, Debug)]
  pub struct SessionTemplate {
    pub name: String,
    pub image: Image,
    pub configuration: String,
    pub bos_parameters: BosParamters,
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImsDetails {
    Name { name: String },
    Id { id: String },
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Image {
    Ims { ims: ImsDetails },
    ImageRef { image_ref: String },
  }

  #[derive(Deserialize, Serialize, Debug)]
  pub struct BosParamters {
    pub boot_sets: HashMap<String, BootSet>,
  }

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

  #[derive(Deserialize, Serialize, Debug, Display)]
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

  #[derive(Deserialize, Serialize, Debug)]
  pub enum Arch {
    #[serde(rename(serialize = "aarch64", deserialize = "aarch64"))]
    Aarch64,
    #[serde(rename(serialize = "x86_64", deserialize = "x86_64"))]
    X86_64,
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImageIms {
    NameIsRecipe { name: String, is_recipe: bool },
    IdIsRecipe { id: String, is_recipe: bool },
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum ImageBaseIms {
    NameType { name: String, r#type: String },
    IdType { id: String, r#type: String },
    BackwardCompatible { is_recipe: Option<bool>, id: String },
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Filter {
    Prefix { prefix: String },
    Wildcard { wildcard: String },
    Arch { arch: Arch },
  }

  #[derive(Deserialize, Serialize, Debug)]
  pub struct Product {
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<String>,
    r#type: String,
    filter: Filter,
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Base {
    Ims { ims: ImageBaseIms },
    Product { product: Product },
    ImageRef { image_ref: String },
  }

  // Used for backguard compatibility
  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum BaseOrIms {
    Base { base: Base },
    Ims { ims: ImageIms },
  }

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

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
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

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum Git {
    GitCommit { url: String, commit: String },
    GitBranch { url: String, branch: String },
    GitTag { url: String, tag: String },
  }

  #[derive(Deserialize, Serialize, Debug)]
  #[serde(untagged)] // <-- this is important. More info https://serde.rs/enum-representations.html#untagged
  pub enum LayerType {
    Git { git: Git },
    Product { product: Product },
  }

  #[derive(Deserialize, Serialize, Debug)]
  pub struct Layer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default = "default_playbook")]
    pub playbook: String, // This field is optional but with default value. Therefore we won't
    #[serde(flatten)]
    pub layer_type: LayerType,
  }

  fn default_playbook() -> String {
    "site.yml".to_string()
  }

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

pub mod sat_file_image_old {
  use serde::{Deserialize, Serialize};

  #[derive(Deserialize, Serialize, Debug)]
  pub struct Ims {
    is_recipe: bool,
    id: String,
  }

  #[derive(Deserialize, Serialize, Debug)]
  pub struct Product {
    name: String,
    version: String,
    r#type: String,
  }
}

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
    return Err(Error::Message("Invalid format".to_string()));
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
        if map.contains_key(&Value::String(key.to_string())) {
          // Use existing map
          map.get_mut(&Value::String(key.to_string())).unwrap()
        } else {
          // Create new map and insert
          map.insert(
            Value::String(key.to_string()),
            Value::Mapping(Mapping::new()),
          );
          map.get_mut(&Value::String(key.to_string())).unwrap()
        }
      } else {
        // In case the structure is not as expected; should not happen in this logic
        return Err(Error::Message(
          "Unexpected structure encountered".to_string(),
        ));
      };
      current_level = next_level;
    }
  }

  Ok(root)
}

pub fn render_jinja2_sat_file_yaml(
  sat_file_content: &String,
  values_file_content_opt: Option<&String>,
  value_cli_vec_opt: Option<Vec<String>>,
) -> Value {
  let mut env = minijinja::Environment::new();
  // Set/enable debug in order to force minijinja to print debug error messages which are more
  // descriptive. Eg https://github.com/mitsuhiko/minijinja/blob/main/examples/error/src/main.rs#L4-L5
  env.set_debug(true);
  // Set lines starting with `#` as comments
  env.set_syntax(
    minijinja::syntax::SyntaxConfig::builder()
      .line_comment_prefix("#")
      .build()
      .unwrap(),
  );
  // Set 'String' as undefined behaviour meaning, missing values won't pass the template
  // rendering
  env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);

  // Render session values file
  let mut values_file_yaml: Value = if let Some(values_file_content) =
    values_file_content_opt
  {
    log::info!("'Session vars' file provided. Going to process SAT file as a jinja template.");
    log::info!("Expand variables in 'session vars' file");
    // Read sesson vars file and parse it to YAML
    let values_file_yaml: Value =
      serde_yaml::from_str(values_file_content).unwrap();
    // Render session vars file with itself (copying ansible behaviour where the ansible vars
    // file is also a jinja template and combine both vars and values in it)
    let values_file_rendered = env
      .render_str(values_file_content, values_file_yaml)
      .expect("ERROR - Error parsing values file to YAML. Exit");
    serde_yaml::from_str(&values_file_rendered).unwrap()
  } else {
    serde_yaml::from_str(sat_file_content).unwrap()
  };

  // Convert variable values sent by cli argument from dot notation to yaml format
  log::debug!("Convert variable values sent by cli argument from dot notation to yaml format");
  if let Some(value_option_vec) = value_cli_vec_opt {
    for value_option in value_option_vec {
      let cli_var_context_yaml_rslt = dot_notation_to_yaml(&value_option);
      let cli_var_context_yaml = match cli_var_context_yaml_rslt {
        Ok(value) => value,
        Err(e) => {
          eprintln!("ERROR - {:#?}", e);
          panic!();
        }
      };
      values_file_yaml =
        merge_yaml(values_file_yaml.clone(), cli_var_context_yaml).unwrap();
    }
  }

  // render sat template file
  log::info!("Expand variables in 'SAT file'");
  let sat_file_rendered_rslt =
    env.render_str(sat_file_content, values_file_yaml);

  let sat_file_rendered = match sat_file_rendered_rslt {
    Ok(sat_file_rendered) => sat_file_rendered,
    Err(err) => {
      eprintln!("ERROR - Could not render template: {:#}", err);
      // render causes as well
      let mut err = &err as &dyn std::error::Error;
      while let Some(next_err) = err.source() {
        eprintln!();
        eprintln!("caused by: {:#}", next_err);
        err = next_err;
      }

      std::process::exit(1);
    }
  };

  // Disable debug
  env.set_debug(false);

  let sat_file_yaml: Value = serde_yaml::from_str(&sat_file_rendered).unwrap();

  sat_file_yaml
}
