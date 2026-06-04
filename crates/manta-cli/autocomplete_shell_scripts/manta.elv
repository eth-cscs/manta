
use builtin;
use str;

set edit:completion:arg-completer[manta] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'manta'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'manta'= {
            cand --site 'Override the active site for this invocation'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand config 'Show or change CLI-side settings (active site, default node group, log level)'
            cand get 'Inspect groups, nodes, hardware, images, configurations, sessions, templates, and boot/kernel parameters'
            cand add 'Register new nodes, groups, boot/kernel parameters, hardware components, or Redfish endpoints'
            cand update 'Modify existing boot parameters or Redfish endpoints in place'
            cand apply 'Roll out configurations, images, session templates, boot/kernel parameters, and hardware rescaling'
            cand delete 'Remove nodes, groups, images, configurations, sessions, boot/kernel parameters, or Redfish endpoints'
            cand migrate 'Move nodes between groups (vCluster backup/restore have moved to ''manta backup''/''manta restore'')'
            cand backup 'Back up a virtual cluster (images, boot settings, group membership) to disk'
            cand restore 'Restore a virtual cluster from a backup bundle'
            cand run 'Create and run a configuration session from a local Ansible repo'
            cand power 'Power nodes on, off, or reset (reboot); waits for the transition unless --no-wait is set'
            cand log 'Stream configuration session logs to stdout (accepts session, node, group, or NID)'
            cand console 'Attach to a node''s serial console, or to a configuration session''s Ansible container'
            cand gen-autocomplete 'Generate shell completion scripts'
            cand upgrade 'Replace this `manta` binary with the latest release'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;config'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand show 'Show current configuration values'
            cand set 'Set a configuration value'
            cand unset 'Clear a configuration value'
            cand gen-autocomplete '[DEPRECATED] Use ''manta gen-autocomplete'' instead'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;config;show'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;set'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand hsm 'Set the active node group'
            cand parent-hsm 'Set the parent node group'
            cand site 'Set the active site'
            cand log 'Set the log verbosity level'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;config;set;hsm'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;set;parent-hsm'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;set;site'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;set;log'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;set;help'= {
            cand hsm 'Set the active node group'
            cand parent-hsm 'Set the parent node group'
            cand site 'Set the active site'
            cand log 'Set the log verbosity level'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;config;set;help;hsm'= {
        }
        &'manta;config;set;help;parent-hsm'= {
        }
        &'manta;config;set;help;site'= {
        }
        &'manta;config;set;help;log'= {
        }
        &'manta;config;set;help;help'= {
        }
        &'manta;config;unset'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand hsm 'Clear the active node group'
            cand parent-hsm 'Clear the parent node group'
            cand auth 'Clear the cached authentication token'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;config;unset;hsm'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;unset;parent-hsm'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;unset;auth'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;unset;help'= {
            cand hsm 'Clear the active node group'
            cand parent-hsm 'Clear the parent node group'
            cand auth 'Clear the cached authentication token'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;config;unset;help;hsm'= {
        }
        &'manta;config;unset;help;parent-hsm'= {
        }
        &'manta;config;unset;help;auth'= {
        }
        &'manta;config;unset;help;help'= {
        }
        &'manta;config;gen-autocomplete'= {
            cand -s 'Shell type (guessed from $SHELL if omitted)'
            cand --shell 'Shell type (guessed from $SHELL if omitted)'
            cand -p 'Directory to write the script (prints to stdout if omitted)'
            cand --path 'Directory to write the script (prints to stdout if omitted)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;help'= {
            cand show 'Show current configuration values'
            cand set 'Set a configuration value'
            cand unset 'Clear a configuration value'
            cand gen-autocomplete '[DEPRECATED] Use ''manta gen-autocomplete'' instead'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;config;help;show'= {
        }
        &'manta;config;help;set'= {
            cand hsm 'Set the active node group'
            cand parent-hsm 'Set the parent node group'
            cand site 'Set the active site'
            cand log 'Set the log verbosity level'
        }
        &'manta;config;help;set;hsm'= {
        }
        &'manta;config;help;set;parent-hsm'= {
        }
        &'manta;config;help;set;site'= {
        }
        &'manta;config;help;set;log'= {
        }
        &'manta;config;help;unset'= {
            cand hsm 'Clear the active node group'
            cand parent-hsm 'Clear the parent node group'
            cand auth 'Clear the cached authentication token'
        }
        &'manta;config;help;unset;hsm'= {
        }
        &'manta;config;help;unset;parent-hsm'= {
        }
        &'manta;config;help;unset;auth'= {
        }
        &'manta;config;help;gen-autocomplete'= {
        }
        &'manta;config;help;help'= {
        }
        &'manta;get'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand groups 'List node groups visible to your token (or look up one by name)'
            cand hardware 'Inspect hardware components'
            cand sessions 'List configuration sessions'
            cand configurations 'List CFS configurations (filter by name, glob, group, or recency)'
            cand templates 'List BOS session templates (filter by name, group, or recency)'
            cand cluster '[DEPRECATED] Use ''manta get group-nodes'' instead'
            cand group-nodes 'Show node details and status for a group'
            cand group-hardware 'Show hardware inventory for a group'
            cand nodes 'Show node details and status'
            cand images 'List IMS images (filter by id, name regex, or recency; sorted most-recent first)'
            cand boot-parameters 'Show the BSS boot parameters (kernel, initrd, params) for nodes or a group'
            cand kernel-parameters 'Show kernel parameters for nodes or a group'
            cand redfish-endpoints 'List the BMCs / controllers the hardware state manager has registered as Redfish endpoints'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;get;groups'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;hardware'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand cluster '[DEPRECATED] Use ''manta get group-hardware'' instead'
            cand nodes 'Show hardware inventory for a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;get;hardware;cluster'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;hardware;nodes'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;hardware;help'= {
            cand cluster '[DEPRECATED] Use ''manta get group-hardware'' instead'
            cand nodes 'Show hardware inventory for a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;get;hardware;help;cluster'= {
        }
        &'manta;get;hardware;help;nodes'= {
        }
        &'manta;get;hardware;help;help'= {
        }
        &'manta;get;sessions'= {
            cand -n 'Return only the session with this name'
            cand --name 'Return only the session with this name'
            cand -a 'Return only sessions older than this age (eg: ''1d'', ''6h'')'
            cand --min-age 'Return only sessions older than this age (eg: ''1d'', ''6h'')'
            cand -A 'Return only sessions younger than this age (eg: ''1d'', ''6h'')'
            cand --max-age 'Return only sessions younger than this age (eg: ''1d'', ''6h'')'
            cand -t 'Return only sessions of this type'
            cand --type 'Return only sessions of this type'
            cand -s 'Return only sessions with this status'
            cand --status 'Return only sessions with this status'
            cand -l 'Return only the <VALUE> most recent sessions'
            cand --limit 'Return only the <VALUE> most recent sessions'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -x 'Xnames, NIDs, or hostlist expression. Returns sessions targeting these nodes or their groups'
            cand --xnames 'Xnames, NIDs, or hostlist expression. Returns sessions targeting these nodes or their groups'
            cand -H 'Node group name. Returns sessions targeting this group or its members'
            cand --group 'Node group name. Returns sessions targeting this group or its members'
            cand --hsm-group 'Node group name. Returns sessions targeting this group or its members'
            cand -m 'Return only the most recent session (equivalent to --limit 1)'
            cand --most-recent 'Return only the most recent session (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;configurations'= {
            cand -n 'Show only the configuration with this exact name'
            cand --name 'Show only the configuration with this exact name'
            cand -p 'Glob pattern to filter by name (eg: ''my-cfg*'')'
            cand --pattern 'Glob pattern to filter by name (eg: ''my-cfg*'')'
            cand -l 'Return only the <VALUE> most recent configurations'
            cand --limit 'Return only the <VALUE> most recent configurations'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -H 'Show only configurations whose layers target this group'
            cand --group 'Show only configurations whose layers target this group'
            cand --hsm-group 'Show only configurations whose layers target this group'
            cand -m 'Return only the most recent (equivalent to --limit 1)'
            cand --most-recent 'Return only the most recent (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;templates'= {
            cand -n 'Show only the template with this exact name'
            cand --name 'Show only the template with this exact name'
            cand -l 'Return only the <VALUE> most recent templates'
            cand --limit 'Return only the <VALUE> most recent templates'
            cand -H 'Show only templates whose boot sets target this group'
            cand --group 'Show only templates whose boot sets target this group'
            cand --hsm-group 'Show only templates whose boot sets target this group'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -m 'Return only the most recent (equivalent to --limit 1)'
            cand --most-recent 'Return only the most recent (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;cluster'= {
            cand -s 'Filter nodes by status'
            cand --status 'Filter nodes by status'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -n 'Print NIDs on a single line'
            cand --nids-only-one-line 'Print NIDs on a single line'
            cand -x 'Print xnames on a single line'
            cand --xnames-only-one-line 'Print xnames on a single line'
            cand -T 'Show a group status summary: OK          — all nodes booted and configured OFF         — at least one node is OFF ON          — no nodes OFF, at least one is ON STANDBY     — at least one node''s heartbeat is lost UNCONFIGURED — all nodes READY but at least one is still being configured FAILED      — at least one node''s configuration failed'
            cand --summary-status 'Show a group status summary: OK          — all nodes booted and configured OFF         — at least one node is OFF ON          — no nodes OFF, at least one is ON STANDBY     — at least one node''s heartbeat is lost UNCONFIGURED — all nodes READY but at least one is still being configured FAILED      — at least one node''s configuration failed'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;group-nodes'= {
            cand -s 'Filter nodes by status'
            cand --status 'Filter nodes by status'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -n 'Print NIDs on a single line'
            cand --nids-only-one-line 'Print NIDs on a single line'
            cand -x 'Print xnames on a single line'
            cand --xnames-only-one-line 'Print xnames on a single line'
            cand -T 'Show a group status summary: OK          — all nodes booted and configured OFF         — at least one node is OFF ON          — no nodes OFF, at least one is ON STANDBY     — at least one node''s heartbeat is lost UNCONFIGURED — all nodes READY but at least one is still being configured FAILED      — at least one node''s configuration failed'
            cand --summary-status 'Show a group status summary: OK          — all nodes booted and configured OFF         — at least one node is OFF ON          — no nodes OFF, at least one is ON STANDBY     — at least one node''s heartbeat is lost UNCONFIGURED — all nodes READY but at least one is still being configured FAILED      — at least one node''s configuration failed'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;group-hardware'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;nodes'= {
            cand -s 'Filter nodes by status'
            cand --status 'Filter nodes by status'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -n 'Print NIDs on a single line'
            cand --nids-only-one-line 'Print NIDs on a single line'
            cand -T 'Show a node status summary: OK          — all nodes booted and configured OFF         — at least one node is OFF ON          — no nodes OFF, at least one is ON STANDBY     — at least one node''s heartbeat is lost UNCONFIGURED — all nodes READY but at least one is still being configured FAILED      — at least one node''s configuration failed'
            cand --summary-status 'Show a node status summary: OK          — all nodes booted and configured OFF         — at least one node is OFF ON          — no nodes OFF, at least one is ON STANDBY     — at least one node''s heartbeat is lost UNCONFIGURED — all nodes READY but at least one is still being configured FAILED      — at least one node''s configuration failed'
            cand -S 'Also show sibling nodes that share a power supply with the requested nodes'
            cand --include-siblings 'Also show sibling nodes that share a power supply with the requested nodes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;images'= {
            cand -i 'Show only the image with this exact ID'
            cand --id 'Show only the image with this exact ID'
            cand -p 'Regex matched against image name (applied client-side)'
            cand --pattern 'Regex matched against image name (applied client-side)'
            cand -l 'Return only the <VALUE> most recent images'
            cand --limit 'Return only the <VALUE> most recent images'
            cand -m 'Return only the most recent (equivalent to --limit 1)'
            cand --most-recent 'Return only the most recent (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;boot-parameters'= {
            cand -H 'Show boot parameters for every node in this group'
            cand --group 'Show boot parameters for every node in this group'
            cand --hsm-group 'Show boot parameters for every node in this group'
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;kernel-parameters'= {
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -H 'Show kernel parameters for all nodes in this group'
            cand --group 'Show kernel parameters for all nodes in this group'
            cand --hsm-group 'Show kernel parameters for all nodes in this group'
            cand -f 'Comma-separated list of parameter names to show. eg: ''console,bad_page,crashkernel,hugepagelist,root'''
            cand --filter 'Comma-separated list of parameter names to show. eg: ''console,bad_page,crashkernel,hugepagelist,root'''
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;redfish-endpoints'= {
            cand -i 'Filter by xname (can be specified multiple times)'
            cand --id 'Filter by xname (can be specified multiple times)'
            cand -f 'Filter by FQDN'
            cand --fqdn 'Filter by FQDN'
            cand -u 'Filter by UUID'
            cand --uuid 'Filter by UUID'
            cand -m 'Filter by MAC address'
            cand --macaddr 'Filter by MAC address'
            cand -I 'Filter by IP address (empty string matches endpoints without an IP)'
            cand --ipaddress 'Filter by IP address (empty string matches endpoints without an IP)'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;help'= {
            cand groups 'List node groups visible to your token (or look up one by name)'
            cand hardware 'Inspect hardware components'
            cand sessions 'List configuration sessions'
            cand configurations 'List CFS configurations (filter by name, glob, group, or recency)'
            cand templates 'List BOS session templates (filter by name, group, or recency)'
            cand cluster '[DEPRECATED] Use ''manta get group-nodes'' instead'
            cand group-nodes 'Show node details and status for a group'
            cand group-hardware 'Show hardware inventory for a group'
            cand nodes 'Show node details and status'
            cand images 'List IMS images (filter by id, name regex, or recency; sorted most-recent first)'
            cand boot-parameters 'Show the BSS boot parameters (kernel, initrd, params) for nodes or a group'
            cand kernel-parameters 'Show kernel parameters for nodes or a group'
            cand redfish-endpoints 'List the BMCs / controllers the hardware state manager has registered as Redfish endpoints'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;get;help;groups'= {
        }
        &'manta;get;help;hardware'= {
            cand cluster '[DEPRECATED] Use ''manta get group-hardware'' instead'
            cand nodes 'Show hardware inventory for a set of nodes'
        }
        &'manta;get;help;hardware;cluster'= {
        }
        &'manta;get;help;hardware;nodes'= {
        }
        &'manta;get;help;sessions'= {
        }
        &'manta;get;help;configurations'= {
        }
        &'manta;get;help;templates'= {
        }
        &'manta;get;help;cluster'= {
        }
        &'manta;get;help;group-nodes'= {
        }
        &'manta;get;help;group-hardware'= {
        }
        &'manta;get;help;nodes'= {
        }
        &'manta;get;help;images'= {
        }
        &'manta;get;help;boot-parameters'= {
        }
        &'manta;get;help;kernel-parameters'= {
        }
        &'manta;get;help;redfish-endpoints'= {
        }
        &'manta;get;help;help'= {
        }
        &'manta;add'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand node 'Register a new node in the hardware state manager'
            cand nodes 'Add existing nodes to a group'
            cand group 'Create a node group'
            cand hardware '[experimental] Add hardware components to a group'
            cand boot-parameters 'Create a BSS boot-parameters entry (kernel, initrd, params, cloud-init) for one or more nodes'
            cand kernel-parameters 'Append kernel parameters to nodes (leaves existing parameters untouched unless --overwrite is set)'
            cand redfish-endpoints 'Register a new Redfish endpoint'
            cand redfish-endpoint 'Register a new Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;add;node'= {
            cand -i 'Xname to register'
            cand --id 'Xname to register'
            cand -g 'Group to put the new node into'
            cand --group 'Group to put the new node into'
            cand -H 'File containing hardware information'
            cand --hardware 'File containing hardware information'
            cand -a 'Node architecture'
            cand --arch 'Node architecture'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Register the node as disabled'
            cand --disabled 'Register the node as disabled'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;add;nodes'= {
            cand -g 'Group to add the nodes to'
            cand --group 'Group to add the nodes to'
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;add;group'= {
            cand -l 'Group name'
            cand --label 'Group name'
            cand -d 'Group description'
            cand --description 'Group description'
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;add;hardware'= {
            cand -P 'Hardware component pattern'
            cand --pattern 'Hardware component pattern'
            cand -t 'Group to add components to'
            cand --target-group 'Group to add components to'
            cand --target-cluster 'Group to add components to'
            cand -p 'Group that donates the components'
            cand --parent-group 'Group that donates the components'
            cand --parent-cluster 'Group that donates the components'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -c 'Create the target group if it does not exist'
            cand --create-group 'Create the target group if it does not exist'
            cand --create-hsm-group 'Create the target group if it does not exist'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;add;boot-parameters'= {
            cand -H 'Xnames of the nodes'
            cand --hosts 'Xnames of the nodes'
            cand -n 'Comma-separated NIDs of the nodes'
            cand --nids 'Comma-separated NIDs of the nodes'
            cand -m 'Comma-separated MAC addresses of the nodes'
            cand --macs 'Comma-separated MAC addresses of the nodes'
            cand -p 'Kernel parameters'
            cand --params 'Kernel parameters'
            cand -k 'S3 path to the kernel file'
            cand --kernel 'S3 path to the kernel file'
            cand -i 'S3 path to the initrd file'
            cand --initrd 'S3 path to the initrd file'
            cand -c 'Cloud-init script'
            cand --cloud-init 'Cloud-init script'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;add;kernel-parameters'= {
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -H 'Append kernel parameters to every node in this group'
            cand --group 'Append kernel parameters to every node in this group'
            cand --hsm-group 'Append kernel parameters to every node in this group'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -O 'Overwrite the value if the parameter already exists'
            cand --overwrite 'Overwrite the value if the parameter already exists'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --do-not-reboot 'Do not reboot nodes after applying changes'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;add;redfish-endpoints'= {
            cand -i 'Xname of the BMC or controller'
            cand --id 'Xname of the BMC or controller'
            cand -n 'Arbitrary user-provided name for the endpoint'
            cand --name 'Arbitrary user-provided name for the endpoint'
            cand -H 'Hostname (FQDN host portion); normally identical to the xname'
            cand --hostname 'Hostname (FQDN host portion); normally identical to the xname'
            cand -d 'Domain (FQDN domain portion)'
            cand --domain 'Domain (FQDN domain portion)'
            cand -f 'Fully-qualified domain name on the management network (derived from hostname + domain)'
            cand --fqdn 'Fully-qualified domain name on the management network (derived from hostname + domain)'
            cand -u 'Username for endpoint authentication'
            cand --user 'Username for endpoint authentication'
            cand -p 'Password for endpoint authentication'
            cand --password 'Password for endpoint authentication'
            cand -M 'MAC address of the Redfish endpoint on the management network'
            cand --macaddr 'MAC address of the Redfish endpoint on the management network'
            cand -I 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)'
            cand --ipaddress 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)'
            cand -t 'Discovery template ID'
            cand --template-id 'Discovery template ID'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -e 'Enable the endpoint upon creation'
            cand --enabled 'Enable the endpoint upon creation'
            cand -U 'Use SSDP for discovery if the endpoint supports it'
            cand --use-ssdp 'Use SSDP for discovery if the endpoint supports it'
            cand -m 'Require a MAC address for geolocation'
            cand --mac-required 'Require a MAC address for geolocation'
            cand -r 'Trigger rediscovery when endpoint information is updated'
            cand --rediscover-on-update 'Trigger rediscovery when endpoint information is updated'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;add;redfish-endpoint'= {
            cand -i 'Xname of the BMC or controller'
            cand --id 'Xname of the BMC or controller'
            cand -n 'Arbitrary user-provided name for the endpoint'
            cand --name 'Arbitrary user-provided name for the endpoint'
            cand -H 'Hostname (FQDN host portion); normally identical to the xname'
            cand --hostname 'Hostname (FQDN host portion); normally identical to the xname'
            cand -d 'Domain (FQDN domain portion)'
            cand --domain 'Domain (FQDN domain portion)'
            cand -f 'Fully-qualified domain name on the management network (derived from hostname + domain)'
            cand --fqdn 'Fully-qualified domain name on the management network (derived from hostname + domain)'
            cand -u 'Username for endpoint authentication'
            cand --user 'Username for endpoint authentication'
            cand -p 'Password for endpoint authentication'
            cand --password 'Password for endpoint authentication'
            cand -M 'MAC address of the Redfish endpoint on the management network'
            cand --macaddr 'MAC address of the Redfish endpoint on the management network'
            cand -I 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)'
            cand --ipaddress 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)'
            cand -t 'Discovery template ID'
            cand --template-id 'Discovery template ID'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -e 'Enable the endpoint upon creation'
            cand --enabled 'Enable the endpoint upon creation'
            cand -U 'Use SSDP for discovery if the endpoint supports it'
            cand --use-ssdp 'Use SSDP for discovery if the endpoint supports it'
            cand -m 'Require a MAC address for geolocation'
            cand --mac-required 'Require a MAC address for geolocation'
            cand -r 'Trigger rediscovery when endpoint information is updated'
            cand --rediscover-on-update 'Trigger rediscovery when endpoint information is updated'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;add;help'= {
            cand node 'Register a new node in the hardware state manager'
            cand nodes 'Add existing nodes to a group'
            cand group 'Create a node group'
            cand hardware '[experimental] Add hardware components to a group'
            cand boot-parameters 'Create a BSS boot-parameters entry (kernel, initrd, params, cloud-init) for one or more nodes'
            cand kernel-parameters 'Append kernel parameters to nodes (leaves existing parameters untouched unless --overwrite is set)'
            cand redfish-endpoints 'Register a new Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;add;help;node'= {
        }
        &'manta;add;help;nodes'= {
        }
        &'manta;add;help;group'= {
        }
        &'manta;add;help;hardware'= {
        }
        &'manta;add;help;boot-parameters'= {
        }
        &'manta;add;help;kernel-parameters'= {
        }
        &'manta;add;help;redfish-endpoints'= {
        }
        &'manta;add;help;help'= {
        }
        &'manta;update'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand boot-parameters 'Update boot parameters for nodes'
            cand redfish-endpoints 'Update a Redfish endpoint'
            cand redfish-endpoint 'Update a Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;update;boot-parameters'= {
            cand -H 'Xnames of the nodes to update'
            cand --hosts 'Xnames of the nodes to update'
            cand -p 'Kernel parameters'
            cand --params 'Kernel parameters'
            cand -k 'S3 path to the kernel file'
            cand --kernel 'S3 path to the kernel file'
            cand -i 'S3 path to the initrd file'
            cand --initrd 'S3 path to the initrd file'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;update;redfish-endpoints'= {
            cand -i 'Xname of the endpoint to update'
            cand --id 'Xname of the endpoint to update'
            cand -n 'Arbitrary user-provided name for the endpoint'
            cand --name 'Arbitrary user-provided name for the endpoint'
            cand -H 'Hostname (FQDN host portion)'
            cand --hostname 'Hostname (FQDN host portion)'
            cand -d 'Domain (FQDN domain portion)'
            cand --domain 'Domain (FQDN domain portion)'
            cand -f 'Fully-qualified domain name on the management network'
            cand --fqdn 'Fully-qualified domain name on the management network'
            cand -u 'Username for endpoint authentication'
            cand --user 'Username for endpoint authentication'
            cand -p 'Password for endpoint authentication'
            cand --password 'Password for endpoint authentication'
            cand -M 'MAC address of the Redfish endpoint on the management network'
            cand --macaddr 'MAC address of the Redfish endpoint on the management network'
            cand -I 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)'
            cand --ipaddress 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)'
            cand -t 'Discovery template ID'
            cand --template-id 'Discovery template ID'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -e 'Enable the endpoint'
            cand --enabled 'Enable the endpoint'
            cand -U 'Use SSDP for discovery if the endpoint supports it'
            cand --use-ssdp 'Use SSDP for discovery if the endpoint supports it'
            cand -m 'Require a MAC address for geolocation'
            cand --mac-required 'Require a MAC address for geolocation'
            cand -r 'Trigger rediscovery when endpoint information is updated'
            cand --rediscover-on-update 'Trigger rediscovery when endpoint information is updated'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;update;redfish-endpoint'= {
            cand -i 'Xname of the endpoint to update'
            cand --id 'Xname of the endpoint to update'
            cand -n 'Arbitrary user-provided name for the endpoint'
            cand --name 'Arbitrary user-provided name for the endpoint'
            cand -H 'Hostname (FQDN host portion)'
            cand --hostname 'Hostname (FQDN host portion)'
            cand -d 'Domain (FQDN domain portion)'
            cand --domain 'Domain (FQDN domain portion)'
            cand -f 'Fully-qualified domain name on the management network'
            cand --fqdn 'Fully-qualified domain name on the management network'
            cand -u 'Username for endpoint authentication'
            cand --user 'Username for endpoint authentication'
            cand -p 'Password for endpoint authentication'
            cand --password 'Password for endpoint authentication'
            cand -M 'MAC address of the Redfish endpoint on the management network'
            cand --macaddr 'MAC address of the Redfish endpoint on the management network'
            cand -I 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)'
            cand --ipaddress 'IP address of the Redfish endpoint on the management network (IPv4 or IPv6)'
            cand -t 'Discovery template ID'
            cand --template-id 'Discovery template ID'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -e 'Enable the endpoint'
            cand --enabled 'Enable the endpoint'
            cand -U 'Use SSDP for discovery if the endpoint supports it'
            cand --use-ssdp 'Use SSDP for discovery if the endpoint supports it'
            cand -m 'Require a MAC address for geolocation'
            cand --mac-required 'Require a MAC address for geolocation'
            cand -r 'Trigger rediscovery when endpoint information is updated'
            cand --rediscover-on-update 'Trigger rediscovery when endpoint information is updated'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;update;help'= {
            cand boot-parameters 'Update boot parameters for nodes'
            cand redfish-endpoints 'Update a Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;update;help;boot-parameters'= {
        }
        &'manta;update;help;redfish-endpoints'= {
        }
        &'manta;update;help;help'= {
        }
        &'manta;apply'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand hardware '[experimental] Rescale a group''s hardware allocation'
            cand configuration 'Create a configuration (deprecated — use ''apply sat-file'')'
            cand sat-file 'Process a SAT file to create configurations, images, and session templates'
            cand boot 'Update boot parameters and runtime configuration'
            cand kernel-parameters 'Replace the full kernel-parameters string on nodes (drops any existing parameters not listed)'
            cand session '[DEPRECATED] Use ''manta run session'' instead'
            cand ephemeral-environment 'Launch an ephemeral SSH environment from an image'
            cand template 'Boot nodes using an existing session template'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;hardware'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand group '[experimental] Rescale a group''s hardware allocation'
            cand cluster '[DEPRECATED] Use ''manta apply hardware group'' instead'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;hardware;group'= {
            cand -P 'Hardware pattern: <component>:<qty>[:<component>:<qty>...]. eg: ''a100:12:epyc:5'''
            cand --pattern 'Hardware pattern: <component>:<qty>[:<component>:<qty>...]. eg: ''a100:12:epyc:5'''
            cand -t 'Group to rescale'
            cand --target-group 'Group to rescale'
            cand --target-cluster 'Group to rescale'
            cand -p 'Group that donates or receives the redistributed nodes'
            cand --parent-group 'Group that donates or receives the redistributed nodes'
            cand --parent-cluster 'Group that donates or receives the redistributed nodes'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -c 'Create the target group if it does not exist'
            cand --create-target-group 'Create the target group if it does not exist'
            cand --create-target-hsm-group 'Create the target group if it does not exist'
            cand -D 'Delete the parent group if empty after this operation'
            cand --delete-empty-parent-group 'Delete the parent group if empty after this operation'
            cand --delete-empty-parent-hsm-group 'Delete the parent group if empty after this operation'
            cand -u 'Allow any available nodes to be selected'
            cand --unpin-nodes 'Allow any available nodes to be selected'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;apply;hardware;cluster'= {
            cand -P 'Hardware pattern: <component>:<qty>[:<component>:<qty>...]. eg: ''a100:12:epyc:5'''
            cand --pattern 'Hardware pattern: <component>:<qty>[:<component>:<qty>...]. eg: ''a100:12:epyc:5'''
            cand -t 'Group to rescale'
            cand --target-group 'Group to rescale'
            cand --target-cluster 'Group to rescale'
            cand -p 'Group that donates or receives the redistributed nodes'
            cand --parent-group 'Group that donates or receives the redistributed nodes'
            cand --parent-cluster 'Group that donates or receives the redistributed nodes'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -c 'Create the target group if it does not exist'
            cand --create-target-group 'Create the target group if it does not exist'
            cand --create-target-hsm-group 'Create the target group if it does not exist'
            cand -D 'Delete the parent group if empty after this operation'
            cand --delete-empty-parent-group 'Delete the parent group if empty after this operation'
            cand --delete-empty-parent-hsm-group 'Delete the parent group if empty after this operation'
            cand -u 'Allow any available nodes to be selected'
            cand --unpin-nodes 'Allow any available nodes to be selected'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;hardware;help'= {
            cand group '[experimental] Rescale a group''s hardware allocation'
            cand cluster '[DEPRECATED] Use ''manta apply hardware group'' instead'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;hardware;help;group'= {
        }
        &'manta;apply;hardware;help;cluster'= {
        }
        &'manta;apply;hardware;help;help'= {
        }
        &'manta;apply;configuration'= {
            cand -t 'SAT file path'
            cand --sat-template-file 'SAT file path'
            cand -f 'Values file for SAT jinja2 templates'
            cand --values-file 'Values file for SAT jinja2 templates'
            cand -V 'Inline values for SAT jinja2 templates (overrides --values-file)'
            cand --values 'Inline values for SAT jinja2 templates (overrides --values-file)'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -H 'Node group name'
            cand --group 'Node group name'
            cand --hsm-group 'Node group name'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;sat-file'= {
            cand -t 'SAT file path (may be a jinja2 template)'
            cand --sat-template-file 'SAT file path (may be a jinja2 template)'
            cand -f 'Values file to expand jinja2 variables in the SAT file'
            cand --values-file 'Values file to expand jinja2 variables in the SAT file'
            cand -V 'Inline values to expand jinja2 variables (overrides --values-file)'
            cand --values 'Inline values to expand jinja2 variables (overrides --values-file)'
            cand -v 'Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)'
            cand --ansible-verbosity 'Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)'
            cand -P 'Additional Ansible flags (limited to --extra-vars, --forks, --skip-tags, --start-at-task, --tags)'
            cand --ansible-passthrough 'Additional Ansible flags (limited to --extra-vars, --forks, --skip-tags, --start-at-task, --tags)'
            cand -p 'Command to run before processing. eg: --pre-hook "echo hello"'
            cand --pre-hook 'Command to run before processing. eg: --pre-hook "echo hello"'
            cand -a 'Command to run after successful processing. eg: --post-hook "echo hello"'
            cand --post-hook 'Command to run after successful processing. eg: --post-hook "echo hello"'
            cand --output 'Output format'
            cand --reboot 'Reboot nodes after applying session templates'
            cand -o 'Overwrite an existing configuration with the same name'
            cand --overwrite-configuration 'Overwrite an existing configuration with the same name'
            cand -w 'Stream session logs to stdout'
            cand --watch-logs 'Stream session logs to stdout'
            cand -T 'Show log timestamps'
            cand --timestamps 'Show log timestamps'
            cand -i 'Process only the `configurations` and `images` sections'
            cand --image-only 'Process only the `configurations` and `images` sections'
            cand -s 'Process only the `configurations` and `session_templates` sections'
            cand --sessiontemplate-only 'Process only the `configurations` and `session_templates` sections'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;apply;boot'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand nodes 'Update boot parameters for a set of nodes'
            cand group 'Update boot parameters for all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta apply boot group'' instead'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;boot;nodes'= {
            cand -i 'Image ID to boot the nodes'
            cand --boot-image 'Image ID to boot the nodes'
            cand -b 'Configuration name used to build the boot image (uses the most recent matching image)'
            cand --boot-image-configuration 'Configuration name used to build the boot image (uses the most recent matching image)'
            cand -r 'Configuration to apply to nodes after booting'
            cand --runtime-configuration 'Configuration to apply to nodes after booting'
            cand -k 'Kernel parameters to assign to the nodes'
            cand --kernel-parameters 'Kernel parameters to assign to the nodes'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --do-not-reboot 'Suppress the automatic reboot after updating boot parameters'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;apply;boot;group'= {
            cand -i 'Image ID to boot the nodes'
            cand --boot-image 'Image ID to boot the nodes'
            cand -b 'Configuration name used to build the boot image (uses the most recent matching image)'
            cand --boot-image-configuration 'Configuration name used to build the boot image (uses the most recent matching image)'
            cand -r 'Configuration to apply to nodes after booting'
            cand --runtime-configuration 'Configuration to apply to nodes after booting'
            cand -k 'Kernel parameters to assign to all group members'
            cand --kernel-parameters 'Kernel parameters to assign to all group members'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --do-not-reboot 'Suppress the automatic reboot after updating boot parameters'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;apply;boot;cluster'= {
            cand -i 'Image ID to boot the nodes'
            cand --boot-image 'Image ID to boot the nodes'
            cand -b 'Configuration name used to build the boot image (uses the most recent matching image)'
            cand --boot-image-configuration 'Configuration name used to build the boot image (uses the most recent matching image)'
            cand -r 'Configuration to apply to nodes after booting'
            cand --runtime-configuration 'Configuration to apply to nodes after booting'
            cand -k 'Kernel parameters to assign to all group members'
            cand --kernel-parameters 'Kernel parameters to assign to all group members'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --do-not-reboot 'Suppress the automatic reboot after updating boot parameters'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;apply;boot;help'= {
            cand nodes 'Update boot parameters for a set of nodes'
            cand group 'Update boot parameters for all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta apply boot group'' instead'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;boot;help;nodes'= {
        }
        &'manta;apply;boot;help;group'= {
        }
        &'manta;apply;boot;help;cluster'= {
        }
        &'manta;apply;boot;help;help'= {
        }
        &'manta;apply;kernel-parameters'= {
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -H 'Replace kernel parameters on every node in this group'
            cand --group 'Replace kernel parameters on every node in this group'
            cand --hsm-group 'Replace kernel parameters on every node in this group'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --do-not-reboot 'Do not reboot nodes after applying changes'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;session'= {
            cand -n 'Session name'
            cand --name 'Session name'
            cand -p 'Ansible playbook filename'
            cand --playbook-name 'Ansible playbook filename'
            cand -r 'Path to the local git repo containing the Ansible playbook'
            cand --repo-path 'Path to the local git repo containing the Ansible playbook'
            cand -v 'Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)'
            cand --ansible-verbosity 'Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)'
            cand -P 'Additional Ansible flags (limited to --extra-vars, --forks, --skip-tags, --start-at-task, --tags)'
            cand --ansible-passthrough 'Additional Ansible flags (limited to --extra-vars, --forks, --skip-tags, --start-at-task, --tags)'
            cand -l 'Limit the session to specific nodes (must be a subset of --group if both are provided)'
            cand --ansible-limit 'Limit the session to specific nodes (must be a subset of --group if both are provided)'
            cand -H 'Run the session against every node in this group'
            cand --group 'Run the session against every node in this group'
            cand --hsm-group 'Run the session against every node in this group'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -w 'Stream session logs to stdout'
            cand --watch-logs 'Stream session logs to stdout'
            cand -t 'Show log timestamps'
            cand --timestamps 'Show log timestamps'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;apply;ephemeral-environment'= {
            cand -i 'Image ID to use'
            cand --image-id 'Image ID to use'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;apply;template'= {
            cand -n 'Name of the boot session to create'
            cand --name 'Name of the boot session to create'
            cand -o 'Boot operation to perform'
            cand --operation 'Boot operation to perform'
            cand -t 'Session template name'
            cand --template 'Session template name'
            cand -l 'Limit to specific nodes, groups, or roles (OR by default; prefix with ''&'' for AND or ''!'' for NOT)'
            cand --limit 'Limit to specific nodes, groups, or roles (OR by default; prefix with ''&'' for AND or ''!'' for NOT)'
            cand -i 'Include nodes marked as disabled in the hardware state manager'
            cand --include-disabled 'Include nodes marked as disabled in the hardware state manager'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;help'= {
            cand hardware '[experimental] Rescale a group''s hardware allocation'
            cand configuration 'Create a configuration (deprecated — use ''apply sat-file'')'
            cand sat-file 'Process a SAT file to create configurations, images, and session templates'
            cand boot 'Update boot parameters and runtime configuration'
            cand kernel-parameters 'Replace the full kernel-parameters string on nodes (drops any existing parameters not listed)'
            cand session '[DEPRECATED] Use ''manta run session'' instead'
            cand ephemeral-environment 'Launch an ephemeral SSH environment from an image'
            cand template 'Boot nodes using an existing session template'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;help;hardware'= {
            cand group '[experimental] Rescale a group''s hardware allocation'
            cand cluster '[DEPRECATED] Use ''manta apply hardware group'' instead'
        }
        &'manta;apply;help;hardware;group'= {
        }
        &'manta;apply;help;hardware;cluster'= {
        }
        &'manta;apply;help;configuration'= {
        }
        &'manta;apply;help;sat-file'= {
        }
        &'manta;apply;help;boot'= {
            cand nodes 'Update boot parameters for a set of nodes'
            cand group 'Update boot parameters for all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta apply boot group'' instead'
        }
        &'manta;apply;help;boot;nodes'= {
        }
        &'manta;apply;help;boot;group'= {
        }
        &'manta;apply;help;boot;cluster'= {
        }
        &'manta;apply;help;kernel-parameters'= {
        }
        &'manta;apply;help;session'= {
        }
        &'manta;apply;help;ephemeral-environment'= {
        }
        &'manta;apply;help;template'= {
        }
        &'manta;apply;help;help'= {
        }
        &'manta;delete'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand group 'Delete a node group'
            cand node 'Remove a node from the hardware state manager'
            cand nodes 'Remove nodes from a group'
            cand kernel-parameters 'Remove kernel parameters from nodes (parameter values are ignored — match is by name)'
            cand boot-parameters 'Delete boot parameters for nodes'
            cand configurations 'Delete configurations and all associated data'
            cand session 'Delete a configuration session'
            cand images '[experimental] Delete IMS images by ID (refuses to delete images currently booting a node)'
            cand hardware '[experimental] Remove hardware components from a group'
            cand redfish-endpoints 'Delete a Redfish endpoint'
            cand redfish-endpoint 'Delete a Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;delete;group'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -f 'Force deletion'
            cand --force 'Force deletion'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;delete;node'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;delete;nodes'= {
            cand -g 'Group to remove the nodes from'
            cand --group 'Group to remove the nodes from'
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;delete;kernel-parameters'= {
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -H 'Remove the listed kernel parameters from every node in this group'
            cand --group 'Remove the listed kernel parameters from every node in this group'
            cand --hsm-group 'Remove the listed kernel parameters from every node in this group'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --do-not-reboot 'Do not reboot nodes after applying changes'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;delete;boot-parameters'= {
            cand -H 'Xnames of the nodes'
            cand --hosts 'Xnames of the nodes'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;delete;configurations'= {
            cand -n 'Glob pattern to filter by name. eg: my-config*, my-config-v[1,2]'
            cand --configuration-name 'Glob pattern to filter by name. eg: my-config*, my-config-v[1,2]'
            cand -s 'Delete configurations last updated after this date (format: %Y-%m-%d)'
            cand --since 'Delete configurations last updated after this date (format: %Y-%m-%d)'
            cand -u 'Delete configurations last updated before this date (format: %Y-%m-%d)'
            cand --until 'Delete configurations last updated before this date (format: %Y-%m-%d)'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;delete;session'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;delete;images'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;delete;hardware'= {
            cand -P 'Hardware component pattern'
            cand --pattern 'Hardware component pattern'
            cand -t 'Group to remove components from'
            cand --target-group 'Group to remove components from'
            cand --target-cluster 'Group to remove components from'
            cand -p 'Group that receives the freed components'
            cand --parent-group 'Group that receives the freed components'
            cand --parent-cluster 'Group that receives the freed components'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -D 'Delete the group if empty after this operation'
            cand --delete-group 'Delete the group if empty after this operation'
            cand --delete-hsm-group 'Delete the group if empty after this operation'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;delete;redfish-endpoints'= {
            cand -i 'Xname of the Redfish endpoint to delete'
            cand --id 'Xname of the Redfish endpoint to delete'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;delete;redfish-endpoint'= {
            cand -i 'Xname of the Redfish endpoint to delete'
            cand --id 'Xname of the Redfish endpoint to delete'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;delete;help'= {
            cand group 'Delete a node group'
            cand node 'Remove a node from the hardware state manager'
            cand nodes 'Remove nodes from a group'
            cand kernel-parameters 'Remove kernel parameters from nodes (parameter values are ignored — match is by name)'
            cand boot-parameters 'Delete boot parameters for nodes'
            cand configurations 'Delete configurations and all associated data'
            cand session 'Delete a configuration session'
            cand images '[experimental] Delete IMS images by ID (refuses to delete images currently booting a node)'
            cand hardware '[experimental] Remove hardware components from a group'
            cand redfish-endpoints 'Delete a Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;delete;help;group'= {
        }
        &'manta;delete;help;node'= {
        }
        &'manta;delete;help;nodes'= {
        }
        &'manta;delete;help;kernel-parameters'= {
        }
        &'manta;delete;help;boot-parameters'= {
        }
        &'manta;delete;help;configurations'= {
        }
        &'manta;delete;help;session'= {
        }
        &'manta;delete;help;images'= {
        }
        &'manta;delete;help;hardware'= {
        }
        &'manta;delete;help;redfish-endpoints'= {
        }
        &'manta;delete;help;help'= {
        }
        &'manta;migrate'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand vCluster '[experimental] Migrate a cluster'
            cand nodes 'Move nodes between clusters'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;migrate;vCluster'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand backup '[DEPRECATED] Use ''manta backup vcluster'' instead'
            cand restore '[DEPRECATED] Use ''manta restore vcluster'' instead. The old path keeps working for one release.'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;migrate;vCluster;backup'= {
            cand -b 'Session template to derive the backup from'
            cand --bos 'Session template to derive the backup from'
            cand -d 'Destination directory for the backup files'
            cand --destination 'Destination directory for the backup files'
            cand -p 'Command to run before the backup. eg: --pre-hook "echo hello"'
            cand --pre-hook 'Command to run before the backup. eg: --pre-hook "echo hello"'
            cand -a 'Command to run after a successful backup. eg: --post-hook "echo hello"'
            cand --post-hook 'Command to run after a successful backup. eg: --post-hook "echo hello"'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;migrate;vCluster;restore'= {
            cand -b 'Session template backup file'
            cand --bos-file 'Session template backup file'
            cand -c 'Configuration backup file'
            cand --cfs-file 'Configuration backup file'
            cand -j 'Group description backup file'
            cand --hsm-file 'Group description backup file'
            cand -m 'Image metadata backup file'
            cand --ims-file 'Image metadata backup file'
            cand -i 'Directory containing the image files'
            cand --image-dir 'Directory containing the image files'
            cand -p 'Command to run before the restore. eg: --pre-hook "echo hello"'
            cand --pre-hook 'Command to run before the restore. eg: --pre-hook "echo hello"'
            cand -a 'Command to run after a successful restore. eg: --post-hook "echo hello"'
            cand --post-hook 'Command to run after a successful restore. eg: --post-hook "echo hello"'
            cand --output 'Output format'
            cand -o 'Overwrite existing data'
            cand --overwrite 'Overwrite existing data'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;migrate;vCluster;help'= {
            cand backup '[DEPRECATED] Use ''manta backup vcluster'' instead'
            cand restore '[DEPRECATED] Use ''manta restore vcluster'' instead. The old path keeps working for one release.'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;migrate;vCluster;help;backup'= {
        }
        &'manta;migrate;vCluster;help;restore'= {
        }
        &'manta;migrate;vCluster;help;help'= {
        }
        &'manta;migrate;nodes'= {
            cand -f 'Source cluster to move nodes from'
            cand --from 'Source cluster to move nodes from'
            cand -t 'Destination cluster to move nodes to'
            cand --to 'Destination cluster to move nodes to'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;migrate;help'= {
            cand vCluster '[experimental] Migrate a cluster'
            cand nodes 'Move nodes between clusters'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;migrate;help;vCluster'= {
            cand backup '[DEPRECATED] Use ''manta backup vcluster'' instead'
            cand restore '[DEPRECATED] Use ''manta restore vcluster'' instead. The old path keeps working for one release.'
        }
        &'manta;migrate;help;vCluster;backup'= {
        }
        &'manta;migrate;help;vCluster;restore'= {
        }
        &'manta;migrate;help;nodes'= {
        }
        &'manta;migrate;help;help'= {
        }
        &'manta;backup'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand vcluster 'Back up a virtual cluster (images, boot settings, group membership)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;backup;vcluster'= {
            cand -b 'Session template to derive the backup from'
            cand --bos 'Session template to derive the backup from'
            cand -d 'Destination directory for the backup files'
            cand --destination 'Destination directory for the backup files'
            cand -p 'Command to run before the backup. eg: --pre-hook "echo hello"'
            cand --pre-hook 'Command to run before the backup. eg: --pre-hook "echo hello"'
            cand -a 'Command to run after a successful backup. eg: --post-hook "echo hello"'
            cand --post-hook 'Command to run after a successful backup. eg: --post-hook "echo hello"'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;backup;help'= {
            cand vcluster 'Back up a virtual cluster (images, boot settings, group membership)'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;backup;help;vcluster'= {
        }
        &'manta;backup;help;help'= {
        }
        &'manta;restore'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand vcluster 'Restore a virtual cluster from a backup bundle'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;restore;vcluster'= {
            cand -b 'Session template backup file'
            cand --bos-file 'Session template backup file'
            cand -c 'Configuration backup file'
            cand --cfs-file 'Configuration backup file'
            cand -j 'Group description backup file'
            cand --hsm-file 'Group description backup file'
            cand -m 'Image metadata backup file'
            cand --ims-file 'Image metadata backup file'
            cand -i 'Directory containing the image files'
            cand --image-dir 'Directory containing the image files'
            cand -p 'Command to run before the restore. eg: --pre-hook "echo hello"'
            cand --pre-hook 'Command to run before the restore. eg: --pre-hook "echo hello"'
            cand -a 'Command to run after a successful restore. eg: --post-hook "echo hello"'
            cand --post-hook 'Command to run after a successful restore. eg: --post-hook "echo hello"'
            cand --output 'Output format'
            cand -o 'Overwrite existing data'
            cand --overwrite 'Overwrite existing data'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;restore;help'= {
            cand vcluster 'Restore a virtual cluster from a backup bundle'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;restore;help;vcluster'= {
        }
        &'manta;restore;help;help'= {
        }
        &'manta;run'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand session 'Create and run a configuration session from a local repo'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;run;session'= {
            cand -n 'Session name'
            cand --name 'Session name'
            cand -p 'Ansible playbook filename'
            cand --playbook-name 'Ansible playbook filename'
            cand -r 'Path to the local git repo containing the Ansible playbook'
            cand --repo-path 'Path to the local git repo containing the Ansible playbook'
            cand -v 'Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)'
            cand --ansible-verbosity 'Ansible verbosity level (1 = -v, 2 = -vv, …, max 4)'
            cand -P 'Additional Ansible flags (limited to --extra-vars, --forks, --skip-tags, --start-at-task, --tags)'
            cand --ansible-passthrough 'Additional Ansible flags (limited to --extra-vars, --forks, --skip-tags, --start-at-task, --tags)'
            cand -l 'Limit the session to specific nodes (must be a subset of --group if both are provided)'
            cand --ansible-limit 'Limit the session to specific nodes (must be a subset of --group if both are provided)'
            cand -H 'Run the session against every node in this group'
            cand --group 'Run the session against every node in this group'
            cand --hsm-group 'Run the session against every node in this group'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -w 'Stream session logs to stdout'
            cand --watch-logs 'Stream session logs to stdout'
            cand -t 'Show log timestamps'
            cand --timestamps 'Show log timestamps'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;run;help'= {
            cand session 'Create and run a configuration session from a local repo'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;run;help;session'= {
        }
        &'manta;run;help;help'= {
        }
        &'manta;power'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand on 'Power on nodes or a group'
            cand off 'Power off nodes or a group'
            cand reset 'Reset (reboot) nodes or a group'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;on'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand group 'Power on all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power on group'' instead'
            cand nodes 'Power on a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;on;group'= {
            cand -R 'Reason for the power operation'
            cand --reason 'Reason for the power operation'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --no-wait 'Return as soon as the transition is queued; don''t poll for completion'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;on;cluster'= {
            cand -R 'Reason for the power operation'
            cand --reason 'Reason for the power operation'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --no-wait 'Return as soon as the transition is queued; don''t poll for completion'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;on;nodes'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --no-wait 'Return as soon as the transition is queued; don''t poll for completion'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;on;help'= {
            cand group 'Power on all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power on group'' instead'
            cand nodes 'Power on a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;on;help;group'= {
        }
        &'manta;power;on;help;cluster'= {
        }
        &'manta;power;on;help;nodes'= {
        }
        &'manta;power;on;help;help'= {
        }
        &'manta;power;off'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand group 'Power off all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power off group'' instead'
            cand nodes 'Power off a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;off;group'= {
            cand -R 'Reason for the power operation'
            cand --reason 'Reason for the power operation'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -g 'Perform a graceful shutdown'
            cand --graceful 'Perform a graceful shutdown'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --no-wait 'Return as soon as the transition is queued; don''t poll for completion'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;off;cluster'= {
            cand -R 'Reason for the power operation'
            cand --reason 'Reason for the power operation'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -g 'Perform a graceful shutdown'
            cand --graceful 'Perform a graceful shutdown'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --no-wait 'Return as soon as the transition is queued; don''t poll for completion'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;off;nodes'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -g 'Perform a graceful shutdown'
            cand --graceful 'Perform a graceful shutdown'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --no-wait 'Return as soon as the transition is queued; don''t poll for completion'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;off;help'= {
            cand group 'Power off all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power off group'' instead'
            cand nodes 'Power off a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;off;help;group'= {
        }
        &'manta;power;off;help;cluster'= {
        }
        &'manta;power;off;help;nodes'= {
        }
        &'manta;power;off;help;help'= {
        }
        &'manta;power;reset'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand group 'Reset all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power reset group'' instead'
            cand nodes 'Reset a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;reset;group'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -r 'Reason for the power operation'
            cand --reason 'Reason for the power operation'
            cand -g 'Perform a graceful reboot'
            cand --graceful 'Perform a graceful reboot'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --no-wait 'Return as soon as the transition is queued; don''t poll for completion'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;reset;cluster'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -r 'Reason for the power operation'
            cand --reason 'Reason for the power operation'
            cand -g 'Perform a graceful reboot'
            cand --graceful 'Perform a graceful reboot'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --no-wait 'Return as soon as the transition is queued; don''t poll for completion'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;reset;nodes'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -g 'Perform a graceful reboot'
            cand --graceful 'Perform a graceful reboot'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --no-wait 'Return as soon as the transition is queued; don''t poll for completion'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;reset;help'= {
            cand group 'Reset all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power reset group'' instead'
            cand nodes 'Reset a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;reset;help;group'= {
        }
        &'manta;power;reset;help;cluster'= {
        }
        &'manta;power;reset;help;nodes'= {
        }
        &'manta;power;reset;help;help'= {
        }
        &'manta;power;help'= {
            cand on 'Power on nodes or a group'
            cand off 'Power off nodes or a group'
            cand reset 'Reset (reboot) nodes or a group'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;help;on'= {
            cand group 'Power on all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power on group'' instead'
            cand nodes 'Power on a set of nodes'
        }
        &'manta;power;help;on;group'= {
        }
        &'manta;power;help;on;cluster'= {
        }
        &'manta;power;help;on;nodes'= {
        }
        &'manta;power;help;off'= {
            cand group 'Power off all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power off group'' instead'
            cand nodes 'Power off a set of nodes'
        }
        &'manta;power;help;off;group'= {
        }
        &'manta;power;help;off;cluster'= {
        }
        &'manta;power;help;off;nodes'= {
        }
        &'manta;power;help;reset'= {
            cand group 'Reset all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power reset group'' instead'
            cand nodes 'Reset a set of nodes'
        }
        &'manta;power;help;reset;group'= {
        }
        &'manta;power;help;reset;cluster'= {
        }
        &'manta;power;help;reset;nodes'= {
        }
        &'manta;power;help;help'= {
        }
        &'manta;log'= {
            cand -t 'Show log timestamps'
            cand --timestamps 'Show log timestamps'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;console'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand node 'Connect to a node''s serial console'
            cand target-ansible 'Connect to the Ansible target container of a configuration session'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;console;node'= {
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;console;target-ansible'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;console;help'= {
            cand node 'Connect to a node''s serial console'
            cand target-ansible 'Connect to the Ansible target container of a configuration session'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;console;help;node'= {
        }
        &'manta;console;help;target-ansible'= {
        }
        &'manta;console;help;help'= {
        }
        &'manta;gen-autocomplete'= {
            cand -s 'Shell type (guessed from $SHELL if omitted)'
            cand --shell 'Shell type (guessed from $SHELL if omitted)'
            cand -p 'Directory to write the script (prints to stdout if omitted)'
            cand --path 'Directory to write the script (prints to stdout if omitted)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;upgrade'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -c 'Check for a newer version and print it, but don''t apply'
            cand --check 'Check for a newer version and print it, but don''t apply'
            cand -d 'Show what would happen without downloading or replacing'
            cand --dry-run 'Show what would happen without downloading or replacing'
            cand -y 'Skip the confirmation prompt before replacing the binary'
            cand --assume-yes 'Skip the confirmation prompt before replacing the binary'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta;help'= {
            cand config 'Show or change CLI-side settings (active site, default node group, log level)'
            cand get 'Inspect groups, nodes, hardware, images, configurations, sessions, templates, and boot/kernel parameters'
            cand add 'Register new nodes, groups, boot/kernel parameters, hardware components, or Redfish endpoints'
            cand update 'Modify existing boot parameters or Redfish endpoints in place'
            cand apply 'Roll out configurations, images, session templates, boot/kernel parameters, and hardware rescaling'
            cand delete 'Remove nodes, groups, images, configurations, sessions, boot/kernel parameters, or Redfish endpoints'
            cand migrate 'Move nodes between groups (vCluster backup/restore have moved to ''manta backup''/''manta restore'')'
            cand backup 'Back up a virtual cluster (images, boot settings, group membership) to disk'
            cand restore 'Restore a virtual cluster from a backup bundle'
            cand run 'Create and run a configuration session from a local Ansible repo'
            cand power 'Power nodes on, off, or reset (reboot); waits for the transition unless --no-wait is set'
            cand log 'Stream configuration session logs to stdout (accepts session, node, group, or NID)'
            cand console 'Attach to a node''s serial console, or to a configuration session''s Ansible container'
            cand gen-autocomplete 'Generate shell completion scripts'
            cand upgrade 'Replace this `manta` binary with the latest release'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;help;config'= {
            cand show 'Show current configuration values'
            cand set 'Set a configuration value'
            cand unset 'Clear a configuration value'
            cand gen-autocomplete '[DEPRECATED] Use ''manta gen-autocomplete'' instead'
        }
        &'manta;help;config;show'= {
        }
        &'manta;help;config;set'= {
            cand hsm 'Set the active node group'
            cand parent-hsm 'Set the parent node group'
            cand site 'Set the active site'
            cand log 'Set the log verbosity level'
        }
        &'manta;help;config;set;hsm'= {
        }
        &'manta;help;config;set;parent-hsm'= {
        }
        &'manta;help;config;set;site'= {
        }
        &'manta;help;config;set;log'= {
        }
        &'manta;help;config;unset'= {
            cand hsm 'Clear the active node group'
            cand parent-hsm 'Clear the parent node group'
            cand auth 'Clear the cached authentication token'
        }
        &'manta;help;config;unset;hsm'= {
        }
        &'manta;help;config;unset;parent-hsm'= {
        }
        &'manta;help;config;unset;auth'= {
        }
        &'manta;help;config;gen-autocomplete'= {
        }
        &'manta;help;get'= {
            cand groups 'List node groups visible to your token (or look up one by name)'
            cand hardware 'Inspect hardware components'
            cand sessions 'List configuration sessions'
            cand configurations 'List CFS configurations (filter by name, glob, group, or recency)'
            cand templates 'List BOS session templates (filter by name, group, or recency)'
            cand cluster '[DEPRECATED] Use ''manta get group-nodes'' instead'
            cand group-nodes 'Show node details and status for a group'
            cand group-hardware 'Show hardware inventory for a group'
            cand nodes 'Show node details and status'
            cand images 'List IMS images (filter by id, name regex, or recency; sorted most-recent first)'
            cand boot-parameters 'Show the BSS boot parameters (kernel, initrd, params) for nodes or a group'
            cand kernel-parameters 'Show kernel parameters for nodes or a group'
            cand redfish-endpoints 'List the BMCs / controllers the hardware state manager has registered as Redfish endpoints'
        }
        &'manta;help;get;groups'= {
        }
        &'manta;help;get;hardware'= {
            cand cluster '[DEPRECATED] Use ''manta get group-hardware'' instead'
            cand nodes 'Show hardware inventory for a set of nodes'
        }
        &'manta;help;get;hardware;cluster'= {
        }
        &'manta;help;get;hardware;nodes'= {
        }
        &'manta;help;get;sessions'= {
        }
        &'manta;help;get;configurations'= {
        }
        &'manta;help;get;templates'= {
        }
        &'manta;help;get;cluster'= {
        }
        &'manta;help;get;group-nodes'= {
        }
        &'manta;help;get;group-hardware'= {
        }
        &'manta;help;get;nodes'= {
        }
        &'manta;help;get;images'= {
        }
        &'manta;help;get;boot-parameters'= {
        }
        &'manta;help;get;kernel-parameters'= {
        }
        &'manta;help;get;redfish-endpoints'= {
        }
        &'manta;help;add'= {
            cand node 'Register a new node in the hardware state manager'
            cand nodes 'Add existing nodes to a group'
            cand group 'Create a node group'
            cand hardware '[experimental] Add hardware components to a group'
            cand boot-parameters 'Create a BSS boot-parameters entry (kernel, initrd, params, cloud-init) for one or more nodes'
            cand kernel-parameters 'Append kernel parameters to nodes (leaves existing parameters untouched unless --overwrite is set)'
            cand redfish-endpoints 'Register a new Redfish endpoint'
        }
        &'manta;help;add;node'= {
        }
        &'manta;help;add;nodes'= {
        }
        &'manta;help;add;group'= {
        }
        &'manta;help;add;hardware'= {
        }
        &'manta;help;add;boot-parameters'= {
        }
        &'manta;help;add;kernel-parameters'= {
        }
        &'manta;help;add;redfish-endpoints'= {
        }
        &'manta;help;update'= {
            cand boot-parameters 'Update boot parameters for nodes'
            cand redfish-endpoints 'Update a Redfish endpoint'
        }
        &'manta;help;update;boot-parameters'= {
        }
        &'manta;help;update;redfish-endpoints'= {
        }
        &'manta;help;apply'= {
            cand hardware '[experimental] Rescale a group''s hardware allocation'
            cand configuration 'Create a configuration (deprecated — use ''apply sat-file'')'
            cand sat-file 'Process a SAT file to create configurations, images, and session templates'
            cand boot 'Update boot parameters and runtime configuration'
            cand kernel-parameters 'Replace the full kernel-parameters string on nodes (drops any existing parameters not listed)'
            cand session '[DEPRECATED] Use ''manta run session'' instead'
            cand ephemeral-environment 'Launch an ephemeral SSH environment from an image'
            cand template 'Boot nodes using an existing session template'
        }
        &'manta;help;apply;hardware'= {
            cand group '[experimental] Rescale a group''s hardware allocation'
            cand cluster '[DEPRECATED] Use ''manta apply hardware group'' instead'
        }
        &'manta;help;apply;hardware;group'= {
        }
        &'manta;help;apply;hardware;cluster'= {
        }
        &'manta;help;apply;configuration'= {
        }
        &'manta;help;apply;sat-file'= {
        }
        &'manta;help;apply;boot'= {
            cand nodes 'Update boot parameters for a set of nodes'
            cand group 'Update boot parameters for all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta apply boot group'' instead'
        }
        &'manta;help;apply;boot;nodes'= {
        }
        &'manta;help;apply;boot;group'= {
        }
        &'manta;help;apply;boot;cluster'= {
        }
        &'manta;help;apply;kernel-parameters'= {
        }
        &'manta;help;apply;session'= {
        }
        &'manta;help;apply;ephemeral-environment'= {
        }
        &'manta;help;apply;template'= {
        }
        &'manta;help;delete'= {
            cand group 'Delete a node group'
            cand node 'Remove a node from the hardware state manager'
            cand nodes 'Remove nodes from a group'
            cand kernel-parameters 'Remove kernel parameters from nodes (parameter values are ignored — match is by name)'
            cand boot-parameters 'Delete boot parameters for nodes'
            cand configurations 'Delete configurations and all associated data'
            cand session 'Delete a configuration session'
            cand images '[experimental] Delete IMS images by ID (refuses to delete images currently booting a node)'
            cand hardware '[experimental] Remove hardware components from a group'
            cand redfish-endpoints 'Delete a Redfish endpoint'
        }
        &'manta;help;delete;group'= {
        }
        &'manta;help;delete;node'= {
        }
        &'manta;help;delete;nodes'= {
        }
        &'manta;help;delete;kernel-parameters'= {
        }
        &'manta;help;delete;boot-parameters'= {
        }
        &'manta;help;delete;configurations'= {
        }
        &'manta;help;delete;session'= {
        }
        &'manta;help;delete;images'= {
        }
        &'manta;help;delete;hardware'= {
        }
        &'manta;help;delete;redfish-endpoints'= {
        }
        &'manta;help;migrate'= {
            cand vCluster '[experimental] Migrate a cluster'
            cand nodes 'Move nodes between clusters'
        }
        &'manta;help;migrate;vCluster'= {
            cand backup '[DEPRECATED] Use ''manta backup vcluster'' instead'
            cand restore '[DEPRECATED] Use ''manta restore vcluster'' instead. The old path keeps working for one release.'
        }
        &'manta;help;migrate;vCluster;backup'= {
        }
        &'manta;help;migrate;vCluster;restore'= {
        }
        &'manta;help;migrate;nodes'= {
        }
        &'manta;help;backup'= {
            cand vcluster 'Back up a virtual cluster (images, boot settings, group membership)'
        }
        &'manta;help;backup;vcluster'= {
        }
        &'manta;help;restore'= {
            cand vcluster 'Restore a virtual cluster from a backup bundle'
        }
        &'manta;help;restore;vcluster'= {
        }
        &'manta;help;run'= {
            cand session 'Create and run a configuration session from a local repo'
        }
        &'manta;help;run;session'= {
        }
        &'manta;help;power'= {
            cand on 'Power on nodes or a group'
            cand off 'Power off nodes or a group'
            cand reset 'Reset (reboot) nodes or a group'
        }
        &'manta;help;power;on'= {
            cand group 'Power on all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power on group'' instead'
            cand nodes 'Power on a set of nodes'
        }
        &'manta;help;power;on;group'= {
        }
        &'manta;help;power;on;cluster'= {
        }
        &'manta;help;power;on;nodes'= {
        }
        &'manta;help;power;off'= {
            cand group 'Power off all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power off group'' instead'
            cand nodes 'Power off a set of nodes'
        }
        &'manta;help;power;off;group'= {
        }
        &'manta;help;power;off;cluster'= {
        }
        &'manta;help;power;off;nodes'= {
        }
        &'manta;help;power;reset'= {
            cand group 'Reset all nodes in a group'
            cand cluster '[DEPRECATED] Use ''manta power reset group'' instead'
            cand nodes 'Reset a set of nodes'
        }
        &'manta;help;power;reset;group'= {
        }
        &'manta;help;power;reset;cluster'= {
        }
        &'manta;help;power;reset;nodes'= {
        }
        &'manta;help;log'= {
        }
        &'manta;help;console'= {
            cand node 'Connect to a node''s serial console'
            cand target-ansible 'Connect to the Ansible target container of a configuration session'
        }
        &'manta;help;console;node'= {
        }
        &'manta;help;console;target-ansible'= {
        }
        &'manta;help;gen-autocomplete'= {
        }
        &'manta;help;upgrade'= {
        }
        &'manta;help;help'= {
        }
    ]
    $completions[$command]
}
