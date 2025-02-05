
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
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
            cand config 'Manta''s configuration'
            cand get 'Get information from CSM system'
            cand add 'Add hw components to cluster'
            cand apply 'Make changes to Shasta system'
            cand delete 'Deletes data'
            cand migrate 'migrate'
            cand power 'Command to submit commands related to cluster/node power management'
            cand log 'get cfs session logs'
            cand console 'Opens an interective session to a node or CFS session ansible target container'
            cand validate-local-repo 'Check all tags and HEAD information related to a local repo exists in Gitea'
            cand add-nodes-to-groups 'Add nodes to a list of groups'
            cand remove-nodes-from-groups 'Remove nodes from groups'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;config'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand show 'Show config values'
            cand set 'Change config values'
            cand unset 'Reset config values'
            cand generate-autocomplete 'Generate shell auto completion script'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;config;show'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;set'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand hsm 'Set target HSM group'
            cand parent-hsm 'Set parent HSM group'
            cand site 'Set site to work on'
            cand log 'Set site to work on'
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
            cand hsm 'Set target HSM group'
            cand parent-hsm 'Set parent HSM group'
            cand site 'Set site to work on'
            cand log 'Set site to work on'
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
            cand hsm 'Unset target HSM group'
            cand parent-hsm 'Unset parent HSM group'
            cand auth 'Unset authentication token'
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
            cand hsm 'Unset target HSM group'
            cand parent-hsm 'Unset parent HSM group'
            cand auth 'Unset authentication token'
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
        &'manta;config;generate-autocomplete'= {
            cand -s 'Shell type. Will try to guess from $SHELL if missing'
            cand --shell 'Shell type. Will try to guess from $SHELL if missing'
            cand -p 'Path to put the autocomplete script or prints to stdout if missing. NOTE: Do not specify filename, only path to directory'
            cand --path 'Path to put the autocomplete script or prints to stdout if missing. NOTE: Do not specify filename, only path to directory'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;config;help'= {
            cand show 'Show config values'
            cand set 'Change config values'
            cand unset 'Reset config values'
            cand generate-autocomplete 'Generate shell auto completion script'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;config;help;show'= {
        }
        &'manta;config;help;set'= {
            cand hsm 'Set target HSM group'
            cand parent-hsm 'Set parent HSM group'
            cand site 'Set site to work on'
            cand log 'Set site to work on'
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
            cand hsm 'Unset target HSM group'
            cand parent-hsm 'Unset parent HSM group'
            cand auth 'Unset authentication token'
        }
        &'manta;config;help;unset;hsm'= {
        }
        &'manta;config;help;unset;parent-hsm'= {
        }
        &'manta;config;help;unset;auth'= {
        }
        &'manta;config;help;generate-autocomplete'= {
        }
        &'manta;config;help;help'= {
        }
        &'manta;get'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand hw-component 'Get hardware components1 for a cluster or a node'
            cand sessions 'Get information from Shasta CFS session'
            cand configurations 'Get information from Shasta CFS configuration'
            cand templates 'Get information from Shasta BOS template'
            cand cluster 'Get cluster details'
            cand nodes 'Get node details'
            cand hsm-groups 'DEPRECATED - Please do not use this command. Get HSM groups details'
            cand images 'Get image information'
            cand kernel-parameters 'Get kernel-parameters information'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;get;hw-component'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand cluster 'Get hw components for a cluster'
            cand node 'Get hw components for some nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;get;hw-component;cluster'= {
            cand -o 'Output format. If missing it will print output data in human redeable (table) format'
            cand --output 'Output format. If missing it will print output data in human redeable (table) format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;hw-component;node'= {
            cand -t 'Filters output to specific type'
            cand --type 'Filters output to specific type'
            cand -o 'Output format. If missing it will print output data in human redeable (table) format'
            cand --output 'Output format. If missing it will print output data in human redeable (table) format'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;hw-component;help'= {
            cand cluster 'Get hw components for a cluster'
            cand node 'Get hw components for some nodes'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;get;hw-component;help;cluster'= {
        }
        &'manta;get;hw-component;help;node'= {
        }
        &'manta;get;hw-component;help;help'= {
        }
        &'manta;get;sessions'= {
            cand -n 'Return only sessions with the given session name'
            cand --name 'Return only sessions with the given session name'
            cand -a 'Return only sessions older than the given age. Age is given in the format ''1d'' or ''6h'''
            cand --min-age 'Return only sessions older than the given age. Age is given in the format ''1d'' or ''6h'''
            cand -A 'Return only sessions younger than the given age. Age is given in the format ''1d'' or ''6h'''
            cand --max-age 'Return only sessions younger than the given age. Age is given in the format ''1d'' or ''6h'''
            cand -s 'Return only sessions with the given status'
            cand --status 'Return only sessions with the given status'
            cand -l 'Return only last <VALUE> sessions created'
            cand --limit 'Return only last <VALUE> sessions created'
            cand -o 'Output format. If missing, it will print output data in human redeable (table) format'
            cand --output 'Output format. If missing, it will print output data in human redeable (table) format'
            cand -x 'Comma separated list of xnames. eg ''x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'''
            cand --xnames 'Comma separated list of xnames. eg ''x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'''
            cand -H 'hsm group name'
            cand --hsm-group 'hsm group name'
            cand -m 'Return only the most recent session created (equivalent to --limit 1)'
            cand --most-recent 'Return only the most recent session created (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;configurations'= {
            cand -n 'configuration name'
            cand --name 'configuration name'
            cand -p 'Glob pattern for configuration name'
            cand --pattern 'Glob pattern for configuration name'
            cand -l 'Filter records to the <VALUE> most common number of CFS configurations created'
            cand --limit 'Filter records to the <VALUE> most common number of CFS configurations created'
            cand -o 'Output format. If missing, it will print output data in human redeable (table) format'
            cand --output 'Output format. If missing, it will print output data in human redeable (table) format'
            cand -H 'hsm group name'
            cand --hsm-group 'hsm group name'
            cand -m 'Only shows the most recent (equivalent to --limit 1)'
            cand --most-recent 'Only shows the most recent (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;templates'= {
            cand -n 'template name'
            cand --name 'template name'
            cand -l 'Filter records to the <VALUE> most common number of BOS templates created'
            cand --limit 'Filter records to the <VALUE> most common number of BOS templates created'
            cand -H 'hsm group name'
            cand --hsm-group 'hsm group name'
            cand -o 'Output format. If missing it will print output data in human redeable (table) format'
            cand --output 'Output format. If missing it will print output data in human redeable (table) format'
            cand -m 'Only shows the most recent (equivalent to --limit 1)'
            cand --most-recent 'Only shows the most recent (equivalent to --limit 1)'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;cluster'= {
            cand -o 'Output format. If missing it will print output data in human readable (table) format'
            cand --output 'Output format. If missing it will print output data in human readable (table) format'
            cand -n 'Prints nids in one line eg nidxxxxxx,nidyyyyyy,nidzzzzzz,...'
            cand --nids-only-one-line 'Prints nids in one line eg nidxxxxxx,nidyyyyyy,nidzzzzzz,...'
            cand -x 'Prints xnames in one line eg x1001c1s5b0n0,x1001c1s5b0n1,...'
            cand --xnames-only-one-line 'Prints xnames in one line eg x1001c1s5b0n0,x1001c1s5b0n1,...'
            cand -s 'Get cluster status:  - OK: All nodes are operational (booted and configured)  - OFF: At least one node is OFF  - ON: No nodes OFF and at least one is ON  - STANDBY: At least one node''s heartbeat is lost  - UNCONFIGURED: All nodes are READY but at least one of them is being configured  - FAILED: At least one node configuration failed'
            cand --status 'Get cluster status:  - OK: All nodes are operational (booted and configured)  - OFF: At least one node is OFF  - ON: No nodes OFF and at least one is ON  - STANDBY: At least one node''s heartbeat is lost  - UNCONFIGURED: All nodes are READY but at least one of them is being configured  - FAILED: At least one node configuration failed'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;nodes'= {
            cand -o 'Output format. If missing it will print output data in human readable (table) format'
            cand --output 'Output format. If missing it will print output data in human readable (table) format'
            cand -n 'Prints nids in one line eg nidxxxxxx,nidyyyyyy,nidzzzzzz,...'
            cand --nids-only-one-line 'Prints nids in one line eg nidxxxxxx,nidyyyyyy,nidzzzzzz,...'
            cand -s 'Get cluster status:  - OK: All nodes are operational (booted and configured)  - OFF: At least one node is OFF  - ON: No nodes OFF and at least one is ON  - STANDBY: At least one node''s heartbeat is lost  - UNCONFIGURED: All nodes are READY but at least one of them is being configured  - FAILED: At least one node configuration failed'
            cand --status 'Get cluster status:  - OK: All nodes are operational (booted and configured)  - OFF: At least one node is OFF  - ON: No nodes OFF and at least one is ON  - STANDBY: At least one node''s heartbeat is lost  - UNCONFIGURED: All nodes are READY but at least one of them is being configured  - FAILED: At least one node configuration failed'
            cand -S 'Output includes extra nodes related to the ones requested by used. 2 nodes are siblings if they share the same power supply.'
            cand --include-siblings 'Output includes extra nodes related to the ones requested by used. 2 nodes are siblings if they share the same power supply.'
            cand -r 'Input nodes in regex format.'
            cand --regex 'Input nodes in regex format.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;hsm-groups'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;images'= {
            cand -i 'Image ID'
            cand --id 'Image ID'
            cand -l 'Filter records to the <VALUE> most common number of images created'
            cand --limit 'Filter records to the <VALUE> most common number of images created'
            cand -H 'hsm group name'
            cand --hsm-group 'hsm group name'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;kernel-parameters'= {
            cand -x 'Comma separated list of xnames to retreive the kernel parameters from. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
            cand --xnames 'Comma separated list of xnames to retreive the kernel parameters from. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
            cand -H 'List kernel parameters for all nodes in a HSM group name'
            cand --hsm-group 'List kernel parameters for all nodes in a HSM group name'
            cand -f 'Comma separated list of kernel parameters to filter. eg: ''console,bad_page,crashkernel,hugepagelist,root'''
            cand --filter 'Comma separated list of kernel parameters to filter. eg: ''console,bad_page,crashkernel,hugepagelist,root'''
            cand -o 'Output format.'
            cand --output 'Output format.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;get;help'= {
            cand hw-component 'Get hardware components1 for a cluster or a node'
            cand sessions 'Get information from Shasta CFS session'
            cand configurations 'Get information from Shasta CFS configuration'
            cand templates 'Get information from Shasta BOS template'
            cand cluster 'Get cluster details'
            cand nodes 'Get node details'
            cand hsm-groups 'DEPRECATED - Please do not use this command. Get HSM groups details'
            cand images 'Get image information'
            cand kernel-parameters 'Get kernel-parameters information'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;get;help;hw-component'= {
            cand cluster 'Get hw components for a cluster'
            cand node 'Get hw components for some nodes'
        }
        &'manta;get;help;hw-component;cluster'= {
        }
        &'manta;get;help;hw-component;node'= {
        }
        &'manta;get;help;sessions'= {
        }
        &'manta;get;help;configurations'= {
        }
        &'manta;get;help;templates'= {
        }
        &'manta;get;help;cluster'= {
        }
        &'manta;get;help;nodes'= {
        }
        &'manta;get;help;hsm-groups'= {
        }
        &'manta;get;help;images'= {
        }
        &'manta;get;help;kernel-parameters'= {
        }
        &'manta;get;help;help'= {
        }
        &'manta;add'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand hw-component 'WIP - Add hw components from a cluster'
            cand kernel-parameters 'Delete kernel parameters'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;add;hw-component'= {
            cand -P 'Pattern'
            cand --pattern 'Pattern'
            cand -t 'Target cluster name. This is the name of the cluster the pattern is applying to.'
            cand --target-cluster 'Target cluster name. This is the name of the cluster the pattern is applying to.'
            cand -p 'Parent cluster name. The parent cluster is the one offering and receiving resources from the target cluster.'
            cand --parent-cluster 'Parent cluster name. The parent cluster is the one offering and receiving resources from the target cluster.'
            cand -x 'No dry-run, actually change the status of the system. The default for this command is a dry-run.'
            cand --no-dryrun 'No dry-run, actually change the status of the system. The default for this command is a dry-run.'
            cand -c 'If the target cluster name does not exist as HSM group, create it.'
            cand --create-hsm-group 'If the target cluster name does not exist as HSM group, create it.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;add;kernel-parameters'= {
            cand -x 'Comma separated list of nodes to set runtime configuration. eg ''x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'''
            cand --xnames 'Comma separated list of nodes to set runtime configuration. eg ''x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'''
            cand -H 'Cluster to set runtime configuration'
            cand --hsm-group 'Cluster to set runtime configuration'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;add;help'= {
            cand hw-component 'WIP - Add hw components from a cluster'
            cand kernel-parameters 'Delete kernel parameters'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;add;help;hw-component'= {
        }
        &'manta;add;help;kernel-parameters'= {
        }
        &'manta;add;help;help'= {
        }
        &'manta;apply'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand hw-configuration 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
            cand configuration 'DEPRECATED - Please use ''manta apply sat-file'' command instead. Create a CFS configuration'
            cand sat-file 'Process a SAT file and creates the configurations, images, boot parameters and runtime configurations. If runtime configuration and boot parameters are defined, then, reboots the nodes to configure. The ansible container for the session building the image will remain running after an Ansible failure.  The container will remain running for a number of seconds specified by the ''debug_wait_time options'''
            cand boot 'Change boot operations'
            cand session 'Runs the ansible script in local directory against HSM group or xnames. Note: the local repo must alrady exists in Shasta VCS'
            cand ephemeral-environment 'Returns a hostname use can ssh with the image ID provided. This call is async which means, the user will have to wait a few seconds for the environment to be ready, normally, this takes a few seconds.'
            cand template 'Create a new BOS session from an existing BOS sessiontemplate'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;hw-configuration'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand cluster 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;hw-configuration;cluster'= {
            cand -P 'Hw pattern with keywords to fuzzy find hardware componented to assign to the cluster like <hw component name>:<hw component quantity>[:<hw component name>:<hw component quantity>]. Eg ''a100:12:epic:5'' will update the nodes assigned to cluster ''zinal'' with 4 nodes:  - 3 nodes with 4 Nvidia gpus A100 and 1 epyc AMD cpu each  - 1 node with 2 epyc AMD cpus'
            cand --pattern 'Hw pattern with keywords to fuzzy find hardware componented to assign to the cluster like <hw component name>:<hw component quantity>[:<hw component name>:<hw component quantity>]. Eg ''a100:12:epic:5'' will update the nodes assigned to cluster ''zinal'' with 4 nodes:  - 3 nodes with 4 Nvidia gpus A100 and 1 epyc AMD cpu each  - 1 node with 2 epyc AMD cpus'
            cand -t 'Target cluster name. This is the name of the cluster the pattern is applying to.'
            cand --target-cluster 'Target cluster name. This is the name of the cluster the pattern is applying to.'
            cand -p 'Parent cluster name. The parent cluster is the one offering and receiving resources from the target cluster.'
            cand --parent-cluster 'Parent cluster name. The parent cluster is the one offering and receiving resources from the target cluster.'
            cand -x 'No dry-run, actually change the status of the system. The default for this command is a dry-run.'
            cand --no-dryrun 'No dry-run, actually change the status of the system. The default for this command is a dry-run.'
            cand -c 'If the target cluster name does not exist as HSM group, create it.'
            cand --create-target-hsm-group 'If the target cluster name does not exist as HSM group, create it.'
            cand -d 'If the target HSM group is empty after this action, remove it.'
            cand --delete-empty-parent-hsm-group 'If the target HSM group is empty after this action, remove it.'
            cand -u 'It will try to get any nodes available.'
            cand --unpin-nodes 'It will try to get any nodes available.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;hw-configuration;help'= {
            cand cluster 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;hw-configuration;help;cluster'= {
        }
        &'manta;apply;hw-configuration;help;help'= {
        }
        &'manta;apply;configuration'= {
            cand -t 'SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.'
            cand --sat-template-file 'SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.'
            cand -f 'If the SAT file is a jinja2 template, then variables values can be expanded using this values file.'
            cand --values-file 'If the SAT file is a jinja2 template, then variables values can be expanded using this values file.'
            cand -V 'If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided.'
            cand --values 'If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided.'
            cand -o 'Output format. If missing it will print output data in human redeable (table) format'
            cand --output 'Output format. If missing it will print output data in human redeable (table) format'
            cand -H 'hsm group name linked to this configuration'
            cand --hsm-group 'hsm group name linked to this configuration'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;sat-file'= {
            cand -t 'SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.'
            cand --sat-template-file 'SAT file with CFS configuration, CFS image and BOS session template details to create a cluster. The SAT file can be a jinja2 template, if this is the case, then a values file must be provided.'
            cand -f 'If the SAT file is a jinja2 template, then variables values can be expanded using this values file.'
            cand --values-file 'If the SAT file is a jinja2 template, then variables values can be expanded using this values file.'
            cand -V 'If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided.'
            cand --values 'If the SAT file is a jinja2 template, then variables values can be expanded using these values. Overwrites values-file if both provided.'
            cand -v 'Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command. 1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.'
            cand --ansible-verbosity 'Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command. 1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.'
            cand -P 'Additional parameters that are added to all Ansible calls for the session to create an image. This field is currently limited to the following Ansible parameters: "--extra-vars", "--forks", "--skip-tags", "--start-at-task", and "--tags". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs.'
            cand --ansible-passthrough 'Additional parameters that are added to all Ansible calls for the session to create an image. This field is currently limited to the following Ansible parameters: "--extra-vars", "--forks", "--skip-tags", "--start-at-task", and "--tags". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs.'
            cand -p 'Command to run before processing SAT file. If need to pass a command with params. Use " or ''. eg: --pre-hook "echo hello"'
            cand --pre-hook 'Command to run before processing SAT file. If need to pass a command with params. Use " or ''. eg: --pre-hook "echo hello"'
            cand -a 'Command to run immediately after processing SAT file successfully. Use " or ''. eg: --post-hook "echo hello".'
            cand --post-hook 'Command to run immediately after processing SAT file successfully. Use " or ''. eg: --post-hook "echo hello".'
            cand --do-not-reboot 'By default, nodes will restart if SAT file builds an image which is assigned to the nodes through a BOS sessiontemplate, if you do not want to reboot the nodes, then use this flag. The SAT file will be processeed as usual and different elements created but the nodes won''t reboot. This means, you will have to run ''manta apply template'' command with the sessoin_template created'''
            cand -w 'Watch logs. Hooks stdout to see container running ansible scripts'
            cand --watch-logs 'Watch logs. Hooks stdout to see container running ansible scripts'
            cand -i 'Only process `configurations` and `images` sections in SAT file. The `session_templates` section will be ignored.'
            cand --image-only 'Only process `configurations` and `images` sections in SAT file. The `session_templates` section will be ignored.'
            cand -s 'Only process `configurations` and `session_templates` sections in SAT file. The `images` section will be ignored.'
            cand --sessiontemplate-only 'Only process `configurations` and `session_templates` sections in SAT file. The `images` section will be ignored.'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;boot'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand nodes 'Update the boot parameters (boot image id, runtime configuration and kernel parameters) for a list of nodes. The boot image could be specified by either image id or the configuration name used to create the image id. eg: manta apply boot nodes --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>'
            cand cluster 'Update the boot parameters (boot image id, runtime configuration and kernel params) for all nodes in a cluster. The boot image could be specified by either image id or the configuration name used to create the image id. eg: manta apply boot cluster --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;boot;nodes'= {
            cand -i 'Image ID to boot the nodes'
            cand --boot-image 'Image ID to boot the nodes'
            cand -b 'CFS configuration name related to the image to boot the nodes. The most recent image id created using this configuration will be used to boot the nodes'
            cand --boot-image-configuration 'CFS configuration name related to the image to boot the nodes. The most recent image id created using this configuration will be used to boot the nodes'
            cand -r 'CFS configuration name to configure the nodes after booting'
            cand --runtime-configuration 'CFS configuration name to configure the nodes after booting'
            cand -k 'Kernel boot parameters to assign to the nodes while booting'
            cand --kernel-parameters 'Kernel boot parameters to assign to the nodes while booting'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -d 'Simulates the execution of the command without making any actual changes.'
            cand --dry-run 'Simulates the execution of the command without making any actual changes.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;boot;cluster'= {
            cand -i 'Image ID to boot the nodes'
            cand --boot-image 'Image ID to boot the nodes'
            cand -b 'CFS configuration name related to the image to boot the nodes. The most recent image id created using this configuration will be used to boot the nodes'
            cand --boot-image-configuration 'CFS configuration name related to the image to boot the nodes. The most recent image id created using this configuration will be used to boot the nodes'
            cand -r 'CFS configuration name to configure the nodes after booting'
            cand --runtime-configuration 'CFS configuration name to configure the nodes after booting'
            cand -k 'Kernel boot parameters to assign to all cluster nodes while booting'
            cand --kernel-parameters 'Kernel boot parameters to assign to all cluster nodes while booting'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -d 'Simulates the execution of the command without making any actual changes.'
            cand --dry-run 'Simulates the execution of the command without making any actual changes.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;boot;help'= {
            cand nodes 'Update the boot parameters (boot image id, runtime configuration and kernel parameters) for a list of nodes. The boot image could be specified by either image id or the configuration name used to create the image id. eg: manta apply boot nodes --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>'
            cand cluster 'Update the boot parameters (boot image id, runtime configuration and kernel params) for all nodes in a cluster. The boot image could be specified by either image id or the configuration name used to create the image id. eg: manta apply boot cluster --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;boot;help;nodes'= {
        }
        &'manta;apply;boot;help;cluster'= {
        }
        &'manta;apply;boot;help;help'= {
        }
        &'manta;apply;session'= {
            cand -n 'Session name'
            cand --name 'Session name'
            cand -p 'Playbook YAML file name. eg (site.yml)'
            cand --playbook-name 'Playbook YAML file name. eg (site.yml)'
            cand -r 'Repo path. The path with a git repo and an ansible-playbook to configure the CFS image'
            cand --repo-path 'Repo path. The path with a git repo and an ansible-playbook to configure the CFS image'
            cand -v 'Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command. 1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.'
            cand --ansible-verbosity 'Ansible verbosity. The verbose mode to use in the call to the ansible-playbook command. 1 = -v, 2 = -vv, etc. Valid values range from 0 to 4. See the ansible-playbook help for more information.'
            cand -P 'Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: "--extra-vars", "--forks", "--skip-tags", "--start-at-task", and "--tags". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs.'
            cand --ansible-passthrough 'Additional parameters that are added to all Ansible calls for the session. This field is currently limited to the following Ansible parameters: "--extra-vars", "--forks", "--skip-tags", "--start-at-task", and "--tags". WARNING: Parameters passed to Ansible in this way should be used with caution. State will not be recorded for components when using these flags to avoid incorrect reporting of partial playbook runs.'
            cand -l 'Ansible limit. Target xnames to the CFS session. Note: ansible-limit must be a subset of hsm-group if both parameters are provided'
            cand --ansible-limit 'Ansible limit. Target xnames to the CFS session. Note: ansible-limit must be a subset of hsm-group if both parameters are provided'
            cand -H 'hsm group name'
            cand --hsm-group 'hsm group name'
            cand -w 'Watch logs. Hooks stdout to see container running ansible scripts'
            cand --watch-logs 'Watch logs. Hooks stdout to see container running ansible scripts'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;ephemeral-environment'= {
            cand -i 'Image ID to use as a container image'
            cand --image-id 'Image ID to use as a container image'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;template'= {
            cand -n 'Name of the Session'
            cand --name 'Name of the Session'
            cand -o 'An operation to perform on Components in this Session. Boot Applies the Template to the Components and boots/reboots if necessary. Reboot Applies the Template to the Components; guarantees a reboot. Shutdown Power down Components that are on'
            cand --operation 'An operation to perform on Components in this Session. Boot Applies the Template to the Components and boots/reboots if necessary. Reboot Applies the Template to the Components; guarantees a reboot. Shutdown Power down Components that are on'
            cand -t 'Name of the Session Template'
            cand --template 'Name of the Session Template'
            cand -l 'A comma-separated list of nodes, groups, or roles to which the Session will be limited. Components are treated as OR operations unless preceded by ''&'' for AND or ''!'' for NOT'
            cand --limit 'A comma-separated list of nodes, groups, or roles to which the Session will be limited. Components are treated as OR operations unless preceded by ''&'' for AND or ''!'' for NOT'
            cand -i 'Set to include nodes that have been disabled as indicated in the Hardware State Manager (HSM)'
            cand --include-disabled 'Set to include nodes that have been disabled as indicated in the Hardware State Manager (HSM)'
            cand -d 'Simulates the execution of the command without making any actual changes.'
            cand --dry-run 'Simulates the execution of the command without making any actual changes.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;apply;help'= {
            cand hw-configuration 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
            cand configuration 'DEPRECATED - Please use ''manta apply sat-file'' command instead. Create a CFS configuration'
            cand sat-file 'Process a SAT file and creates the configurations, images, boot parameters and runtime configurations. If runtime configuration and boot parameters are defined, then, reboots the nodes to configure. The ansible container for the session building the image will remain running after an Ansible failure.  The container will remain running for a number of seconds specified by the ''debug_wait_time options'''
            cand boot 'Change boot operations'
            cand session 'Runs the ansible script in local directory against HSM group or xnames. Note: the local repo must alrady exists in Shasta VCS'
            cand ephemeral-environment 'Returns a hostname use can ssh with the image ID provided. This call is async which means, the user will have to wait a few seconds for the environment to be ready, normally, this takes a few seconds.'
            cand template 'Create a new BOS session from an existing BOS sessiontemplate'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;apply;help;hw-configuration'= {
            cand cluster 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
        }
        &'manta;apply;help;hw-configuration;cluster'= {
        }
        &'manta;apply;help;configuration'= {
        }
        &'manta;apply;help;sat-file'= {
        }
        &'manta;apply;help;boot'= {
            cand nodes 'Update the boot parameters (boot image id, runtime configuration and kernel parameters) for a list of nodes. The boot image could be specified by either image id or the configuration name used to create the image id. eg: manta apply boot nodes --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>'
            cand cluster 'Update the boot parameters (boot image id, runtime configuration and kernel params) for all nodes in a cluster. The boot image could be specified by either image id or the configuration name used to create the image id. eg: manta apply boot cluster --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>'
        }
        &'manta;apply;help;boot;nodes'= {
        }
        &'manta;apply;help;boot;cluster'= {
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
            cand kernel-parameters 'Delete kernel parameters'
            cand session 'Deletes a session. For ''image'' sessions, it also removes the associated image. For ''dynamic'' sessions, it sets the ''error count'' to its maximum value.'
            cand images 'WIP - Deletes a list of images.'
            cand hw-component 'WIP - Remove hw components from a cluster'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;delete;kernel-parameters'= {
            cand -x 'Comma separated list of nodes to set runtime configuration. eg ''x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'''
            cand --xnames 'Comma separated list of nodes to set runtime configuration. eg ''x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'''
            cand -H 'Cluster to set runtime configuration'
            cand --hsm-group 'Cluster to set runtime configuration'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;delete;session'= {
            cand -d 'Simulates the execution of the command without making any actual changes.'
            cand --dry-run 'Simulates the execution of the command without making any actual changes.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;delete;images'= {
            cand -d 'Simulates the execution of the command without making any actual changes.'
            cand --dry-run 'Simulates the execution of the command without making any actual changes.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;delete;hw-component'= {
            cand -P 'Pattern'
            cand --pattern 'Pattern'
            cand -t 'Target cluster name. This is the name of the cluster the pattern is applying to (resources move from here).'
            cand --target-cluster 'Target cluster name. This is the name of the cluster the pattern is applying to (resources move from here).'
            cand -p 'Parent cluster name. The parent cluster is the one receiving resources from the target cluster (resources move here).'
            cand --parent-cluster 'Parent cluster name. The parent cluster is the one receiving resources from the target cluster (resources move here).'
            cand -x 'No dry-run, actually change the status of the system. The default for this command is a dry-run.'
            cand --no-dryrun 'No dry-run, actually change the status of the system. The default for this command is a dry-run.'
            cand -d 'Delete the HSM group if empty after this action.'
            cand --delete-hsm-group 'Delete the HSM group if empty after this action.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;delete;help'= {
            cand kernel-parameters 'Delete kernel parameters'
            cand session 'Deletes a session. For ''image'' sessions, it also removes the associated image. For ''dynamic'' sessions, it sets the ''error count'' to its maximum value.'
            cand images 'WIP - Deletes a list of images.'
            cand hw-component 'WIP - Remove hw components from a cluster'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;delete;help;kernel-parameters'= {
        }
        &'manta;delete;help;session'= {
        }
        &'manta;delete;help;images'= {
        }
        &'manta;delete;help;hw-component'= {
        }
        &'manta;delete;help;help'= {
        }
        &'manta;migrate'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand vCluster 'WIP - Migrate vCluster'
            cand nodes 'Migrate nodes across vClusters'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;migrate;vCluster'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand backup 'Backup the configuration (BOS, CFS, image and HSM group) of a given vCluster/BOS session template.'
            cand restore 'MIGRATE RESTORE of all the nodes in a HSM group. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted. eg: manta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;migrate;vCluster;backup'= {
            cand -b 'BOS Sessiontemplate to use to derive CFS, boot parameters and HSM group'
            cand --bos 'BOS Sessiontemplate to use to derive CFS, boot parameters and HSM group'
            cand -d 'Destination folder to store the backup on'
            cand --destination 'Destination folder to store the backup on'
            cand -p 'Command to run before doing the backup. If need to pass a command with params. Use " or ''. eg: --pre-hook "echo hello"'
            cand --pre-hook 'Command to run before doing the backup. If need to pass a command with params. Use " or ''. eg: --pre-hook "echo hello"'
            cand -a 'Command to run immediately after the backup is completed successfully. Use " or ''. eg: --post-hook "echo hello".'
            cand --post-hook 'Command to run immediately after the backup is completed successfully. Use " or ''. eg: --post-hook "echo hello".'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;migrate;vCluster;restore'= {
            cand -b 'BOS session template of the cluster backed previously with migrate backup'
            cand --bos-file 'BOS session template of the cluster backed previously with migrate backup'
            cand -c 'CFS session template of the cluster backed previously with migrate backup'
            cand --cfs-file 'CFS session template of the cluster backed previously with migrate backup'
            cand -j 'HSM group description file of the cluster backed previously with migrate backup'
            cand --hsm-file 'HSM group description file of the cluster backed previously with migrate backup'
            cand -m 'IMS file backed previously with migrate backup'
            cand --ims-file 'IMS file backed previously with migrate backup'
            cand -i 'Path where the image files are stored.'
            cand --image-dir 'Path where the image files are stored.'
            cand -p 'Command to run before doing the backup. If need to pass a command with params. Use " or ''. eg: --pre-hook "echo hello"'
            cand --pre-hook 'Command to run before doing the backup. If need to pass a command with params. Use " or ''. eg: --pre-hook "echo hello"'
            cand -a 'Command to run immediately after the backup is completed successfully. Use " or ''. eg: --pre-hook "echo hello".'
            cand --post-hook 'Command to run immediately after the backup is completed successfully. Use " or ''. eg: --pre-hook "echo hello".'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;migrate;vCluster;help'= {
            cand backup 'Backup the configuration (BOS, CFS, image and HSM group) of a given vCluster/BOS session template.'
            cand restore 'MIGRATE RESTORE of all the nodes in a HSM group. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted. eg: manta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;migrate;vCluster;help;backup'= {
        }
        &'manta;migrate;vCluster;help;restore'= {
        }
        &'manta;migrate;vCluster;help;help'= {
        }
        &'manta;migrate;nodes'= {
            cand -f 'The name of the source vCluster from which the compute nodes will be moved.'
            cand --from 'The name of the source vCluster from which the compute nodes will be moved.'
            cand -t 'The name of the target vCluster to which the compute nodes will be moved.'
            cand --to 'The name of the target vCluster to which the compute nodes will be moved.'
            cand -d 'Simulates the execution of the command without making any actual changes.'
            cand --dry-run 'Simulates the execution of the command without making any actual changes.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;migrate;help'= {
            cand vCluster 'WIP - Migrate vCluster'
            cand nodes 'Migrate nodes across vClusters'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;migrate;help;vCluster'= {
            cand backup 'Backup the configuration (BOS, CFS, image and HSM group) of a given vCluster/BOS session template.'
            cand restore 'MIGRATE RESTORE of all the nodes in a HSM group. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted. eg: manta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>'
        }
        &'manta;migrate;help;vCluster;backup'= {
        }
        &'manta;migrate;help;vCluster;restore'= {
        }
        &'manta;migrate;help;nodes'= {
        }
        &'manta;migrate;help;help'= {
        }
        &'manta;power'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand on 'Command to power on cluster/node'
            cand off 'Command to power off cluster/node'
            cand reset 'Command to power reset cluster/node'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;on'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand cluster 'Command to power on all nodes in a cluster'
            cand nodes 'Command to power on a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;on;cluster'= {
            cand -R 'reason to power on'
            cand --reason 'reason to power on'
            cand -o 'Output format.'
            cand --output 'Output format.'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;on;nodes'= {
            cand -o 'Output format.'
            cand --output 'Output format.'
            cand -r 'Input nodes in regex format.'
            cand --regex 'Input nodes in regex format.'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;on;help'= {
            cand cluster 'Command to power on all nodes in a cluster'
            cand nodes 'Command to power on a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
            cand help 'Print this message or the help of the given subcommand(s)'
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
            cand cluster 'Command to power off all nodes in a cluster'
            cand nodes 'Command to power off a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;off;cluster'= {
            cand -R 'reason to power off'
            cand --reason 'reason to power off'
            cand -o 'Output format.'
            cand --output 'Output format.'
            cand -f 'force'
            cand --force 'force'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;off;nodes'= {
            cand -n 'Comma separated list of nodes'
            cand --nodes 'Comma separated list of nodes'
            cand -o 'Output format.'
            cand --output 'Output format.'
            cand -r 'Input nodes in regex format. eg ''x1003c1s.*'''
            cand --regex 'Input nodes in regex format. eg ''x1003c1s.*'''
            cand -f 'force'
            cand --force 'force'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;off;help'= {
            cand cluster 'Command to power off all nodes in a cluster'
            cand nodes 'Command to power off a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
            cand help 'Print this message or the help of the given subcommand(s)'
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
            cand cluster 'Command to power reset all nodes in a cluster'
            cand nodes 'Command to power reset a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;reset;cluster'= {
            cand -o 'Output format.'
            cand --output 'Output format.'
            cand -r 'reason to power reset'
            cand --reason 'reason to power reset'
            cand -f 'force'
            cand --force 'force'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;reset;nodes'= {
            cand -o 'Output format.'
            cand --output 'Output format.'
            cand -r 'Input nodes in regex format.'
            cand --regex 'Input nodes in regex format.'
            cand -f 'force'
            cand --force 'force'
            cand -y 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand --assume-yes 'Automatic yes to prompts; assume ''yes'' as answer to all prompts and run non-interactively.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;power;reset;help'= {
            cand cluster 'Command to power reset all nodes in a cluster'
            cand nodes 'Command to power reset a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;reset;help;cluster'= {
        }
        &'manta;power;reset;help;nodes'= {
        }
        &'manta;power;reset;help;help'= {
        }
        &'manta;power;help'= {
            cand on 'Command to power on cluster/node'
            cand off 'Command to power off cluster/node'
            cand reset 'Command to power reset cluster/node'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;power;help;on'= {
            cand cluster 'Command to power on all nodes in a cluster'
            cand nodes 'Command to power on a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
        }
        &'manta;power;help;on;cluster'= {
        }
        &'manta;power;help;on;nodes'= {
        }
        &'manta;power;help;off'= {
            cand cluster 'Command to power off all nodes in a cluster'
            cand nodes 'Command to power off a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
        }
        &'manta;power;help;off;cluster'= {
        }
        &'manta;power;help;off;nodes'= {
        }
        &'manta;power;help;reset'= {
            cand cluster 'Command to power reset all nodes in a cluster'
            cand nodes 'Command to power reset a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
        }
        &'manta;power;help;reset;cluster'= {
        }
        &'manta;power;help;reset;nodes'= {
        }
        &'manta;power;help;help'= {
        }
        &'manta;log'= {
            cand -c 'Show logs most recent CFS session logs created for cluster.'
            cand --cluster 'Show logs most recent CFS session logs created for cluster.'
            cand -n 'Show logs most recent CFS session logs created for a node.'
            cand --node 'Show logs most recent CFS session logs created for a node.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;console'= {
            cand -h 'Print help'
            cand --help 'Print help'
            cand node 'Connects to a node''s console'
            cand target-ansible 'Opens an interactive session to the ansible target container of a CFS session'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;console;node'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;console;target-ansible'= {
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;console;help'= {
            cand node 'Connects to a node''s console'
            cand target-ansible 'Opens an interactive session to the ansible target container of a CFS session'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;console;help;node'= {
        }
        &'manta;console;help;target-ansible'= {
        }
        &'manta;console;help;help'= {
        }
        &'manta;validate-local-repo'= {
            cand -r 'Repo path. The path to a local a git repo related to a CFS configuration layer to test against Gitea'
            cand --repo-path 'Repo path. The path to a local a git repo related to a CFS configuration layer to test against Gitea'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;add-nodes-to-groups'= {
            cand -g 'HSM group to assign the nodes to'
            cand --group 'HSM group to assign the nodes to'
            cand -n 'List of xnames or nids. eg ''x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'' or ''nid001313,nid001314''  Host list also accepted eg ''x1003c1s7b0n[0-1],x1003c1s7b1n0'' or ''nid00131[0-9]'''
            cand --nodes 'List of xnames or nids. eg ''x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'' or ''nid001313,nid001314''  Host list also accepted eg ''x1003c1s7b0n[0-1],x1003c1s7b1n0'' or ''nid00131[0-9]'''
            cand -r 'Input nodes in regex format.'
            cand --regex 'Input nodes in regex format.'
            cand -d 'Simulates the execution of the command without making any actual changes.'
            cand --dry-run 'Simulates the execution of the command without making any actual changes.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;remove-nodes-from-groups'= {
            cand -g 'HSM group to remove the nodes from'
            cand --group 'HSM group to remove the nodes from'
            cand -n 'List of xnames or nids. eg ''x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'' or ''nid001313,nid001314''  Host list also accepted eg ''x1003c1s7b0n[0-1],x1003c1s7b1n0'' or ''nid00131[0-9]'''
            cand --nodes 'List of xnames or nids. eg ''x1003c1s7b0n0,1003c1s7b0n1,x1003c1s7b1n0'' or ''nid001313,nid001314''  Host list also accepted eg ''x1003c1s7b0n[0-1],x1003c1s7b1n0'' or ''nid00131[0-9]'''
            cand -r 'Input nodes in regex format.'
            cand --regex 'Input nodes in regex format.'
            cand -d 'Simulates the execution of the command without making any actual changes.'
            cand --dry-run 'Simulates the execution of the command without making any actual changes.'
            cand -h 'Print help'
            cand --help 'Print help'
        }
        &'manta;help'= {
            cand config 'Manta''s configuration'
            cand get 'Get information from CSM system'
            cand add 'Add hw components to cluster'
            cand apply 'Make changes to Shasta system'
            cand delete 'Deletes data'
            cand migrate 'migrate'
            cand power 'Command to submit commands related to cluster/node power management'
            cand log 'get cfs session logs'
            cand console 'Opens an interective session to a node or CFS session ansible target container'
            cand validate-local-repo 'Check all tags and HEAD information related to a local repo exists in Gitea'
            cand add-nodes-to-groups 'Add nodes to a list of groups'
            cand remove-nodes-from-groups 'Remove nodes from groups'
            cand help 'Print this message or the help of the given subcommand(s)'
        }
        &'manta;help;config'= {
            cand show 'Show config values'
            cand set 'Change config values'
            cand unset 'Reset config values'
            cand generate-autocomplete 'Generate shell auto completion script'
        }
        &'manta;help;config;show'= {
        }
        &'manta;help;config;set'= {
            cand hsm 'Set target HSM group'
            cand parent-hsm 'Set parent HSM group'
            cand site 'Set site to work on'
            cand log 'Set site to work on'
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
            cand hsm 'Unset target HSM group'
            cand parent-hsm 'Unset parent HSM group'
            cand auth 'Unset authentication token'
        }
        &'manta;help;config;unset;hsm'= {
        }
        &'manta;help;config;unset;parent-hsm'= {
        }
        &'manta;help;config;unset;auth'= {
        }
        &'manta;help;config;generate-autocomplete'= {
        }
        &'manta;help;get'= {
            cand hw-component 'Get hardware components1 for a cluster or a node'
            cand sessions 'Get information from Shasta CFS session'
            cand configurations 'Get information from Shasta CFS configuration'
            cand templates 'Get information from Shasta BOS template'
            cand cluster 'Get cluster details'
            cand nodes 'Get node details'
            cand hsm-groups 'DEPRECATED - Please do not use this command. Get HSM groups details'
            cand images 'Get image information'
            cand kernel-parameters 'Get kernel-parameters information'
        }
        &'manta;help;get;hw-component'= {
            cand cluster 'Get hw components for a cluster'
            cand node 'Get hw components for some nodes'
        }
        &'manta;help;get;hw-component;cluster'= {
        }
        &'manta;help;get;hw-component;node'= {
        }
        &'manta;help;get;sessions'= {
        }
        &'manta;help;get;configurations'= {
        }
        &'manta;help;get;templates'= {
        }
        &'manta;help;get;cluster'= {
        }
        &'manta;help;get;nodes'= {
        }
        &'manta;help;get;hsm-groups'= {
        }
        &'manta;help;get;images'= {
        }
        &'manta;help;get;kernel-parameters'= {
        }
        &'manta;help;add'= {
            cand hw-component 'WIP - Add hw components from a cluster'
            cand kernel-parameters 'Delete kernel parameters'
        }
        &'manta;help;add;hw-component'= {
        }
        &'manta;help;add;kernel-parameters'= {
        }
        &'manta;help;apply'= {
            cand hw-configuration 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
            cand configuration 'DEPRECATED - Please use ''manta apply sat-file'' command instead. Create a CFS configuration'
            cand sat-file 'Process a SAT file and creates the configurations, images, boot parameters and runtime configurations. If runtime configuration and boot parameters are defined, then, reboots the nodes to configure. The ansible container for the session building the image will remain running after an Ansible failure.  The container will remain running for a number of seconds specified by the ''debug_wait_time options'''
            cand boot 'Change boot operations'
            cand session 'Runs the ansible script in local directory against HSM group or xnames. Note: the local repo must alrady exists in Shasta VCS'
            cand ephemeral-environment 'Returns a hostname use can ssh with the image ID provided. This call is async which means, the user will have to wait a few seconds for the environment to be ready, normally, this takes a few seconds.'
            cand template 'Create a new BOS session from an existing BOS sessiontemplate'
        }
        &'manta;help;apply;hw-configuration'= {
            cand cluster 'WIP - Upscale/downscale hw components in a cluster based on user input pattern. If the cluster does not exists, then a new one will be created, otherwise, the nodes of the existing cluster will be changed according to the new configuration'
        }
        &'manta;help;apply;hw-configuration;cluster'= {
        }
        &'manta;help;apply;configuration'= {
        }
        &'manta;help;apply;sat-file'= {
        }
        &'manta;help;apply;boot'= {
            cand nodes 'Update the boot parameters (boot image id, runtime configuration and kernel parameters) for a list of nodes. The boot image could be specified by either image id or the configuration name used to create the image id. eg: manta apply boot nodes --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>'
            cand cluster 'Update the boot parameters (boot image id, runtime configuration and kernel params) for all nodes in a cluster. The boot image could be specified by either image id or the configuration name used to create the image id. eg: manta apply boot cluster --boot-image-configuration <cfs configuration name used to build an image> --runtime-configuration <cfs configuration name to apply during runtime configuration>'
        }
        &'manta;help;apply;boot;nodes'= {
        }
        &'manta;help;apply;boot;cluster'= {
        }
        &'manta;help;apply;session'= {
        }
        &'manta;help;apply;ephemeral-environment'= {
        }
        &'manta;help;apply;template'= {
        }
        &'manta;help;delete'= {
            cand kernel-parameters 'Delete kernel parameters'
            cand session 'Deletes a session. For ''image'' sessions, it also removes the associated image. For ''dynamic'' sessions, it sets the ''error count'' to its maximum value.'
            cand images 'WIP - Deletes a list of images.'
            cand hw-component 'WIP - Remove hw components from a cluster'
        }
        &'manta;help;delete;kernel-parameters'= {
        }
        &'manta;help;delete;session'= {
        }
        &'manta;help;delete;images'= {
        }
        &'manta;help;delete;hw-component'= {
        }
        &'manta;help;migrate'= {
            cand vCluster 'WIP - Migrate vCluster'
            cand nodes 'Migrate nodes across vClusters'
        }
        &'manta;help;migrate;vCluster'= {
            cand backup 'Backup the configuration (BOS, CFS, image and HSM group) of a given vCluster/BOS session template.'
            cand restore 'MIGRATE RESTORE of all the nodes in a HSM group. Boot configuration means updating the image used to boot the machine. Configuration of a node means the CFS configuration with the ansible scripts running once a node has been rebooted. eg: manta update hsm-group --boot-image <boot cfs configuration name> --desired-configuration <desired cfs configuration name>'
        }
        &'manta;help;migrate;vCluster;backup'= {
        }
        &'manta;help;migrate;vCluster;restore'= {
        }
        &'manta;help;migrate;nodes'= {
        }
        &'manta;help;power'= {
            cand on 'Command to power on cluster/node'
            cand off 'Command to power off cluster/node'
            cand reset 'Command to power reset cluster/node'
        }
        &'manta;help;power;on'= {
            cand cluster 'Command to power on all nodes in a cluster'
            cand nodes 'Command to power on a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
        }
        &'manta;help;power;on;cluster'= {
        }
        &'manta;help;power;on;nodes'= {
        }
        &'manta;help;power;off'= {
            cand cluster 'Command to power off all nodes in a cluster'
            cand nodes 'Command to power off a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
        }
        &'manta;help;power;off;cluster'= {
        }
        &'manta;help;power;off;nodes'= {
        }
        &'manta;help;power;reset'= {
            cand cluster 'Command to power reset all nodes in a cluster'
            cand nodes 'Command to power reset a group of nodes. eg: ''x1001c1s0b0n1,x1001c1s0b1n0'''
        }
        &'manta;help;power;reset;cluster'= {
        }
        &'manta;help;power;reset;nodes'= {
        }
        &'manta;help;log'= {
        }
        &'manta;help;console'= {
            cand node 'Connects to a node''s console'
            cand target-ansible 'Opens an interactive session to the ansible target container of a CFS session'
        }
        &'manta;help;console;node'= {
        }
        &'manta;help;console;target-ansible'= {
        }
        &'manta;help;validate-local-repo'= {
        }
        &'manta;help;add-nodes-to-groups'= {
        }
        &'manta;help;remove-nodes-from-groups'= {
        }
        &'manta;help;help'= {
        }
    ]
    $completions[$command]
}
