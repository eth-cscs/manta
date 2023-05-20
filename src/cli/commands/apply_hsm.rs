/// Manipulates HSM groups `apply hsm-group <hsm group name>:<prop1>:<prop2>:...:<num nodes> [<hsm group name>:<prop1>:<prop2>:...:<num nodes>] ...`
/// each propX is a propery from https://api.cmn.alps.cscs.ch/apis/smd/hsm/v2/Inventory/Hardware/Query/{xname} 
/// for each <hsm group name>:<prop1>:<prop2>:...:<num nodes>: 
///   - Calculate `delta number`: number of nodes affected. Eg zinal hsm group has 4 nodes and
///   apply hsm-group has zinal:6, this means delta number would be +2 meaning 2 nodes needs to be
///   removed from the free pool and added to the zinal hsm group. Eg 2 zinal hsm group has 4 nodes and apply
///   hsm-group has zinal:2, this means delta number is -2 meaning 2 nodes from zinal hsm group
///   needs to be moved to the free pool hsm group
///   - Calculate `delta operation`: 
///      - `remove`: if delta number is negative, then nodes are movec form <hsm> group to `free pool` hsm group
///      - `add`: if delta number if positive, then nodes are moved from the `free pool` hsm group to the <hsm> group
///   - Fetch nodes props from https://api.cmn.alps.cscs.ch/apis/smd/hsm/v2/Inventory/Hardware/Query/{xname} from all nodes in HSM groups affected
///   - Create a plan:
///      1) adds all nodes with negative delta numbers to the free pool
///      2) calculates there are enough resources to add all nodes to all hsm groups
///   - Checks operations for all hsm groups can be fullfilled
///      NOTE: apply hsm-group is a unique transaction, if any hsm group update can't be done, then
///      the command will fail.
///   - Operates the deltas for each group
