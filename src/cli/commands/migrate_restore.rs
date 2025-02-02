use chrono::Local;
use dialoguer::Confirm;
use humansize::DECIMAL;
use indicatif::{ProgressBar, ProgressStyle};
use md5::Digest;
use mesa::bos::template::http_client::v2::types::BosSessionTemplate;
use mesa::cfs::configuration::http_client::v3::types::{
    cfs_configuration_request::CfsConfigurationRequest,
    cfs_configuration_response::CfsConfigurationResponse,
};
use mesa::hsm::group::{
    http_client::{create_new_group, delete_group},
    types::Group,
};
use mesa::ims::image::utils::get_by_name;
use mesa::ims::image::{
    http_client::{
        patch,
        types::{Image, ImsImageRecord2Update, Link},
    },
    utils::get_fuzzy,
};
use mesa::ims::s3_client::BAR_FORMAT;
use mesa::{bos, cfs, ims};
use serde::{Deserialize, Serialize};
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
use std::path::PathBuf;
use std::process::exit;

// As per https://cray-hpe.github.io/docs-csm/en-13/operations/image_management/import_external_image_to_ims/
/* #[derive(Serialize, Deserialize, Debug, Clone)]
struct Link {
    pub path: String,
    #[serde(rename = "type", default = "default_link_type")]
    pub r#type: String,
} */

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Artifact {
    pub link: Link,
    pub md5: String,
    #[serde(rename = "type")]
    pub r#type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct ImageManifest {
    pub created: String,
    #[serde(default = "default_version")]
    pub version: String,
    pub artifacts: Vec<Artifact>,
}
// This is ridiculous
// fn default_link_type() -> String {
//     "s3".to_string()
// }

fn default_version() -> String {
    "1.0".to_string()
}

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_file: Option<&String>,
    cfs_file: Option<&String>,
    hsm_file: Option<&String>,
    ims_file: Option<&String>,
    image_dir: Option<&String>,
    prehook: Option<&String>,
    posthook: Option<&String>,
) {
    log::info!(
        "Migrate_restore \n Pre-hook: {}\n Post-hook: {}\n BOS_file: {}\n CFS_file: {}\n IMS_file: {}\n HSM_file: {}",
        &prehook.unwrap_or(&"none".to_string()),
        &posthook.unwrap_or(&"none".to_string()),
        bos_file.unwrap(),
        cfs_file.unwrap(),
        ims_file.unwrap(),
        hsm_file.unwrap()
    );
    println!(
        "Migrate_restore\n Prehook: {}\n Posthook: {}\n BOS_file: {}\n CFS_file: {}\n IMS_file: {}\n HSM_file: {}",
        &prehook.unwrap_or(&"none".to_string()),
        &posthook.unwrap_or(&"none".to_string()),
        bos_file.unwrap(),
        cfs_file.unwrap(),
        ims_file.unwrap(),
        hsm_file.unwrap()
    );
    if prehook.is_some() {
        match crate::common::hooks::check_hook_perms(prehook).await {
            Ok(_) => log::debug!("Pre-hook script exists and is executable."),
            Err(e) => {
                log::error!("{}. File: {}", e, &prehook.unwrap());
                exit(2);
            }
        };
    }
    if posthook.is_some() {
        match crate::common::hooks::check_hook_perms(posthook).await {
            Ok(_) => log::debug!("Post-hook script exists and is executable."),
            Err(e) => {
                log::error!("{}. File: {}", e, &posthook.unwrap());
                exit(2);
            }
        };
    }
    // println!("Migrate restore of the following image:\n\tBOS file: {}\n\tCFS file: {}\n\tIMS file: {}\n\tHSM file: {}", &bos_file.unwrap(), &cfs_file.unwrap(), &ims_file.unwrap(), &hsm_file.unwrap() );
    if !PathBuf::from(&bos_file.unwrap()).exists() {
        eprintln!(
            "Error, file {} does not exist or cannot be open.",
            &bos_file.unwrap()
        );
        std::process::exit(1)
    }
    if !PathBuf::from(&cfs_file.unwrap()).exists() {
        eprintln!(
            "Error, file {} does not exist or cannot be open.",
            &cfs_file.unwrap()
        );
        std::process::exit(1)
    }
    if !PathBuf::from(&ims_file.unwrap()).exists() {
        eprintln!(
            "Error, file {} does not exist or cannot be open.",
            &ims_file.unwrap()
        );
        std::process::exit(1)
    }
    if !PathBuf::from(&hsm_file.unwrap()).exists() {
        eprintln!(
            "Error, file {} does not exist or cannot be open.",
            &hsm_file.unwrap()
        );
        std::process::exit(1)
    }

    // ========================================================================================================
    let current_timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let mut ims_image_manifest = ImageManifest {
        created: current_timestamp.to_string(),
        version: "1.0".to_string(),
        artifacts: vec![],
    };

    let backup_ims_file = ims_file.unwrap().to_string();
    let backup_cfs_file = cfs_file.unwrap().to_string();
    let backup_bos_file = bos_file.unwrap().to_string();
    let backup_hsm_file = hsm_file.unwrap().to_string();

    let ims_image_name: String = get_image_name_from_ims_file(&backup_ims_file);
    println!(" Image name: {}", ims_image_name);

    println!(
        "\tinitrd file: {}",
        image_dir.unwrap().to_string() + "/initrd"
    );
    println!(
        "\tkernel file: {}",
        image_dir.unwrap().to_string() + "/kernel"
    );
    println!(
        "\trootfs file: {}",
        image_dir.unwrap().to_string() + "/rootfs"
    );

    // These should come from the manifest, but let's assume these values are correct
    let vec_backup_image_files = vec![
        image_dir.unwrap().to_string() + "/initrd",
        image_dir.unwrap().to_string() + "/kernel",
        image_dir.unwrap().to_string() + "/rootfs",
    ];

    for file in &vec_backup_image_files {
        if !PathBuf::from(&file).exists() {
            eprintln!("Error, file {} does not exist or cannot be open.", &file);
            std::process::exit(1)
        }
    }

    println!();
    if prehook.is_some() {
        println!("Running the pre-hook {}", &prehook.unwrap());
        match crate::common::hooks::run_hook(prehook).await {
            Ok(_code) => log::debug!("Pre-hook script completed ok. RT={}", _code),
            Err(_error) => {
                log::error!("{}", _error);
                exit(2);
            }
        };
    }

    println!("Calculating image artifact checksum...");
    calculate_image_checksums(&mut ims_image_manifest, &vec_backup_image_files);

    // println!("{:?}", ims_image_manifest);

    // Do we have another image with this name?
    println!("\n\nRegistering image with IMS...");
    let ims_image_id_rslt = ims_register_image(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &ims_image_name,
    )
    .await;

    let ims_image_id: String = match ims_image_id_rslt {
        Ok(value) => value,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };

    println!("Ok, IMS image ID: {}", &ims_image_id);

    println!("\nUploading image artifacts to s3...");
    s3_upload_image_artifacts(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &ims_image_id,
        &mut ims_image_manifest,
        &vec_backup_image_files,
    )
    .await;
    // println!();
    // println!("Image manifest: {:?}", ims_image_manifest);
    println!("\nUpdating IMS image record with the new location in s3...");
    log::debug!("Updating image record with location of the newly generated manifest.json data");
    ims_update_image_add_manifest(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &ims_image_name,
        &ims_image_id,
    )
    .await;
    println!("Ok");

    println!("\nCreating HSM group...");
    create_hsm_group(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &backup_hsm_file,
    )
    .await;
    println!("Ok");

    println!("\nUploading CFS configuration...");
    // create a new CFS configuration based on the original CFS file backed up previously
    // this operation is simple as the file only has git repos and commits
    create_cfs_config(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &backup_cfs_file,
    )
    .await;

    println!("\nUploading BOS sessiontemplate...");
    // Create a new BOS session template based on the original BOS file backed previously
    create_bos_sessiontemplate(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &backup_bos_file,
        &ims_image_id,
    )
    .await;
    if posthook.is_some() {
        println!("Running the post-hook {}", &posthook.unwrap());
        match crate::common::hooks::run_hook(posthook).await {
            Ok(_code) => log::debug!("Post-hook script completed ok. RT={}", _code),
            Err(_error) => {
                log::error!("{}", _error);
                exit(2);
            }
        };
    }
    println!("\nDone, the image bundle, HSM group, CFS configuration and BOS sessiontemplate have been restored.");

    // ========================================================================================================
}

async fn create_bos_sessiontemplate(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_file: &String,
    ims_image_id: &String,
) {
    let file_content =
        File::open(bos_file).expect(&format!("Unable to read BOS JSON file '{}'", bos_file));

    let bos_json: BosSessionTemplate = serde_json::from_reader(BufReader::new(file_content))
        .expect("BOS JSON file does not have correct format.");

    let bos_sessiontemplate_name = bos_json.name.unwrap();

    // BOS sessiontemplates need the new ID of the image!
    log::debug!("BOS sessiontemplate name: {}", &bos_sessiontemplate_name);

    let vector = bos::template::http_client::v2::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&bos_sessiontemplate_name),
    )
    .await
    .unwrap_or_else(|error| {
        eprint!(
            "Error: unable to query CSM to get list of BOS sessiontemplates. Error returned: {}",
            error
        );
        std::process::exit(1);
    });

    log::debug!("BOS sessiontemplate filtered: {:#?}", vector);

    if !vector.is_empty() {
        println!("There already exists a BOS sessiontemplate with name '{}'. It can be replaced, but it's dangerous.", &bos_sessiontemplate_name);
        let confirmation = Confirm::new()
            .with_prompt("Do you want to overwrite it?")
            .interact()
            .unwrap();

        if !confirmation {
            println!("Looks like you do not want to continue, bailing out.");
            std::process::exit(2)
        } else {
            match bos::template::http_client::v2::delete(
                shasta_token,
                shasta_base_url,
                shasta_root_cert,
                &bos_sessiontemplate_name,
            )
            .await
            {
                Ok(_) => log::debug!(
                    "Ok BOS session template {}, deleted.",
                    &bos_sessiontemplate_name
                ),
                Result::Err(err1) => panic!(
                    "Error, unable to delete BOS session template. Cannot continue. Error: {}",
                    err1
                ),
            };
        }
    }

    let file_content =
        File::open(bos_file).expect(&format!("Unable to read BOS JSON file '{}'", bos_file));

    let mut bos_sessiontemplate: BosSessionTemplate =
        serde_json::from_reader(BufReader::new(file_content))
            .expect("BOS JSON file does not have correct format.");

    // This is as ugly as it can be
    // println!("Path: {}", bos_sessiontemplate.clone().boot_sets.clone().unwrap().compute.clone().unwrap().path.clone().unwrap().to_string());
    let path_modified = format!("s3://boot-images/{}/manifest.json", ims_image_id);
    bos_sessiontemplate
        .boot_sets
        .as_mut()
        .unwrap()
        .get_mut("compute")
        .unwrap()
        .path = Some(path_modified);

    log::debug!("BOS sessiontemplate loaded:\n{:#?}", bos_sessiontemplate);
    log::debug!("BOS sessiontemplate modified:\n{:#?}", &bos_sessiontemplate);

    match bos::template::http_client::v2::put(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &bos_sessiontemplate,
        &bos_sessiontemplate_name,
    )
    .await
    {
        Ok(_result) => println!(
            "Ok, BOS session template {} created successfully.",
            &bos_sessiontemplate_name
        ),
        Err(e1) => panic!(
            "Error, unable to create BOS sesiontemplate. Error returned by CSM API: {}",
            e1
        ),
    }
}

/// Creates a CFS config on the current CSM system, based on the CFS file generated by manta migrate backup
/// panics with an error message if creation fails
async fn create_cfs_config(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    cfs_file: &String,
) {
    let file_content =
        File::open(cfs_file).expect(&format!("Unable to read CFS JSON file '{}'", cfs_file));

    let cfs_configuration: CfsConfigurationResponse =
        serde_json::from_reader(BufReader::new(file_content))
            .expect("CFS JSON file does not have correct format.");

    // CFS needs to be cleaned up when loading into the system, the filed lastUpdate should not exist
    let cfs_config_name = cfs_configuration.name;

    // Get all CFS configurations, this is ugly
    let cfs_config_vec = cfs::configuration::http_client::v3::get(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        Some(&cfs_config_name),
    )
    .await
    .unwrap_or_else(|error| {
        eprint!(
            "Error: Unable to fetch CFS configuration. Error returned by CSM API: {}",
            error
        );
        std::process::exit(1);
    });

    if !cfs_config_vec.is_empty() {
        println!("There already exists a CFS configuration with name {}. It can be replaced, but it's dangerous as it can trigger automated node reconfiguration.", &cfs_config_name);
        let confirmation = Confirm::new()
            .with_prompt("Do you want to overwrite it?")
            .interact()
            .unwrap();

        if !confirmation {
            println!("Looks like you do not want to continue, bailing out.");
            std::process::exit(2)
        }

        match cfs::configuration::http_client::v3::delete(
            shasta_token,
            shasta_base_url,
            shasta_root_cert,
            cfs_config_name.as_str(),
        )
        .await
        {
            Ok(_) => log::debug!("Ok CFS configuration {}, deleted.", cfs_config_name),
            Result::Err(error) => panic!(
                "Error, unable to delete configuration. Cannot continue. Error: {}",
                error
            ),
        };
    }
    // At this point we're sure there's either no CFS config with that name
    // or that the user wants to overwrite it, so let's do it

    let file_content =
        File::open(cfs_file).expect(&format!("Unable to read CFS JSON file '{}'", cfs_file));

    let cfs_configuration: CfsConfigurationRequest =
        serde_json::from_reader(BufReader::new(file_content))
            .expect("CFS JSON file does not have correct format.");

    log::debug!("CFS config:\n{:#?}", &cfs_configuration);

    match cfs::configuration::http_client::v3::put(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &cfs_configuration,
        cfs_config_name.as_str(),
    )
    .await
    {
        Ok(result) => {
            log::debug!("Ok, result: {:#?}", result);
            println!(
                "Ok, CFS configuration {} created successfully.",
                &cfs_config_name
            );
        }
        Err(e1) => panic!(
            "Error, unable to create CFS configuration. Error returned by CSM API: {}",
            e1
        ),
    }
}
/// Add the image manifest field to an IMS image record
/// the manifest field will be: s3://boot-images/{ims_image_id}/manifest.json
async fn ims_update_image_add_manifest(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    ims_image_name: &String,
    ims_image_id: &String,
) {
    match get_fuzzy(shasta_token,
                         shasta_base_url,
                         shasta_root_cert,
                         &["".to_string()], // hsm_group_name
                         Some(ims_image_name.clone().as_str()),
                         None).await {
        Ok(_vector) => {
            if _vector.is_empty() {
                panic!("Error: there are no images stored with id {} in IMS. Unable to update the image manifest", &ims_image_id);
            }
        },
        Err(error) =>  panic!("Error: Unable to determine if there are other images in IMS with the name {}. Error code: {}", &ims_image_name, &error),
    };

    let _ims_record = ims::image::http_client::types::Image {
        name: ims_image_name.clone().to_string(),
        id: Some(ims_image_id.clone().to_string()),
        created: None,
        arch: None,
        link: Some(ims::image::http_client::types::Link {
            etag: None,
            path: format!(
                "s3://boot-images/{}/manifest.json",
                &ims_image_id.to_string()
            ),
            r#type: "s3".to_string(),
        }),
    };

    // arch is not on CSM 1.3
    // {
    //   "link": {
    //     "path": "s3://boot-images/1fb58f4e-ad23-489b-89b7-95868fca7ee6/manifest.json",
    //     "etag": "f04af5f34635ae7c507322985e60c00c-131",
    //     "type": "s3"
    //   },
    //   "arch": "aarch64"
    // }

    let ims_link = Link {
        etag: None,
        path: format!(
            "s3://boot-images/{}/manifest.json",
            &ims_image_id.to_string()
        ),
        r#type: "s3".to_string(),
    };
    let rec = ImsImageRecord2Update {
        link: ims_link,
        arch: None,
    };

    // println!("New IMS link {:?}", &rec);
    match patch(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &ims_image_id.to_string(),
        &rec,
    )
    .await
    {
        Ok(_returned) => log::debug!("Returned json: {}", _returned),
        Err(e) => panic!(
            "Error, unable to modify the record of the image. Err msg: {}",
            e
        ),
    };
}

/// Uploads to s3 under boot-images/ims_image_id all the files that
/// vec_image_files refers to. If upload successful, it modifies
/// ImageManifest to point to the right place within s3
async fn s3_upload_image_artifacts(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    ims_image_id: &String,
    ims_image_manifest: &mut ImageManifest,
    vec_image_files: &Vec<String>,
) {
    let bucket_name = "boot-images";
    let object_path = ims_image_id;

    // Connect and auth to S3
    let sts_value =
        match ims::s3_client::s3_auth(shasta_token, shasta_base_url, shasta_root_cert).await {
            Ok(sts_value) => {
                log::debug!("STS token:\n{:#?}", sts_value);
                sts_value
            }
            Err(error) => panic!(
                "Unable to authenticate with s3 when uploading images. Error: {}",
                error
            ),
        };

    for file in vec_image_files {
        let filename = Path::new(file).file_name().unwrap();
        let file_size = match fs::metadata(file) {
            Ok(_file_metadata) => {
                let res: String = humansize::format_size(_file_metadata.len(), DECIMAL);
                res
            }
            Err(e) => {
                eprintln!(
                    "Unable to fetch file metadata info, faking the value. Error: {}",
                    e
                );
                "-1".to_string()
            }
        };

        let full_object_path = format!("{}/{}", &object_path, &filename.to_string_lossy());
        println!(
            "File {:?} ({}) to s3://{}/{}.",
            &file, &file_size, &bucket_name, &full_object_path
        );
        let etag: String;
        if fs::metadata(file).unwrap().len() > 1024 * 1024 * 5 {
            etag = match ims::s3_client::s3_multipart_upload_object(
                &sts_value,
                &full_object_path,
                bucket_name,
                file,
            )
            .await
            {
                Ok(result) => {
                    log::debug!("Artifact uploaded successfully.");
                    result
                }
                Err(error) => panic!("Unable to upload file to s3. Error {}", error),
            };
        } else {
            etag = match ims::s3_client::s3_upload_object(
                &sts_value,
                &full_object_path,
                bucket_name,
                file,
            )
            .await
            {
                Ok(result) => {
                    println!("Ok");
                    result
                }
                Err(error) => panic!("Unable to upload file to s3. Error {}", error),
            };
        }

        // I'm pretty sure there's a better way to do this...
        if file.contains("kernel") {
            for artifact in ims_image_manifest.artifacts.iter_mut() {
                if !etag.is_empty() {
                    // assign eTag if returned by s3, otherwise set to none
                    artifact.link.etag = Some(etag.clone());
                }
                if artifact.r#type.contains("kernel") {
                    artifact.link.path = "s3://".to_string()
                        + bucket_name
                        + "/"
                        + &object_path.to_string()
                        + "/kernel";
                    break;
                }
            }
        } else if file.contains("rootfs") {
            for artifact in ims_image_manifest.artifacts.iter_mut() {
                if !etag.is_empty() {
                    // assign eTag if returned by s3, otherwise set to none
                    artifact.link.etag = Some(etag.clone());
                }
                if artifact.r#type.contains("rootfs") {
                    artifact.link.path = "s3://".to_string()
                        + bucket_name
                        + "/"
                        + &object_path.to_string()
                        + "/rootfs";
                    break;
                }
            }
        } else if file.contains("initrd") {
            for artifact in ims_image_manifest.artifacts.iter_mut() {
                if !etag.is_empty() {
                    // assign eTag if returned by s3, otherwise set to none
                    artifact.link.etag = Some(etag.clone());
                }
                if artifact.r#type.contains("initrd") {
                    artifact.link.path = "s3://".to_string()
                        + bucket_name
                        + "/"
                        + &object_path.to_string()
                        + "/initrd";
                    break;
                }
            }
        }
    }
    log::debug!("Writing the new manifest.json file with the correct new ID");
    let new_manifest_file_name = String::from("new-manifest.json");
    let new_manifest_file_path = Path::new(vec_image_files.first().unwrap())
        .parent()
        .unwrap()
        .join(&new_manifest_file_name);
    let new_manifest_file = File::create(&new_manifest_file_path)
        .expect("new manifest.json file could not be created.");
    serde_json::to_writer_pretty(&new_manifest_file, &ims_image_manifest)
        .expect("Unable to write new manifest.json file");

    log::debug!("Uploading the new manifest.json file");
    let manifest_full_object_path = format!("{}/manifest.json", &object_path);
    println!(
        "File {:?} -> s3://{}/{}.",
        &new_manifest_file_name, &bucket_name, &manifest_full_object_path
    );

    match ims::s3_client::s3_upload_object(
        &sts_value,
        &manifest_full_object_path,
        bucket_name,
        &new_manifest_file_path.to_owned().to_string_lossy(),
    )
    .await
    {
        Ok(_result) => {
            println!("OK");
        }
        Err(error) => panic!("Unable to upload file to s3. Error {}", error),
    };
}
/// Return the md5sum of a file
fn file_md5sum(filename: PathBuf) -> Digest {
    log::debug!("File {:?}...", &filename);

    let f = File::open(filename).unwrap();
    // Find the length of the file
    let len = f.metadata().unwrap().len();
    // Decide on a reasonable buffer size (100MB in this case, fastest will depend on hardware)
    let buf_len = len.min(100_000_000) as usize;
    let mut buf = BufReader::with_capacity(buf_len, f);
    let mut context = md5::Context::new();
    let bar = ProgressBar::new(len);
    bar.set_style(ProgressStyle::with_template(BAR_FORMAT).unwrap());

    loop {
        // Get a chunk of the file
        let part = buf.fill_buf().unwrap();
        // If that chunk was empty, the reader has reached EOF
        if part.is_empty() {
            break;
        }
        // Add chunk to the md5
        context.consume(part);
        // Tell the buffer that the chunk is consumed
        let part_len = part.len();
        buf.consume(part_len);
        bar.inc(part_len as u64);
        // println!("Consumed {} out of {}; step {}/{}",
        //          humansize::format_size(part_len, DECIMAL),
        //          humansize::format_size(len, DECIMAL),
        //          i, len as usize/buf_len);
    }
    let digest = context.compute();
    bar.finish();

    // println!("{:x}\t{:?}", digest, &filename);
    digest
}
/// Calculates the md5sum of all the files in the `vec_backup_image_files` vector and updates
///  the image manifest at `ims_image_manifest`
fn calculate_image_checksums(
    image_manifest: &mut ImageManifest,
    vec_backup_image_files: &Vec<String>,
) {
    for file in vec_backup_image_files {
        let file_size = match fs::metadata(file) {
            Ok(_file_metadata) => {
                let res: String = humansize::format_size(_file_metadata.len(), DECIMAL);
                res
            }
            Err(e) => {
                eprintln!(
                    "Unable to fetch file metadata info, faking the value. Error: {}",
                    e
                );
                "-1".to_string()
            }
        };
        println!("File {:?} ({})...", &file, &file_size);
        let artifact;
        let mut fp = PathBuf::new();
        fp.push(file);
        let digest = file_md5sum(fp);
        // println!("{:x}\t{:?}", digest, file);

        if file.contains("kernel") {
            artifact = Artifact {
                md5: format!("{:x}", digest),
                link: Link {
                    path: "path".to_string(),
                    r#type: "s3".to_string(),
                    etag: None,
                },
                r#type: "application/vnd.cray.image.kernel".to_string(),
            };
        } else if file.contains("rootfs") {
            artifact = Artifact {
                md5: format!("{:x}", digest),
                link: Link {
                    path: "path".to_string(),
                    r#type: "s3".to_string(),
                    etag: None,
                },
                r#type: "application/vnd.cray.image.rootfs.squashfs".to_string(),
            };
        } else {
            artifact = Artifact {
                md5: format!("{:x}", digest),
                link: Link {
                    path: "path".to_string(),
                    r#type: "s3".to_string(),
                    etag: None,
                },
                r#type: "application/vnd.cray.image.initrd".to_string(),
            };
        }
        image_manifest.artifacts.push(artifact);
    }
}

/// Registers in IMS a new image and returns the new id to pass to s3
async fn ims_register_image(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    ims_image_name: &String,
) -> anyhow::Result<String> {
    let ims_record = Image {
        name: ims_image_name.clone().to_string(),
        id: None,
        created: None,
        link: None,
        arch: None,
    };

    let list_images_with_same_name = get_by_name(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &["".to_string()], // hsm_group_name
        Some(ims_image_name.clone().as_str()),
        None,
    )
    .await?;

    if !list_images_with_same_name.is_empty() {
        println!("There is already at least one record for image name {} in IMS do you want to create a new one (the previous one will not be deleted).", &ims_image_name);
        println!("Current IMS record(s): {:?}", &list_images_with_same_name);
        let confirmation = Confirm::new()
            .with_prompt("Do you want to create a new record?")
            .interact()
            .unwrap();

        if !confirmation {
            println!("Looks like you do not want to continue, bailing out.");
            std::process::exit(2)
        }
    }

    let json_response =
        ims::image::http_client::post(shasta_token, shasta_base_url, shasta_root_cert, &ims_record)
            .await?;

    Ok(json_response["id"].to_string().replace('"', ""))
}

/// Gets the image name off an IMS yaml file
pub fn get_image_name_from_ims_file(ims_file: &String) -> String {
    // load into memory
    let ims_data =
        fs::read_to_string(PathBuf::from(&ims_file)).expect("Unable to read IMS file file");

    let ims_json: serde_json::Value =
        serde_json::from_str(&ims_data).expect("HSM JSON file does not have correct format.");

    // The file looks like this, we only want the field "name"
    // {
    //   "created": "2023-10-13T19:13:46.558252+00:00",
    //   "id": "58a205ff-d98a-46ad-a32d-87657c90814e",
    //   "link": {
    //     "etag": "d1f2a80c4725dc0d42b809dabcc065d8",
    //     "path": "s3://boot-images/58a205ff-d98a-46ad-a32d-87657c90814e/manifest.json",
    //     "type": "s3"
    //   },
    //   "name": "gele-cos-3.2.2"
    // }
    //
    ims_json[0]["name"].clone().to_string().replace('"', "")
}

// Anything in this function is critical, so the asserts will kill further processing
pub async fn create_hsm_group(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_file: &String,
) {
    // load into memory
    let hsm_data =
        fs::read_to_string(PathBuf::from(hsm_file)).expect("Unable to read HSM JSON file");

    let _hsm_json: serde_json::Value =
        serde_json::from_str(&hsm_data).expect("HSM JSON file does not have correct format.");

    // Create new HSM group if not existing

    // Parse HSM group file
    // The file looks like this: [{"gele":["x1001c7s1b1n1","x1001c7s1b0n0","x1001c7s1b1n0","x1001c7s1b0n1"]}]
    let mut hsm_vec: Vec<Group> = serde_json::from_str(hsm_data.as_str()).unwrap();
    log::debug!("HSM vector {:#?}", &hsm_vec);

    // for hsm in hsm_vec.iter() {
    //     let mut hsm: HsmGroup = hsm.clone();
    // }
    let mut hsm: Group = hsm_vec.remove(0);
    log::debug!("HSM group to create {:#?}", &hsm_data.as_str());

    // let exclusive:bool = false; // Make sure this is false, so we can test this without impacting other HSM groups
    // // the following xnames are part of HSM group "gele"
    // let xnames:Vec<String> = vec!["x1001c7s1b0n0".to_string(),
    //                               "x1001c7s1b0n1".to_string(),
    //                               "x1001c7s1b1n0".to_string(),
    //                               "x1001c7s1b1n1".to_string()];
    // let description = "Test group created by function mesa test_1_hsm";
    // let tags:Vec<String> = vec!["dummyTag1".to_string(), "dummyTag2".to_string()];
    // // let tags= vec![]; // sending an empty vector works
    // let hsm_group_name_opt = "manta_created_hsm".to_string();
    if hsm.tags.is_none() {
        hsm.tags = vec![].into();
    }
    if hsm.exclusive_group.is_none() {
        hsm.exclusive_group = Some(false.to_string());
    }
    // This couldn't be uglier, I know
    let hsm2: Group = hsm.clone();

    // Create the HSM group
    match create_new_group(
        shasta_token,
        shasta_base_url,
        shasta_root_cert,
        &hsm.label,
        &hsm.members.unwrap().ids.unwrap(),
        &hsm.exclusive_group.unwrap(),
        &hsm.description.unwrap(),
        &hsm.tags.unwrap(),
    )
    .await
    {
        Ok(_) => {
            println!(
                "The HSM group {} has been created successfully.",
                &hsm.label
            );
        }
        Err(error) => {
            if error.to_string().to_lowercase().contains("409") {
                println!("The HSM group {} already exists, it is possible to recreate it, but is a dangerous operation", &hsm.label);
                log::error!("Error message {}", error);
                let confirmation = Confirm::new()
                    .with_prompt("Do you want to recreate it?")
                    .interact()
                    .unwrap();

                if confirmation {
                    println!("Looks like you want to continue");
                    match delete_group(shasta_token, shasta_base_url, shasta_root_cert, &hsm.label)
                        .await
                    {
                        Ok(_) => {
                            // try creating the group again
                            match create_new_group(
                                shasta_token,
                                shasta_base_url,
                                shasta_root_cert,
                                &hsm2.label,
                                &hsm2.members.unwrap().ids.unwrap(),
                                &hsm2.exclusive_group.unwrap(),
                                &hsm2.description.unwrap(),
                                &hsm2.tags.unwrap(),
                            )
                            .await
                            {
                                Ok(_json) => {
                                    println!(
                                        "The HSM group {} has been created successfully.",
                                        &hsm2.label
                                    );
                                }
                                Err(e2) => {
                                    log::error!("Error message {}", e2);
                                    panic!("Second error creating a new HSM group. Bailing out. Error returned: '{}'", e2)
                                }
                            }
                        }
                        Err(e1) => {
                            log::error!("Error message {}", e1);
                            panic!(
                                "Error deleting the HSM group {}. Error returned: '{}'",
                                &hsm.label, e1
                            )
                        }
                    }
                } else {
                    println!("Not deleting the group, cannot continue the operation.");
                    std::process::exit(2);
                }
            } else if error.to_string().to_lowercase().contains("400") {
                eprintln!("Unable to create the group, the API returned code 400. This usually means the HSM file is malformed, or has incorrect xnames for this site in it.");
                std::process::exit(2);
            }
        }
    };
}
