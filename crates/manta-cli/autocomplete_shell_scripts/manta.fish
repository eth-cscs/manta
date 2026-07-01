# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_manta_global_optspecs
    string join \n site= h/help V/version
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

complete -c manta -n "__fish_manta_needs_command" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_needs_command" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_needs_command" -s V -l version -d 'Print version'
complete -c manta -n "__fish_manta_needs_command" -f -a "config" -d 'Show or change CLI-side settings (active site, default node group, log level)'
complete -c manta -n "__fish_manta_needs_command" -f -a "get" -d 'Inspect groups, nodes, hardware, images, configurations, sessions, templates, and boot/kernel parameters'
complete -c manta -n "__fish_manta_needs_command" -f -a "add" -d 'Register new nodes, groups, boot/kernel parameters, hardware components, or Redfish endpoints'
complete -c manta -n "__fish_manta_needs_command" -f -a "apply" -d 'Roll out configurations, images, session templates, boot/kernel parameters, and hardware rescaling'
complete -c manta -n "__fish_manta_needs_command" -f -a "delete" -d 'Remove nodes, groups, images, configurations, sessions, boot/kernel parameters, or Redfish endpoints'
complete -c manta -n "__fish_manta_needs_command" -f -a "migrate" -d 'Move nodes between groups'
complete -c manta -n "__fish_manta_needs_command" -f -a "backup" -d 'Back up a virtual cluster (images, boot settings, group membership) to disk'
complete -c manta -n "__fish_manta_needs_command" -f -a "restore" -d 'Restore a virtual cluster from a backup bundle'
complete -c manta -n "__fish_manta_needs_command" -f -a "run" -d 'Create and run a configuration session from a local Ansible repo'
complete -c manta -n "__fish_manta_needs_command" -f -a "power" -d 'Power nodes on, off, or reset (reboot); waits for the transition unless --no-wait is set'
complete -c manta -n "__fish_manta_needs_command" -f -a "log" -d 'Stream configuration session logs to stdout (accepts session, node, group, or NID)'
complete -c manta -n "__fish_manta_needs_command" -f -a "console" -d 'Attach to a node\'s serial console, or to a configuration session\'s Ansible container'
complete -c manta -n "__fish_manta_needs_command" -f -a "gen-autocomplete" -d 'Generate and install shell completion scripts'
complete -c manta -n "__fish_manta_needs_command" -f -a "gen-man" -d 'Generate and install the manta man page'
complete -c manta -n "__fish_manta_needs_command" -f -a "upgrade" -d 'Replace this `manta` binary with the latest release'
complete -c manta -n "__fish_manta_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset help" -f -a "show" -d 'Show current configuration values'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset help" -f -a "set" -d 'Set a configuration value'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset help" -f -a "unset" -d 'Clear a configuration value'
complete -c manta -n "__fish_manta_using_subcommand config; and not __fish_seen_subcommand_from show set unset help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from show" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from show" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from show" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -f -a "hsm" -d 'Set the active node group'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -f -a "log" -d 'Set the log verbosity level'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -f -a "read-only" -d 'Refuse every backend-mutating command until unset'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -f -a "site" -d 'Set the active site'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from set" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -f -a "auth" -d 'Clear the cached authentication token'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -f -a "hsm" -d 'Clear the active node group'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -f -a "read-only" -d 'Allow backend-mutating commands again'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from unset" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "show" -d 'Show current configuration values'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "set" -d 'Set a configuration value'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "unset" -d 'Clear a configuration value'
complete -c manta -n "__fish_manta_using_subcommand config; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "groups" -d 'List node groups visible to your token (or look up one by name)'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "hardware" -d 'Inspect hardware components'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "sessions" -d 'List configuration sessions'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "configurations" -d 'List CFS configurations (filter by name, glob, group, or recency)'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "templates" -d 'List BOS session templates (filter by name, group, or recency)'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "group-nodes" -d 'Show node details and status for a group'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "nodes" -d 'Show node details and status'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "images" -d 'List IMS images (filter by id, name glob, or recency; sorted most-recent first)'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "boot-parameters" -d 'Show the BSS boot parameters (kernel, initrd, params) for nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "kernel-parameters" -d 'Show kernel parameters for nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "redfish-endpoints" -d 'List the BMCs / controllers the hardware state manager has registered as Redfish endpoints'
complete -c manta -n "__fish_manta_using_subcommand get; and not __fish_seen_subcommand_from groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from groups" -s o -l output -d 'Output format' -r -f -a "json\t''
table\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from groups" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from groups" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from hardware" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from hardware" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from hardware" -f -a "nodes" -d 'Show hardware inventory for a set of nodes'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from hardware" -f -a "group" -d 'Show hardware inventory for a group'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from hardware" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s n -l name -d 'Return only the session with this name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s a -l min-age -d 'Return only sessions older than this age (eg: \'1d\', \'6h\')' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s A -l max-age -d 'Return only sessions younger than this age (eg: \'1d\', \'6h\')' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s t -l type -d 'Return only sessions of this type' -r -f -a "image\t''
runtime\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s s -l status -d 'Return only sessions with this status' -r -f -a "pending\t''
running\t''
complete\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s l -l limit -d 'Return only the <VALUE> most recent sessions' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s o -l output -d 'Output format' -r -f -a "json\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s x -l xnames -d 'Xnames, NIDs, or hostlist expression. Returns sessions targeting these nodes or their groups' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s H -l group -l hsm-group -d 'Node group name. Returns sessions targeting this group or its members' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s m -l most-recent -d 'Return only the most recent session (equivalent to --limit 1)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from sessions" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s n -l name -d 'Show only the configuration with this exact name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s p -l pattern -d 'Glob pattern to filter by name (eg: \'my-cfg*\')' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s l -l limit -d 'Return only the <VALUE> most recent configurations' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s o -l output -d 'Output format' -r -f -a "json\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s H -l group -l hsm-group -d 'Show only configurations whose layers target this group' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s m -l most-recent -d 'Return only the most recent (equivalent to --limit 1)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -l only-safe-to-delete -d 'Show only configurations that are safe to delete'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -l only-unsafe-to-delete -d 'Show only configurations that are NOT safe to delete (in use)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from configurations" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s n -l name -d 'Show only the template with this exact name' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s l -l limit -d 'Return only the <VALUE> most recent templates' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s H -l group -l hsm-group -d 'Show only templates whose boot sets target this group' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s o -l output -d 'Output format' -r -f -a "json\t''
table\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s m -l most-recent -d 'Return only the most recent (equivalent to --limit 1)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from templates" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from group-nodes" -s s -l status -d 'Filter nodes by status' -r -f -a "OFF\t''
ON\t''
READY\t''
STANDBY\t''
PENDING\t''
FAILED\t''
CONFIGURED\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from group-nodes" -s o -l output -d 'Output format' -r -f -a "table\t''
table-wide\t''
json\t''
summary\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from group-nodes" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from group-nodes" -s n -l nids-only-one-line -d 'Print NIDs on a single line'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from group-nodes" -s x -l xnames-only-one-line -d 'Print xnames on a single line'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from group-nodes" -s T -l summary-status -d 'Show a group status summary: OK          — all nodes booted and configured OFF         — at least one node is OFF ON          — no nodes OFF, at least one is ON STANDBY     — at least one node\'s heartbeat is lost UNCONFIGURED — all nodes READY but at least one is still being configured FAILED      — at least one node\'s configuration failed'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from group-nodes" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s s -l status -d 'Filter nodes by status' -r -f -a "OFF\t''
ON\t''
READY\t''
STANDBY\t''
PENDING\t''
FAILED\t''
CONFIGURED\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s o -l output -d 'Output format' -r -f -a "table\t''
table-wide\t''
json\t''
summary\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s n -l nids-only-one-line -d 'Print NIDs on a single line'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s T -l summary-status -d 'Show a node status summary: OK          — all nodes booted and configured OFF         — at least one node is OFF ON          — no nodes OFF, at least one is ON STANDBY     — at least one node\'s heartbeat is lost UNCONFIGURED — all nodes READY but at least one is still being configured FAILED      — at least one node\'s configuration failed'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s S -l include-siblings -d 'Also show sibling nodes that share a power supply with the requested nodes'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from nodes" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -s i -l id -d 'Show only the image with this exact ID' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -s p -l pattern -d 'Glob matched against image name (e.g. \'compute-*\'); applied server-side. Invalid glob returns 400.' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -s l -l limit -d 'Return only the <VALUE> most recent images' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -s m -l most-recent -d 'Return only the most recent (equivalent to --limit 1)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -l only-safe-to-delete -d 'Show only images that are safe to delete'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -l only-unsafe-to-delete -d 'Show only images that are NOT safe to delete (currently used as a node\'s boot image)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from images" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from boot-parameters" -s H -l group -l hsm-group -d 'Show boot parameters for every node in this group' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from boot-parameters" -s n -l nodes -d 'Xnames, NIDs, or a hostlist expression. eg: \'x1003c1s7b0n0,x1003c1s7b0n1\', \'nid001313,nid001314\', \'x1003c1s7b0n[0-1],x1003c1s7b1n0\', \'nid00131[0-9]\'' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from boot-parameters" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from boot-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -s n -l nodes -d 'Xnames, NIDs, or a hostlist expression. eg: \'x1003c1s7b0n0,x1003c1s7b0n1\', \'nid001313,nid001314\', \'x1003c1s7b0n[0-1],x1003c1s7b1n0\', \'nid00131[0-9]\'' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -s H -l group -l hsm-group -d 'Show kernel parameters for all nodes in this group' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -s f -l filter -d 'Comma-separated list of parameter names to show. eg: \'console,bad_page,crashkernel,hugepagelist,root\'' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from kernel-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from redfish-endpoints" -s i -l id -d 'Filter by xname (can be specified multiple times)' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from redfish-endpoints" -s f -l fqdn -d 'Filter by FQDN' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from redfish-endpoints" -s u -l uuid -d 'Filter by UUID' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from redfish-endpoints" -s m -l macaddr -d 'Filter by MAC address' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from redfish-endpoints" -s I -l ipaddress -d 'Filter by IP address (empty string matches endpoints without an IP)' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from redfish-endpoints" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from redfish-endpoints" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from redfish-endpoints" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "groups" -d 'List node groups visible to your token (or look up one by name)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "hardware" -d 'Inspect hardware components'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "sessions" -d 'List configuration sessions'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "configurations" -d 'List CFS configurations (filter by name, glob, group, or recency)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "templates" -d 'List BOS session templates (filter by name, group, or recency)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "group-nodes" -d 'Show node details and status for a group'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "nodes" -d 'Show node details and status'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "images" -d 'List IMS images (filter by id, name glob, or recency; sorted most-recent first)'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "boot-parameters" -d 'Show the BSS boot parameters (kernel, initrd, params) for nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "kernel-parameters" -d 'Show kernel parameters for nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "redfish-endpoints" -d 'List the BMCs / controllers the hardware state manager has registered as Redfish endpoints'
complete -c manta -n "__fish_manta_using_subcommand get; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -f -a "node" -d 'Register a new node in the hardware state manager'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -f -a "nodes" -d 'Add existing nodes to a group'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -f -a "group" -d 'Create a node group'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -f -a "hardware" -d '[experimental] Add hardware components to a group'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -f -a "boot-parameters" -d 'Create a BSS boot-parameters entry (kernel, initrd, params, cloud-init) for one or more nodes'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -f -a "kernel-parameters" -d 'Append kernel parameters to nodes (leaves existing parameters untouched unless --overwrite is set)'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -f -a "redfish-endpoints" -d 'Register a new Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -f -a "redfish-endpoint" -d 'Register a new Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand add; and not __fish_seen_subcommand_from node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s i -l id -d 'Xname to register' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s g -l group -d 'Group to put the new node into' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s H -l hardware -d 'File containing hardware information' -r -F
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s a -l arch -d 'Node architecture' -r -f -a "X86\t''
ARM\t''
Other\t''"
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s D -l disabled -d 'Register the node as disabled'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from node" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from nodes" -s g -l group -d 'Group to add the nodes to' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from nodes" -s n -l nodes -d 'Xnames, NIDs, or a hostlist expression. eg: \'x1003c1s7b0n0,x1003c1s7b0n1\', \'nid001313,nid001314\', \'x1003c1s7b0n[0-1],x1003c1s7b1n0\', \'nid00131[0-9]\'' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from nodes" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from nodes" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from nodes" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from nodes" -s y -l assume-yes -d 'Skip confirmation prompts'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from nodes" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from group" -s l -l label -d 'Group name' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from group" -s D -l description -d 'Group description' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from group" -s n -l nodes -d 'Xnames, NIDs, or a hostlist expression. eg: \'x1003c1s7b0n0,x1003c1s7b0n1\', \'nid001313,nid001314\', \'x1003c1s7b0n[0-1],x1003c1s7b1n0\', \'nid00131[0-9]\'' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from group" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from group" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from group" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from group" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hardware" -s P -l pattern -d 'Hardware component pattern' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hardware" -s t -l target-group -l target-cluster -d 'Group to add components to' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hardware" -s p -l parent-group -l parent-cluster -d 'Group that donates the components' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hardware" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hardware" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hardware" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hardware" -s c -l create-group -l create-hsm-group -d 'Create the target group if it does not exist'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from hardware" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s H -l hosts -d 'Xnames of the nodes' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s n -l nids -d 'Comma-separated NIDs of the nodes' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s m -l macs -d 'Comma-separated MAC addresses of the nodes' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s p -l params -d 'Kernel parameters' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s k -l kernel -d 'S3 path to the kernel file' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s i -l initrd -d 'S3 path to the initrd file' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s c -l cloud-init -d 'Cloud-init script' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s y -l assume-yes -d 'Skip confirmation prompts'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from boot-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s n -l nodes -d 'Xnames, NIDs, or a hostlist expression. eg: \'x1003c1s7b0n0,x1003c1s7b0n1\', \'nid001313,nid001314\', \'x1003c1s7b0n[0-1],x1003c1s7b1n0\', \'nid00131[0-9]\'' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s H -l group -l hsm-group -d 'Append kernel parameters to every node in this group' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s O -l overwrite -d 'Overwrite the value if the parameter already exists'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s y -l assume-yes -d 'Skip confirmation prompts'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -l do-not-reboot -d 'Do not reboot nodes after applying changes'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from kernel-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s i -l id -d 'Xname of the BMC or controller' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s n -l name -d 'Arbitrary user-provided name for the endpoint' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s H -l hostname -d 'Hostname (FQDN host portion); normally identical to the xname' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s D -l domain -d 'Domain (FQDN domain portion)' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s f -l fqdn -d 'Fully-qualified domain name on the management network (derived from hostname + domain)' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s u -l user -d 'Username for endpoint authentication' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s p -l password -d 'Password for endpoint authentication' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s M -l macaddr -d 'MAC address of the Redfish endpoint on the management network' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s I -l ipaddress -d 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s t -l template-id -d 'Discovery template ID' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s e -l enabled -d 'Enable the endpoint upon creation'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s U -l use-ssdp -d 'Use SSDP for discovery if the endpoint supports it'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s m -l mac-required -d 'Require a MAC address for geolocation'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s r -l rediscover-on-update -d 'Trigger rediscovery when endpoint information is updated'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoints" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s i -l id -d 'Xname of the BMC or controller' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s n -l name -d 'Arbitrary user-provided name for the endpoint' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s H -l hostname -d 'Hostname (FQDN host portion); normally identical to the xname' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s D -l domain -d 'Domain (FQDN domain portion)' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s f -l fqdn -d 'Fully-qualified domain name on the management network (derived from hostname + domain)' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s u -l user -d 'Username for endpoint authentication' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s p -l password -d 'Password for endpoint authentication' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s M -l macaddr -d 'MAC address of the Redfish endpoint on the management network' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s I -l ipaddress -d 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s t -l template-id -d 'Discovery template ID' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s e -l enabled -d 'Enable the endpoint upon creation'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s U -l use-ssdp -d 'Use SSDP for discovery if the endpoint supports it'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s m -l mac-required -d 'Require a MAC address for geolocation'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s r -l rediscover-on-update -d 'Trigger rediscovery when endpoint information is updated'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from redfish-endpoint" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "node" -d 'Register a new node in the hardware state manager'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "nodes" -d 'Add existing nodes to a group'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "group" -d 'Create a node group'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "hardware" -d '[experimental] Add hardware components to a group'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "boot-parameters" -d 'Create a BSS boot-parameters entry (kernel, initrd, params, cloud-init) for one or more nodes'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "kernel-parameters" -d 'Append kernel parameters to nodes (leaves existing parameters untouched unless --overwrite is set)'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "redfish-endpoints" -d 'Register a new Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand add; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -f -a "hardware" -d '[experimental] Rescale a group\'s hardware allocation'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -f -a "sat-file" -d 'Process a SAT file to create configurations, images, and session templates'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -f -a "boot" -d 'Update boot parameters and runtime configuration'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -f -a "boot-parameters" -d 'Update boot parameters for nodes'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -f -a "redfish-endpoints" -d 'Update an existing Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -f -a "redfish-endpoint" -d 'Update an existing Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -f -a "kernel-parameters" -d 'Replace the full kernel-parameters string on nodes (drops any existing parameters not listed)'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -f -a "ephemeral-environment" -d 'Launch an ephemeral SSH environment from an image'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -f -a "template" -d 'Boot nodes using an existing session template'
complete -c manta -n "__fish_manta_using_subcommand apply; and not __fish_seen_subcommand_from hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from hardware" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from hardware" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from hardware" -f -a "group" -d '[experimental] Rescale a group\'s hardware allocation'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from hardware" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s t -l sat-template-file -d 'SAT file path (may be a jinja2 template)' -r -F
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s f -l values-file -d 'Values file to expand jinja2 variables in the SAT file' -r -F
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s V -l values -d 'Inline values to expand jinja2 variables (overrides --values-file)' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s v -l ansible-verbosity -d 'Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)' -r -f -a "1\t''
2\t''
3\t''
4\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s P -l ansible-passthrough -d 'Additional Ansible flags (limited to --extra-vars, --forks, --skip-tags, --start-at-task, --tags)' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s p -l pre-hook -d 'Command to run before processing. eg: --pre-hook "echo hello"' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s a -l post-hook -d 'Command to run after successful processing. eg: --post-hook "echo hello"' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -l create-bos-session -d 'After each BOS session template is created, create a BOS session from it so its target nodes boot via the new template (this typically causes a reboot)'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s o -l overwrite-configuration -d 'Overwrite an existing configuration with the same name'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s w -l watch-logs -d 'Stream session logs to stdout'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s T -l timestamps -d 'Show log timestamps'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s i -l image-only -d 'Process only the `configurations` and `images` sections'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s s -l sessiontemplate-only -d 'Process only the `configurations` and `session_templates` sections'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s y -l assume-yes -d 'Skip confirmation prompts'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from sat-file" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot" -f -a "nodes" -d 'Update boot parameters for a set of nodes'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot" -f -a "group" -d 'Update boot parameters for all nodes in a group'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot-parameters" -s H -l hosts -d 'Xnames of the nodes to update' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot-parameters" -s p -l params -d 'Kernel parameters' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot-parameters" -s k -l kernel -d 'S3 path to the kernel file' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot-parameters" -s i -l initrd -d 'S3 path to the initrd file' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot-parameters" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot-parameters" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot-parameters" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from boot-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s i -l id -d 'Xname of the endpoint to update' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s n -l name -d 'Arbitrary user-provided name for the endpoint' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s H -l hostname -d 'Hostname (FQDN host portion)' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s D -l domain -d 'Domain (FQDN domain portion)' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s f -l fqdn -d 'Fully-qualified domain name on the management network' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s u -l user -d 'Username for endpoint authentication' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s p -l password -d 'Password for endpoint authentication' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s M -l macaddr -d 'MAC address of the Redfish endpoint on the management network' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s I -l ipaddress -d 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s t -l template-id -d 'Discovery template ID' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s e -l enabled -d 'Enable the endpoint'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s U -l use-ssdp -d 'Use SSDP for discovery if the endpoint supports it'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s m -l mac-required -d 'Require a MAC address for geolocation'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s r -l rediscover-on-update -d 'Trigger rediscovery when endpoint information is updated'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoints" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s i -l id -d 'Xname of the endpoint to update' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s n -l name -d 'Arbitrary user-provided name for the endpoint' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s H -l hostname -d 'Hostname (FQDN host portion)' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s D -l domain -d 'Domain (FQDN domain portion)' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s f -l fqdn -d 'Fully-qualified domain name on the management network' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s u -l user -d 'Username for endpoint authentication' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s p -l password -d 'Password for endpoint authentication' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s M -l macaddr -d 'MAC address of the Redfish endpoint on the management network' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s I -l ipaddress -d 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s t -l template-id -d 'Discovery template ID' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s e -l enabled -d 'Enable the endpoint'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s U -l use-ssdp -d 'Use SSDP for discovery if the endpoint supports it'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s m -l mac-required -d 'Require a MAC address for geolocation'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s r -l rediscover-on-update -d 'Trigger rediscovery when endpoint information is updated'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from redfish-endpoint" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from kernel-parameters" -s n -l nodes -d 'Xnames, NIDs, or a hostlist expression. eg: \'x1003c1s7b0n0,x1003c1s7b0n1\', \'nid001313,nid001314\', \'x1003c1s7b0n[0-1],x1003c1s7b1n0\', \'nid00131[0-9]\'' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from kernel-parameters" -s H -l group -l hsm-group -d 'Replace kernel parameters on every node in this group' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from kernel-parameters" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from kernel-parameters" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from kernel-parameters" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from kernel-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from ephemeral-environment" -s i -l image-id -d 'Image ID to use' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from ephemeral-environment" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from ephemeral-environment" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from ephemeral-environment" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from ephemeral-environment" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s n -l name -d 'Name of the boot session to create' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s o -l operation -d 'Boot operation to perform' -r -f -a "reboot\t''
boot\t''
shutdown\t''"
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s t -l template -d 'Session template name' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s l -l limit -d 'Limit to specific nodes, groups, or roles (OR by default; prefix with \'&\' for AND or \'!\' for NOT)' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s i -l include-disabled -d 'Include nodes marked as disabled in the hardware state manager'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from template" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "hardware" -d '[experimental] Rescale a group\'s hardware allocation'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "sat-file" -d 'Process a SAT file to create configurations, images, and session templates'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "boot" -d 'Update boot parameters and runtime configuration'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "boot-parameters" -d 'Update boot parameters for nodes'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "redfish-endpoints" -d 'Update an existing Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "kernel-parameters" -d 'Replace the full kernel-parameters string on nodes (drops any existing parameters not listed)'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "ephemeral-environment" -d 'Launch an ephemeral SSH environment from an image'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "template" -d 'Boot nodes using an existing session template'
complete -c manta -n "__fish_manta_using_subcommand apply; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "group" -d 'Delete a node group'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "node" -d 'Remove a node from the hardware state manager'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "nodes" -d 'Remove nodes from a group'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "kernel-parameters" -d 'Remove kernel parameters from nodes (parameter values are ignored — match is by name)'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "boot-parameters" -d 'Delete boot parameters for nodes'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "configurations" -d 'Delete configurations and all associated data'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "session" -d 'Delete a configuration session'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "images" -d '[experimental] Delete IMS images by ID (refuses to delete images currently booting a node)'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "hardware" -d '[experimental] Remove hardware components from a group'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "redfish-endpoints" -d 'Delete a Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "redfish-endpoint" -d 'Delete a Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand delete; and not __fish_seen_subcommand_from group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from group" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from group" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from group" -s f -l force -d 'Force deletion'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from group" -s d -l dry-run -d 'Validate input and print the request that would be sent to the backend without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from group" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from node" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from node" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from node" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from node" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from nodes" -s g -l group -d 'Group to remove the nodes from' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from nodes" -s n -l nodes -d 'Xnames, NIDs, or a hostlist expression. eg: \'x1003c1s7b0n0,x1003c1s7b0n1\', \'nid001313,nid001314\', \'x1003c1s7b0n[0-1],x1003c1s7b1n0\', \'nid00131[0-9]\'' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from nodes" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from nodes" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from nodes" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from nodes" -s y -l assume-yes -d 'Skip confirmation prompts'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from nodes" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from kernel-parameters" -s n -l nodes -d 'Xnames, NIDs, or a hostlist expression. eg: \'x1003c1s7b0n0,x1003c1s7b0n1\', \'nid001313,nid001314\', \'x1003c1s7b0n[0-1],x1003c1s7b1n0\', \'nid00131[0-9]\'' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from kernel-parameters" -s H -l group -l hsm-group -d 'Remove the listed kernel parameters from every node in this group' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from kernel-parameters" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from kernel-parameters" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from kernel-parameters" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from kernel-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from boot-parameters" -s H -l hosts -d 'Xnames of the nodes' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from boot-parameters" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from boot-parameters" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from boot-parameters" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from boot-parameters" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s n -l configuration-name -d 'Glob pattern to filter by name. eg: my-config*, my-config-v[1,2]' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s s -l since -d 'Delete configurations last updated after this date (format: %Y-%m-%d)' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s u -l until -d 'Delete configurations last updated before this date (format: %Y-%m-%d)' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from configurations" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from session" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from session" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from session" -s y -l assume-yes -d 'Skip confirmation prompts'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from session" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from session" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from images" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from images" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from images" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from images" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hardware" -s P -l pattern -d 'Hardware component pattern' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hardware" -s t -l target-group -l target-cluster -d 'Group to remove components from' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hardware" -s p -l parent-group -l parent-cluster -d 'Group that receives the freed components' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hardware" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hardware" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hardware" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hardware" -s D -l delete-group -l delete-hsm-group -d 'Delete the group if empty after this operation'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from hardware" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from redfish-endpoints" -s i -l id -d 'Xname of the Redfish endpoint to delete' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from redfish-endpoints" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from redfish-endpoints" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from redfish-endpoints" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from redfish-endpoints" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from redfish-endpoint" -s i -l id -d 'Xname of the Redfish endpoint to delete' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from redfish-endpoint" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from redfish-endpoint" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from redfish-endpoint" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from redfish-endpoint" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "group" -d 'Delete a node group'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "node" -d 'Remove a node from the hardware state manager'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "nodes" -d 'Remove nodes from a group'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "kernel-parameters" -d 'Remove kernel parameters from nodes (parameter values are ignored — match is by name)'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "boot-parameters" -d 'Delete boot parameters for nodes'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "configurations" -d 'Delete configurations and all associated data'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "session" -d 'Delete a configuration session'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "images" -d '[experimental] Delete IMS images by ID (refuses to delete images currently booting a node)'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "hardware" -d '[experimental] Remove hardware components from a group'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "redfish-endpoints" -d 'Delete a Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand delete; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand migrate; and not __fish_seen_subcommand_from nodes help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand migrate; and not __fish_seen_subcommand_from nodes help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand migrate; and not __fish_seen_subcommand_from nodes help" -f -a "nodes" -d 'Move nodes between clusters'
complete -c manta -n "__fish_manta_using_subcommand migrate; and not __fish_seen_subcommand_from nodes help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from nodes" -s f -l from -d 'Source cluster to move nodes from' -r
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from nodes" -s t -l to -d 'Destination cluster to move nodes to' -r
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from nodes" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from nodes" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from nodes" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from nodes" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from help" -f -a "nodes" -d 'Move nodes between clusters'
complete -c manta -n "__fish_manta_using_subcommand migrate; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand backup; and not __fish_seen_subcommand_from vcluster help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand backup; and not __fish_seen_subcommand_from vcluster help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand backup; and not __fish_seen_subcommand_from vcluster help" -f -a "vcluster" -d 'Back up a virtual cluster (images, boot settings, group membership)'
complete -c manta -n "__fish_manta_using_subcommand backup; and not __fish_seen_subcommand_from vcluster help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand backup; and __fish_seen_subcommand_from vcluster" -s b -l bos -d 'Session template to derive the backup from' -r
complete -c manta -n "__fish_manta_using_subcommand backup; and __fish_seen_subcommand_from vcluster" -s d -l destination -d 'Destination directory for the backup files' -r -f -a "(__fish_complete_directories)"
complete -c manta -n "__fish_manta_using_subcommand backup; and __fish_seen_subcommand_from vcluster" -s p -l pre-hook -d 'Command to run before the backup. eg: --pre-hook "echo hello"' -r
complete -c manta -n "__fish_manta_using_subcommand backup; and __fish_seen_subcommand_from vcluster" -s a -l post-hook -d 'Command to run after a successful backup. eg: --post-hook "echo hello"' -r
complete -c manta -n "__fish_manta_using_subcommand backup; and __fish_seen_subcommand_from vcluster" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand backup; and __fish_seen_subcommand_from vcluster" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand backup; and __fish_seen_subcommand_from vcluster" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand backup; and __fish_seen_subcommand_from help" -f -a "vcluster" -d 'Back up a virtual cluster (images, boot settings, group membership)'
complete -c manta -n "__fish_manta_using_subcommand backup; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand restore; and not __fish_seen_subcommand_from vcluster help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand restore; and not __fish_seen_subcommand_from vcluster help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand restore; and not __fish_seen_subcommand_from vcluster help" -f -a "vcluster" -d 'Restore a virtual cluster from a backup bundle'
complete -c manta -n "__fish_manta_using_subcommand restore; and not __fish_seen_subcommand_from vcluster help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -s b -l bos-file -d 'Session template backup file' -r -F
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -s c -l cfs-file -d 'Configuration backup file' -r -F
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -s j -l hsm-file -d 'Group description backup file' -r -F
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -s m -l ims-file -d 'Image metadata backup file' -r -F
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -s i -l image-dir -d 'Directory containing the image files' -r -f -a "(__fish_complete_directories)"
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -s p -l pre-hook -d 'Command to run before the restore. eg: --pre-hook "echo hello"' -r
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -s a -l post-hook -d 'Command to run after a successful restore. eg: --post-hook "echo hello"' -r
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -s o -l overwrite -d 'Overwrite existing data'
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from vcluster" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from help" -f -a "vcluster" -d 'Restore a virtual cluster from a backup bundle'
complete -c manta -n "__fish_manta_using_subcommand restore; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand run; and not __fish_seen_subcommand_from session help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand run; and not __fish_seen_subcommand_from session help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand run; and not __fish_seen_subcommand_from session help" -f -a "session" -d 'Create and run a configuration session from a local repo'
complete -c manta -n "__fish_manta_using_subcommand run; and not __fish_seen_subcommand_from session help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s n -l name -d 'Session name' -r
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s p -l playbook-name -d 'Ansible playbook filename' -r
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s r -l repo-path -d 'Path to the local git repo containing the Ansible playbook' -r -f -a "(__fish_complete_directories)"
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s v -l ansible-verbosity -d 'Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)' -r -f -a "0\t''
1\t''
2\t''
3\t''
4\t''"
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s P -l ansible-passthrough -d 'Additional Ansible flags (limited to --extra-vars, --forks, --skip-tags, --start-at-task, --tags)' -r
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s l -l ansible-limit -d 'Limit the session to specific nodes (must be a subset of --group if both are provided)' -r
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s H -l group -l hsm-group -d 'Run the session against every node in this group' -r
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s w -l watch-logs -d 'Stream session logs to stdout'
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s t -l timestamps -d 'Show log timestamps'
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s d -l dry-run -d 'Simulate the operation without making changes'
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from session" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from help" -f -a "session" -d 'Create and run a configuration session from a local repo'
complete -c manta -n "__fish_manta_using_subcommand run; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -f -a "on" -d 'Power on nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -f -a "off" -d 'Power off nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -f -a "reset" -d 'Reset (reboot) nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand power; and not __fish_seen_subcommand_from on off reset help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from on" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from on" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from on" -f -a "group" -d 'Power on all nodes in a group'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from on" -f -a "nodes" -d 'Power on a set of nodes'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from on" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from off" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from off" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from off" -f -a "group" -d 'Power off all nodes in a group'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from off" -f -a "nodes" -d 'Power off a set of nodes'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from off" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from reset" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from reset" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from reset" -f -a "group" -d 'Reset all nodes in a group'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from reset" -f -a "nodes" -d 'Reset a set of nodes'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from reset" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from help" -f -a "on" -d 'Power on nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from help" -f -a "off" -d 'Power off nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from help" -f -a "reset" -d 'Reset (reboot) nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand power; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand log" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand log" -s t -l timestamps -d 'Show log timestamps'
complete -c manta -n "__fish_manta_using_subcommand log" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand console; and not __fish_seen_subcommand_from node target-ansible help" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand console; and not __fish_seen_subcommand_from node target-ansible help" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand console; and not __fish_seen_subcommand_from node target-ansible help" -f -a "node" -d 'Connect to a node\'s serial console'
complete -c manta -n "__fish_manta_using_subcommand console; and not __fish_seen_subcommand_from node target-ansible help" -f -a "target-ansible" -d 'Connect to the Ansible target container of a configuration session'
complete -c manta -n "__fish_manta_using_subcommand console; and not __fish_seen_subcommand_from node target-ansible help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from node" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from node" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from target-ansible" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from target-ansible" -s h -l help -d 'Print help'
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from help" -f -a "node" -d 'Connect to a node\'s serial console'
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from help" -f -a "target-ansible" -d 'Connect to the Ansible target container of a configuration session'
complete -c manta -n "__fish_manta_using_subcommand console; and __fish_seen_subcommand_from help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand gen-autocomplete" -s s -l shell -d 'Shell type (guessed from $SHELL if omitted)' -r -f -a "bash\t''
zsh\t''
fish\t''"
complete -c manta -n "__fish_manta_using_subcommand gen-autocomplete" -s p -l path -d 'Override the default install directory' -r -f -a "(__fish_complete_directories)"
complete -c manta -n "__fish_manta_using_subcommand gen-autocomplete" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand gen-autocomplete" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand gen-autocomplete" -l print -d 'Emit the script to stdout instead of installing it'
complete -c manta -n "__fish_manta_using_subcommand gen-autocomplete" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand gen-man" -s p -l path -d 'Directory to write `manta.1` into (defaults to $XDG_DATA_HOME/man/man1)' -r -f -a "(__fish_complete_directories)"
complete -c manta -n "__fish_manta_using_subcommand gen-man" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand gen-man" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand gen-man" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand upgrade" -s o -l output -d 'Output format' -r -f -a "table\t''
json\t''"
complete -c manta -n "__fish_manta_using_subcommand upgrade" -l site -d 'Override the active site for this invocation' -r
complete -c manta -n "__fish_manta_using_subcommand upgrade" -s c -l check -d 'Check for a newer version and print it, but don\'t apply'
complete -c manta -n "__fish_manta_using_subcommand upgrade" -s d -l dry-run -d 'Show what would happen without downloading or replacing'
complete -c manta -n "__fish_manta_using_subcommand upgrade" -s y -l assume-yes -d 'Skip the confirmation prompt before replacing the binary'
complete -c manta -n "__fish_manta_using_subcommand upgrade" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "config" -d 'Show or change CLI-side settings (active site, default node group, log level)'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "get" -d 'Inspect groups, nodes, hardware, images, configurations, sessions, templates, and boot/kernel parameters'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "add" -d 'Register new nodes, groups, boot/kernel parameters, hardware components, or Redfish endpoints'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "apply" -d 'Roll out configurations, images, session templates, boot/kernel parameters, and hardware rescaling'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "delete" -d 'Remove nodes, groups, images, configurations, sessions, boot/kernel parameters, or Redfish endpoints'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "migrate" -d 'Move nodes between groups'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "backup" -d 'Back up a virtual cluster (images, boot settings, group membership) to disk'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "restore" -d 'Restore a virtual cluster from a backup bundle'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "run" -d 'Create and run a configuration session from a local Ansible repo'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "power" -d 'Power nodes on, off, or reset (reboot); waits for the transition unless --no-wait is set'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "log" -d 'Stream configuration session logs to stdout (accepts session, node, group, or NID)'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "console" -d 'Attach to a node\'s serial console, or to a configuration session\'s Ansible container'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "gen-autocomplete" -d 'Generate and install shell completion scripts'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "gen-man" -d 'Generate and install the manta man page'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "upgrade" -d 'Replace this `manta` binary with the latest release'
complete -c manta -n "__fish_manta_using_subcommand help; and not __fish_seen_subcommand_from config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "show" -d 'Show current configuration values'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "set" -d 'Set a configuration value'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from config" -f -a "unset" -d 'Clear a configuration value'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "groups" -d 'List node groups visible to your token (or look up one by name)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "hardware" -d 'Inspect hardware components'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "sessions" -d 'List configuration sessions'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "configurations" -d 'List CFS configurations (filter by name, glob, group, or recency)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "templates" -d 'List BOS session templates (filter by name, group, or recency)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "group-nodes" -d 'Show node details and status for a group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "nodes" -d 'Show node details and status'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "images" -d 'List IMS images (filter by id, name glob, or recency; sorted most-recent first)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "boot-parameters" -d 'Show the BSS boot parameters (kernel, initrd, params) for nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "kernel-parameters" -d 'Show kernel parameters for nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from get" -f -a "redfish-endpoints" -d 'List the BMCs / controllers the hardware state manager has registered as Redfish endpoints'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "node" -d 'Register a new node in the hardware state manager'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "nodes" -d 'Add existing nodes to a group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "group" -d 'Create a node group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "hardware" -d '[experimental] Add hardware components to a group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "boot-parameters" -d 'Create a BSS boot-parameters entry (kernel, initrd, params, cloud-init) for one or more nodes'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "kernel-parameters" -d 'Append kernel parameters to nodes (leaves existing parameters untouched unless --overwrite is set)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from add" -f -a "redfish-endpoints" -d 'Register a new Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "hardware" -d '[experimental] Rescale a group\'s hardware allocation'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "sat-file" -d 'Process a SAT file to create configurations, images, and session templates'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "boot" -d 'Update boot parameters and runtime configuration'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "boot-parameters" -d 'Update boot parameters for nodes'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "redfish-endpoints" -d 'Update an existing Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "kernel-parameters" -d 'Replace the full kernel-parameters string on nodes (drops any existing parameters not listed)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "ephemeral-environment" -d 'Launch an ephemeral SSH environment from an image'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from apply" -f -a "template" -d 'Boot nodes using an existing session template'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "group" -d 'Delete a node group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "node" -d 'Remove a node from the hardware state manager'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "nodes" -d 'Remove nodes from a group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "kernel-parameters" -d 'Remove kernel parameters from nodes (parameter values are ignored — match is by name)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "boot-parameters" -d 'Delete boot parameters for nodes'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "configurations" -d 'Delete configurations and all associated data'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "session" -d 'Delete a configuration session'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "images" -d '[experimental] Delete IMS images by ID (refuses to delete images currently booting a node)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "hardware" -d '[experimental] Remove hardware components from a group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from delete" -f -a "redfish-endpoints" -d 'Delete a Redfish endpoint'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from migrate" -f -a "nodes" -d 'Move nodes between clusters'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from backup" -f -a "vcluster" -d 'Back up a virtual cluster (images, boot settings, group membership)'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from restore" -f -a "vcluster" -d 'Restore a virtual cluster from a backup bundle'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from run" -f -a "session" -d 'Create and run a configuration session from a local repo'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from power" -f -a "on" -d 'Power on nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from power" -f -a "off" -d 'Power off nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from power" -f -a "reset" -d 'Reset (reboot) nodes or a group'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from console" -f -a "node" -d 'Connect to a node\'s serial console'
complete -c manta -n "__fish_manta_using_subcommand help; and __fish_seen_subcommand_from console" -f -a "target-ansible" -d 'Connect to the Ansible target container of a configuration session'
