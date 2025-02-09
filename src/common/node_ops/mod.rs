#[cfg(test)]
pub mod tests;

use std::collections::HashMap;

use backend_dispatcher::{
    error::Error,
    interfaces::hsm::{component::ComponentTrait, group::GroupTrait},
    types::Component,
};
use comfy_table::{Cell, Table};
use hostlist_parser::parse;
use mesa::{bss::types::BootParameters, node::types::NodeDetails};
use regex::Regex;

use crate::backend_dispatcher::StaticBackendDispatcher;

// Validate and get short nid
pub fn get_short_nid(long_nid: &str) -> Result<usize, Error> {
    // Validate nid has the right length
    if long_nid.len() != 9 {
        return Err(Error::Message(format!(
            "Nid '{}' not valid, Nid does not have 9 characters",
            long_nid
        )));
    }

    long_nid.strip_prefix("nid")
        .ok_or_else(|| Error::Message(format!("Nid '{}' not valid, 'nid' prefix missing", long_nid)))
        .and_then(|nid_number| nid_number.to_string().parse::<usize>()
                            .map_err(|e| Error::Message(format!("Intermediate operation to convert Nid {} from long to short format. Reason:\n{}", nid_number, e.to_string())))
        )
}

/// Check if user input is 'nid'
pub fn is_user_input_nids(user_input: &str) -> bool {
    user_input.to_lowercase().contains("nid") // using function contains in case user input is a
                                              // regex like ˆnid
}

pub async fn get_xname_from_nid_hostlist(
    node_vec: &Vec<String>,
    node_metadata_available_vec: &Vec<Component>,
) -> Result<Vec<String>, Error> {
    // Convert long nids to short nids
    // Get xnames from short nids
    let short_nid_vec: Vec<usize> = node_vec
        .clone()
        .iter()
        .map(|nid_long| get_short_nid(nid_long))
        .collect::<Result<Vec<_>, Error>>()?;

    log::debug!("short Nid list expanded: {:?}", short_nid_vec);

    let xname_vec: Vec<String> = node_metadata_available_vec
        .into_iter()
        .filter(|node_metadata_available| {
            short_nid_vec.contains(&node_metadata_available.nid.unwrap())
        })
        .map(|node_metadata_available| node_metadata_available.id.as_ref().unwrap())
        .cloned()
        .collect();

    Ok(xname_vec)
}

pub async fn get_xname_from_xname_hostlist(
    node_vec: &Vec<String>,
    node_metadata_available_vec: &Vec<Component>,
) -> Result<Vec<String>, Error> {
    // If hostlist of XNAMEs, return hostlist expanded xnames
    // Validate XNAMEs
    log::debug!("XNAME format are valid");

    let xname_vec: Vec<String> = node_metadata_available_vec
        .into_iter()
        .filter(|node_metadata_available| {
            node_vec.contains(&node_metadata_available.id.as_ref().unwrap())
        })
        .map(|node_metadata_available| node_metadata_available.id.as_ref().unwrap())
        .cloned()
        .collect();

    Ok(xname_vec)
}

pub async fn get_xname_from_nid_regex(
    regex: &Regex,
    node_metadata_available_vec: &Vec<Component>,
) -> Result<Vec<String>, Error> {
    let xname_vec: Vec<String> = node_metadata_available_vec
        .clone()
        .into_iter()
        .filter(|node_metadata_available: &Component| {
            regex.is_match(&format!("nid{:06}", node_metadata_available.nid.unwrap()))
        })
        .map(|node_metadata_available| node_metadata_available.id.unwrap())
        .collect();

    Ok(xname_vec)
}

pub async fn get_xname_from_xname_regex(
    regex: &Regex,
    node_metadata_available_vec: &Vec<Component>,
) -> Result<Vec<String>, Error> {
    let xname_vec = node_metadata_available_vec
        .clone()
        .into_iter()
        .filter(|node_metadata_available: &Component| {
            regex.is_match(&node_metadata_available.id.as_ref().unwrap())
        })
        .map(|node_metadata_available| node_metadata_available.id.unwrap())
        .collect();

    Ok(xname_vec)
}

pub async fn resolve_node_list_user_input_to_xname_2(
    user_input: &str,
    is_include_siblings: bool,
    node_metadata_available_vec: Vec<Component>,
) -> Result<Vec<String>, Error> {
    // Check if hostlist
    // Expand user input to list of nids
    let hostlist_expanded_vec_rslt = parse(user_input).map_err(|e| Error::Message(e.to_string()));

    // Check if regex
    let regexexp_rslt = Regex::new(user_input).map_err(|e| Error::Message(e.to_string()));

    let xname_vec = if let Ok(node_vec) = hostlist_expanded_vec_rslt {
        // If hostlist, expand hostlist
        let xname_vec: Vec<String> = if mesa::node::utils::validate_nid_format_vec(node_vec.clone())
        {
            // If hostlist of NIDs, convert to xname
            // Validate NIDs
            log::debug!("NID format is valid");
            log::debug!("hostlist Nids: {}", user_input);
            log::debug!("hostlist Nids expanded: {:?}", node_vec);

            get_xname_from_nid_hostlist(&node_vec, &node_metadata_available_vec).await?
        } else if mesa::node::utils::validate_xname_format_vec(node_vec.clone()) {
            // If hostlist of XNAMEs, return hostlist expanded xnames
            // Validate XNAMEs
            log::debug!("NID format is valid");
            log::debug!("hostlist Nids: {}", user_input);
            log::debug!("hostlist Nids expanded: {:?}", node_vec);

            get_xname_from_xname_hostlist(&node_vec, &node_metadata_available_vec).await?
        } else {
            eprintln!("Node format not valid");
            std::process::exit(1);
        };

        xname_vec
    } else if let Ok(regex) = regexexp_rslt {
        log::debug!("Regex format is valid");
        // If regex, return regex
        // Filter, validate and translate list of regex nids to xnames
        let xname_vec = get_xname_from_nid_regex(&regex, &node_metadata_available_vec).await?;

        log::debug!("Regex format: {}", regex);
        log::debug!("NID list from regex: {:#?}", xname_vec);

        let xname_vec: Vec<String> = if xname_vec.is_empty() {
            log::debug!("No NIDs found from regex");
            // Filter, validate and translate list of regex xnames to xnames
            get_xname_from_xname_regex(&regex, &node_metadata_available_vec).await?
        } else {
            xname_vec
        };

        xname_vec
    } else {
        eprintln!(
            "Could not parse list of nodes as a hostlist or regex. Reason:\n{}",
            user_input
        );
        std::process::exit(1);
    };

    // Include siblings if requested
    let xname_vec: Vec<String> = if is_include_siblings {
        log::debug!("Include siblings");
        let xname_blade_vec: Vec<String> = xname_vec
            .iter()
            .map(|xname| xname[0..10].to_string())
            .collect();

        log::debug!("XNAME blades:\n{:?}", xname_blade_vec);

        // Filter xnames to the ones the user has access to
        let xname_vec = node_metadata_available_vec
            .into_iter()
            .filter(|node_metadata_available| {
                xname_blade_vec.iter().any(|xname_blade| {
                    node_metadata_available
                        .id
                        .as_ref()
                        .unwrap()
                        .starts_with(xname_blade)
                })
            })
            .map(|node_metadata_available| node_metadata_available.id.unwrap())
            .collect();

        xname_vec
    } else {
        xname_vec
    };

    Ok(xname_vec)
}

pub async fn resolve_node_list_user_input_to_xname(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    user_input: &str,
    is_include_siblings: bool,
    is_regex: bool,
) -> Result<Vec<String>, Error> {
    // Get list of xnames available to the user
    let xname_available_vec: Vec<String> = backend
        .get_group_available(shasta_token)
        .await?
        .iter()
        .flat_map(|group| group.get_members())
        .collect();

    let node_metadata = backend
        .get_all_nodes(shasta_token, Some("true"))
        .await?
        .components
        .unwrap_or_default();

    // Expand user input to list of xnames
    let xname_requested_by_user_vec = if is_user_input_nids(user_input) {
        log::debug!("User input seems to be NID");
        let all_short_nid_vec: Vec<usize> = node_metadata
            .iter()
            .map(|node| node.nid)
            .collect::<Option<Vec<usize>>>()
            .unwrap_or_default();

        let requested_short_nid_vec =
            get_xname_from_user_nid_expression(user_input, is_regex, &all_short_nid_vec)?;

        // Get list of xnames from short nids requested by user
        let requested_xname_vec: Vec<String> = node_metadata
            .iter()
            .filter(|component| requested_short_nid_vec.contains(&component.nid.clone().unwrap()))
            .map(|component| component.id.clone().unwrap())
            .collect();

        log::debug!("XNAME list:\n{:#?}", requested_xname_vec);

        // let xname_vec_rslt = nid_to_xname(backend, shasta_token, user_input, is_regex).await;

        requested_xname_vec
    } else {
        log::debug!("User input seems to be XNAME");
        let all_xname_vec: Vec<String> = node_metadata
            .iter()
            .map(|node| node.id.clone())
            .collect::<Option<Vec<String>>>()
            .unwrap_or_default();

        let xname_vec: Vec<String> =
            get_xname_list_from_xname_expression(user_input, &all_xname_vec)?;
        // get_curated_hsm_group_from_xname_regex(backend, shasta_token, user_input).await

        xname_vec
    };

    let xname_vec: Vec<String> = if is_include_siblings {
        let xname_blade_requested_by_user_vec: Vec<String> = xname_requested_by_user_vec
            .iter()
            .map(|xname| xname[0..10].to_string())
            .collect();

        log::debug!("XNAME blades:\n{:?}", xname_blade_requested_by_user_vec);

        // Filter xnames to the ones the user has access to
        xname_available_vec
            .into_iter()
            .filter(|xname| {
                xname_blade_requested_by_user_vec
                    .iter()
                    .any(|xname_blade| xname.starts_with(xname_blade))
            })
            .collect()
    } else {
        // Filter xnames to the ones the user has access to
        xname_requested_by_user_vec
            .into_iter()
            .filter(|xname| xname_available_vec.contains(&xname))
            .collect()
    };

    Ok(xname_vec)
}

/// Get list of nids from a list of user expressions related to NIDs
/// A user expressions related to NID can be:
///     - comma separated list of NIDs (eg: nid000001,nid000002,nid000003)
///     - regex (eg: nid00000.*)
///     - hostlist (eg: nid0000[01-15])
pub fn get_xname_from_user_nid_expression(
    user_input_nid: &str,
    is_regex: bool,
    short_nid_available_vec: &Vec<usize>,
) -> Result<Vec<usize>, Error> {
    let short_nid_vec = if is_regex {
        log::debug!("Regex found, getting xnames from NIDs");
        // Get list of regex
        let regex_vec: Vec<Regex> = user_input_nid
            .split(",")
            .map(|regex_str| {
                Regex::new(regex_str.trim()).map_err(|e| Error::Message(e.to_string()))
            })
            .collect::<Result<Vec<_>, Error>>()?;

        // Filter nids available to the ones that match the regex
        short_nid_available_vec
            .clone()
            .into_iter()
            .filter(|nid_short| {
                let nid_long = format!("nid{:06}", nid_short);
                regex_vec.iter().any(|regex| regex.is_match(&nid_long))
            })
            .collect()
    } else {
        log::debug!("No regex found, getting xnames from list of NIDs or NIDs hostlist");
        // Expand user input to list of nids
        let long_nid_hostlist_expanded_vec =
            parse(user_input_nid).map_err(|e| Error::Message(e.to_string()))?;

        log::debug!("hostlist Nids: {}", user_input_nid);
        log::debug!(
            "hostlist Nids expanded: {:?}",
            long_nid_hostlist_expanded_vec
        );

        // Validate and convert long nids to short nids
        let mut short_nid_vec: Vec<usize> = long_nid_hostlist_expanded_vec
            .iter()
            .map(|nid_long| get_short_nid(nid_long))
            .collect::<Result<Vec<_>, Error>>()?;

        log::debug!("short Nid list expanded: {:?}", short_nid_vec);

        // Filter nids available to the ones that match the hostlist
        short_nid_vec.retain(|nid| short_nid_available_vec.contains(&nid));

        short_nid_vec
    };

    log::debug!("short Nid list requested by the user: {:?}", short_nid_vec);

    return Ok(short_nid_vec);
}

/* /// Get list of xnames from NIDs
/// The list of NIDs can be:
///     - comma separated list of NIDs (eg: nid000001,nid000002,nid000003)
///     - regex (eg: nid00000.*)
///     - hostlist (eg: nid0000[01-15])
pub async fn nid_to_xname(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    user_input_nid: &str,
    is_regex: bool,
) -> Result<Vec<String>, Error> {
    if is_regex {
        log::debug!("Regex found, getting xnames from NIDs");
        // Get list of regex
        let regex_vec: Vec<Regex> = user_input_nid
            .split(",")
            .map(|regex_str| {
                Regex::new(regex_str.trim())
                    .expect(format!("Regex '{}' not valid", regex_str).as_str())
            })
            .collect();

        // Get all HSM components (list of xnames + nids)
        let hsm_component_vec = backend
            .get_all_nodes(shasta_token, Some("true"))
            .await?
            .components
            .unwrap_or_default();

        let mut xname_vec: Vec<String> = vec![];

        // Get list of xnames the user is asking for
        for hsm_component in hsm_component_vec {
            let nid_long = format!("nid{:06}", &hsm_component.nid.expect("No NID found"));
            for regex in &regex_vec {
                if regex.is_match(&nid_long) {
                    log::debug!(
                        "Nid '{}' IS included in regex '{}'",
                        nid_long,
                        regex.as_str()
                    );
                    xname_vec.push(hsm_component.id.clone().expect("No XName found"));
                }
            }
        }

        return Ok(xname_vec);
    } else {
        log::debug!("No regex found, getting xnames from list of NIDs or NIDs hostlist");
        let nid_hostlist_expanded_vec_rslt = parse(user_input_nid);

        let nid_hostlist_expanded_vec = match nid_hostlist_expanded_vec_rslt {
            Ok(xname_requested_vec) => xname_requested_vec,
            Err(e) => {
                println!(
                    "Could not parse list of nodes as a hostlist. Reason:\n{}Exit",
                    e
                );
                std::process::exit(1);
            }
        };

        log::debug!("hostlist: {}", user_input_nid);
        log::debug!("hostlist expanded: {:?}", nid_hostlist_expanded_vec);

        let nid_short = nid_hostlist_expanded_vec
            .iter()
            .map(|nid_long| {
                nid_long
                    .strip_prefix("nid")
                    .expect(format!("Nid '{}' not valid, 'nid' prefix missing", nid_long).as_str())
                    .trim_start_matches("0")
            })
            .collect::<Vec<&str>>()
            .join(",");

        log::debug!("short NID list: {}", nid_short);

        // Get all HSM components (list of xnames + nids)
        let hsm_component_vec = backend
            .get_all_nodes(shasta_token, Some("true"))
            .await?
            .components
            .unwrap_or_default();

        // Get list of xnames from HSM components
        let xname_vec: Vec<String> = hsm_component_vec
            .iter()
            .map(|component| component.id.clone().unwrap())
            .collect();

        log::debug!("xname list:\n{:#?}", xname_vec);

        return Ok(xname_vec);
    };
} */

/// Get list of xnames user has access to based on input regex.
/// This function expands and filters the list of xnames available to the user based on the regex
/// provided
pub fn get_xname_list_from_xname_expression(
    regex_exp: &str,
    xname_available_vec: &[String],
) -> Result<Vec<String>, Error> {
    // Get list of regex
    let regex_vec_rslt: Result<Vec<Regex>, Error> = regex_exp
        .split(",")
        .map(|regex_str| Regex::new(regex_str.trim()).map_err(|e| Error::Message(e.to_string())))
        .collect();

    // Filter xnames available to the ones that match the regex
    regex_vec_rslt.map(|regex_vec| {
        xname_available_vec
            .iter()
            .map(|xname| xname.to_string())
            .filter(|xname| regex_vec.iter().any(|regex| regex.is_match(xname)))
            .collect()
    })
}

/* /// Get list of xnames user has access to based on input regex.
/// This method will:
/// 1) Break down all regex in user input
/// 2) Fetch all HSM groups user has access to
/// 3) For each HSM group, get the list of xnames and filter the ones that matches the regex
pub async fn get_curated_hsm_group_from_xname_regex(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    /* shasta_base_url: &str,
    shasta_root_cert: &[u8], */
    xname_requested_regex: &str,
) -> HashMap<String, Vec<String>> {
    let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();

    // Get list of regex
    let regex_vec: Vec<Regex> = xname_requested_regex
        .split(",")
        .map(|regex_str| {
            Regex::new(regex_str.trim()).expect(format!("Regex '{}' not valid", regex_str).as_str())
        })
        .collect();

    let hsm_name_available_vec = backend
        .get_group_name_available(shasta_token)
        .await
        .unwrap();

    // Get HSM group user has access to
    let hsm_group_available_map = backend
        .get_hsm_map_and_filter_by_hsm_name_vec(
            shasta_token,
            hsm_name_available_vec
                .iter()
                .map(|hsm_name| hsm_name.as_str())
                .collect(),
        )
        .await
        .expect("ERROR - could not get HSM group summary");

    // Filter hsm group members
    for (hsm_name, xnames) in hsm_group_available_map {
        for xname in xnames {
            for regex in &regex_vec {
                if regex.is_match(&xname) {
                    hsm_group_summary
                        .entry(hsm_name.clone())
                        .and_modify(|member_vec| member_vec.push(xname.clone()))
                        .or_insert(vec![xname.clone()]);
                }
            }
        }
    }

    hsm_group_summary
} */

/* /// Get list of xnames user has access to based on input regex.
/// This function expands and filters the list of xnames available to the user based on the regex
/// provided
pub fn get_xname_list_from_hostlist_expression(
    hostlist_exp: &str,
    xname_vec: &[String],
) -> Result<Vec<String>, Error> {
    // Get list of xnames
    let mut xname_requested_vec = parse(hostlist_exp).map_err(|e| Error::Message(e.to_string()))?;

    log::info!("hostlist: {}", hostlist_exp);
    log::info!("hostlist expanded: {:?}", xname_requested_vec);

    // Filter xnames to the ones members to groups the user has access to
    xname_requested_vec.retain(|xname| xname_vec.contains(xname));

    Ok(xname_requested_vec)
} */

/// Returns a HashMap with keys HSM group names the user has access to and values a curated list of memembers that matches
/// hostlist
pub async fn get_curated_hsm_group_from_xname_hostlist(
    backend: &StaticBackendDispatcher,
    auth_token: &str,
    xname_requested_hostlist: &str,
) -> HashMap<String, Vec<String>> {
    // Create a summary of HSM groups and the list of members filtered by the list of nodes the
    // user is targeting
    let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();

    let xname_requested_vec_rslt = parse(xname_requested_hostlist);

    let xname_requested_vec = match xname_requested_vec_rslt {
        Ok(xname_requested_vec) => xname_requested_vec,
        Err(e) => {
            println!(
                "Could not parse list of nodes as a hostlist. Reason:\n{}Exit",
                e
            );
            std::process::exit(1);
        }
    };

    log::info!("hostlist: {}", xname_requested_hostlist);
    log::info!("hostlist expanded: {:?}", xname_requested_vec);

    /* // Get final list of xnames to operate on
    // Get list of HSM groups available
    // NOTE: HSM available are the ones the user has access to
    // let hsm_group_name_available: Vec<String> = get_hsm_name_available_from_jwt(shasta_token).await;

    // Get all HSM groups in the system
    // FIXME: client should not fetch all info in backend. Create a method in backend to do provide
    // information already filtered to the client:
    // hsm::groups::utils::get_hsm_group_available_vec(shasta_token, shasta_base_url,
    // shasta_root_cert) -> Vec<HsmGroup> to get the list of HSM available to the user and return
    // a Vec of HsmGroups the user has access to
    let hsm_group_vec_all =
        hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await
            .expect("Error - fetching HSM groups"); */

    let hsm_name_available_vec = backend.get_group_name_available(auth_token).await.unwrap();

    // Get HSM group user has access to
    let hsm_group_available_map = backend
        .get_group_map_and_filter_by_group_vec(
            auth_token,
            hsm_name_available_vec
                .iter()
                .map(|hsm_name| hsm_name.as_str())
                .collect(),
        )
        .await
        .expect("ERROR - could not get HSM group summary");

    // Filter hsm group members
    for (hsm_name, hsm_members) in hsm_group_available_map {
        let xname_filtered: Vec<String> = hsm_members
            .iter()
            .filter(|&xname| xname_requested_vec.contains(&xname))
            .cloned()
            .collect();
        if !xname_filtered.is_empty() {
            hsm_group_summary.insert(hsm_name, xname_filtered);
        }
    }

    hsm_group_summary
}

/* /// Returns a HashMap with keys HSM group names the user has access to and values a curated list of memembers that matches
/// hostlist
pub async fn get_curated_hsm_group_from_hostlist(
    backend: &StaticBackendDispatcher,
    shasta_token: &str,
    xname_requested_hostlist: &str,
) -> HashMap<String, Vec<String>> {
    // Create a summary of HSM groups and the list of members filtered by the list of nodes the
    // user is targeting
    let mut hsm_group_summary: HashMap<String, Vec<String>> = HashMap::new();

    let xname_requested_vec_rslt = parse(xname_requested_hostlist);

    let xname_requested_vec = match xname_requested_vec_rslt {
        Ok(xname_requested_vec) => xname_requested_vec,
        Err(e) => {
            println!(
                "Could not parse list of nodes as a hostlist. Reason:\n{}Exit",
                e
            );
            std::process::exit(1);
        }
    };

    log::info!("hostlist: {}", xname_requested_hostlist);
    log::info!("hostlist expanded: {:?}", xname_requested_vec);

    /* // Get final list of xnames to operate on
    // Get list of HSM groups available
    // NOTE: HSM available are the ones the user has access to
    // let hsm_group_name_available: Vec<String> = get_hsm_name_available_from_jwt(shasta_token).await;

    // Get all HSM groups in the system
    // FIXME: client should not fetch all info in backend. Create a method in backend to do provide
    // information already filtered to the client:
    // hsm::groups::utils::get_hsm_group_available_vec(shasta_token, shasta_base_url,
    // shasta_root_cert) -> Vec<HsmGroup> to get the list of HSM available to the user and return
    // a Vec of HsmGroups the user has access to
    let hsm_group_vec_all =
        hsm::group::http_client::get_all(shasta_token, shasta_base_url, shasta_root_cert)
            .await
            .expect("Error - fetching HSM groups"); */

    let hsm_name_available_vec = backend
        .get_group_name_available(shasta_token)
        .await
        .unwrap();
    /* let hsm_name_available_vec =
    get_hsm_name_available_from_jwt_or_all(shasta_token, shasta_base_url, shasta_root_cert)
        .await; */

    // Get HSM group user has access to
    let hsm_group_available_map = backend
        .get_group_map_and_filter_by_group_vec(
            shasta_token,
            hsm_name_available_vec
                .iter()
                .map(|hsm_name| hsm_name.as_str())
                .collect(),
        )
        .await
        .expect("ERROR - could not get HSM group summary");

    // Filter hsm group members
    for (hsm_name, hsm_members) in hsm_group_available_map {
        let xname_filtered: Vec<String> = hsm_members
            .iter()
            .filter(|&xname| xname_requested_vec.contains(&xname))
            .cloned()
            .collect();
        if !xname_filtered.is_empty() {
            hsm_group_summary.insert(hsm_name, xname_filtered);
        }
    }

    hsm_group_summary
} */

pub fn print_table(nodes_status: Vec<NodeDetails>) {
    let mut table = Table::new();

    table.set_header(vec![
        "XNAME",
        "NID",
        "HSM",
        "Power Status",
        "Runtime Configuration",
        "Configuration Status",
        "Enabled",
        "Error Count",
        "Image Configuration",
        "Image ID",
    ]);

    for node_status in nodes_status {
        let mut node_vec: Vec<String> = node_status
            .hsm
            .split(",")
            .map(|xname_str| xname_str.trim().to_string())
            .collect();
        node_vec.sort();

        table.add_row(vec![
            Cell::new(node_status.xname),
            Cell::new(node_status.nid),
            Cell::new(nodes_to_string_format_discrete_columns(Some(&node_vec), 1)),
            Cell::new(node_status.power_status),
            Cell::new(node_status.desired_configuration),
            Cell::new(node_status.configuration_status),
            Cell::new(node_status.enabled),
            Cell::new(node_status.error_count),
            Cell::new(node_status.boot_configuration),
            Cell::new(node_status.boot_image_id),
        ]);
    }

    println!("{table}");
}

pub fn print_table_wide(nodes_status: Vec<NodeDetails>) {
    let mut table = Table::new();

    table.set_header(vec![
        "XNAME",
        "NID",
        "HSM",
        "Power Status",
        "Runtime Configuration",
        "Configuration Status",
        "Enabled",
        "Error Count",
        "Image Configuration",
        "Image ID",
        "Kernel Params",
    ]);

    for node_status in nodes_status {
        let kernel_params_vec: Vec<&str> = node_status.kernel_params.split_whitespace().collect();
        let cell_max_width = kernel_params_vec
            .iter()
            .map(|value| value.len())
            .max()
            .unwrap_or(0);

        let mut kernel_params_string: String = kernel_params_vec[0].to_string();
        let mut cell_width = kernel_params_string.len();

        for kernel_param in kernel_params_vec.iter().skip(1) {
            cell_width += kernel_param.len();

            if cell_width + kernel_param.len() >= cell_max_width {
                kernel_params_string.push_str("\n");
                cell_width = 0;
            } else {
                kernel_params_string.push_str(" ");
            }

            kernel_params_string.push_str(kernel_param);
        }

        let mut node_vec: Vec<String> = node_status
            .hsm
            .split(",")
            .map(|xname_str| xname_str.trim().to_string())
            .collect();
        node_vec.sort();

        table.add_row(vec![
            Cell::new(node_status.xname),
            Cell::new(node_status.nid),
            Cell::new(nodes_to_string_format_discrete_columns(Some(&node_vec), 1)),
            Cell::new(node_status.power_status),
            Cell::new(node_status.desired_configuration),
            Cell::new(node_status.configuration_status),
            Cell::new(node_status.enabled),
            Cell::new(node_status.error_count),
            Cell::new(node_status.boot_configuration),
            Cell::new(node_status.boot_image_id),
            Cell::new(kernel_params_string),
        ]);
    }

    println!("{table}");
}

pub fn print_summary(node_details_list: Vec<NodeDetails>) {
    let mut power_status_counters: HashMap<String, usize> = HashMap::new();
    let mut boot_configuration_counters: HashMap<String, usize> = HashMap::new();
    let mut runtime_configuration_counters: HashMap<String, usize> = HashMap::new();
    let mut boot_image_counters: HashMap<String, usize> = HashMap::new();

    for node in node_details_list {
        power_status_counters
            .entry(node.power_status)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);

        boot_configuration_counters
            .entry(node.boot_configuration)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);

        runtime_configuration_counters
            .entry(node.desired_configuration)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);

        boot_image_counters
            .entry(node.boot_image_id)
            .and_modify(|power_status_counter| *power_status_counter += 1)
            .or_insert(1);
    }

    let mut table = Table::new();

    table.set_header(vec!["Power status", "Num nodes"]);

    for power_status in ["FAILED", "ON", "OFF", "READY", "STANDBY", "UNCONFIGURED"] {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(power_status),
                Cell::new(power_status_counters.get(power_status).unwrap_or(&0))
                    .set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");

    let mut table = Table::new();

    table.set_header(vec!["Boot configuration name", "Num nodes"]);

    for (config_name, counter) in boot_configuration_counters {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(config_name),
                Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");

    let mut table = Table::new();

    table.set_header(vec!["Boot image id", "Num nodes"]);

    for (image_id, counter) in boot_image_counters {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(image_id),
                Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");

    let mut table = Table::new();

    table.set_header(vec!["Runtime configuration name", "Num nodes"]);

    for (config_name, counter) in runtime_configuration_counters {
        table
            .load_preset(comfy_table::presets::ASCII_FULL_CONDENSED)
            .add_row(vec![
                Cell::new(config_name),
                Cell::new(counter).set_alignment(comfy_table::CellAlignment::Center),
            ]);
    }

    println!("{table}");
}

pub fn nodes_to_string_format_one_line(nodes: Option<&Vec<String>>) -> String {
    if let Some(nodes_content) = nodes {
        nodes_to_string_format_discrete_columns(nodes, nodes_content.len() + 1)
    } else {
        "".to_string()
    }
}

pub fn nodes_to_string_format_discrete_columns(
    nodes: Option<&Vec<String>>,
    num_columns: usize,
) -> String {
    let mut members: String;

    match nodes {
        Some(nodes) if !nodes.is_empty() => {
            members = nodes[0].clone(); // take first element

            for (i, _) in nodes.iter().enumerate().skip(1) {
                // iterate for the rest of the list
                if i % num_columns == 0 {
                    // breaking the cell content into multiple lines (only 2 xnames per line)

                    members.push_str(",\n");
                } else {
                    members.push(',');
                }

                members.push_str(&nodes[i]);
            }
        }
        _ => members = "".to_string(),
    }

    members
}

/// Given a list of boot params, this function returns the list of hosts booting an image_id
pub fn get_node_vec_booting_image(
    image_id: &str,
    boot_param_vec: &[BootParameters],
) -> Vec<String> {
    let mut node_booting_image_vec = boot_param_vec
        .iter()
        .cloned()
        .filter(|boot_param| boot_param.get_boot_image().eq(image_id))
        .flat_map(|boot_param| boot_param.hosts)
        .collect::<Vec<_>>();

    node_booting_image_vec.sort();

    node_booting_image_vec
}
