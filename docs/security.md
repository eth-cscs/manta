# Security

??? info "**WIP**"
    This is work in progress

Manta validates the users command against the information in their JWT authentication token.

If the user tries to submit an operation against a cluster or node that does not have access, then the operation will fail.

Discuss about how manta uses the JWT auth token to restrict user actions

Alps system management APIs are not publicly available, manta needs to run from a hardened environment within CSCS
