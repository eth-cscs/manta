# Wellcome to Manta

Another CLI tool for [Alps](https://www.cscs.ch/science/computer-science-hpc/2021/cscs-hewlett-packard-enterprise-and-nvidia-announce-worlds-most-powerful-ai-capable-supercomputer).

Manta is a frontend CLI to interact and extend CSM, it uses [mesa](https://github.com/eth-cscs/mesa) as a backend to interact with the infrastructe (CSM) APIs.

!!! info "Documentation is work in progress"
    This documentation is "work in progress" expect changes in the near future

Manta's goals:

 - Release operators from repetitive tasks.
 - Provide quick system feedback.
 - Simplify CICD pipelines

Manta aggregates information from multiple sources:

 - CSM Keycloak
 - CSM API
 - CSM K8s API
 - local git repo
 - Gitea API (CSM VCS)
 - Hashicorp Vault

## Features

- List and filter CFS configurations based on cluster name or configuration name
- List and filter CFS sessions based on cluster name or session name
- List and filter BOS session templates based on cluster name or session name
- List nodes in HSM groups
- List hw configuration/components
- Create CFS configuration and session (target dynamic) from local repository
- Create CFS configuration and session (target image) from CSCS SAT input file
- Watch logs of a CFS session
- Connect to a node's console
- Power On/Off or restart nodes individually, in a list or per cluster
- Restrict operations to nodes belonging to a specific HSM group
- Filter information to a HSM group
- Update node boot image based on CFS configuration name
- Audit/Log
- Delete all data related to CFS configuration
- Migrate nodes from HSM group based on hw components profile

## Introduction

???+ warning "**WIP**"
      trying to fill the gap in cray and sat CLIs when managing clusters

### Configuration immutability

Overwritting configurations used by clusters (nodes booting with an image linked to a configuration or runtime configuration) is a bad practice because the history of that configuration is lost. 

### Configuration vs images

Image IDs are typically opaque and do not provide any insight into the content of the image. In other words, you cannot tell what software, configurations, or versions are included in an image just by looking at its ID or name. Due to the obscurity of image IDs, it's crucial to maintain a clear relationship between configurations (the settings and scripts used to create the image) and the images themselves. This means keeping detailed records of which configurations were used to build each image. 
Due to the obscurity of image IDs, it's crucial to keep the relationship between configurations and the images themselves. This means keeping the sessions used to create the images and/or configure the nodes. The sessions are important because they are the link between the configurations and the images. They contain the information about which configurations were used to create each image, thus ensuring traceability. 

Instead of using image IDs directly, the best practice is to refer to the configurations when assigning images to nodes for booting. This ensures that you have a clear understanding of what is being deployed and can replicate or troubleshoot it if necessary.

### Cluster vs nodes operations

Provide operations on a list no nodes or on all nodes in a cluster
