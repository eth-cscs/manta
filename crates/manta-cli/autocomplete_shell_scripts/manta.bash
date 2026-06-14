_manta() {
    local i cur prev opts cmd
    COMPREPLY=()
    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
        cur="$2"
    else
        cur="${COMP_WORDS[COMP_CWORD]}"
    fi
    prev="$3"
    cmd=""
    opts=""

    for i in "${COMP_WORDS[@]:0:COMP_CWORD}"
    do
        case "${cmd},${i}" in
            ",$1")
                cmd="manta"
                ;;
            manta,add)
                cmd="manta__subcmd__add"
                ;;
            manta,apply)
                cmd="manta__subcmd__apply"
                ;;
            manta,backup)
                cmd="manta__subcmd__backup"
                ;;
            manta,config)
                cmd="manta__subcmd__config"
                ;;
            manta,console)
                cmd="manta__subcmd__console"
                ;;
            manta,delete)
                cmd="manta__subcmd__delete"
                ;;
            manta,gen-autocomplete)
                cmd="manta__subcmd__gen__subcmd__autocomplete"
                ;;
            manta,gen-man)
                cmd="manta__subcmd__gen__subcmd__man"
                ;;
            manta,get)
                cmd="manta__subcmd__get"
                ;;
            manta,help)
                cmd="manta__subcmd__help"
                ;;
            manta,log)
                cmd="manta__subcmd__log"
                ;;
            manta,migrate)
                cmd="manta__subcmd__migrate"
                ;;
            manta,power)
                cmd="manta__subcmd__power"
                ;;
            manta,restore)
                cmd="manta__subcmd__restore"
                ;;
            manta,run)
                cmd="manta__subcmd__run"
                ;;
            manta,upgrade)
                cmd="manta__subcmd__upgrade"
                ;;
            manta__subcmd__add,boot-parameters)
                cmd="manta__subcmd__add__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__add,group)
                cmd="manta__subcmd__add__subcmd__group"
                ;;
            manta__subcmd__add,hardware)
                cmd="manta__subcmd__add__subcmd__hardware"
                ;;
            manta__subcmd__add,help)
                cmd="manta__subcmd__add__subcmd__help"
                ;;
            manta__subcmd__add,kernel-parameters)
                cmd="manta__subcmd__add__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__add,node)
                cmd="manta__subcmd__add__subcmd__node"
                ;;
            manta__subcmd__add,nodes)
                cmd="manta__subcmd__add__subcmd__nodes"
                ;;
            manta__subcmd__add,redfish-endpoint)
                cmd="manta__subcmd__add__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__add,redfish-endpoints)
                cmd="manta__subcmd__add__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__add__subcmd__help,boot-parameters)
                cmd="manta__subcmd__add__subcmd__help__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__add__subcmd__help,group)
                cmd="manta__subcmd__add__subcmd__help__subcmd__group"
                ;;
            manta__subcmd__add__subcmd__help,hardware)
                cmd="manta__subcmd__add__subcmd__help__subcmd__hardware"
                ;;
            manta__subcmd__add__subcmd__help,help)
                cmd="manta__subcmd__add__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__add__subcmd__help,kernel-parameters)
                cmd="manta__subcmd__add__subcmd__help__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__add__subcmd__help,node)
                cmd="manta__subcmd__add__subcmd__help__subcmd__node"
                ;;
            manta__subcmd__add__subcmd__help,nodes)
                cmd="manta__subcmd__add__subcmd__help__subcmd__nodes"
                ;;
            manta__subcmd__add__subcmd__help,redfish-endpoints)
                cmd="manta__subcmd__add__subcmd__help__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__apply,boot)
                cmd="manta__subcmd__apply__subcmd__boot"
                ;;
            manta__subcmd__apply,boot-parameters)
                cmd="manta__subcmd__apply__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__apply,ephemeral-environment)
                cmd="manta__subcmd__apply__subcmd__ephemeral__subcmd__environment"
                ;;
            manta__subcmd__apply,hardware)
                cmd="manta__subcmd__apply__subcmd__hardware"
                ;;
            manta__subcmd__apply,help)
                cmd="manta__subcmd__apply__subcmd__help"
                ;;
            manta__subcmd__apply,kernel-parameters)
                cmd="manta__subcmd__apply__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__apply,redfish-endpoint)
                cmd="manta__subcmd__apply__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__apply,redfish-endpoints)
                cmd="manta__subcmd__apply__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__apply,sat-file)
                cmd="manta__subcmd__apply__subcmd__sat__subcmd__file"
                ;;
            manta__subcmd__apply,template)
                cmd="manta__subcmd__apply__subcmd__template"
                ;;
            manta__subcmd__apply__subcmd__boot,group)
                cmd="manta__subcmd__apply__subcmd__boot__subcmd__group"
                ;;
            manta__subcmd__apply__subcmd__boot,help)
                cmd="manta__subcmd__apply__subcmd__boot__subcmd__help"
                ;;
            manta__subcmd__apply__subcmd__boot,nodes)
                cmd="manta__subcmd__apply__subcmd__boot__subcmd__nodes"
                ;;
            manta__subcmd__apply__subcmd__boot__subcmd__help,group)
                cmd="manta__subcmd__apply__subcmd__boot__subcmd__help__subcmd__group"
                ;;
            manta__subcmd__apply__subcmd__boot__subcmd__help,help)
                cmd="manta__subcmd__apply__subcmd__boot__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__apply__subcmd__boot__subcmd__help,nodes)
                cmd="manta__subcmd__apply__subcmd__boot__subcmd__help__subcmd__nodes"
                ;;
            manta__subcmd__apply__subcmd__hardware,group)
                cmd="manta__subcmd__apply__subcmd__hardware__subcmd__group"
                ;;
            manta__subcmd__apply__subcmd__hardware,help)
                cmd="manta__subcmd__apply__subcmd__hardware__subcmd__help"
                ;;
            manta__subcmd__apply__subcmd__hardware__subcmd__help,group)
                cmd="manta__subcmd__apply__subcmd__hardware__subcmd__help__subcmd__group"
                ;;
            manta__subcmd__apply__subcmd__hardware__subcmd__help,help)
                cmd="manta__subcmd__apply__subcmd__hardware__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__apply__subcmd__help,boot)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__boot"
                ;;
            manta__subcmd__apply__subcmd__help,boot-parameters)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__apply__subcmd__help,ephemeral-environment)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__ephemeral__subcmd__environment"
                ;;
            manta__subcmd__apply__subcmd__help,hardware)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__hardware"
                ;;
            manta__subcmd__apply__subcmd__help,help)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__apply__subcmd__help,kernel-parameters)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__apply__subcmd__help,redfish-endpoints)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__apply__subcmd__help,sat-file)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__sat__subcmd__file"
                ;;
            manta__subcmd__apply__subcmd__help,template)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__template"
                ;;
            manta__subcmd__apply__subcmd__help__subcmd__boot,group)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__boot__subcmd__group"
                ;;
            manta__subcmd__apply__subcmd__help__subcmd__boot,nodes)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__boot__subcmd__nodes"
                ;;
            manta__subcmd__apply__subcmd__help__subcmd__hardware,group)
                cmd="manta__subcmd__apply__subcmd__help__subcmd__hardware__subcmd__group"
                ;;
            manta__subcmd__backup,help)
                cmd="manta__subcmd__backup__subcmd__help"
                ;;
            manta__subcmd__backup,vcluster)
                cmd="manta__subcmd__backup__subcmd__vcluster"
                ;;
            manta__subcmd__backup__subcmd__help,help)
                cmd="manta__subcmd__backup__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__backup__subcmd__help,vcluster)
                cmd="manta__subcmd__backup__subcmd__help__subcmd__vcluster"
                ;;
            manta__subcmd__config,help)
                cmd="manta__subcmd__config__subcmd__help"
                ;;
            manta__subcmd__config,set)
                cmd="manta__subcmd__config__subcmd__set"
                ;;
            manta__subcmd__config,show)
                cmd="manta__subcmd__config__subcmd__show"
                ;;
            manta__subcmd__config,unset)
                cmd="manta__subcmd__config__subcmd__unset"
                ;;
            manta__subcmd__config__subcmd__help,help)
                cmd="manta__subcmd__config__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__config__subcmd__help,set)
                cmd="manta__subcmd__config__subcmd__help__subcmd__set"
                ;;
            manta__subcmd__config__subcmd__help,show)
                cmd="manta__subcmd__config__subcmd__help__subcmd__show"
                ;;
            manta__subcmd__config__subcmd__help,unset)
                cmd="manta__subcmd__config__subcmd__help__subcmd__unset"
                ;;
            manta__subcmd__config__subcmd__help__subcmd__set,hsm)
                cmd="manta__subcmd__config__subcmd__help__subcmd__set__subcmd__hsm"
                ;;
            manta__subcmd__config__subcmd__help__subcmd__set,log)
                cmd="manta__subcmd__config__subcmd__help__subcmd__set__subcmd__log"
                ;;
            manta__subcmd__config__subcmd__help__subcmd__set,site)
                cmd="manta__subcmd__config__subcmd__help__subcmd__set__subcmd__site"
                ;;
            manta__subcmd__config__subcmd__help__subcmd__unset,auth)
                cmd="manta__subcmd__config__subcmd__help__subcmd__unset__subcmd__auth"
                ;;
            manta__subcmd__config__subcmd__help__subcmd__unset,hsm)
                cmd="manta__subcmd__config__subcmd__help__subcmd__unset__subcmd__hsm"
                ;;
            manta__subcmd__config__subcmd__set,help)
                cmd="manta__subcmd__config__subcmd__set__subcmd__help"
                ;;
            manta__subcmd__config__subcmd__set,hsm)
                cmd="manta__subcmd__config__subcmd__set__subcmd__hsm"
                ;;
            manta__subcmd__config__subcmd__set,log)
                cmd="manta__subcmd__config__subcmd__set__subcmd__log"
                ;;
            manta__subcmd__config__subcmd__set,site)
                cmd="manta__subcmd__config__subcmd__set__subcmd__site"
                ;;
            manta__subcmd__config__subcmd__set__subcmd__help,help)
                cmd="manta__subcmd__config__subcmd__set__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__config__subcmd__set__subcmd__help,hsm)
                cmd="manta__subcmd__config__subcmd__set__subcmd__help__subcmd__hsm"
                ;;
            manta__subcmd__config__subcmd__set__subcmd__help,log)
                cmd="manta__subcmd__config__subcmd__set__subcmd__help__subcmd__log"
                ;;
            manta__subcmd__config__subcmd__set__subcmd__help,site)
                cmd="manta__subcmd__config__subcmd__set__subcmd__help__subcmd__site"
                ;;
            manta__subcmd__config__subcmd__unset,auth)
                cmd="manta__subcmd__config__subcmd__unset__subcmd__auth"
                ;;
            manta__subcmd__config__subcmd__unset,help)
                cmd="manta__subcmd__config__subcmd__unset__subcmd__help"
                ;;
            manta__subcmd__config__subcmd__unset,hsm)
                cmd="manta__subcmd__config__subcmd__unset__subcmd__hsm"
                ;;
            manta__subcmd__config__subcmd__unset__subcmd__help,auth)
                cmd="manta__subcmd__config__subcmd__unset__subcmd__help__subcmd__auth"
                ;;
            manta__subcmd__config__subcmd__unset__subcmd__help,help)
                cmd="manta__subcmd__config__subcmd__unset__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__config__subcmd__unset__subcmd__help,hsm)
                cmd="manta__subcmd__config__subcmd__unset__subcmd__help__subcmd__hsm"
                ;;
            manta__subcmd__console,help)
                cmd="manta__subcmd__console__subcmd__help"
                ;;
            manta__subcmd__console,node)
                cmd="manta__subcmd__console__subcmd__node"
                ;;
            manta__subcmd__console,target-ansible)
                cmd="manta__subcmd__console__subcmd__target__subcmd__ansible"
                ;;
            manta__subcmd__console__subcmd__help,help)
                cmd="manta__subcmd__console__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__console__subcmd__help,node)
                cmd="manta__subcmd__console__subcmd__help__subcmd__node"
                ;;
            manta__subcmd__console__subcmd__help,target-ansible)
                cmd="manta__subcmd__console__subcmd__help__subcmd__target__subcmd__ansible"
                ;;
            manta__subcmd__delete,boot-parameters)
                cmd="manta__subcmd__delete__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__delete,configurations)
                cmd="manta__subcmd__delete__subcmd__configurations"
                ;;
            manta__subcmd__delete,group)
                cmd="manta__subcmd__delete__subcmd__group"
                ;;
            manta__subcmd__delete,hardware)
                cmd="manta__subcmd__delete__subcmd__hardware"
                ;;
            manta__subcmd__delete,help)
                cmd="manta__subcmd__delete__subcmd__help"
                ;;
            manta__subcmd__delete,images)
                cmd="manta__subcmd__delete__subcmd__images"
                ;;
            manta__subcmd__delete,kernel-parameters)
                cmd="manta__subcmd__delete__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__delete,node)
                cmd="manta__subcmd__delete__subcmd__node"
                ;;
            manta__subcmd__delete,nodes)
                cmd="manta__subcmd__delete__subcmd__nodes"
                ;;
            manta__subcmd__delete,redfish-endpoint)
                cmd="manta__subcmd__delete__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__delete,redfish-endpoints)
                cmd="manta__subcmd__delete__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__delete,session)
                cmd="manta__subcmd__delete__subcmd__session"
                ;;
            manta__subcmd__delete__subcmd__help,boot-parameters)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__delete__subcmd__help,configurations)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__configurations"
                ;;
            manta__subcmd__delete__subcmd__help,group)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__group"
                ;;
            manta__subcmd__delete__subcmd__help,hardware)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__hardware"
                ;;
            manta__subcmd__delete__subcmd__help,help)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__delete__subcmd__help,images)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__images"
                ;;
            manta__subcmd__delete__subcmd__help,kernel-parameters)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__delete__subcmd__help,node)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__node"
                ;;
            manta__subcmd__delete__subcmd__help,nodes)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__nodes"
                ;;
            manta__subcmd__delete__subcmd__help,redfish-endpoints)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__delete__subcmd__help,session)
                cmd="manta__subcmd__delete__subcmd__help__subcmd__session"
                ;;
            manta__subcmd__get,boot-parameters)
                cmd="manta__subcmd__get__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__get,configurations)
                cmd="manta__subcmd__get__subcmd__configurations"
                ;;
            manta__subcmd__get,group-nodes)
                cmd="manta__subcmd__get__subcmd__group__subcmd__nodes"
                ;;
            manta__subcmd__get,groups)
                cmd="manta__subcmd__get__subcmd__groups"
                ;;
            manta__subcmd__get,hardware)
                cmd="manta__subcmd__get__subcmd__hardware"
                ;;
            manta__subcmd__get,help)
                cmd="manta__subcmd__get__subcmd__help"
                ;;
            manta__subcmd__get,images)
                cmd="manta__subcmd__get__subcmd__images"
                ;;
            manta__subcmd__get,kernel-parameters)
                cmd="manta__subcmd__get__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__get,nodes)
                cmd="manta__subcmd__get__subcmd__nodes"
                ;;
            manta__subcmd__get,redfish-endpoints)
                cmd="manta__subcmd__get__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__get,sessions)
                cmd="manta__subcmd__get__subcmd__sessions"
                ;;
            manta__subcmd__get,templates)
                cmd="manta__subcmd__get__subcmd__templates"
                ;;
            manta__subcmd__get__subcmd__hardware,group)
                cmd="manta__subcmd__get__subcmd__hardware__subcmd__group"
                ;;
            manta__subcmd__get__subcmd__hardware,help)
                cmd="manta__subcmd__get__subcmd__hardware__subcmd__help"
                ;;
            manta__subcmd__get__subcmd__hardware,nodes)
                cmd="manta__subcmd__get__subcmd__hardware__subcmd__nodes"
                ;;
            manta__subcmd__get__subcmd__hardware__subcmd__help,group)
                cmd="manta__subcmd__get__subcmd__hardware__subcmd__help__subcmd__group"
                ;;
            manta__subcmd__get__subcmd__hardware__subcmd__help,help)
                cmd="manta__subcmd__get__subcmd__hardware__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__get__subcmd__hardware__subcmd__help,nodes)
                cmd="manta__subcmd__get__subcmd__hardware__subcmd__help__subcmd__nodes"
                ;;
            manta__subcmd__get__subcmd__help,boot-parameters)
                cmd="manta__subcmd__get__subcmd__help__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__get__subcmd__help,configurations)
                cmd="manta__subcmd__get__subcmd__help__subcmd__configurations"
                ;;
            manta__subcmd__get__subcmd__help,group-nodes)
                cmd="manta__subcmd__get__subcmd__help__subcmd__group__subcmd__nodes"
                ;;
            manta__subcmd__get__subcmd__help,groups)
                cmd="manta__subcmd__get__subcmd__help__subcmd__groups"
                ;;
            manta__subcmd__get__subcmd__help,hardware)
                cmd="manta__subcmd__get__subcmd__help__subcmd__hardware"
                ;;
            manta__subcmd__get__subcmd__help,help)
                cmd="manta__subcmd__get__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__get__subcmd__help,images)
                cmd="manta__subcmd__get__subcmd__help__subcmd__images"
                ;;
            manta__subcmd__get__subcmd__help,kernel-parameters)
                cmd="manta__subcmd__get__subcmd__help__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__get__subcmd__help,nodes)
                cmd="manta__subcmd__get__subcmd__help__subcmd__nodes"
                ;;
            manta__subcmd__get__subcmd__help,redfish-endpoints)
                cmd="manta__subcmd__get__subcmd__help__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__get__subcmd__help,sessions)
                cmd="manta__subcmd__get__subcmd__help__subcmd__sessions"
                ;;
            manta__subcmd__get__subcmd__help,templates)
                cmd="manta__subcmd__get__subcmd__help__subcmd__templates"
                ;;
            manta__subcmd__get__subcmd__help__subcmd__hardware,group)
                cmd="manta__subcmd__get__subcmd__help__subcmd__hardware__subcmd__group"
                ;;
            manta__subcmd__get__subcmd__help__subcmd__hardware,nodes)
                cmd="manta__subcmd__get__subcmd__help__subcmd__hardware__subcmd__nodes"
                ;;
            manta__subcmd__help,add)
                cmd="manta__subcmd__help__subcmd__add"
                ;;
            manta__subcmd__help,apply)
                cmd="manta__subcmd__help__subcmd__apply"
                ;;
            manta__subcmd__help,backup)
                cmd="manta__subcmd__help__subcmd__backup"
                ;;
            manta__subcmd__help,config)
                cmd="manta__subcmd__help__subcmd__config"
                ;;
            manta__subcmd__help,console)
                cmd="manta__subcmd__help__subcmd__console"
                ;;
            manta__subcmd__help,delete)
                cmd="manta__subcmd__help__subcmd__delete"
                ;;
            manta__subcmd__help,gen-autocomplete)
                cmd="manta__subcmd__help__subcmd__gen__subcmd__autocomplete"
                ;;
            manta__subcmd__help,gen-man)
                cmd="manta__subcmd__help__subcmd__gen__subcmd__man"
                ;;
            manta__subcmd__help,get)
                cmd="manta__subcmd__help__subcmd__get"
                ;;
            manta__subcmd__help,help)
                cmd="manta__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__help,log)
                cmd="manta__subcmd__help__subcmd__log"
                ;;
            manta__subcmd__help,migrate)
                cmd="manta__subcmd__help__subcmd__migrate"
                ;;
            manta__subcmd__help,power)
                cmd="manta__subcmd__help__subcmd__power"
                ;;
            manta__subcmd__help,restore)
                cmd="manta__subcmd__help__subcmd__restore"
                ;;
            manta__subcmd__help,run)
                cmd="manta__subcmd__help__subcmd__run"
                ;;
            manta__subcmd__help,upgrade)
                cmd="manta__subcmd__help__subcmd__upgrade"
                ;;
            manta__subcmd__help__subcmd__add,boot-parameters)
                cmd="manta__subcmd__help__subcmd__add__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__help__subcmd__add,group)
                cmd="manta__subcmd__help__subcmd__add__subcmd__group"
                ;;
            manta__subcmd__help__subcmd__add,hardware)
                cmd="manta__subcmd__help__subcmd__add__subcmd__hardware"
                ;;
            manta__subcmd__help__subcmd__add,kernel-parameters)
                cmd="manta__subcmd__help__subcmd__add__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__help__subcmd__add,node)
                cmd="manta__subcmd__help__subcmd__add__subcmd__node"
                ;;
            manta__subcmd__help__subcmd__add,nodes)
                cmd="manta__subcmd__help__subcmd__add__subcmd__nodes"
                ;;
            manta__subcmd__help__subcmd__add,redfish-endpoints)
                cmd="manta__subcmd__help__subcmd__add__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__help__subcmd__apply,boot)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__boot"
                ;;
            manta__subcmd__help__subcmd__apply,boot-parameters)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__help__subcmd__apply,ephemeral-environment)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__ephemeral__subcmd__environment"
                ;;
            manta__subcmd__help__subcmd__apply,hardware)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__hardware"
                ;;
            manta__subcmd__help__subcmd__apply,kernel-parameters)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__help__subcmd__apply,redfish-endpoints)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__help__subcmd__apply,sat-file)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__sat__subcmd__file"
                ;;
            manta__subcmd__help__subcmd__apply,template)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__template"
                ;;
            manta__subcmd__help__subcmd__apply__subcmd__boot,group)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__boot__subcmd__group"
                ;;
            manta__subcmd__help__subcmd__apply__subcmd__boot,nodes)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__boot__subcmd__nodes"
                ;;
            manta__subcmd__help__subcmd__apply__subcmd__hardware,group)
                cmd="manta__subcmd__help__subcmd__apply__subcmd__hardware__subcmd__group"
                ;;
            manta__subcmd__help__subcmd__backup,vcluster)
                cmd="manta__subcmd__help__subcmd__backup__subcmd__vcluster"
                ;;
            manta__subcmd__help__subcmd__config,set)
                cmd="manta__subcmd__help__subcmd__config__subcmd__set"
                ;;
            manta__subcmd__help__subcmd__config,show)
                cmd="manta__subcmd__help__subcmd__config__subcmd__show"
                ;;
            manta__subcmd__help__subcmd__config,unset)
                cmd="manta__subcmd__help__subcmd__config__subcmd__unset"
                ;;
            manta__subcmd__help__subcmd__config__subcmd__set,hsm)
                cmd="manta__subcmd__help__subcmd__config__subcmd__set__subcmd__hsm"
                ;;
            manta__subcmd__help__subcmd__config__subcmd__set,log)
                cmd="manta__subcmd__help__subcmd__config__subcmd__set__subcmd__log"
                ;;
            manta__subcmd__help__subcmd__config__subcmd__set,site)
                cmd="manta__subcmd__help__subcmd__config__subcmd__set__subcmd__site"
                ;;
            manta__subcmd__help__subcmd__config__subcmd__unset,auth)
                cmd="manta__subcmd__help__subcmd__config__subcmd__unset__subcmd__auth"
                ;;
            manta__subcmd__help__subcmd__config__subcmd__unset,hsm)
                cmd="manta__subcmd__help__subcmd__config__subcmd__unset__subcmd__hsm"
                ;;
            manta__subcmd__help__subcmd__console,node)
                cmd="manta__subcmd__help__subcmd__console__subcmd__node"
                ;;
            manta__subcmd__help__subcmd__console,target-ansible)
                cmd="manta__subcmd__help__subcmd__console__subcmd__target__subcmd__ansible"
                ;;
            manta__subcmd__help__subcmd__delete,boot-parameters)
                cmd="manta__subcmd__help__subcmd__delete__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__help__subcmd__delete,configurations)
                cmd="manta__subcmd__help__subcmd__delete__subcmd__configurations"
                ;;
            manta__subcmd__help__subcmd__delete,group)
                cmd="manta__subcmd__help__subcmd__delete__subcmd__group"
                ;;
            manta__subcmd__help__subcmd__delete,hardware)
                cmd="manta__subcmd__help__subcmd__delete__subcmd__hardware"
                ;;
            manta__subcmd__help__subcmd__delete,images)
                cmd="manta__subcmd__help__subcmd__delete__subcmd__images"
                ;;
            manta__subcmd__help__subcmd__delete,kernel-parameters)
                cmd="manta__subcmd__help__subcmd__delete__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__help__subcmd__delete,node)
                cmd="manta__subcmd__help__subcmd__delete__subcmd__node"
                ;;
            manta__subcmd__help__subcmd__delete,nodes)
                cmd="manta__subcmd__help__subcmd__delete__subcmd__nodes"
                ;;
            manta__subcmd__help__subcmd__delete,redfish-endpoints)
                cmd="manta__subcmd__help__subcmd__delete__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__help__subcmd__delete,session)
                cmd="manta__subcmd__help__subcmd__delete__subcmd__session"
                ;;
            manta__subcmd__help__subcmd__get,boot-parameters)
                cmd="manta__subcmd__help__subcmd__get__subcmd__boot__subcmd__parameters"
                ;;
            manta__subcmd__help__subcmd__get,configurations)
                cmd="manta__subcmd__help__subcmd__get__subcmd__configurations"
                ;;
            manta__subcmd__help__subcmd__get,group-nodes)
                cmd="manta__subcmd__help__subcmd__get__subcmd__group__subcmd__nodes"
                ;;
            manta__subcmd__help__subcmd__get,groups)
                cmd="manta__subcmd__help__subcmd__get__subcmd__groups"
                ;;
            manta__subcmd__help__subcmd__get,hardware)
                cmd="manta__subcmd__help__subcmd__get__subcmd__hardware"
                ;;
            manta__subcmd__help__subcmd__get,images)
                cmd="manta__subcmd__help__subcmd__get__subcmd__images"
                ;;
            manta__subcmd__help__subcmd__get,kernel-parameters)
                cmd="manta__subcmd__help__subcmd__get__subcmd__kernel__subcmd__parameters"
                ;;
            manta__subcmd__help__subcmd__get,nodes)
                cmd="manta__subcmd__help__subcmd__get__subcmd__nodes"
                ;;
            manta__subcmd__help__subcmd__get,redfish-endpoints)
                cmd="manta__subcmd__help__subcmd__get__subcmd__redfish__subcmd__endpoints"
                ;;
            manta__subcmd__help__subcmd__get,sessions)
                cmd="manta__subcmd__help__subcmd__get__subcmd__sessions"
                ;;
            manta__subcmd__help__subcmd__get,templates)
                cmd="manta__subcmd__help__subcmd__get__subcmd__templates"
                ;;
            manta__subcmd__help__subcmd__get__subcmd__hardware,group)
                cmd="manta__subcmd__help__subcmd__get__subcmd__hardware__subcmd__group"
                ;;
            manta__subcmd__help__subcmd__get__subcmd__hardware,nodes)
                cmd="manta__subcmd__help__subcmd__get__subcmd__hardware__subcmd__nodes"
                ;;
            manta__subcmd__help__subcmd__migrate,nodes)
                cmd="manta__subcmd__help__subcmd__migrate__subcmd__nodes"
                ;;
            manta__subcmd__help__subcmd__power,off)
                cmd="manta__subcmd__help__subcmd__power__subcmd__off"
                ;;
            manta__subcmd__help__subcmd__power,on)
                cmd="manta__subcmd__help__subcmd__power__subcmd__on"
                ;;
            manta__subcmd__help__subcmd__power,reset)
                cmd="manta__subcmd__help__subcmd__power__subcmd__reset"
                ;;
            manta__subcmd__help__subcmd__power__subcmd__off,group)
                cmd="manta__subcmd__help__subcmd__power__subcmd__off__subcmd__group"
                ;;
            manta__subcmd__help__subcmd__power__subcmd__off,nodes)
                cmd="manta__subcmd__help__subcmd__power__subcmd__off__subcmd__nodes"
                ;;
            manta__subcmd__help__subcmd__power__subcmd__on,group)
                cmd="manta__subcmd__help__subcmd__power__subcmd__on__subcmd__group"
                ;;
            manta__subcmd__help__subcmd__power__subcmd__on,nodes)
                cmd="manta__subcmd__help__subcmd__power__subcmd__on__subcmd__nodes"
                ;;
            manta__subcmd__help__subcmd__power__subcmd__reset,group)
                cmd="manta__subcmd__help__subcmd__power__subcmd__reset__subcmd__group"
                ;;
            manta__subcmd__help__subcmd__power__subcmd__reset,nodes)
                cmd="manta__subcmd__help__subcmd__power__subcmd__reset__subcmd__nodes"
                ;;
            manta__subcmd__help__subcmd__restore,vcluster)
                cmd="manta__subcmd__help__subcmd__restore__subcmd__vcluster"
                ;;
            manta__subcmd__help__subcmd__run,session)
                cmd="manta__subcmd__help__subcmd__run__subcmd__session"
                ;;
            manta__subcmd__migrate,help)
                cmd="manta__subcmd__migrate__subcmd__help"
                ;;
            manta__subcmd__migrate,nodes)
                cmd="manta__subcmd__migrate__subcmd__nodes"
                ;;
            manta__subcmd__migrate__subcmd__help,help)
                cmd="manta__subcmd__migrate__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__migrate__subcmd__help,nodes)
                cmd="manta__subcmd__migrate__subcmd__help__subcmd__nodes"
                ;;
            manta__subcmd__power,help)
                cmd="manta__subcmd__power__subcmd__help"
                ;;
            manta__subcmd__power,off)
                cmd="manta__subcmd__power__subcmd__off"
                ;;
            manta__subcmd__power,on)
                cmd="manta__subcmd__power__subcmd__on"
                ;;
            manta__subcmd__power,reset)
                cmd="manta__subcmd__power__subcmd__reset"
                ;;
            manta__subcmd__power__subcmd__help,help)
                cmd="manta__subcmd__power__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__power__subcmd__help,off)
                cmd="manta__subcmd__power__subcmd__help__subcmd__off"
                ;;
            manta__subcmd__power__subcmd__help,on)
                cmd="manta__subcmd__power__subcmd__help__subcmd__on"
                ;;
            manta__subcmd__power__subcmd__help,reset)
                cmd="manta__subcmd__power__subcmd__help__subcmd__reset"
                ;;
            manta__subcmd__power__subcmd__help__subcmd__off,group)
                cmd="manta__subcmd__power__subcmd__help__subcmd__off__subcmd__group"
                ;;
            manta__subcmd__power__subcmd__help__subcmd__off,nodes)
                cmd="manta__subcmd__power__subcmd__help__subcmd__off__subcmd__nodes"
                ;;
            manta__subcmd__power__subcmd__help__subcmd__on,group)
                cmd="manta__subcmd__power__subcmd__help__subcmd__on__subcmd__group"
                ;;
            manta__subcmd__power__subcmd__help__subcmd__on,nodes)
                cmd="manta__subcmd__power__subcmd__help__subcmd__on__subcmd__nodes"
                ;;
            manta__subcmd__power__subcmd__help__subcmd__reset,group)
                cmd="manta__subcmd__power__subcmd__help__subcmd__reset__subcmd__group"
                ;;
            manta__subcmd__power__subcmd__help__subcmd__reset,nodes)
                cmd="manta__subcmd__power__subcmd__help__subcmd__reset__subcmd__nodes"
                ;;
            manta__subcmd__power__subcmd__off,group)
                cmd="manta__subcmd__power__subcmd__off__subcmd__group"
                ;;
            manta__subcmd__power__subcmd__off,help)
                cmd="manta__subcmd__power__subcmd__off__subcmd__help"
                ;;
            manta__subcmd__power__subcmd__off,nodes)
                cmd="manta__subcmd__power__subcmd__off__subcmd__nodes"
                ;;
            manta__subcmd__power__subcmd__off__subcmd__help,group)
                cmd="manta__subcmd__power__subcmd__off__subcmd__help__subcmd__group"
                ;;
            manta__subcmd__power__subcmd__off__subcmd__help,help)
                cmd="manta__subcmd__power__subcmd__off__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__power__subcmd__off__subcmd__help,nodes)
                cmd="manta__subcmd__power__subcmd__off__subcmd__help__subcmd__nodes"
                ;;
            manta__subcmd__power__subcmd__on,group)
                cmd="manta__subcmd__power__subcmd__on__subcmd__group"
                ;;
            manta__subcmd__power__subcmd__on,help)
                cmd="manta__subcmd__power__subcmd__on__subcmd__help"
                ;;
            manta__subcmd__power__subcmd__on,nodes)
                cmd="manta__subcmd__power__subcmd__on__subcmd__nodes"
                ;;
            manta__subcmd__power__subcmd__on__subcmd__help,group)
                cmd="manta__subcmd__power__subcmd__on__subcmd__help__subcmd__group"
                ;;
            manta__subcmd__power__subcmd__on__subcmd__help,help)
                cmd="manta__subcmd__power__subcmd__on__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__power__subcmd__on__subcmd__help,nodes)
                cmd="manta__subcmd__power__subcmd__on__subcmd__help__subcmd__nodes"
                ;;
            manta__subcmd__power__subcmd__reset,group)
                cmd="manta__subcmd__power__subcmd__reset__subcmd__group"
                ;;
            manta__subcmd__power__subcmd__reset,help)
                cmd="manta__subcmd__power__subcmd__reset__subcmd__help"
                ;;
            manta__subcmd__power__subcmd__reset,nodes)
                cmd="manta__subcmd__power__subcmd__reset__subcmd__nodes"
                ;;
            manta__subcmd__power__subcmd__reset__subcmd__help,group)
                cmd="manta__subcmd__power__subcmd__reset__subcmd__help__subcmd__group"
                ;;
            manta__subcmd__power__subcmd__reset__subcmd__help,help)
                cmd="manta__subcmd__power__subcmd__reset__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__power__subcmd__reset__subcmd__help,nodes)
                cmd="manta__subcmd__power__subcmd__reset__subcmd__help__subcmd__nodes"
                ;;
            manta__subcmd__restore,help)
                cmd="manta__subcmd__restore__subcmd__help"
                ;;
            manta__subcmd__restore,vcluster)
                cmd="manta__subcmd__restore__subcmd__vcluster"
                ;;
            manta__subcmd__restore__subcmd__help,help)
                cmd="manta__subcmd__restore__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__restore__subcmd__help,vcluster)
                cmd="manta__subcmd__restore__subcmd__help__subcmd__vcluster"
                ;;
            manta__subcmd__run,help)
                cmd="manta__subcmd__run__subcmd__help"
                ;;
            manta__subcmd__run,session)
                cmd="manta__subcmd__run__subcmd__session"
                ;;
            manta__subcmd__run__subcmd__help,help)
                cmd="manta__subcmd__run__subcmd__help__subcmd__help"
                ;;
            manta__subcmd__run__subcmd__help,session)
                cmd="manta__subcmd__run__subcmd__help__subcmd__session"
                ;;
            *)
                ;;
        esac
    done

    case "${cmd}" in
        manta)
            opts="-h -V --site --help --version config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 1 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add)
            opts="-h --site --help node nodes group hardware boot-parameters kernel-parameters redfish-endpoints redfish-endpoint help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__boot__subcmd__parameters)
            opts="-H -n -m -p -k -i -c -d -y -o -h --hosts --nids --macs --params --kernel --initrd --cloud-init --dry-run --assume-yes --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --hosts)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --nids)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --macs)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -m)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --params)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --kernel)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -k)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --initrd)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --cloud-init)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -c)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__group)
            opts="-l -d -n -o -h --label --description --nodes --dry-run --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --label)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --description)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --nodes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__hardware)
            opts="-P -t -p -d -c -o -h --pattern --target-cluster --target-group --parent-cluster --parent-group --dry-run --create-hsm-group --create-group --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --pattern)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -P)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target-cluster)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --parent-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --parent-cluster)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__help)
            opts="node nodes group hardware boot-parameters kernel-parameters redfish-endpoints help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__help__subcmd__boot__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__help__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__help__subcmd__hardware)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__help__subcmd__kernel__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__help__subcmd__node)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__help__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__help__subcmd__redfish__subcmd__endpoints)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__kernel__subcmd__parameters)
            opts="-n -H -o -O -y -d -h --nodes --hsm-group --group --output --overwrite --assume-yes --do-not-reboot --dry-run --site --help <PARAMS>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --nodes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hsm-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__node)
            opts="-i -g -H -a -d -o -h --id --group --hardware --arch --disabled --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -g)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hardware)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --arch)
                    COMPREPLY=($(compgen -W "X86 ARM Other" -- "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -W "X86 ARM Other" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__nodes)
            opts="-g -n -d -o -h --group --nodes --dry-run --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -g)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --nodes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__add__subcmd__redfish__subcmd__endpoints)
            opts="-i -n -H -d -f -e -u -p -U -m -M -I -r -t -o -h --id --name --hostname --domain --fqdn --enabled --user --password --use-ssdp --mac-required --macaddr --ipaddress --rediscover-on-update --template-id --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hostname)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --domain)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --fqdn)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --user)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --macaddr)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -M)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ipaddress)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -I)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --template-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply)
            opts="-h --site --help hardware sat-file boot boot-parameters redfish-endpoints redfish-endpoint kernel-parameters ephemeral-environment template help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__boot)
            opts="-h --site --help nodes group help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__boot__subcmd__parameters)
            opts="-H -p -k -i -d -o -h --hosts --params --kernel --initrd --dry-run --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --hosts)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --params)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --kernel)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -k)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --initrd)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__boot__subcmd__group)
            opts="-i -b -r -k -d -o -h --boot-image --boot-image-configuration --runtime-configuration --kernel-parameters --dry-run --output --site --help <GROUP_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --boot-image)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --boot-image-configuration)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -b)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --runtime-configuration)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -r)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --kernel-parameters)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -k)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__boot__subcmd__help)
            opts="nodes group help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__boot__subcmd__help__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__boot__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__boot__subcmd__help__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__boot__subcmd__nodes)
            opts="-i -b -r -k -d -o -h --boot-image --boot-image-configuration --runtime-configuration --kernel-parameters --dry-run --output --site --help <NODES>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --boot-image)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --boot-image-configuration)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -b)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --runtime-configuration)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -r)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --kernel-parameters)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -k)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__ephemeral__subcmd__environment)
            opts="-i -h --image-id --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --image-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__hardware)
            opts="-h --site --help group help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__hardware__subcmd__group)
            opts="-P -t -p -d -c -D -u -o -h --pattern --target-cluster --target-group --parent-cluster --parent-group --dry-run --create-target-hsm-group --create-target-group --delete-empty-parent-hsm-group --delete-empty-parent-group --unpin-nodes --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --pattern)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -P)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target-cluster)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --parent-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --parent-cluster)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__hardware__subcmd__help)
            opts="group help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__hardware__subcmd__help__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__hardware__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help)
            opts="hardware sat-file boot boot-parameters redfish-endpoints kernel-parameters ephemeral-environment template help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__boot)
            opts="nodes group"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__boot__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__boot__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__boot__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__ephemeral__subcmd__environment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__hardware)
            opts="group"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__hardware__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__kernel__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__redfish__subcmd__endpoints)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__sat__subcmd__file)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__help__subcmd__template)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__kernel__subcmd__parameters)
            opts="-n -H -d -o -h --nodes --hsm-group --group --dry-run --output --site --help <PARAMS>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --nodes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hsm-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__redfish__subcmd__endpoints)
            opts="-i -n -H -d -f -e -u -p -U -m -M -I -r -t -o -h --id --name --hostname --domain --fqdn --enabled --user --password --use-ssdp --mac-required --macaddr --ipaddress --rediscover-on-update --template-id --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hostname)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --domain)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -d)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --fqdn)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --user)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --password)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --macaddr)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -M)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ipaddress)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -I)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --template-id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__sat__subcmd__file)
            opts="-t -f -V -v -P -o -w -T -i -s -p -a -y -d -h --sat-template-file --values-file --values --reboot --ansible-verbosity --ansible-passthrough --overwrite-configuration --watch-logs --timestamps --image-only --sessiontemplate-only --pre-hook --post-hook --assume-yes --dry-run --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --sat-template-file)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                -t)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                --values-file)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                -f)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                --values)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -V)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ansible-verbosity)
                    COMPREPLY=($(compgen -W "1 2 3 4" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "1 2 3 4" -- "${cur}"))
                    return 0
                    ;;
                --ansible-passthrough)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -P)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pre-hook)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --post-hook)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__apply__subcmd__template)
            opts="-n -o -t -l -i -d -h --name --operation --template --limit --include-disabled --dry-run --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --operation)
                    COMPREPLY=($(compgen -W "reboot boot shutdown" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "reboot boot shutdown" -- "${cur}"))
                    return 0
                    ;;
                --template)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__backup)
            opts="-h --site --help vcluster help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__backup__subcmd__help)
            opts="vcluster help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__backup__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__backup__subcmd__help__subcmd__vcluster)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__backup__subcmd__vcluster)
            opts="-b -d -p -a -o -h --bos --destination --pre-hook --post-hook --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --bos)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -b)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --destination)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                -d)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                --pre-hook)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --post-hook)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config)
            opts="-h --site --help show set unset help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__help)
            opts="show set unset help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__help__subcmd__set)
            opts="hsm site log"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__help__subcmd__set__subcmd__hsm)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__help__subcmd__set__subcmd__log)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__help__subcmd__set__subcmd__site)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__help__subcmd__show)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__help__subcmd__unset)
            opts="hsm auth"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__help__subcmd__unset__subcmd__auth)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__help__subcmd__unset__subcmd__hsm)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__set)
            opts="-h --site --help hsm site log help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__set__subcmd__help)
            opts="hsm site log help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__set__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__set__subcmd__help__subcmd__hsm)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__set__subcmd__help__subcmd__log)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__set__subcmd__help__subcmd__site)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__set__subcmd__hsm)
            opts="-h --site --help <GROUP_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__set__subcmd__log)
            opts="-h --site --help error warn info debug trace"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__set__subcmd__site)
            opts="-h --site --help <SITE_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__show)
            opts="-o -h --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__unset)
            opts="-h --site --help hsm auth help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__unset__subcmd__auth)
            opts="-h --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__unset__subcmd__help)
            opts="hsm auth help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__unset__subcmd__help__subcmd__auth)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__unset__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__unset__subcmd__help__subcmd__hsm)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__config__subcmd__unset__subcmd__hsm)
            opts="-h --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__console)
            opts="-h --site --help node target-ansible help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__console__subcmd__help)
            opts="node target-ansible help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__console__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__console__subcmd__help__subcmd__node)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__console__subcmd__help__subcmd__target__subcmd__ansible)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__console__subcmd__node)
            opts="-h --site --help <XNAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__console__subcmd__target__subcmd__ansible)
            opts="-h --site --help <SESSION_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete)
            opts="-h --site --help group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints redfish-endpoint help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__boot__subcmd__parameters)
            opts="-H -o -h --hosts --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --hosts)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__configurations)
            opts="-n -s -u -o -h --configuration-name --since --until --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --configuration-name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --since)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --until)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__group)
            opts="-f -d -o -h --force --dry-run --output --site --help <GROUP_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__hardware)
            opts="-P -t -p -d -D -o -h --pattern --target-cluster --target-group --parent-cluster --parent-group --dry-run --delete-hsm-group --delete-group --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --pattern)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -P)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --target-cluster)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --parent-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --parent-cluster)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help)
            opts="group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__boot__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__configurations)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__hardware)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__images)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__kernel__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__node)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__redfish__subcmd__endpoints)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__help__subcmd__session)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__images)
            opts="-d -o -h --dry-run --output --site --help <IMAGE_IDS>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__kernel__subcmd__parameters)
            opts="-n -H -d -o -h --nodes --hsm-group --group --dry-run --output --site --help <PARAMS>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --nodes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hsm-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__node)
            opts="-o -h --output --site --help <XNAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__nodes)
            opts="-g -n -d -o -h --group --nodes --dry-run --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -g)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --nodes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__redfish__subcmd__endpoints)
            opts="-i -o -h --id --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__delete__subcmd__session)
            opts="-y -d -o -h --assume-yes --dry-run --output --site --help <SESSION_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__gen__subcmd__autocomplete)
            opts="-s -p -o -h --shell --path --print --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --shell)
                    COMPREPLY=($(compgen -W "bash zsh fish" -- "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -W "bash zsh fish" -- "${cur}"))
                    return 0
                    ;;
                --path)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                -p)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__gen__subcmd__man)
            opts="-p -o -h --path --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --path)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                -p)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get)
            opts="-h --site --help groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__boot__subcmd__parameters)
            opts="-H -n -h --hsm-group --group --nodes --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hsm-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --nodes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__configurations)
            opts="-n -p -m -l -o -H -h --name --pattern --most-recent --limit --output --hsm-group --group --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pattern)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "json" -- "${cur}"))
                    return 0
                    ;;
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hsm-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__group__subcmd__nodes)
            opts="-n -x -s -T -o -h --nids-only-one-line --xnames-only-one-line --status --summary-status --output --site --help <GROUP_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --status)
                    COMPREPLY=($(compgen -W "OFF ON READY STANDBY PENDING FAILED CONFIGURED" -- "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -W "OFF ON READY STANDBY PENDING FAILED CONFIGURED" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table table-wide json summary" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table table-wide json summary" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__groups)
            opts="-o -h --output --site --help [GROUP_NAME]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "json table" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "json table" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__hardware)
            opts="-h --site --help nodes group help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__hardware__subcmd__group)
            opts="-o -h --output --site --help <GROUP_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "json summary details pattern" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "json summary details pattern" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__hardware__subcmd__help)
            opts="nodes group help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__hardware__subcmd__help__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__hardware__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__hardware__subcmd__help__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__hardware__subcmd__nodes)
            opts="-o -h --output --site --help <NODES>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help)
            opts="groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__boot__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__configurations)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__group__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__groups)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__hardware)
            opts="nodes group"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__hardware__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__hardware__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__images)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__kernel__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__redfish__subcmd__endpoints)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__sessions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__help__subcmd__templates)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__images)
            opts="-i -p -m -l -h --id --pattern --most-recent --limit --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --pattern)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__kernel__subcmd__parameters)
            opts="-n -H -f -o -h --nodes --hsm-group --group --filter --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --nodes)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hsm-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --filter)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__nodes)
            opts="-n -s -T -S -o -h --nids-only-one-line --status --summary-status --include-siblings --output --site --help <NODES>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --status)
                    COMPREPLY=($(compgen -W "OFF ON READY STANDBY PENDING FAILED CONFIGURED" -- "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -W "OFF ON READY STANDBY PENDING FAILED CONFIGURED" -- "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table table-wide json summary" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table table-wide json summary" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__redfish__subcmd__endpoints)
            opts="-i -f -u -m -I -o -h --id --fqdn --uuid --macaddr --ipaddress --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --id)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -i)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --fqdn)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --uuid)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -u)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --macaddr)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -m)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ipaddress)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -I)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__sessions)
            opts="-n -a -A -t -s -m -l -o -x -H -h --name --min-age --max-age --type --status --most-recent --limit --output --xnames --hsm-group --group --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --min-age)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --max-age)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -A)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --type)
                    COMPREPLY=($(compgen -W "image runtime" -- "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -W "image runtime" -- "${cur}"))
                    return 0
                    ;;
                --status)
                    COMPREPLY=($(compgen -W "pending running complete" -- "${cur}"))
                    return 0
                    ;;
                -s)
                    COMPREPLY=($(compgen -W "pending running complete" -- "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "json" -- "${cur}"))
                    return 0
                    ;;
                --xnames)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -x)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hsm-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__get__subcmd__templates)
            opts="-n -m -l -H -o -h --name --most-recent --limit --hsm-group --group --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hsm-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "json table" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "json table" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help)
            opts="config get add apply delete migrate backup restore run power log console gen-autocomplete gen-man upgrade help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__add)
            opts="node nodes group hardware boot-parameters kernel-parameters redfish-endpoints"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__add__subcmd__boot__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__add__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__add__subcmd__hardware)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__add__subcmd__kernel__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__add__subcmd__node)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__add__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__add__subcmd__redfish__subcmd__endpoints)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply)
            opts="hardware sat-file boot boot-parameters redfish-endpoints kernel-parameters ephemeral-environment template"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__boot)
            opts="nodes group"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__boot__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__boot__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__boot__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__ephemeral__subcmd__environment)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__hardware)
            opts="group"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__hardware__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__kernel__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__redfish__subcmd__endpoints)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__sat__subcmd__file)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__apply__subcmd__template)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__backup)
            opts="vcluster"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__backup__subcmd__vcluster)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__config)
            opts="show set unset"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__config__subcmd__set)
            opts="hsm site log"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__config__subcmd__set__subcmd__hsm)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__config__subcmd__set__subcmd__log)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__config__subcmd__set__subcmd__site)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__config__subcmd__show)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__config__subcmd__unset)
            opts="hsm auth"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__config__subcmd__unset__subcmd__auth)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__config__subcmd__unset__subcmd__hsm)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__console)
            opts="node target-ansible"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__console__subcmd__node)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__console__subcmd__target__subcmd__ansible)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete)
            opts="group node nodes kernel-parameters boot-parameters configurations session images hardware redfish-endpoints"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete__subcmd__boot__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete__subcmd__configurations)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete__subcmd__hardware)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete__subcmd__images)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete__subcmd__kernel__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete__subcmd__node)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete__subcmd__redfish__subcmd__endpoints)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__delete__subcmd__session)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__gen__subcmd__autocomplete)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__gen__subcmd__man)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get)
            opts="groups hardware sessions configurations templates group-nodes nodes images boot-parameters kernel-parameters redfish-endpoints"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__boot__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__configurations)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__group__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__groups)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__hardware)
            opts="nodes group"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__hardware__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__hardware__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__images)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__kernel__subcmd__parameters)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__redfish__subcmd__endpoints)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__sessions)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__get__subcmd__templates)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__log)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__migrate)
            opts="nodes"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__migrate__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__power)
            opts="on off reset"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__power__subcmd__off)
            opts="group nodes"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__power__subcmd__off__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__power__subcmd__off__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__power__subcmd__on)
            opts="group nodes"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__power__subcmd__on__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__power__subcmd__on__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__power__subcmd__reset)
            opts="group nodes"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__power__subcmd__reset__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__power__subcmd__reset__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__restore)
            opts="vcluster"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__restore__subcmd__vcluster)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__run)
            opts="session"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__run__subcmd__session)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__help__subcmd__upgrade)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__log)
            opts="-t -h --timestamps --site --help [TARGET]"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__migrate)
            opts="-h --site --help nodes help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__migrate__subcmd__help)
            opts="nodes help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__migrate__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__migrate__subcmd__help__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__migrate__subcmd__nodes)
            opts="-f -t -d -o -h --from --to --dry-run --output --site --help <XNAMES>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --from)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -f)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --to)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -t)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power)
            opts="-h --site --help on off reset help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help)
            opts="on off reset help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help__subcmd__off)
            opts="group nodes"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help__subcmd__off__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help__subcmd__off__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help__subcmd__on)
            opts="group nodes"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help__subcmd__on__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help__subcmd__on__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help__subcmd__reset)
            opts="group nodes"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help__subcmd__reset__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__help__subcmd__reset__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__off)
            opts="-h --site --help group nodes help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__off__subcmd__group)
            opts="-g -R -y -o -h --graceful --reason --assume-yes --no-wait --output --site --help <GROUP_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --reason)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -R)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__off__subcmd__help)
            opts="group nodes help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__off__subcmd__help__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__off__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__off__subcmd__help__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__off__subcmd__nodes)
            opts="-g -y -o -h --graceful --assume-yes --no-wait --output --site --help <NODES>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__on)
            opts="-h --site --help group nodes help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__on__subcmd__group)
            opts="-R -y -o -h --reason --assume-yes --no-wait --output --site --help <GROUP_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --reason)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -R)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__on__subcmd__help)
            opts="group nodes help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__on__subcmd__help__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__on__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__on__subcmd__help__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__on__subcmd__nodes)
            opts="-y -o -h --assume-yes --no-wait --output --site --help <NODES>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__reset)
            opts="-h --site --help group nodes help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__reset__subcmd__group)
            opts="-g -y -o -r -h --graceful --assume-yes --no-wait --output --reason --site --help <GROUP_NAME>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --reason)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -r)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__reset__subcmd__help)
            opts="group nodes help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__reset__subcmd__help__subcmd__group)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__reset__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__reset__subcmd__help__subcmd__nodes)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 5 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__power__subcmd__reset__subcmd__nodes)
            opts="-g -y -o -h --graceful --assume-yes --no-wait --output --site --help <NODES>"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__restore)
            opts="-h --site --help vcluster help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__restore__subcmd__help)
            opts="vcluster help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__restore__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__restore__subcmd__help__subcmd__vcluster)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__restore__subcmd__vcluster)
            opts="-b -c -j -m -i -p -a -o -h --bos-file --cfs-file --hsm-file --ims-file --image-dir --pre-hook --post-hook --overwrite --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --bos-file)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                -b)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                --cfs-file)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                -c)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                --hsm-file)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                -j)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                --ims-file)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                -m)
                    local oldifs
                    if [ -n "${IFS+x}" ]; then
                        oldifs="$IFS"
                    fi
                    IFS=$'\n'
                    COMPREPLY=($(compgen -f "${cur}"))
                    if [ -n "${oldifs+x}" ]; then
                        IFS="$oldifs"
                    fi
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o filenames
                    fi
                    return 0
                    ;;
                --image-dir)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                -i)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                --pre-hook)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --post-hook)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -a)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__run)
            opts="-h --site --help session help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__run__subcmd__help)
            opts="session help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__run__subcmd__help__subcmd__help)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__run__subcmd__help__subcmd__session)
            opts=""
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 4 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__run__subcmd__session)
            opts="-n -p -r -w -t -v -P -l -H -o -h --name --playbook-name --repo-path --watch-logs --timestamps --ansible-verbosity --ansible-passthrough --ansible-limit --hsm-group --group --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 3 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -n)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --playbook-name)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -p)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --repo-path)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                -r)
                    COMPREPLY=()
                    if [[ "${BASH_VERSINFO[0]}" -ge 4 ]]; then
                        compopt -o plusdirs
                    fi
                    return 0
                    ;;
                --ansible-verbosity)
                    COMPREPLY=($(compgen -W "0 1 2 3 4" -- "${cur}"))
                    return 0
                    ;;
                -v)
                    COMPREPLY=($(compgen -W "0 1 2 3 4" -- "${cur}"))
                    return 0
                    ;;
                --ansible-passthrough)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -P)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --ansible-limit)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -l)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --hsm-group)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                -H)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
        manta__subcmd__upgrade)
            opts="-c -d -y -o -h --check --dry-run --assume-yes --output --site --help"
            if [[ ${cur} == -* || ${COMP_CWORD} -eq 2 ]] ; then
                COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
                return 0
            fi
            case "${prev}" in
                --output)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                -o)
                    COMPREPLY=($(compgen -W "table json" -- "${cur}"))
                    return 0
                    ;;
                --site)
                    COMPREPLY=($(compgen -f "${cur}"))
                    return 0
                    ;;
                *)
                    COMPREPLY=()
                    ;;
            esac
            COMPREPLY=( $(compgen -W "${opts}" -- "${cur}") )
            return 0
            ;;
    esac
}

if [[ "${BASH_VERSINFO[0]}" -eq 4 && "${BASH_VERSINFO[1]}" -ge 4 || "${BASH_VERSINFO[0]}" -gt 4 ]]; then
    complete -F _manta -o nosort -o bashdefault -o default manta
else
    complete -F _manta -o bashdefault -o default manta
fi
