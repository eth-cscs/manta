
use builtin;
use str;

set edit:completion:arg-completer[manta-cli] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'manta-cli'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'manta-cli'= {
            cand --site 'Override the active site for this invocation'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand config 'Manage manta CLI configuration'
            cand get 'Query system resources'
            cand add 'Create system resources'
            cand update 'Update system resources'
            cand apply 'Apply changes to the system'
            cand delete 'Delete system resources'
            cand migrate 'Move nodes or clusters between groups'
            cand power 'Manage node and cluster power state'
            cand log 'Stream configuration session logs'
            cand console 'Open an interactive console to a node or configuration session'
            cand add-nodes-to-groups 'Add nodes to one or more groups'
            cand remove-nodes-from-groups 'Remove nodes from one or more groups'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;config'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand show 'Show current configuration values'
            cand set 'Set a configuration value'
            cand unset 'Clear a configuration value'
            cand gen-autocomplete 'Generate shell completion scripts'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;config;show'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;config;set'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand hsm 'Set the active node group'
            cand parent-hsm 'Set the parent node group'
            cand site 'Set the active site'
            cand log 'Set the log verbosity level'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;config;set;hsm'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;config;set;parent-hsm'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;config;set;site'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;config;set;log'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;config;set;help'= {
            cand hsm 'Set the active node group'
            cand parent-hsm 'Set the parent node group'
            cand site 'Set the active site'
            cand log 'Set the log verbosity level'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;config;set;help;hsm'= {
        }
        &'manta-cli;config;set;help;parent-hsm'= {
        }
        &'manta-cli;config;set;help;site'= {
        }
        &'manta-cli;config;set;help;log'= {
        }
        &'manta-cli;config;set;help;help'= {
        }
        &'manta-cli;config;unset'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand hsm 'Clear the active node group'
            cand parent-hsm 'Clear the parent node group'
            cand auth 'Clear the cached authentication token'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;config;unset;hsm'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;config;unset;parent-hsm'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;config;unset;auth'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;config;unset;help'= {
            cand hsm 'Clear the active node group'
            cand parent-hsm 'Clear the parent node group'
            cand auth 'Clear the cached authentication token'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;config;unset;help;hsm'= {
        }
        &'manta-cli;config;unset;help;parent-hsm'= {
        }
        &'manta-cli;config;unset;help;auth'= {
        }
        &'manta-cli;config;unset;help;help'= {
        }
        &'manta-cli;config;gen-autocomplete'= {
            cand -s 'Shell type (guessed from $SHELL if omitted)'
            cand --shell 'Shell type (guessed from $SHELL if omitted)'
            cand -p 'Directory to write the script (prints to stdout if omitted)'
            cand --path 'Directory to write the script (prints to stdout if omitted)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;config;help'= {
            cand show 'Show current configuration values'
            cand set 'Set a configuration value'
            cand unset 'Clear a configuration value'
            cand gen-autocomplete 'Generate shell completion scripts'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;config;help;show'= {
        }
        &'manta-cli;config;help;set'= {
            cand hsm 'Set the active node group'
            cand parent-hsm 'Set the parent node group'
            cand site 'Set the active site'
            cand log 'Set the log verbosity level'
        }
        &'manta-cli;config;help;set;hsm'= {
        }
        &'manta-cli;config;help;set;parent-hsm'= {
        }
        &'manta-cli;config;help;set;site'= {
        }
        &'manta-cli;config;help;set;log'= {
        }
        &'manta-cli;config;help;unset'= {
            cand hsm 'Clear the active node group'
            cand parent-hsm 'Clear the parent node group'
            cand auth 'Clear the cached authentication token'
        }
        &'manta-cli;config;help;unset;hsm'= {
        }
        &'manta-cli;config;help;unset;parent-hsm'= {
        }
        &'manta-cli;config;help;unset;auth'= {
        }
        &'manta-cli;config;help;gen-autocomplete'= {
        }
        &'manta-cli;config;help;help'= {
        }
        &'manta-cli;get'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand groups 'List node groups'
            cand hardware 'Inspect hardware components'
            cand sessions 'List configuration sessions'
            cand configurations 'List configurations'
            cand templates 'List session templates'
            cand cluster 'Show cluster node details and status'
            cand nodes 'Show node details and status'
            cand images 'List images'
            cand boot-parameters 'Show boot parameters for nodes or a group'
            cand kernel-parameters 'Show kernel parameters for nodes or a group'
            cand redfish-endpoints 'List Redfish endpoints'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;get;groups'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;get;hardware'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand cluster 'Show hardware inventory for a cluster'
            cand nodes 'Show hardware inventory for a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;get;hardware;cluster'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;get;hardware;nodes'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;get;hardware;help'= {
            cand cluster 'Show hardware inventory for a cluster'
            cand nodes 'Show hardware inventory for a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;get;hardware;help;cluster'= {
        }
        &'manta-cli;get;hardware;help;nodes'= {
        }
        &'manta-cli;get;hardware;help;help'= {
        }
        &'manta-cli;get;sessions'= {
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
            cand --hsm-group 'Node group name. Returns sessions targeting this group or its members'
            cand -m 'Return only the most recent session (equivalent to --limit 1)'
            cand --most-recent 'Return only the most recent session (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;get;configurations'= {
            cand -n 'Configuration name'
            cand --name 'Configuration name'
            cand -p 'Glob pattern to filter by name'
            cand --pattern 'Glob pattern to filter by name'
            cand -l 'Return only the <VALUE> most recent configurations'
            cand --limit 'Return only the <VALUE> most recent configurations'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -H 'Node group name'
            cand --hsm-group 'Node group name'
            cand -m 'Return only the most recent (equivalent to --limit 1)'
            cand --most-recent 'Return only the most recent (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;get;templates'= {
            cand -n 'Template name'
            cand --name 'Template name'
            cand -l 'Return only the <VALUE> most recent templates'
            cand --limit 'Return only the <VALUE> most recent templates'
            cand -H 'Node group name'
            cand --hsm-group 'Node group name'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -m 'Return only the most recent (equivalent to --limit 1)'
            cand --most-recent 'Return only the most recent (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;get;cluster'= {
            cand -s 'Filter nodes by status'
            cand --status 'Filter nodes by status'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -n 'Print NIDs on a single line'
            cand --nids-only-one-line 'Print NIDs on a single line'
            cand -x 'Print xnames on a single line'
            cand --xnames-only-one-line 'Print xnames on a single line'
            cand -T 'Show a cluster status summary: OK          — all nodes booted and configured OFF         — at least one node is OFF ON          — no nodes OFF, at least one is ON STANDBY     — at least one node''s heartbeat is lost UNCONFIGURED — all nodes READY but at least one is still being configured FAILED      — at least one node''s configuration failed'
            cand --summary-status 'Show a cluster status summary: OK          — all nodes booted and configured OFF         — at least one node is OFF ON          — no nodes OFF, at least one is ON STANDBY     — at least one node''s heartbeat is lost UNCONFIGURED — all nodes READY but at least one is still being configured FAILED      — at least one node''s configuration failed'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;get;nodes'= {
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
        &'manta-cli;get;images'= {
            cand -i 'Image ID'
            cand --id 'Image ID'
            cand -l 'Return only the <VALUE> most recent images'
            cand --limit 'Return only the <VALUE> most recent images'
            cand -H 'Node group name'
            cand --hsm-group 'Node group name'
            cand -m 'Return only the most recent (equivalent to --limit 1)'
            cand --most-recent 'Return only the most recent (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;get;boot-parameters'= {
            cand -H 'Node group name'
            cand --hsm-group 'Node group name'
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;get;kernel-parameters'= {
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -H 'Show kernel parameters for all nodes in this group'
            cand --hsm-group 'Show kernel parameters for all nodes in this group'
            cand -f 'Comma-separated list of parameter names to show. eg: ''console,bad_page,crashkernel,hugepagelist,root'''
            cand --filter 'Comma-separated list of parameter names to show. eg: ''console,bad_page,crashkernel,hugepagelist,root'''
            cand -o 'Output format'
            cand --output 'Output format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;get;redfish-endpoints'= {
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
        &'manta-cli;get;help'= {
            cand groups 'List node groups'
            cand hardware 'Inspect hardware components'
            cand sessions 'List configuration sessions'
            cand configurations 'List configurations'
            cand templates 'List session templates'
            cand cluster 'Show cluster node details and status'
            cand nodes 'Show node details and status'
            cand images 'List images'
            cand boot-parameters 'Show boot parameters for nodes or a group'
            cand kernel-parameters 'Show kernel parameters for nodes or a group'
            cand redfish-endpoints 'List Redfish endpoints'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;get;help;groups'= {
        }
        &'manta-cli;get;help;hardware'= {
            cand cluster 'Show hardware inventory for a cluster'
            cand nodes 'Show hardware inventory for a set of nodes'
        }
        &'manta-cli;get;help;hardware;cluster'= {
        }
        &'manta-cli;get;help;hardware;nodes'= {
        }
        &'manta-cli;get;help;sessions'= {
        }
        &'manta-cli;get;help;configurations'= {
        }
        &'manta-cli;get;help;templates'= {
        }
        &'manta-cli;get;help;cluster'= {
        }
        &'manta-cli;get;help;nodes'= {
        }
        &'manta-cli;get;help;images'= {
        }
        &'manta-cli;get;help;boot-parameters'= {
        }
        &'manta-cli;get;help;kernel-parameters'= {
        }
        &'manta-cli;get;help;redfish-endpoints'= {
        }
        &'manta-cli;get;help;help'= {
        }
        &'manta-cli;add'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand node 'Register a new node'
            cand group 'Create a node group'
            cand hardware '[experimental] Add hardware components to a cluster'
            cand boot-parameters 'Create boot parameters for nodes'
            cand kernel-parameters 'Append kernel parameters to nodes'
            cand redfish-endpoint 'Register a new Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;add;node'= {
            cand -i 'Node xname'
            cand --id 'Node xname'
            cand -g 'Node group to add the node to'
            cand --group 'Node group to add the node to'
            cand -H 'File containing hardware information'
            cand --hardware 'File containing hardware information'
            cand -a 'Node architecture'
            cand --arch 'Node architecture'
            cand -d 'Register the node as disabled'
            cand --disabled 'Register the node as disabled'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;add;group'= {
            cand -l 'Group name'
            cand --label 'Group name'
            cand -d 'Group description'
            cand --description 'Group description'
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;add;hardware'= {
            cand -P 'Hardware component pattern'
            cand --pattern 'Hardware component pattern'
            cand -t 'Cluster to add components to'
            cand --target-cluster 'Cluster to add components to'
            cand -p 'Cluster that donates the components'
            cand --parent-cluster 'Cluster that donates the components'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -c 'Create the target cluster if it does not exist'
            cand --create-hsm-group 'Create the target cluster if it does not exist'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;add;boot-parameters'= {
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
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;add;kernel-parameters'= {
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -H 'Node group name'
            cand --hsm-group 'Node group name'
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
        &'manta-cli;add;redfish-endpoint'= {
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
        &'manta-cli;add;help'= {
            cand node 'Register a new node'
            cand group 'Create a node group'
            cand hardware '[experimental] Add hardware components to a cluster'
            cand boot-parameters 'Create boot parameters for nodes'
            cand kernel-parameters 'Append kernel parameters to nodes'
            cand redfish-endpoint 'Register a new Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;add;help;node'= {
        }
        &'manta-cli;add;help;group'= {
        }
        &'manta-cli;add;help;hardware'= {
        }
        &'manta-cli;add;help;boot-parameters'= {
        }
        &'manta-cli;add;help;kernel-parameters'= {
        }
        &'manta-cli;add;help;redfish-endpoint'= {
        }
        &'manta-cli;add;help;help'= {
        }
        &'manta-cli;update'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand boot-parameters 'Update boot parameters for nodes'
            cand redfish-endpoint 'Update a Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;update;boot-parameters'= {
            cand -H 'Xnames of the nodes to update'
            cand --hosts 'Xnames of the nodes to update'
            cand -p 'Kernel parameters'
            cand --params 'Kernel parameters'
            cand -k 'S3 path to the kernel file'
            cand --kernel 'S3 path to the kernel file'
            cand -i 'S3 path to the initrd file'
            cand --initrd 'S3 path to the initrd file'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;update;redfish-endpoint'= {
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
        &'manta-cli;update;help'= {
            cand boot-parameters 'Update boot parameters for nodes'
            cand redfish-endpoint 'Update a Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;update;help;boot-parameters'= {
        }
        &'manta-cli;update;help;redfish-endpoint'= {
        }
        &'manta-cli;update;help;help'= {
        }
        &'manta-cli;apply'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand hardware '[experimental] Rescale a cluster''s hardware allocation'
            cand configuration 'Create a configuration (deprecated — use ''apply sat-file'')'
            cand sat-file 'Process a SAT file to create configurations, images, and session templates'
            cand boot 'Update boot parameters and runtime configuration'
            cand kernel-parameters 'Replace kernel parameters on nodes'
            cand session 'Create and run a configuration session from a local repo'
            cand ephemeral-environment 'Launch an ephemeral SSH environment from an image'
            cand template 'Boot nodes using an existing session template'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;apply;hardware'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand cluster '[experimental] Rescale a cluster''s hardware allocation'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;apply;hardware;cluster'= {
            cand -P 'Hardware pattern: <component>:<qty>[:<component>:<qty>...]. eg: ''a100:12:epyc:5'''
            cand --pattern 'Hardware pattern: <component>:<qty>[:<component>:<qty>...]. eg: ''a100:12:epyc:5'''
            cand -t 'Cluster to rescale'
            cand --target-cluster 'Cluster to rescale'
            cand -p 'Cluster that donates or receives the redistributed nodes'
            cand --parent-cluster 'Cluster that donates or receives the redistributed nodes'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -c 'Create the target cluster if it does not exist'
            cand --create-target-hsm-group 'Create the target cluster if it does not exist'
            cand -D 'Delete the parent cluster if empty after this operation'
            cand --delete-empty-parent-hsm-group 'Delete the parent cluster if empty after this operation'
            cand -u 'Allow any available nodes to be selected'
            cand --unpin-nodes 'Allow any available nodes to be selected'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta-cli;apply;hardware;help'= {
            cand cluster '[experimental] Rescale a cluster''s hardware allocation'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;apply;hardware;help;cluster'= {
        }
        &'manta-cli;apply;hardware;help;help'= {
        }
        &'manta-cli;apply;configuration'= {
            cand -t 'SAT file path'
            cand --sat-template-file 'SAT file path'
            cand -f 'Values file for SAT jinja2 templates'
            cand --values-file 'Values file for SAT jinja2 templates'
            cand -V 'Inline values for SAT jinja2 templates (overrides --values-file)'
            cand --values 'Inline values for SAT jinja2 templates (overrides --values-file)'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -H 'Node group name'
            cand --hsm-group 'Node group name'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;apply;sat-file'= {
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
        &'manta-cli;apply;boot'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand nodes 'Update boot parameters for a set of nodes'
            cand cluster 'Update boot parameters for all nodes in a cluster'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;apply;boot;nodes'= {
            cand -i 'Image ID to boot the nodes'
            cand --boot-image 'Image ID to boot the nodes'
            cand -b 'Configuration name used to build the boot image (uses the most recent matching image)'
            cand --boot-image-configuration 'Configuration name used to build the boot image (uses the most recent matching image)'
            cand -r 'Configuration to apply to nodes after booting'
            cand --runtime-configuration 'Configuration to apply to nodes after booting'
            cand -k 'Kernel parameters to assign to the nodes'
            cand --kernel-parameters 'Kernel parameters to assign to the nodes'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --do-not-reboot 'Suppress the automatic reboot after updating boot parameters'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta-cli;apply;boot;cluster'= {
            cand -i 'Image ID to boot the nodes'
            cand --boot-image 'Image ID to boot the nodes'
            cand -b 'Configuration name used to build the boot image (uses the most recent matching image)'
            cand --boot-image-configuration 'Configuration name used to build the boot image (uses the most recent matching image)'
            cand -r 'Configuration to apply to nodes after booting'
            cand --runtime-configuration 'Configuration to apply to nodes after booting'
            cand -k 'Kernel parameters to assign to all cluster nodes'
            cand --kernel-parameters 'Kernel parameters to assign to all cluster nodes'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --do-not-reboot 'Suppress the automatic reboot after updating boot parameters'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta-cli;apply;boot;help'= {
            cand nodes 'Update boot parameters for a set of nodes'
            cand cluster 'Update boot parameters for all nodes in a cluster'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;apply;boot;help;nodes'= {
        }
        &'manta-cli;apply;boot;help;cluster'= {
        }
        &'manta-cli;apply;boot;help;help'= {
        }
        &'manta-cli;apply;kernel-parameters'= {
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -H 'Node group name'
            cand --hsm-group 'Node group name'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --do-not-reboot 'Do not reboot nodes after applying changes'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;apply;session'= {
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
            cand -l 'Limit the session to specific nodes (must be a subset of --hsm-group if both are provided)'
            cand --ansible-limit 'Limit the session to specific nodes (must be a subset of --hsm-group if both are provided)'
            cand -H 'Node group name'
            cand --hsm-group 'Node group name'
            cand -w 'Stream session logs to stdout'
            cand --watch-logs 'Stream session logs to stdout'
            cand -t 'Show log timestamps'
            cand --timestamps 'Show log timestamps'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta-cli;apply;ephemeral-environment'= {
            cand -i 'Image ID to use'
            cand --image-id 'Image ID to use'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta-cli;apply;template'= {
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
        &'manta-cli;apply;help'= {
            cand hardware '[experimental] Rescale a cluster''s hardware allocation'
            cand configuration 'Create a configuration (deprecated — use ''apply sat-file'')'
            cand sat-file 'Process a SAT file to create configurations, images, and session templates'
            cand boot 'Update boot parameters and runtime configuration'
            cand kernel-parameters 'Replace kernel parameters on nodes'
            cand session 'Create and run a configuration session from a local repo'
            cand ephemeral-environment 'Launch an ephemeral SSH environment from an image'
            cand template 'Boot nodes using an existing session template'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;apply;help;hardware'= {
            cand cluster '[experimental] Rescale a cluster''s hardware allocation'
        }
        &'manta-cli;apply;help;hardware;cluster'= {
        }
        &'manta-cli;apply;help;configuration'= {
        }
        &'manta-cli;apply;help;sat-file'= {
        }
        &'manta-cli;apply;help;boot'= {
            cand nodes 'Update boot parameters for a set of nodes'
            cand cluster 'Update boot parameters for all nodes in a cluster'
        }
        &'manta-cli;apply;help;boot;nodes'= {
        }
        &'manta-cli;apply;help;boot;cluster'= {
        }
        &'manta-cli;apply;help;kernel-parameters'= {
        }
        &'manta-cli;apply;help;session'= {
        }
        &'manta-cli;apply;help;ephemeral-environment'= {
        }
        &'manta-cli;apply;help;template'= {
        }
        &'manta-cli;apply;help;help'= {
        }
        &'manta-cli;delete'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand group 'Delete a node group'
            cand node 'Remove a node from the system'
            cand kernel-parameters 'Remove kernel parameters from nodes'
            cand boot-parameters 'Delete boot parameters for nodes'
            cand configurations 'Delete configurations and all associated data'
            cand session 'Delete a configuration session'
            cand images '[experimental] Delete images'
            cand hardware '[experimental] Remove hardware components from a cluster'
            cand redfish-endpoint 'Delete a Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;delete;group'= {
            cand -f 'Force deletion'
            cand --force 'Force deletion'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta-cli;delete;node'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;delete;kernel-parameters'= {
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -H 'Node group name'
            cand --hsm-group 'Node group name'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand --do-not-reboot 'Do not reboot nodes after applying changes'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;delete;boot-parameters'= {
            cand -H 'Xnames of the nodes'
            cand --hosts 'Xnames of the nodes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;delete;configurations'= {
            cand -n 'Glob pattern to filter by name. eg: my-config*, my-config-v[1,2]'
            cand --configuration-name 'Glob pattern to filter by name. eg: my-config*, my-config-v[1,2]'
            cand -s 'Delete configurations last updated after this date (format: %Y-%m-%d)'
            cand --since 'Delete configurations last updated after this date (format: %Y-%m-%d)'
            cand -u 'Delete configurations last updated before this date (format: %Y-%m-%d)'
            cand --until 'Delete configurations last updated before this date (format: %Y-%m-%d)'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta-cli;delete;session'= {
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta-cli;delete;images'= {
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;delete;hardware'= {
            cand -P 'Hardware component pattern'
            cand --pattern 'Hardware component pattern'
            cand -t 'Cluster to remove components from'
            cand --target-cluster 'Cluster to remove components from'
            cand -p 'Cluster that receives the freed components'
            cand --parent-cluster 'Cluster that receives the freed components'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -D 'Delete the group if empty after this operation'
            cand --delete-hsm-group 'Delete the group if empty after this operation'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;delete;redfish-endpoint'= {
            cand -i 'Xname of the Redfish endpoint to delete'
            cand --id 'Xname of the Redfish endpoint to delete'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;delete;help'= {
            cand group 'Delete a node group'
            cand node 'Remove a node from the system'
            cand kernel-parameters 'Remove kernel parameters from nodes'
            cand boot-parameters 'Delete boot parameters for nodes'
            cand configurations 'Delete configurations and all associated data'
            cand session 'Delete a configuration session'
            cand images '[experimental] Delete images'
            cand hardware '[experimental] Remove hardware components from a cluster'
            cand redfish-endpoint 'Delete a Redfish endpoint'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;delete;help;group'= {
        }
        &'manta-cli;delete;help;node'= {
        }
        &'manta-cli;delete;help;kernel-parameters'= {
        }
        &'manta-cli;delete;help;boot-parameters'= {
        }
        &'manta-cli;delete;help;configurations'= {
        }
        &'manta-cli;delete;help;session'= {
        }
        &'manta-cli;delete;help;images'= {
        }
        &'manta-cli;delete;help;hardware'= {
        }
        &'manta-cli;delete;help;redfish-endpoint'= {
        }
        &'manta-cli;delete;help;help'= {
        }
        &'manta-cli;migrate'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand vCluster '[experimental] Migrate a cluster'
            cand nodes 'Move nodes between clusters'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;migrate;vCluster'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand backup 'Back up a cluster''s configuration'
            cand restore 'Restore a cluster from a backup'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;migrate;vCluster;backup'= {
            cand -b 'Session template to derive the backup from'
            cand --bos 'Session template to derive the backup from'
            cand -d 'Destination directory for the backup files'
            cand --destination 'Destination directory for the backup files'
            cand -p 'Command to run before the backup. eg: --pre-hook "echo hello"'
            cand --pre-hook 'Command to run before the backup. eg: --pre-hook "echo hello"'
            cand -a 'Command to run after a successful backup. eg: --post-hook "echo hello"'
            cand --post-hook 'Command to run after a successful backup. eg: --post-hook "echo hello"'
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta-cli;migrate;vCluster;restore'= {
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
            cand -o 'Overwrite existing data'
            cand --overwrite 'Overwrite existing data'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;migrate;vCluster;help'= {
            cand backup 'Back up a cluster''s configuration'
            cand restore 'Restore a cluster from a backup'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;migrate;vCluster;help;backup'= {
        }
        &'manta-cli;migrate;vCluster;help;restore'= {
        }
        &'manta-cli;migrate;vCluster;help;help'= {
        }
        &'manta-cli;migrate;nodes'= {
            cand -f 'Source cluster to move nodes from'
            cand --from 'Source cluster to move nodes from'
            cand -t 'Destination cluster to move nodes to'
            cand --to 'Destination cluster to move nodes to'
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;migrate;help'= {
            cand vCluster '[experimental] Migrate a cluster'
            cand nodes 'Move nodes between clusters'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;migrate;help;vCluster'= {
            cand backup 'Back up a cluster''s configuration'
            cand restore 'Restore a cluster from a backup'
        }
        &'manta-cli;migrate;help;vCluster;backup'= {
        }
        &'manta-cli;migrate;help;vCluster;restore'= {
        }
        &'manta-cli;migrate;help;nodes'= {
        }
        &'manta-cli;migrate;help;help'= {
        }
        &'manta-cli;power'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand on 'Power on nodes or a cluster'
            cand off 'Power off nodes or a cluster'
            cand reset 'Reset (reboot) nodes or a cluster'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;power;on'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand cluster 'Power on all nodes in a cluster'
            cand nodes 'Power on a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;power;on;cluster'= {
            cand -R 'Reason for the power operation'
            cand --reason 'Reason for the power operation'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;power;on;nodes'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;power;on;help'= {
            cand cluster 'Power on all nodes in a cluster'
            cand nodes 'Power on a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;power;on;help;cluster'= {
        }
        &'manta-cli;power;on;help;nodes'= {
        }
        &'manta-cli;power;on;help;help'= {
        }
        &'manta-cli;power;off'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand cluster 'Power off all nodes in a cluster'
            cand nodes 'Power off a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;power;off;cluster'= {
            cand -R 'Reason for the power operation'
            cand --reason 'Reason for the power operation'
            cand -o 'Output format'
            cand --output 'Output format'
            cand -g 'Perform a graceful shutdown'
            cand --graceful 'Perform a graceful shutdown'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;power;off;nodes'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -g 'Perform a graceful shutdown'
            cand --graceful 'Perform a graceful shutdown'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;power;off;help'= {
            cand cluster 'Power off all nodes in a cluster'
            cand nodes 'Power off a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;power;off;help;cluster'= {
        }
        &'manta-cli;power;off;help;nodes'= {
        }
        &'manta-cli;power;off;help;help'= {
        }
        &'manta-cli;power;reset'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand cluster 'Reset all nodes in a cluster'
            cand nodes 'Reset a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;power;reset;cluster'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -r 'Reason for the power operation'
            cand --reason 'Reason for the power operation'
            cand -g 'Perform a graceful reboot'
            cand --graceful 'Perform a graceful reboot'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;power;reset;nodes'= {
            cand -o 'Output format'
            cand --output 'Output format'
            cand -g 'Perform a graceful reboot'
            cand --graceful 'Perform a graceful reboot'
            cand -y 'Skip confirmation prompts'
            cand --assume-yes 'Skip confirmation prompts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;power;reset;help'= {
            cand cluster 'Reset all nodes in a cluster'
            cand nodes 'Reset a set of nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;power;reset;help;cluster'= {
        }
        &'manta-cli;power;reset;help;nodes'= {
        }
        &'manta-cli;power;reset;help;help'= {
        }
        &'manta-cli;power;help'= {
            cand on 'Power on nodes or a cluster'
            cand off 'Power off nodes or a cluster'
            cand reset 'Reset (reboot) nodes or a cluster'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;power;help;on'= {
            cand cluster 'Power on all nodes in a cluster'
            cand nodes 'Power on a set of nodes'
        }
        &'manta-cli;power;help;on;cluster'= {
        }
        &'manta-cli;power;help;on;nodes'= {
        }
        &'manta-cli;power;help;off'= {
            cand cluster 'Power off all nodes in a cluster'
            cand nodes 'Power off a set of nodes'
        }
        &'manta-cli;power;help;off;cluster'= {
        }
        &'manta-cli;power;help;off;nodes'= {
        }
        &'manta-cli;power;help;reset'= {
            cand cluster 'Reset all nodes in a cluster'
            cand nodes 'Reset a set of nodes'
        }
        &'manta-cli;power;help;reset;cluster'= {
        }
        &'manta-cli;power;help;reset;nodes'= {
        }
        &'manta-cli;power;help;help'= {
        }
        &'manta-cli;log'= {
            cand -t 'Show log timestamps'
            cand --timestamps 'Show log timestamps'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;console'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand node 'Connect to a node''s serial console'
            cand target-ansible 'Connect to the Ansible target container of a configuration session'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;console;node'= {
            cand -h 'Print help (see more with ''--help'')'
            cand --help 'Print help (see more with ''--help'')'
        }
        &'manta-cli;console;target-ansible'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;console;help'= {
            cand node 'Connect to a node''s serial console'
            cand target-ansible 'Connect to the Ansible target container of a configuration session'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;console;help;node'= {
        }
        &'manta-cli;console;help;target-ansible'= {
        }
        &'manta-cli;console;help;help'= {
        }
        &'manta-cli;add-nodes-to-groups'= {
            cand -g 'Group to add the nodes to'
            cand --group 'Group to add the nodes to'
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;remove-nodes-from-groups'= {
            cand -g 'Group to remove the nodes from'
            cand --group 'Group to remove the nodes from'
            cand -n 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand --nodes 'Xnames, NIDs, or a hostlist expression. eg: ''x1003c1s7b0n0,x1003c1s7b0n1'', ''nid001313,nid001314'', ''x1003c1s7b0n[0-1],x1003c1s7b1n0'', ''nid00131[0-9]'''
            cand -d 'Simulate the operation without making changes'
            cand --dry-run 'Simulate the operation without making changes'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta-cli;help'= {
            cand config 'Manage manta CLI configuration'
            cand get 'Query system resources'
            cand add 'Create system resources'
            cand update 'Update system resources'
            cand apply 'Apply changes to the system'
            cand delete 'Delete system resources'
            cand migrate 'Move nodes or clusters between groups'
            cand power 'Manage node and cluster power state'
            cand log 'Stream configuration session logs'
            cand console 'Open an interactive console to a node or configuration session'
            cand add-nodes-to-groups 'Add nodes to one or more groups'
            cand remove-nodes-from-groups 'Remove nodes from one or more groups'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta-cli;help;config'= {
            cand show 'Show current configuration values'
            cand set 'Set a configuration value'
            cand unset 'Clear a configuration value'
            cand gen-autocomplete 'Generate shell completion scripts'
        }
        &'manta-cli;help;config;show'= {
        }
        &'manta-cli;help;config;set'= {
            cand hsm 'Set the active node group'
            cand parent-hsm 'Set the parent node group'
            cand site 'Set the active site'
            cand log 'Set the log verbosity level'
        }
        &'manta-cli;help;config;set;hsm'= {
        }
        &'manta-cli;help;config;set;parent-hsm'= {
        }
        &'manta-cli;help;config;set;site'= {
        }
        &'manta-cli;help;config;set;log'= {
        }
        &'manta-cli;help;config;unset'= {
            cand hsm 'Clear the active node group'
            cand parent-hsm 'Clear the parent node group'
            cand auth 'Clear the cached authentication token'
        }
        &'manta-cli;help;config;unset;hsm'= {
        }
        &'manta-cli;help;config;unset;parent-hsm'= {
        }
        &'manta-cli;help;config;unset;auth'= {
        }
        &'manta-cli;help;config;gen-autocomplete'= {
        }
        &'manta-cli;help;get'= {
            cand groups 'List node groups'
            cand hardware 'Inspect hardware components'
            cand sessions 'List configuration sessions'
            cand configurations 'List configurations'
            cand templates 'List session templates'
            cand cluster 'Show cluster node details and status'
            cand nodes 'Show node details and status'
            cand images 'List images'
            cand boot-parameters 'Show boot parameters for nodes or a group'
            cand kernel-parameters 'Show kernel parameters for nodes or a group'
            cand redfish-endpoints 'List Redfish endpoints'
        }
        &'manta-cli;help;get;groups'= {
        }
        &'manta-cli;help;get;hardware'= {
            cand cluster 'Show hardware inventory for a cluster'
            cand nodes 'Show hardware inventory for a set of nodes'
        }
        &'manta-cli;help;get;hardware;cluster'= {
        }
        &'manta-cli;help;get;hardware;nodes'= {
        }
        &'manta-cli;help;get;sessions'= {
        }
        &'manta-cli;help;get;configurations'= {
        }
        &'manta-cli;help;get;templates'= {
        }
        &'manta-cli;help;get;cluster'= {
        }
        &'manta-cli;help;get;nodes'= {
        }
        &'manta-cli;help;get;images'= {
        }
        &'manta-cli;help;get;boot-parameters'= {
        }
        &'manta-cli;help;get;kernel-parameters'= {
        }
        &'manta-cli;help;get;redfish-endpoints'= {
        }
        &'manta-cli;help;add'= {
            cand node 'Register a new node'
            cand group 'Create a node group'
            cand hardware '[experimental] Add hardware components to a cluster'
            cand boot-parameters 'Create boot parameters for nodes'
            cand kernel-parameters 'Append kernel parameters to nodes'
            cand redfish-endpoint 'Register a new Redfish endpoint'
        }
        &'manta-cli;help;add;node'= {
        }
        &'manta-cli;help;add;group'= {
        }
        &'manta-cli;help;add;hardware'= {
        }
        &'manta-cli;help;add;boot-parameters'= {
        }
        &'manta-cli;help;add;kernel-parameters'= {
        }
        &'manta-cli;help;add;redfish-endpoint'= {
        }
        &'manta-cli;help;update'= {
            cand boot-parameters 'Update boot parameters for nodes'
            cand redfish-endpoint 'Update a Redfish endpoint'
        }
        &'manta-cli;help;update;boot-parameters'= {
        }
        &'manta-cli;help;update;redfish-endpoint'= {
        }
        &'manta-cli;help;apply'= {
            cand hardware '[experimental] Rescale a cluster''s hardware allocation'
            cand configuration 'Create a configuration (deprecated — use ''apply sat-file'')'
            cand sat-file 'Process a SAT file to create configurations, images, and session templates'
            cand boot 'Update boot parameters and runtime configuration'
            cand kernel-parameters 'Replace kernel parameters on nodes'
            cand session 'Create and run a configuration session from a local repo'
            cand ephemeral-environment 'Launch an ephemeral SSH environment from an image'
            cand template 'Boot nodes using an existing session template'
        }
        &'manta-cli;help;apply;hardware'= {
            cand cluster '[experimental] Rescale a cluster''s hardware allocation'
        }
        &'manta-cli;help;apply;hardware;cluster'= {
        }
        &'manta-cli;help;apply;configuration'= {
        }
        &'manta-cli;help;apply;sat-file'= {
        }
        &'manta-cli;help;apply;boot'= {
            cand nodes 'Update boot parameters for a set of nodes'
            cand cluster 'Update boot parameters for all nodes in a cluster'
        }
        &'manta-cli;help;apply;boot;nodes'= {
        }
        &'manta-cli;help;apply;boot;cluster'= {
        }
        &'manta-cli;help;apply;kernel-parameters'= {
        }
        &'manta-cli;help;apply;session'= {
        }
        &'manta-cli;help;apply;ephemeral-environment'= {
        }
        &'manta-cli;help;apply;template'= {
        }
        &'manta-cli;help;delete'= {
            cand group 'Delete a node group'
            cand node 'Remove a node from the system'
            cand kernel-parameters 'Remove kernel parameters from nodes'
            cand boot-parameters 'Delete boot parameters for nodes'
            cand configurations 'Delete configurations and all associated data'
            cand session 'Delete a configuration session'
            cand images '[experimental] Delete images'
            cand hardware '[experimental] Remove hardware components from a cluster'
            cand redfish-endpoint 'Delete a Redfish endpoint'
        }
        &'manta-cli;help;delete;group'= {
        }
        &'manta-cli;help;delete;node'= {
        }
        &'manta-cli;help;delete;kernel-parameters'= {
        }
        &'manta-cli;help;delete;boot-parameters'= {
        }
        &'manta-cli;help;delete;configurations'= {
        }
        &'manta-cli;help;delete;session'= {
        }
        &'manta-cli;help;delete;images'= {
        }
        &'manta-cli;help;delete;hardware'= {
        }
        &'manta-cli;help;delete;redfish-endpoint'= {
        }
        &'manta-cli;help;migrate'= {
            cand vCluster '[experimental] Migrate a cluster'
            cand nodes 'Move nodes between clusters'
        }
        &'manta-cli;help;migrate;vCluster'= {
            cand backup 'Back up a cluster''s configuration'
            cand restore 'Restore a cluster from a backup'
        }
        &'manta-cli;help;migrate;vCluster;backup'= {
        }
        &'manta-cli;help;migrate;vCluster;restore'= {
        }
        &'manta-cli;help;migrate;nodes'= {
        }
        &'manta-cli;help;power'= {
            cand on 'Power on nodes or a cluster'
            cand off 'Power off nodes or a cluster'
            cand reset 'Reset (reboot) nodes or a cluster'
        }
        &'manta-cli;help;power;on'= {
            cand cluster 'Power on all nodes in a cluster'
            cand nodes 'Power on a set of nodes'
        }
        &'manta-cli;help;power;on;cluster'= {
        }
        &'manta-cli;help;power;on;nodes'= {
        }
        &'manta-cli;help;power;off'= {
            cand cluster 'Power off all nodes in a cluster'
            cand nodes 'Power off a set of nodes'
        }
        &'manta-cli;help;power;off;cluster'= {
        }
        &'manta-cli;help;power;off;nodes'= {
        }
        &'manta-cli;help;power;reset'= {
            cand cluster 'Reset all nodes in a cluster'
            cand nodes 'Reset a set of nodes'
        }
        &'manta-cli;help;power;reset;cluster'= {
        }
        &'manta-cli;help;power;reset;nodes'= {
        }
        &'manta-cli;help;log'= {
        }
        &'manta-cli;help;console'= {
            cand node 'Connect to a node''s serial console'
            cand target-ansible 'Connect to the Ansible target container of a configuration session'
        }
        &'manta-cli;help;console;node'= {
        }
        &'manta-cli;help;console;target-ansible'= {
        }
        &'manta-cli;help;add-nodes-to-groups'= {
        }
        &'manta-cli;help;remove-nodes-from-groups'= {
        }
        &'manta-cli;help;help'= {
        }
    ]
    $completions[$command]
}
