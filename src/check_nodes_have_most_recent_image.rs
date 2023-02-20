use std::collections::HashMap;

pub async fn get_node_details(
    shasta_token: &String,
    shasta_base_url: &String,
    image_id: Option<String>,
    xnames: &Vec<String>,
) -> Vec<Vec<String>> {
    let mut nodes_status: Vec<Vec<String>> = Vec::new();

    // Get power node status from capmc
    let nodes_power_status_resp = crate::shasta::capmc::http_client::node_power_status::post(
        &shasta_token,
        &shasta_base_url,
        xnames,
    )
    .await
    .unwrap();

    // Get nodes boot params
    let nodes_boot_params =
        crate::shasta::bss::http_client::get_boot_params(shasta_token, shasta_base_url, xnames)
            .await
            .unwrap();

    // println!("nodes_boot_params: {:#?}", nodes_boot_params);

    let mut nodes_images: HashMap<String, String> = HashMap::new();

    // Create dictionary of xname and image_id
    for node_boot_params in nodes_boot_params {
        let nodes: Vec<String> = node_boot_params["hosts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|node| node.as_str().unwrap().to_string())
            .collect();

        let image_id = node_boot_params["kernel"].as_str().unwrap().to_string().trim_start_matches("s3://boot-images/").trim_end_matches("/kernel").to_string();

        for node in nodes {
            nodes_images.insert(node, image_id.clone());
        }
    }

//    println!("node_images dictionary: {:#?}", nodes_images);

    // Group nodes by power status
    let nodes_on: Vec<String> = nodes_power_status_resp["on"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|xname| xname.as_str().unwrap().to_string())
        .collect();
    let nodes_off: Vec<String> = nodes_power_status_resp["off"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|xname| xname.as_str().unwrap().to_string())
        .collect();
    let nodes_disabled: Vec<String> = nodes_power_status_resp["disabled"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|xname| xname.as_str().unwrap().to_string())
        .collect();
    let nodes_ready: Vec<String> = nodes_power_status_resp["ready"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|xname| xname.as_str().unwrap().to_string())
        .collect();
    let nodes_standby: Vec<String> = nodes_power_status_resp["standby"]
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|xname| xname.as_str().unwrap().to_string())
        .collect();

//    println!(
//        "List nodes power status:\nON: {:?}\nOFF: {:?}\nDISABLED: {:?}\nREADY: {:?}\nSTANDBY: {:?}",
//        nodes_on, nodes_off, nodes_disabled, nodes_ready, nodes_standby
//    );

    // Merge nodes power status with boot params
    for xname in xnames {
        let mut node_status: Vec<String> = vec![xname.to_string()];

        // Get node power status
        let node_power_status = if nodes_on.contains(xname) {
            "ON".to_string()
        } else if nodes_off.contains(xname) {
            "OFF".to_string()
        } else if nodes_disabled.contains(xname) {
            "DISABLED".to_string()
        } else if nodes_ready.contains(xname) {
            "READY".to_string()
        } else if nodes_standby.contains(xname) {
            "STANDBY".to_string()
        } else {
            "N/A".to_string()
        };

        node_status.push(node_power_status);

        // Get node boot param image
        let node_image_id = nodes_images.get(&xname.to_string()).unwrap().to_string();

        node_status.push(node_image_id.clone());

        // Has node latest image?
        if image_id.is_some() {
            node_status.push(node_image_id.eq(&image_id.clone().unwrap()).to_string());
        } else {
            node_status.push("N/A".to_string());
        }

        nodes_status.push(node_status);
    }

    // println!("nodes_status: {:#?}", nodes_status);

    nodes_status
}

