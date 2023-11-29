use std::fs;
use std::path::PathBuf;
use mesa::shasta::bos;

pub async fn exec(
    shasta_token: &str,
    shasta_base_url: &str,
    shasta_root_cert: &[u8],
    bos_file:  Option<&String>,
    cfs_file:  Option<&String>,
    hsm_file:  Option<&String>
) {
    println!("Migrate_backup; BOS_file={}, CFS_file={}, HSM_file={}",bos_file.unwrap(), cfs_file.unwrap(), hsm_file.unwrap());
    // HSM -----------------------------------------------------------------------------------------
    // HSM needs to go first as CFS and BOS have references to it
    let hsm_data = fs::read_to_string(PathBuf::from(hsm_file.unwrap()))
        .expect("Unable to read HSM JSON file");

    let hsm_json: serde_json::Value = serde_json::from_str(&hsm_data)
        .expect("HSM JSON file does not have correct format.");

    // CFS -----------------------------------------------------------------------------------------
    let cfs_data = fs::read_to_string(PathBuf::from(cfs_file.unwrap()))
        .expect("Unable to read HSM JSON file");

    let cfs_json: serde_json::Value = serde_json::from_str(&cfs_data)
        .expect("HSM JSON file does not have correct format.");
    // CFS needs to be cleaned up when loading into the system, the filed lastUpdate should not exist

    // BOS -----------------------------------------------------------------------------------------
    let bos_data = fs::read_to_string(PathBuf::from(&bos_file.unwrap()))
        .expect("Unable to read HSM JSON file");

    let bos_json: serde_json::Value = serde_json::from_str(&bos_data)
        .expect("HSM JSON file does not have correct format.");

    println!("All loaded ok");
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