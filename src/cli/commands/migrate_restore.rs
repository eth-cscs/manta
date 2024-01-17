use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use mesa::shasta::hsm::http_client::{create_new_hsm_group, delete_hsm_group};
use mesa::shasta::hsm::HsmGroup;
use mesa::shasta::ims::image::http_client::{register_new_image,update_image};
use mesa::shasta::ims::image::{ImsImage,ImsLink,ImsImageRecord2Update};
use mesa::shasta::ims::s3::s3::{s3_auth, s3_upload_object};
use chrono::Local;
use dialoguer::Confirm;
use md5::Digest;
use std::path::Path;
use serde::{Deserialize,Serialize};


// As per https://cray-hpe.github.io/docs-csm/en-13/operations/image_management/import_external_image_to_ims/
#[derive(Serialize, Deserialize, Debug, Clone)]
struct Link {
    pub path: String,
    #[serde(rename = "type", default = "default_link_type")]
    pub r#type: String,
}

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
fn default_link_type() -> String {
    "s3".to_string()
}

fn default_version() -> String {
    "1.0".to_string()
}

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_file:  Option<&String>,
    cfs_file:  Option<&String>,
    hsm_file:  Option<&String>,
    ims_file:  Option<&String>,
    image_dir:  Option<&String>

) {
    log::info!("Migrate_restore; BOS_file={}, CFS_file={}, IMS_file={}, HSM_file={}",bos_file.unwrap(), cfs_file.unwrap(), ims_file.unwrap(), hsm_file.unwrap());
    println!("Migrate restore of the following image:\n\tBOS file: {}\n\tCFS file: {}\n\tIMS file: {}\n\tHSM file: {}", &bos_file.unwrap(), &cfs_file.unwrap(), &ims_file.unwrap(), &hsm_file.unwrap() );

    // ========================================================================================================
    let current_timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    let mut ims_image_manifest = ImageManifest {
        created: current_timestamp.to_string(),
        version: "1.0".to_string(),
        artifacts: vec![],
    };

    let backup_ims_file = ims_file.clone().unwrap().to_string();
    let backup_cfs_file = cfs_file.clone().unwrap().to_string();
    let backup_bos_file = bos_file.clone().unwrap().to_string();
    let backup_hsm_file = hsm_file.clone().unwrap().to_string();


    let ims_image_name: String = get_image_name_from_ims_file(&backup_ims_file);
    println!("\tImage name: {}", ims_image_name);
    println!("\t\trootfs file: {}", image_dir.unwrap().to_string() + "/" + ims_image_name.clone().as_str() + "/rootfs");
    println!("\t\tinitrd file: {}", image_dir.unwrap().to_string() + "/" + ims_image_name.clone().as_str() + "/initrd");
    println!("\t\tkernel file: {}", image_dir.unwrap().to_string() + "/" + ims_image_name.clone().as_str() + "/kernel");

    // These should come from the manifest, but let's assume these values are correct
    let vec_backup_image_files = vec![image_dir.unwrap().to_string() + "/" + ims_image_name.clone().as_str() + "/rootfs",
                                      image_dir.unwrap().to_string() + "/" + ims_image_name.clone().as_str() + "/initrd",
                                      image_dir.unwrap().to_string() + "/" +  ims_image_name.clone().as_str() + "/kernel",];

    println!();

    calculate_image_checksums(&mut ims_image_manifest, &vec_backup_image_files);

    // println!("{:?}", ims_image_manifest);

    // Do we have another image with this name?
    let ims_image_id: String  = ims_register_image(&shasta_token, &shasta_base_url, &shasta_root_cert, &ims_image_name).await;
    println!("New image record in IMS for image with name {}, new ID: {}", &ims_image_name, &ims_image_id);

    s3_upload_image_artifacts(shasta_token,
                              shasta_base_url,
                              shasta_root_cert,
                              &ims_image_id,
                              &mut ims_image_manifest,
                              &vec_backup_image_files).await;
    // println!();
    // println!("Image manifest: {:?}", ims_image_manifest);
    log::debug!("Updating image record with location of the newly generated manifest.json data");
    ims_update_image_with_manifest(shasta_token, shasta_base_url, shasta_root_cert, &ims_image_name, &ims_image_id,).await;

    println!("Creating group HSM...");
    // hsm_create_group(&shasta_token, &shasta_base_url, &shasta_root_cert, &backup_hsm_file).await;
    create_hsm_group(shasta_token, shasta_base_url, shasta_root_cert, &backup_hsm_file).await;

    // create a new CFS configuration based on the original CFS file backed up previously
    // this operation is simple as the file only has git repos and commits
    create_cfs_config(shasta_token, shasta_base_url, shasta_root_cert, &backup_cfs_file).await;

    // Create a new BOS session template based on the original BOS file backed previously
    // create_bos_sessiontemplate(shasta_token, shasta_base_url, shasta_root_cert, backup_bos_file).await;

    println!("Done, the image bundle, HSM group, CFS configuration and BOS sessiontemplate have been restored.");
    // Everything below can/should be ignored
    // ========================================================================================================



    // CFS =========================================================================================

    // load into memory
    // let cfs_data = fs::read_to_string(PathBuf::from(cfs_file.unwrap()))
    //     .expect("Unable to read HSM JSON file");
    //
    // let cfs_json: serde_json::Value = serde_json::from_str(&cfs_data)
    //     .expect("HSM JSON file does not have correct format.");
    // CFS needs to be cleaned up when loading into the system, the filed lastUpdate should not exist

    // create or update CFS config

    // BOS =========================================================================================

    // load into memory
    // let bos_data = fs::read_to_string(PathBuf::from(&bos_file.unwrap()))
    //     .expect("Unable to read HSM JSON file");
    //
    // let bos_json: serde_json::Value = serde_json::from_str(&bos_data)
    //     .expect("HSM JSON file does not have correct format.");
    //
    // log::debug!("Migrate_restore: all JSON files loaded ok");



    //
    // let bos_templates = bos::template::http_client::get(
    //     shasta_token,
    //     shasta_base_url,
    //     hsm_group_name,
    //     template_name,
    //     limit_number,
    // )
    //     .await
    //     .unwrap_or_default();

    // if bos_templates.is_empty() {
    //     println!("No BOS template found!");
    //     std::process::exit(0);
    // } else {
    //     bos::template::utils::print_table(bos_templates);
    // }
}

async fn create_cfs_config(shasta_token: &str,
                           shasta_base_url: &str,
                           shasta_root_cert: &[u8],
                           cfs_file: &String) {
    let cfs_data = fs::read_to_string(PathBuf::from(cfs_file))
        .expect("Unable to read CFS JSON file");

    let _cfs_json: serde_json::Value = serde_json::from_str(&cfs_data)
        .expect("CFS JSON file does not have correct format.");
    // CFS needs to be cleaned up when loading into the system, the filed lastUpdate should not exist
}

async fn ims_update_image_with_manifest(shasta_token: &str,
                                       shasta_base_url: &str,
                                       shasta_root_cert: &[u8],
                                       ims_image_name: &String,
                                       ims_image_id: &String) {

    match mesa::shasta::ims::image::http_client::get(&shasta_token,
                                                      &shasta_base_url,
                                                      &shasta_root_cert,
                                                      &vec![ims_image_name.clone()],
                                                      None,
                                                      None,
                                                      None).await {
        Ok(_vector) => {
            if _vector.is_empty() {
                panic!("Error: there are no images stored with id {} in IMS. Unable to update the image manifest", &ims_image_id);
            }
        },
        Err(error) => panic!("Error: Unable to determine if there are other images in IMS with the name {}. Error code: {}", &ims_image_name, &error),
    };
    let ims_record = ImsImage {
        name: ims_image_name.clone().to_string(),
        id: Some(ims_image_id.clone().to_string()),
        created: None,
        arch: None,
        link: Some(ImsLink
                   { etag: None,
                       path: format!("s3://boot-images/{}/manifest.json",&ims_image_id.to_string()),
                       r#type: Some("s3".to_string())
                   },
        )
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

    let ims_link = ImsLink { etag: None,
                       path: format!("s3://boot-images/{}/manifest.json",&ims_image_id.to_string()),
                       r#type: Some("s3".to_string())
    };
    let rec = ImsImageRecord2Update {
        link: ims_link,
        arch: None,
    };

    // println!("New IMS link {:?}", &rec);
    match update_image(shasta_token, shasta_base_url, shasta_root_cert, &ims_image_id.to_string(), &rec).await {
        Ok(_returned) => println!("Returned json: {}", _returned),
        Err(e) => panic!("Error, unable to modify the record of the image. Err msg: {}",e),
    };
}

/// Uploads to s3 under boot-images/ims_image_id all the files that
/// vec_image_files refers to. If upload successful, it modifies
/// ImageManifest to point to the right place within s3
async fn s3_upload_image_artifacts(shasta_token: &str,
                                   shasta_base_url: &str,
                                   shasta_root_cert: &[u8],
                                   ims_image_id: &String,
                                   ims_image_manifest: &mut ImageManifest,
                                   vec_image_files: &Vec<String>) {
    let bucket_name = "boot-images";
    let object_path = ims_image_id;

    // Connect and auth to S3
    let sts_value = match s3_auth(shasta_token, shasta_base_url, shasta_root_cert).await {
        Ok(sts_value) => {
            println!("Debug - STS token:\n{:#?}", sts_value);
            sts_value
        }
        Err(error) => panic!("Unable to authenticate with s3 when uploading images. Error: {}", error)
    };

    for file in vec_image_files {
        let filename = Path::new(file).file_name().clone().unwrap();
        let full_object_path = format!("{}/{}", &object_path, &filename.to_string_lossy());
        println!("Uploading file {:?} to s3://{}/{}.", &file, &bucket_name, &full_object_path);
        match s3_upload_object(&sts_value, &full_object_path, bucket_name, file).await {
            Ok(_result) => {
                println!("OK");
            },
            Err(error) => panic!("Unable to upload file to s3. Error {}", error)
        };
        // I'm pretty sure there's a better way to do this...
        if file.contains("kernel") {
            for artifact in ims_image_manifest.artifacts.iter_mut() {
                if artifact.r#type.contains("kernel") {
                    artifact.link.path = "s3://".to_string() +  bucket_name + "/" + &object_path.to_string() + "/kernel";
                    break;
                }
            }
        } else if file.contains("rootfs") {
            for artifact in ims_image_manifest.artifacts.iter_mut() {
                if artifact.r#type.contains("rootfs") {
                    artifact.link.path =  "s3://".to_string() +  bucket_name + "/" + &object_path.to_string() + "/rootfs";
                    break;
                }
            }
        } else if file.contains("initrd")  {
            for artifact in ims_image_manifest.artifacts.iter_mut() {
                if artifact.r#type.contains("initrd") {
                    artifact.link.path =  "s3://".to_string() +  bucket_name + "/" + &object_path.to_string() + "/initrd";
                    break;
                }
            }
        }
    }
    log::debug!("Writing the new manifest.json file with the correct new ID");
    let new_manifest_file_name = String::from("new-manifest.json");
    let new_manifest_file_path= Path::new(vec_image_files.first().unwrap()).parent().unwrap().join(&new_manifest_file_name);
    let new_manifest_file = File::create(&new_manifest_file_path)
        .expect("new manifest.json file could not be created.");
    serde_json::to_writer_pretty(&new_manifest_file, &ims_image_manifest).expect("Unable to write new manifest.json file");

    log::debug!("Uploading the new manifest.json file");
    let manifest_full_object_path = format!("{}/manifest.json", &object_path);
    println!("Uploading file {:?} to s3://{}/{}.", &new_manifest_file_name, &bucket_name, &manifest_full_object_path);

    match s3_upload_object(&sts_value, &manifest_full_object_path, bucket_name, &new_manifest_file_path.to_owned().to_string_lossy()).await {
        Ok(_result) => {
            println!("OK");
        },
        Err(error) => panic!("Unable to upload file to s3. Error {}", error)
    };
}

fn file_md5sum(filename: PathBuf) -> Digest {

    // let current_file_name= PathBuf::from(image_dir.unwrap()).join(file_name);
    // println!("Calculating md5sum of file {:?}...", &filename);

    // let k = Path::new(std::env::current_dir()); //(std::env::current_dir().unwrap().to_str().unwrap().to_string() + "/" + file_name;
    // println!("file: {}", k);
    let f = File::open(filename).unwrap();
    // Find the length of the file
    let len = f.metadata().unwrap().len();
    // Decide on a reasonable buffer size (300MB in this case, fastest will depend on hardware)
    let buf_len = len.min(300_000_000) as usize;
    let mut buf = BufReader::with_capacity(buf_len, f);
    let mut context = md5::Context::new();
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
    }
    let digest = context.compute();
    // println!("{:x}\t{:?}", digest, &filename);
    digest
}
/// Calculates the md5sum of all the files in the `vec_backup_image_files` vector and updates
///  the image manifest at `ims_image_manifest`
fn calculate_image_checksums(image_manifest: &mut ImageManifest, vec_backup_image_files: &Vec<String>) {

    for file in vec_backup_image_files {
        println!("Calculating md5sum of file {:?}...", file);
        let mut artifact = Artifact {
            link: Link { path: "".to_string(), r#type: "".to_string() },
            md5: "".to_string(),
            r#type: "".to_string(),
        };
        let mut fp = PathBuf::new();
        fp.push(file);
        let digest = file_md5sum(fp);
        println!("{:x}\t{:?}", digest, file);

        if file.contains("kernel") {
            artifact = Artifact {
                md5: format!("{:x}", digest),
                link: Link {
                    path: "path".to_string(),
                    r#type: "s3".to_string(),
                },
                r#type: "application/vnd.cray.image.kernel".to_string(),
            };

        }
        else if file.contains("rootfs") {
             artifact = Artifact {
                 md5: format!("{:x}", digest),
                link: Link {
                    path: "path".to_string(),
                    r#type: "s3".to_string(),
                },
                r#type: "application/vnd.cray.image.rootfs.squashfs".to_string(),
            };
        } else {
            artifact = Artifact {
                md5: format!("{:x}", digest),
                link: Link {
                    path: "path".to_string(),
                    r#type: "s3".to_string(),
                },
                r#type: "application/vnd.cray.image.initrd".to_string(),
            };
        }
        image_manifest.artifacts.push(artifact);
    }
}

/// Registers in IMS a new image and returns the new id to pass to s3
async fn ims_register_image(shasta_token: &str,
                            shasta_base_url: &str,
                            shasta_root_cert: &[u8],
                            ims_image_name: &String) -> String {
    let ims_record = ImsImage {
        name: ims_image_name.clone().to_string(),
        id: None,
        created: None,
        link: None,
        arch: None
    };
    let list_images_with_same_name = match mesa::shasta::ims::image::http_client::get(&shasta_token,
                                               &shasta_base_url,
                                               &shasta_root_cert,
                                               &vec![ims_image_name.clone()],
                                               None,
                                               None,
                                               None).await {
        Ok(vector) => vector,
        Err(error) => panic!("Error: Unable to determine if there are other images in IMS with the name {}. Error code: {}", &ims_image_name, &error),
    };

    if ! list_images_with_same_name.is_empty() {
        println!("There is already a record for image name {} in IMS do you want to create a new one (the previous one will not be deleted).", &ims_image_name);
        println!("Current IMS record(s): {:?}", &list_images_with_same_name);
        let confirmation = Confirm::new()
            .with_prompt("Do you want to create a new record?")
            .interact()
            .unwrap();

        if ! confirmation {
            println!("Looks like you do not want to continue, bailing out.");
            std::process::exit(2)
        }
    }

    let json_response = match register_new_image(&shasta_token, &shasta_base_url, &shasta_root_cert, &ims_record).await {
        Ok(json_response) => {
            json_response
        },
        Err(error) => panic!("Error: Unable to register a new image {} into IMS {}", &ims_image_name.to_string(), error.to_string())
    };
    json_response["id"].to_string().replace('"',"")
}

/// Gets the image name off an IMS yaml file
fn get_image_name_from_ims_file(ims_file: &String) -> String {
    // load into memory
    let ims_data = fs::read_to_string(PathBuf::from(&ims_file))
        .expect("Unable to read IMS file file");

    let ims_json: serde_json::Value = serde_json::from_str(&ims_data)
        .expect("HSM JSON file does not have correct format.");

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
    ims_json["name"].clone().to_string().replace('"', "")
}

// Anything in this function is critical, so the asserts will kill further processing
pub async fn create_hsm_group(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    hsm_file:  &String
) {

    // load into memory
    let hsm_data = fs::read_to_string(PathBuf::from(hsm_file))
        .expect("Unable to read HSM JSON file");

    let _hsm_json: serde_json::Value = serde_json::from_str(&hsm_data)
        .expect("HSM JSON file does not have correct format.");

    // Create new HSM group if not existing

    // Parse HSM group file
    // The file looks like this: {"gele":["x1001c7s1b1n1","x1001c7s1b0n0","x1001c7s1b1n0","x1001c7s1b0n1"]}
    let mut hsm :HsmGroup = serde_json::from_str(hsm_data.as_str()).unwrap();
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
    if hsm.exclusiveGroup.is_none() {
        hsm.exclusiveGroup = Some(false.to_string());
    }
    // This couldn't be uglier, I know
    let hsm2 :HsmGroup = hsm.clone();

    // Create the HSM group
    match create_new_hsm_group(shasta_token,
                               shasta_base_url,
                               shasta_root_cert,
                               &hsm.label,
                               &hsm.members.unwrap().ids.unwrap(),
                               &hsm.exclusiveGroup.unwrap(),
                               &hsm.description.unwrap(),
                               &hsm.tags.unwrap()).await {
        Ok(_json) => {
            println!("The HSM group {} has been created successfully.", &hsm.label);
        },
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
                    match delete_hsm_group(shasta_token,
                                           shasta_base_url,
                                           shasta_root_cert,
                                           &hsm.label).await {
                        Ok(_) => {
                            // try creating the group again
                            match create_new_hsm_group(shasta_token,
                                                       shasta_base_url,
                                                       shasta_root_cert,
                                                       &hsm2.label,
                                                       &hsm2.members.unwrap().ids.unwrap(),
                                                       &hsm2.exclusiveGroup.unwrap(),
                                                       &hsm2.description.unwrap(),
                                                       &hsm2.tags.unwrap()).await {
                                Ok(_json) => {
                                    println!("The HSM group {} has been created successfully.", &hsm2.label);
                                }
                                Err(e2) => {
                                    log::error!("Error message {}", e2);
                                    panic!("Second error creating a new HSM group. Bailing out. Error returned: '{}'", e2)
                                }
                            }
                        },
                        Err(e1) => {
                            log::error!("Error message {}", e1);
                            panic!("Error deleting the HSM group {}. Error returned: '{}'", &hsm.label, e1)
                        }
                    }
                } else {
                    println!("Not deleting the group, cannot continue the operation.");
                }
            }
        },
    };
}