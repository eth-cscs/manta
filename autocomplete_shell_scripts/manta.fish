# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_manta_global_optspecs
	string join \n h/help V/version
end

function __fish_manta_needs_command
	# Figure out if the current invocation already has a command.
	set -l cmd (commandline -opc)
	set -e cmd[1]
	argparse -s (__fish_manta_global_optspecs) -- $cmd 2>/dev/null
	or return
	if set -q argv[1]
		# Also print the command, so this can be used to figure out what it is.
		echo $argv[1]
		return 1
	end
	return 0
end

function __fish_manta_using_subcommand
	set -l cmd (__fish_manta_needs_command)
	test -z "$cmd"
	and return 1
	contains -- $cmd[1] $argv
end

complete -c manta -n "__fish_manta_needs_command" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_needs_command" -s V -l version -d 'Print version'
complete -c manta -n "__fish_manta_needs_command" -f -a "config" -d 'Manta\'s configuration'
complete -c manta -n "__fish_manta_needs_command" -f -a "get" -d 'Get information from CSM system'
complete -c manta -n "__fish_manta_needs_command" -f -a "add" -d 'Add/Create new elements to system. Nodes will be added to the user\'s \'parent\' group'
complete -c manta -n "__fish_manta_needs_command" -f -a "apply" -d 'Make changes to Shasta system'
complete -c manta -n "__fish_manta_needs_command" -f -a "delete" -d 'Deletes data'
complete -c manta -n "__fish_manta_needs_command" -f -a "migrate"
complete -c manta -n "__fish_manta_needs_command" -f -a "power" -d 'Command to submit commands related to cluster/node power management'
complete -c manta -n "__fish_manta_needs_command" -f -a "log" -d 'get cfs session logs'
complete -c manta -n "__fish_manta_needs_command" -f -a "console" -d 'Opens an interective session to a node or CFS session ansible target container'
complete -c manta -n "__fish_manta_needs_command" -f -a "validate-local-repo" -d 'Check all tags and HEAD information related to a local repo exists in Gitea'
complete -c manta -n "__fish_manta_needs_command" -f -a "add-nodes-to-groups" -d 'Add nodes to a list of groups'
complete -c manta -n "__fish_manta_needs_command" -f -a "remove-nodes-from-groups" -d 'Remove nodes from groups'
complete -c manta -n "__fish_manta_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset gen-autocomplete help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset gen-autocomplete help" -f -a "show" -d 'Show config values'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset gen-autocomplete help" -f -a "set" -d 'Change config values'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset gen-autocomplete help" -f -a "unset" -d 'Reset config values'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset gen-autocomplete help" -f -a "gen-autocomplete" -d 'Generate shell auto completion script'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset gen-autocomplete help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -f -a "hsm" -d 'Set target HSM group'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -f -a "parent-hsm" -d 'Set parent HSM group'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -f -a "site" -d 'Set site to work on'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -f -a "log" -d 'Set site to work on'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -f -a "hsm" -d 'Unset target HSM group'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -f -a "parent-hsm" -d 'Unset parent HSM group'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -f -a "auth" -d 'Unset authentication token'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from gen-autocomplete" -s s -l shell -d 'Shell type. Will try to guess from $SHELL if missing' -r -f -a "bash\t''
zsh\t''
fish\t''"
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from gen-autocomplete" -s p -l path -d 'Path to put the autocomplete script or prints to stdout if missing. NOTE: Do not specify filename, only path to directory' -r -f -a "(__fish_complete_directories)"
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from gen-autocomplete" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show config values'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "set" -d 'Change config values'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "unset" -d 'Reset config values'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "gen-autocomplete" -d 'Generate shell auto completion script'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "group" -d 'Get group details'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "hw-component" -d 'Get hardware components1 for a cluster or a node'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "sessions" -d 'Get information from Shasta CFS session'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "configurations" -d 'Get information from Shasta CFS configuration'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "templates" -d 'Get information from Shasta BOS template'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "cluster" -d 'Get cluster details'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "nodes" -d 'Get node details'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "hsm-groups" -d 'DEPRECATED - Please do not use this command. Get HSM groups details'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "images" -d 'Get image information'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "kernel-parameters" -d 'Get kernel-parameters information'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from group hw-component sessions configurations templates cluster nodes hsm-groups images kernel-parameters help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from group" -s o -l output -d 'Output format' -r -f -a "json\t''
table\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from group" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from hw-component" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from hw-component" -f -a "cluster" -d 'Get hw components for a cluster'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from hw-component" -f -a "node" -d 'Get hw components for some nodes'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from hw-component" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s n -l name -d 'Return only sessions with the given session name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s a -l min-age -d 'Return only sessions older than the given age. Age is given in the format \'1d\' or \'6h\'' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s A -l max-age -d 'Return only sessions younger than the given age. Age is given in the format \'1d\' or \'6h\'' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s s -l status -d 'Return only sessions with the given status' -r -f -a "pending\t''
running\t''
complete\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s l -l limit -d 'Return only last <VALUE> sessions created' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s o -l output -d 'Output format. If missing, it will print output data in human redeable (table) format' -r -f -a "json\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s x -l xnames -d 'Comma separated list of xnames. eg \'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0\'' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s H -l hsm-group -d 'hsm group name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s m -l most-recent -d 'Return only the most recent session created (equivalent to --limit 1)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s n -l name -d 'configuration name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s p -l pattern -d 'Glob pattern for configuration name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s l -l limit -d 'Filter records to the <VALUE> most common number of CFS configurations created' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s o -l output -d 'Output format. If missing, it will print output data in human redeable (table) format' -r -f -a "json\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s H -l hsm-group -d 'hsm group name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s m -l most-recent -d 'Only shows the most recent (equivalent to --limit 1)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s n -l name -d 'template name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s l -l limit -d 'Filter records to the <VALUE> most common number of BOS templates created' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s H -l hsm-group -d 'hsm group name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s o -l output -d 'Output format. If missing it will print output data in human redeable (table) format' -r -f -a "json\t''
table\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s m -l most-recent -d 'Only shows the most recent (equivalent to --limit 1)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from cluster" -s o -l output -d 'Output format. If missing it will print output data in human readable (table) format' -r -f -a "table\t''
table-wide\t''
json\t''
summary\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from cluster" -s n -l nids-only-one-line -d 'Prints nids in one line eg nidxxxxxx,nidyyyyyy,nidzzzzzz,...'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from cluster" -s x -l xnames-only-one-line -d 'Prints xnames in one line eg x1001c1s5b0n0,x1001c1s5b0n1,...'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from cluster" -s s -l status -d 'Get cluster status:  - OK: All nodes are operational (booted and configured)  - OFF: At least one node is OFF  - ON: No nodes OFF and at least one is ON  - STANDBY: At least one node\'s heartbeat is lost  - UNCONFIGURED: All nodes are READY but at least one of them is being configured  - FAILED: At least one node configuration failed'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from cluster" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s o -l output -d 'Output format. If missing it will print output data in human readable (table) format' -r -f -a "table\t''
table-wide\t''
json\t''
summary\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s n -l nids-only-one-line -d 'Prints nids in one line eg nidxxxxxx,nidyyyyyy,nidzzzzzz,...'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s s -l status -d 'Get cluster status:  - OK: All nodes are operational (booted and configured)  - OFF: At least one node is OFF  - ON: No nodes OFF and at least one is ON  - STANDBY: At least one node\'s heartbeat is lost  - UNCONFIGURED: All nodes are READY but at least one of them is being configured  - FAILED: At least one node configuration failed'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s S -l include-siblings -d 'Output includes extra nodes related to the ones requested by used. 2 nodes are siblings if they share the same power supply.'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s r -l regex -d 'Input nodes in regex format.'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from hsm-groups" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -s i -l id -d 'Image ID' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -s l -l limit -d 'Filter records to the <VALUE> most common number of images created' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -s H -l hsm-group -d 'hsm group name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -s x -l xnames -d 'Comma separated list of xnames to retrieve the kernel parameters from. eg: \'x1001c1s0b0n1,x1001c1s0b1n0\'' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -s H -l hsm-group -d 'List kernel parameters for all nodes in a HSM group name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -s f -l filter -d 'Comma separated list of kernel parameters to filter. eg: \'console,bad_page,crashkernel,hugepagelist,root\'' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -s o -l output -d 'Output format.' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "group" -d 'Get group details'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "hw-component" -d 'Get hardware components1 for a cluster or a node'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "sessions" -d 'Get information from Shasta CFS session'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "configurations" -d 'Get information from Shasta CFS configuration'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "templates" -d 'Get information from Shasta BOS template'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "cluster" -d 'Get cluster details'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "nodes" -d 'Get node details'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "hsm-groups" -d 'DEPRECATED - Please do not use this command. Get HSM groups details'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "images" -d 'Get image information'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "kernel-parameters" -d 'Get kernel-parameters information'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node group hw-component kernel-parameters help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node group hw-component kernel-parameters help" -f -a "node" -d 'Add/Create new node'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node group hw-component kernel-parameters help" -f -a "group" -d 'Add/Create new group'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node group hw-component kernel-parameters help" -f -a "hw-component" -d 'WIP - Add hw components from a cluster'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node group hw-component kernel-parameters help" -f -a "kernel-parameters" -d 'Add/Create kernel parameters'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node group hw-component kernel-parameters help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s i -l id -d 'Xname' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s g -l group -d 'Group name the node belongs to' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s H -l hardware -d 'hardware' -r -F
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s a -l arch -d 'Architecture' -r -f -a "X86\t''
ARM\t''
Other\t''"
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s d -l disabled -d 'Disable node upon creation'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from group" -s l -l label -d 'Group name' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from group" -s x -l xnames -d 'Comma separated list of nodes to set in new group. eg \'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0\'' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from group" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hw-component" -s P -l pattern -d 'Pattern' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hw-component" -s t -l target-cluster -d 'Target cluster name. This is the name of the cluster the pattern is applying to.' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hw-component" -s p -l parent-cluster -d 'Parent cluster name. The parent cluster is the one offering and receiving resources from the target cluster.' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hw-component" -s x -l no-dryrun -d 'No dry-run, actually change the status of the system. The default for this command is a dry-run.'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hw-component" -s c -l create-hsm-group -d 'If the target cluster name does not exist as HSM group, create it.'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hw-component" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s x -l xnames -d 'Comma separated list of nodes to set kernel parameters. eg \'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0\'' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s H -l hsm-group -d 'Cluster to set kernel parameters' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s y -l assume-yes -d 'Automatic yes to prompts; assume \'yes\' as answer to all prompts and run non-interactively.'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "node" -d 'Add/Create new node'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "group" -d 'Add/Create new group'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "hw-component" -d 'WIP - Add hw components from a cluster'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "kernel-parameters" -d 'Add/Create kernel parameters'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hw-configuration configuration sat-file boot session ephemeral-environment template help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hw-configuration configuration sat-file boot session ephemeral-environment template help" -f -a "hw-configuration" -d 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hw-configuration configuration sat-file boot session ephemeral-environment template help" -f -a "configuration" -d 'DEPRECATED - Please use \'manta apply sat-file\' command instead. Create a CFS configuration'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hw-configuration configuration sat-file boot session ephemeral-environment template help" -f -a "sat-file" -d 'Process a SAT file and creates the configurations, images, boot parameters and runtime configurations. If runtime configuration and boot parameters are defined, then, reboots the nodes to configure. The ansible container for the session building the image will remain running after an Ansible failure.  The container will remain running for a number of seconds specified by the \'debug_wait_time options\''
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hw-configuration configuration sat-file boot session ephemeral-environment template help" -f -a "boot" -d 'Change boot operations'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hw-configuration configuration sat-file boot session ephemeral-environment template help" -f -a "session" -d 'Runs the ansible script in local directory against HSM group or xnames. Note: the local repo must alrady exists in Shasta VCS'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hw-configuration configuration sat-file boot session ephemeral-environment template help" -f -a "ephemeral-environment" -d 'Returns a hostname use can ssh with the image ID provided. This call is async which means, the user will have to wait a few seconds for the environment to be ready, normally, this takes a few seconds.'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hw-configuration configuration sat-file boot session ephemeral-environment template help" -f -a "template" -d 'Create a new BOS session from an existing BOS sessiontemplate'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hw-configuration configuration sat-file boot session ephemeral-environment template help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from hw-configuration" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from hw-configuration" -f -a "cluster" -d 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from hw-configuration" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from configuration" -s t -l sat-template-file -d 'SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.' -r -F
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from configuration" -s f -l values-file -d 'If the SAT file is a jinja2 template, then variables values can be expanded using this values file.' -r -F
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from configuration" -s V -l values -d 'If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided.' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from configuration" -s o -l output -d 'Output format. If missing it will print output data in human redeable (table) format' -r -f -a "json\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from configuration" -s H -l hsm-group -d 'hsm group name linked to this configuration' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from configuration" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s t -l sat-template-file -d 'SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.' -r -F
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s f -l values-file -d 'If the SAT file is a jinja2 template, then variables values can be expanded using this values file.' -r -F
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s V -l values -d 'If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided.' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s v -l ansible-verbosity -d 'Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command. 1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.' -r -f -a "1\t''
2\t''
3\t''
4\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s P -l ansible-passthrough -d 'Additional parameters that are added to all Ansible calls for the session to create an image. This field is currently limited to the following Ansible parameters: "--extra-vars", "--forks", "--skip-tags", "--start-at-task", and "--tags". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs.' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s p -l pre-hook -d 'Command to run before processing SAT file. If need to pass a command with params. Use " or \'. eg: --pre-hook "echo hello"' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s a -l post-hook -d 'Command to run immediately after processing SAT file successfully. Use " or \'. eg: --post-hook "echo hello".' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -l do-not-reboot -d 'By default, nodes will restart if SAT file builds an image which is assigned to the nodes through a BOS sessiontemplate, if you do not want to reboot the nodes, then use this flag. The SAT file will be processeed as usual and different elements created but the nodes won\'t reboot. This means, you will have to run \'manta apply template\' command with the sessoin_template created\''
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s w -l watch-logs -d 'Watch logs. Hooks stdout to see container running ansible scripts'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s i -l image-only -d 'Only process `configurations` and `images` sections in SAT file. The `session_templates` section will be ignored.'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s s -l sessiontemplate-only -d 'Only process `configurations` and `session_templates` sections in SAT file. The `images` section will be ignored.'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s y -l assume-yes -d 'Automatic yes to prompts; assume \'yes\' as answer to all prompts and run non-interactively.'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot" -f -a "nodes" -d 'Update the boot parameters (boot image id, runtime configuration and kernel parameters) for a list of nodes. The boot image could be specified by either image id or the configuration name used to create the image id. eg: manta apply boot nodes --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot" -f -a "cluster" -d 'Update the boot parameters (boot image id, runtime configuration and kernel params) for all nodes in a cluster. The boot image could be specified by either image id or the configuration name used to create the image id. eg: manta apply boot cluster --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from session" -s n -l name -d 'Session name' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from session" -s p -l playbook-name -d 'Playbook YAML file name. eg (site.yml)' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from session" -s r -l repo-path -d 'Repo path. The path with a git repo and an ansible-playbook to configure the CFS image' -r -f -a "(__fish_complete_directories)"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from session" -s v -l ansible-verbosity -d 'Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command. 1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.' -r -f -a "0\t''
1\t''
2\t''
3\t''
4\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from session" -s P -l ansible-passthrough -d 'Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: "--extra-vars", "--forks", "--skip-tags", "--start-at-task", and "--tags". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs.' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from session" -s l -l ansible-limit -d 'Ansible limit. Target xnames to the CFS session. Note: ansible-limit must be a subset of hsm-group if both parameters are provided' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from session" -s H -l hsm-group -d 'hsm group name' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from session" -s w -l watch-logs -d 'Watch logs. Hooks stdout to see container running ansible scripts'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from session" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from ephemeral-environment" -s i -l image-id -d 'Image ID to use as a container image' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from ephemeral-environment" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s n -l name -d 'Name of the Session' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s o -l operation -d 'An operation to perform on Components in this Session. Boot Applies the Template to the Components and boots/reboots if necessary. Reboot Applies the Template to the Components; guarantees a reboot. Shutdown Power down Components that are on' -r -f -a "reboot\t''
boot\t''
shutdown\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s t -l template -d 'Name of the Session Template' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s l -l limit -d 'A comma-separated list of nodes, groups, or roles to which the Session will be limited. Components are treated as OR operations unless preceded by \'&\' for AND or \'!\' for NOT' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s i -l include-disabled -d 'Set to include nodes that have been disabled as indicated in the Hardware State Manager (HSM)'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s d -l dry-run -d 'Simulates the execution of the command without making any actual changes.'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "hw-configuration" -d 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "configuration" -d 'DEPRECATED - Please use \'manta apply sat-file\' command instead. Create a CFS configuration'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "sat-file" -d 'Process a SAT file and creates the configurations, images, boot parameters and runtime configurations. If runtime configuration and boot parameters are defined, then, reboots the nodes to configure. The ansible container for the session building the image will remain running after an Ansible failure.  The container will remain running for a number of seconds specified by the \'debug_wait_time options\''
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "boot" -d 'Change boot operations'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "session" -d 'Runs the ansible script in local directory against HSM group or xnames. Note: the local repo must alrady exists in Shasta VCS'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "ephemeral-environment" -d 'Returns a hostname use can ssh with the image ID provided. This call is async which means, the user will have to wait a few seconds for the environment to be ready, normally, this takes a few seconds.'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "template" -d 'Create a new BOS session from an existing BOS sessiontemplate'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group kernel-parameters configurations session images hw-component help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group kernel-parameters configurations session images hw-component help" -f -a "group" -d 'Delete group. This command will fail if the group is not empty, please move group members to another group using command \'migrate nodes\' before deletion'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group kernel-parameters configurations session images hw-component help" -f -a "kernel-parameters" -d 'Delete kernel parameters'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group kernel-parameters configurations session images hw-component help" -f -a "configurations" -d 'Deletes CFS configurations and all data related (CFS sessions, BOS sessiontemplates, BOS sessions and images).'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group kernel-parameters configurations session images hw-component help" -f -a "session" -d 'Deletes a session. For \'image\' sessions, it also removes the associated image. For \'dynamic\' sessions, it sets the \'error count\' to its maximum value.'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group kernel-parameters configurations session images hw-component help" -f -a "images" -d 'WIP - Deletes a list of images.'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group kernel-parameters configurations session images hw-component help" -f -a "hw-component" -d 'WIP - Remove hw components from a cluster'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group kernel-parameters configurations session images hw-component help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from group" -s l -l label -d 'Group name to delete' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from group" -s y -l assume-yes -d 'Automatic yes to prompts; assume \'yes\' as answer to all prompts and run non-interactively.'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from group" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from kernel-parameters" -s x -l xnames -d 'Comma separated list of nodes to set runtime configuration. eg \'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0\'' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from kernel-parameters" -s H -l hsm-group -d 'Cluster to set runtime configuration' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from kernel-parameters" -s y -l assume-yes -d 'Automatic yes to prompts; assume \'yes\' as answer to all prompts and run non-interactively.'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from kernel-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s n -l configuration-name -d 'CFS configuration, CFS sessions, BOS sessiontemplate, BOS sessions and IMS images related to the CFS configuration will be deleted. eg: manta delete --configuration-name my-config-v1.0 Deletes all data related to CFS configuration with name \'my-config-v0.1\'' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s p -l pattern -d 'Glob pattern for configuration name' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s s -l since -d 'Deletes CFS configurations, CFS sessions, BOS sessiontemplate, BOS sessions and images related to CFS configurations with \'last updated\' after since date. Note: date format is %Y-%m-%d eg: manta delete --since 2023-01-01 --until 2023-10-01 Deletes all data related to CFS configurations created or updated between 01/01/2023T00:00:00Z and 01/10/2023T00:00:00Z' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s u -l until -d 'Deletes CFS configuration, CFS sessions, BOS sessiontemplate, BOS sessions and images related to the CFS configuration with \'last updated\' before until date. Note: date format is %Y-%m-%d eg: manta delete --until 2023-10-01 Deletes all data related to CFS configurations created or updated before 01/10/2023T00:00:00Z' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s y -l assume-yes -d 'Automatic yes to prompts; assume \'yes\' as answer to all prompts and run non-interactively. Image artifacts and configurations used by nodes will not be deleted'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from session" -s d -l dry-run -d 'Simulates the execution of the command without making any actual changes.'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from session" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from images" -s d -l dry-run -d 'Simulates the execution of the command without making any actual changes.'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from images" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hw-component" -s P -l pattern -d 'Pattern' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hw-component" -s t -l target-cluster -d 'Target cluster name. This is the name of the cluster the pattern is applying to (resources move from here).' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hw-component" -s p -l parent-cluster -d 'Parent cluster name. The parent cluster is the one receiving resources from the target cluster (resources move here).' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hw-component" -s x -l no-dryrun -d 'No dry-run, actually change the status of the system. The default for this command is a dry-run.'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hw-component" -s d -l delete-hsm-group -d 'Delete the HSM group if empty after this action.'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hw-component" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "group" -d 'Delete group. This command will fail if the group is not empty, please move group members to another group using command \'migrate nodes\' before deletion'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "kernel-parameters" -d 'Delete kernel parameters'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "configurations" -d 'Deletes CFS configurations and all data related (CFS sessions, BOS sessiontemplates, BOS sessions and images).'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "session" -d 'Deletes a session. For \'image\' sessions, it also removes the associated image. For \'dynamic\' sessions, it sets the \'error count\' to its maximum value.'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "images" -d 'WIP - Deletes a list of images.'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "hw-component" -d 'WIP - Remove hw components from a cluster'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand migrate; and not __fish_seen_subcommand_from vCluster nodes help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand migrate; and not __fish_seen_subcommand_from vCluster nodes help" -f -a "vCluster" -d 'WIP - Migrate vCluster'
complete -c manta -n "__fish_manta_using_subcommand migrate; and not __fish_seen_subcommand_from vCluster nodes help" -f -a "nodes" -d 'Migrate nodes across vClusters'
complete -c manta -n "__fish_manta_using_subcommand migrate; and not __fish_seen_subcommand_from vCluster nodes help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from vCluster" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from vCluster" -f -a "backup" -d 'Backup the configuration (BOS, CFS, image and HSM group) of a given vCluster/BOS session template.'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from vCluster" -f -a "restore" -d 'MIGRATE RESTORE of all the nodes in a HSM group. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted. eg: manta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from vCluster" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from nodes" -s f -l from -d 'The name of the source vCluster from which the compute nodes will be moved.' -r
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from nodes" -s t -l to -d 'The name of the target vCluster to which the compute nodes will be moved.' -r
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from nodes" -s d -l dry-run -d 'Simulates the execution of the command without making any actual changes.'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from nodes" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from help" -f -a "vCluster" -d 'WIP - Migrate vCluster'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from help" -f -a "nodes" -d 'Migrate nodes across vClusters'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -f -a "on" -d 'Command to power on cluster/node'
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -f -a "off" -d 'Command to power off cluster/node'
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -f -a "reset" -d 'Command to power reset cluster/node'
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from on" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from on" -f -a "cluster" -d 'Command to power on all nodes in a cluster'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from on" -f -a "nodes" -d 'Command to power on a group of nodes. eg: \'x1001c1s0b0n1,x1001c1s0b1n0\''
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from on" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from off" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from off" -f -a "cluster" -d 'Command to power off all nodes in a cluster'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from off" -f -a "nodes" -d 'Command to power off a group of nodes. eg: \'x1001c1s0b0n1,x1001c1s0b1n0\''
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from off" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from reset" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from reset" -f -a "cluster" -d 'Command to power reset all nodes in a cluster'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from reset" -f -a "nodes" -d 'Command to power reset a group of nodes. eg: \'x1001c1s0b0n1,x1001c1s0b1n0\''
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from reset" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from help" -f -a "on" -d 'Command to power on cluster/node'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from help" -f -a "off" -d 'Command to power off cluster/node'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from help" -f -a "reset" -d 'Command to power reset cluster/node'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand log" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand console; and not __fish_seen_subcommand_from node target-ansible help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand console; and not __fish_seen_subcommand_from node target-ansible help" -f -a "node" -d 'Connects to a node\'s console'
complete -c manta -n "__fish_manta_using_subcommand console; and not __fish_seen_subcommand_from node target-ansible help" -f -a "target-ansible" -d 'Opens an interactive session to the ansible target container of a CFS session'
complete -c manta -n "__fish_manta_using_subcommand console; and not __fish_seen_subcommand_from node target-ansible help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from node" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from target-ansible" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from help" -f -a "node" -d 'Connects to a node\'s console'
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from help" -f -a "target-ansible" -d 'Opens an interactive session to the ansible target container of a CFS session'
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand validate-local-repo" -s r -l repo-path -d 'Repo path. The path to a local a git repo related to a CFS configuration layer to test against Gitea' -r
complete -c manta -n "__fish_manta_using_subcommand validate-local-repo" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add-nodes-to-groups" -s g -l group -d 'HSM group to assign the nodes to' -r
complete -c manta -n "__fish_manta_using_subcommand add-nodes-to-groups" -s n -l nodes -d 'Comma separated list of nids or xnames. eg \'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0\' or \'nid001313,nid001314\'  Hostlist format also accepted eg \'x1003c1s7b0n[0-1],x1003c1s7b1n0\' or \'nid0000[10-15]\'' -r
complete -c manta -n "__fish_manta_using_subcommand add-nodes-to-groups" -s r -l regex -d 'Input nodes in regex format.'
complete -c manta -n "__fish_manta_using_subcommand add-nodes-to-groups" -s d -l dry-run -d 'Simulates the execution of the command without making any actual changes.'
complete -c manta -n "__fish_manta_using_subcommand add-nodes-to-groups" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand remove-nodes-from-groups" -s g -l group -d 'HSM group to remove the nodes from' -r
complete -c manta -n "__fish_manta_using_subcommand remove-nodes-from-groups" -s n -l nodes -d 'Comma separated list of nids or xnames. eg \'x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0\' or \'nid001313,nid001314\'  Hostlist format also accepted eg \'x1003c1s7b0n[0-1],x1003c1s7b1n0\' or \'nid0000[10-15]\'' -r
complete -c manta -n "__fish_manta_using_subcommand remove-nodes-from-groups" -s r -l regex -d 'Input nodes in regex format.'
complete -c manta -n "__fish_manta_using_subcommand remove-nodes-from-groups" -s d -l dry-run -d 'Simulates the execution of the command without making any actual changes.'
complete -c manta -n "__fish_manta_using_subcommand remove-nodes-from-groups" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "config" -d 'Manta\'s configuration'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "get" -d 'Get information from CSM system'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "add" -d 'Add/Create new elements to system. Nodes will be added to the user\'s \'parent\' group'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "apply" -d 'Make changes to Shasta system'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "delete" -d 'Deletes data'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "migrate"
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "power" -d 'Command to submit commands related to cluster/node power management'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "log" -d 'get cfs session logs'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "console" -d 'Opens an interective session to a node or CFS session ansible target container'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "validate-local-repo" -d 'Check all tags and HEAD information related to a local repo exists in Gitea'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "add-nodes-to-groups" -d 'Add nodes to a list of groups'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "remove-nodes-from-groups" -d 'Remove nodes from groups'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate power log console validate-local-repo add-nodes-to-groups remove-nodes-from-groups help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "show" -d 'Show config values'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "set" -d 'Change config values'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "unset" -d 'Reset config values'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "gen-autocomplete" -d 'Generate shell auto completion script'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "group" -d 'Get group details'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "hw-component" -d 'Get hardware components1 for a cluster or a node'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "sessions" -d 'Get information from Shasta CFS session'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "configurations" -d 'Get information from Shasta CFS configuration'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "templates" -d 'Get information from Shasta BOS template'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "cluster" -d 'Get cluster details'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "nodes" -d 'Get node details'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "hsm-groups" -d 'DEPRECATED - Please do not use this command. Get HSM groups details'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "images" -d 'Get image information'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "kernel-parameters" -d 'Get kernel-parameters information'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "node" -d 'Add/Create new node'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "group" -d 'Add/Create new group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "hw-component" -d 'WIP - Add hw components from a cluster'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "kernel-parameters" -d 'Add/Create kernel parameters'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "hw-configuration" -d 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "configuration" -d 'DEPRECATED - Please use \'manta apply sat-file\' command instead. Create a CFS configuration'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "sat-file" -d 'Process a SAT file and creates the configurations, images, boot parameters and runtime configurations. If runtime configuration and boot parameters are defined, then, reboots the nodes to configure. The ansible container for the session building the image will remain running after an Ansible failure.  The container will remain running for a number of seconds specified by the \'debug_wait_time options\''
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "boot" -d 'Change boot operations'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "session" -d 'Runs the ansible script in local directory against HSM group or xnames. Note: the local repo must alrady exists in Shasta VCS'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "ephemeral-environment" -d 'Returns a hostname use can ssh with the image ID provided. This call is async which means, the user will have to wait a few seconds for the environment to be ready, normally, this takes a few seconds.'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "template" -d 'Create a new BOS session from an existing BOS sessiontemplate'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "group" -d 'Delete group. This command will fail if the group is not empty, please move group members to another group using command \'migrate nodes\' before deletion'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "kernel-parameters" -d 'Delete kernel parameters'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "configurations" -d 'Deletes CFS configurations and all data related (CFS sessions, BOS sessiontemplates, BOS sessions and images).'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "session" -d 'Deletes a session. For \'image\' sessions, it also removes the associated image. For \'dynamic\' sessions, it sets the \'error count\' to its maximum value.'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "images" -d 'WIP - Deletes a list of images.'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "hw-component" -d 'WIP - Remove hw components from a cluster'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from migrate" -f -a "vCluster" -d 'WIP - Migrate vCluster'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from migrate" -f -a "nodes" -d 'Migrate nodes across vClusters'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from power" -f -a "on" -d 'Command to power on cluster/node'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from power" -f -a "off" -d 'Command to power off cluster/node'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from power" -f -a "reset" -d 'Command to power reset cluster/node'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from console" -f -a "node" -d 'Connects to a node\'s console'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from console" -f -a "target-ansible" -d 'Opens an interactive session to the ansible target container of a CFS session'
