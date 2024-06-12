# Changelog

All notable changes to this project will be documented in this file.

## [1.37.0] - 2024-06-12

### FEAT

- Add new config file parameter to store CSM CA public root filename

### FIX

- Improve errors in configuration file

## [1.36.3] - 2024-06-12

### FEAT

- Update manta version

### Miscellaneous Tasks

- Release manta version 1.36.3

### Refactor

- Clean code

## [1.36.2] - 2024-06-09

### Bug Fixes

- HSM list to validate in apply sat file no longer takes into

### FIX

- Delete now does not deletes images from BOS sessiontemplate params.
- Process cli with wrong commands

### Miscellaneous Tasks

- Release manta version 1.36.2

### REFACTOR

- Fix subcommands

### Refactor

- Clean code

## [1.36.1] - 2024-06-02

### Bug Fixes

- Use new mesa library to fix issue getting commit id details form gitea

### Miscellaneous Tasks

- Release manta version 1.36.1

## [1.36.0] - 2024-06-02

### Bug Fixes

- Cli help

### Co-authored-by

- Manuel Sopena Ballesteros <manuel.sopena@cscs.ch>

### Features

- Apply sat command now support pre and port hooks

### Miscellaneous Tasks

- Release manta version 1.36.0

## [1.35.8] - 2024-05-29

### Bug Fixes

- Enable openssl-vendores feature got git2 crate to avoid breaking apple images during CI/CD pipeline
- Update boot parameters

### Miscellaneous Tasks

- Release manta version 1.35.8

## [1.35.7] - 2024-05-28

### Bug Fixes

- Try to fix ci/cd pipeline building openssl-sys

### Miscellaneous Tasks

- Release manta version 1.35.7

## [1.35.6] - 2024-05-28

### Bug Fixes

- Init cargo dist

### Miscellaneous Tasks

- Release manta version 1.35.6

## [1.35.5] - 2024-05-28

### Bug Fixes

- Update rust toolchain and cargo-dist in CI pipeline

### Miscellaneous Tasks

- Release manta version 1.35.5

## [1.35.4] - 2024-05-28

### Bug Fixes

- Update rust toolchain and cargo-dist in CI pipeline

### Miscellaneous Tasks

- Release manta version 1.35.4

## [1.35.3] - 2024-05-28

### Bug Fixes

- Downgrade cargo-dist in CI pipeline

### Miscellaneous Tasks

- Release manta version 1.35.3

## [1.35.2] - 2024-05-28

### Bug Fixes

- Update rust toolchain and cargo-dist in CI pipeline

### Miscellaneous Tasks

- Release manta version 1.35.2

## [1.35.1] - 2024-05-28

### Bug Fixes

- Bug in apply session processing the wrong HSM group

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.35.1

### Refactor

- Clean code
- Clean crates

## [1.35.0] - 2024-05-23

### Features

- New command 'get cluster <cluster name> --output summary'

### Miscellaneous Tasks

- Release manta version 1.35.0

## [1.34.0] - 2024-05-22

### Features

- Update mesa library
- Filter CFS configurations by name through glob pattern matching

### Miscellaneous Tasks

- Release manta version 1.34.0

## [1.33.0] - 2024-05-21

### Features

- Get configuration with details now shows CFS configuration derivatives (CFS sessions, BOS sessiontemplate and IMS images related to a CFS configuration)

### Miscellaneous Tasks

- Release manta version 1.33.0

## [1.32.5] - 2024-05-20

### Bug Fixes

- Improve the way hw component scarcity scores is calculated
- Print log messages

### Features

- Update hsm group members
- Update mesa version
- Update mesa version
- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.32.4
- Release manta version 1.32.5

### Refactor

- Change var names

## [1.32.3] - 2024-05-02

### Bug Fixes

- Update boot only chaning the boot image of one node

### Features

- Refactor template output information
- Improve user feecback when restarting nodes

### Miscellaneous Tasks

- Release manta version 1.32.3

## [1.32.2] - 2024-05-02

### Bug Fixes

- Error management improvement

### Features

- Add semaphores when making multiples calls to CSM APIs to throttle
- Upgrade mesa version

### Miscellaneous Tasks

- Release manta version 1.32.2

## [1.32.1] - 2024-05-01

### Features

- Filter image by id
- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.32.1

### Refactor

- Clean output of command validate-local-repo

## [1.32.0] - 2024-05-01

### Bug Fixes

- Command get cfs configuration details failing in fetching details

### Features

- Local repo validation against gitea

### Miscellaneous Tasks

- Release manta version 1.32.0

## [1.31.2] - 2024-04-30

### Features

- Format data in manta get configuration -n

### Miscellaneous Tasks

- Release manta version 1.31.2

## [1.31.1] - 2024-04-29

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.31.1

### Refactor

- Manta get cluster --> rename table results column names

## [1.31.0] - 2024-04-28

### Bug Fixes

- Bug in `apply hw cluster` command where nodes in final target hsm group would not be accurate if both target and parent hsm groups have nodes in common
- Disable DHAT crate since it is not beind used

### Features

- Add new method to change boot parameters `manta apply boot`
- Add new argument to set the image id to a cluster or nodes for
- Update mesa
- Add tests to `apply hw cluster` functionality

### Miscellaneous Tasks

- Release manta version 1.31.0

### Refactor

- Clean code
- Comment code

## [1.30.1] - 2024-04-25

### Miscellaneous Tasks

- Release manta version 1.30.1

### Refactor

- Remove 2 columns from the CFS session output table
- Rename "DEsired Configuration" to "Running configuration" in

## [1.30.0] - 2024-04-23

### Bug Fixes

- Update mesa library
- Restore BOS sessiontemplate to v1
- Restore BOS session to v1
- Improve BOS sessiontemplate by processing multiple boot_sets
- Improve BOS sessiontemplate by passing kernel parameters from SAT
- Process multiple BOS sessiontemplate in SAT file

### Miscellaneous Tasks

- Release manta version 1.30.0

## [1.29.5] - 2024-04-17

### Features

- Print most recent CFS session logs related to a cluster

### Miscellaneous Tasks

- Release manta version 1.29.5

### Refactor

- Fix mesa library location

## [1.29.4] - 2024-04-17

### Features

- Add new feature to filter CFS sessions by min_age and max_age parameters

### Miscellaneous Tasks

- Release manta version 1.29.4

## [1.29.3] - 2024-04-17

### Features

- Add new feature to filter CFS sessions by min_age and max_age parameters

### Miscellaneous Tasks

- Release manta version 1.29.3

## [1.29.2] - 2024-04-16

### Bug Fixes

- Bug filtering CFS sessions through HSM groups
- Fix bug filtering bos sessiontemplate by HSM group
- Print BOS sessiontemplate information properly by removing the type column in table

### Features

- Add functionality to filter CFS sessions by state

### Miscellaneous Tasks

- Release manta version 1.29.2

## [1.29.1] - 2024-04-03

### Bug Fixes

- CFS layer branch lookup not printing branch name properly if they

### Miscellaneous Tasks

- Release manta version 1.29.1

### Refactor

- Clean code

## [1.29.0] - 2024-04-02

### Bug Fixes

- Stop fetching all HSM groups available to the user if no roles

### Features

- SAT file processing now accepts

### Miscellaneous Tasks

- Release manta version 1.29.0

### Refactor

- Cean code

## [1.28.14] - 2024-03-17

### Miscellaneous Tasks

- Release manta version 1.28.14

### Refactor

- Move tests to /test/ directory

## [1.28.13] - 2024-03-15

### Features

- Add selection prompmt to delete auth token

### Miscellaneous Tasks

- Release manta version 1.28.13

## [1.28.12] - 2024-03-15

### Bug Fixes

- Config unset hsm command

### Miscellaneous Tasks

- Release manta version 1.28.12

## [1.28.11] - 2024-03-15

### Bug Fixes

- Format cfs layer data and clean stoud log traces

### Miscellaneous Tasks

- Release manta version 1.28.11

## [1.28.10] - 2024-03-15

### Features

- Show cfs configuration layer table in different rows
- User new mesa library version

### Miscellaneous Tasks

- Release manta version 1.28.10

## [1.28.9] - 2024-03-14

### Features

- Handle auth tokens for multiple sites at the same time

### Miscellaneous Tasks

- Release manta version 1.28.9

## [1.28.8] - 2024-03-13

### Bug Fixes

- BUG SAT file session_template validation ignoring previous SAT file version

### Miscellaneous Tasks

- Release manta version 1.28.8

## [1.28.7] - 2024-03-12

### Co-authored-by

- Manuel Sopena Ballesteros <manuel.sopena@cscs.ch>

### Features

- Improve SAT file validation to improve user feedback
- Test new cc to build apple target binaries

### Miscellaneous Tasks

- Release manta version 1.28.7

## [1.28.6] - 2024-03-04

### Bug Fixes

- Remove apply artifacts/targets from CI pipeline

### Miscellaneous Tasks

- Release manta version 1.28.6

## [1.28.5] - 2024-03-03

### Features

- Update mesa

### Miscellaneous Tasks

- Release manta version 1.28.5

### Refactor

- Clean code and adapt to new mesa version

## [1.28.4] - 2024-03-01

### Features

- Update command no longer reboot nodes if boot image did not change
- Update manta version

### Miscellaneous Tasks

- Release manta version 1.28.4

### Refactor

- Clean code

## [1.28.3] - 2024-03-01

### Features

- Prepare to substitute apply configuration, apply image and apply cluster to apply sat-file

### Miscellaneous Tasks

- Release manta version 1.28.3

## [1.28.2] - 2024-03-01

### Bug Fixes

- Test apple artifacts

### Miscellaneous Tasks

- Release manta version 1.28.2

## [1.28.1] - 2024-03-01

### Features

- Get configuration command shows layer information including tag
- Update manta version

### Miscellaneous Tasks

- Release manta version 1.28.1

### Refactor

- Clean code

## [1.28.0] - 2024-02-28

### Bug Fixes

- Add dialog asking user to validate SAT file for commands apply

### Miscellaneous Tasks

- Release manta version 1.28.0

## [1.27.0] - 2024-02-28

### Bug Fixes

- Bug in apply cluster subcommand where it was failing when reading

### Features

- Show more detailed information related to CFS configuration
- Commands add node, remove, node, add, hw, remove, hw and apply hw
- Update manta version

### Miscellaneous Tasks

- Release manta version 1.27.0

### Refactor

- Clean code

## [1.26.0] - 2024-02-25

### Features

- Get hw components subcommands now can print information as a summary of all hw components in a cluster

### Miscellaneous Tasks

- Release manta version 1.26.0

## [1.25.1] - 2024-02-25

### Miscellaneous Tasks

- Release manta version 1.25.1

### Refactor

- Reformat how CFS configuration layer details are printed on screen

## [1.25.0] - 2024-02-24

### Bug Fixes

- Get configuration, get image, get template was filtering way too
- Ci pipeline not generating homebrew installation command till cc

### Features

- Get configuration now resolves gitea information like branch and
- Update mesa library

### Miscellaneous Tasks

- Release manta version 1.25.0

## [1.24.2] - 2024-02-23

### Bug Fixes

- SAT file templating
- Disable apple targets due to a bug in cc crate used by openssl crate

### Features

- Replace __DATE__ in SAT file vars file and cli vars for timestamp

### Miscellaneous Tasks

- Release manta version 1.24.2

### Refactor

- Clean stdout messages

## [1.24.1] - 2024-02-23

### Bug Fixes

- HSM validation for admin users

### Features

- Integrate parent HSM to config file
- Integratino parent HSM to `add hw`, `add node`, `remove hw`, `remove node` and `apply hw` subcommands

### Miscellaneous Tasks

- Release manta version 1.24.1

### Refactor

- Improve stdout messages

## [1.24.0] - 2024-02-22

### Bug Fixes

- Mesa library
- Manta version

### Features

- Initial woking state
- Improve the function that merges 2 yaml structs by avoiding having to rewrite siblings

### Miscellaneous Tasks

- Release manta version 1.24.0
- Release manta version 1.24.0

## [1.23.0] - 2024-02-20

### Co-authored-by

- Manuel Sopena Ballesteros <manuel.sopena@cscs.ch>

### Features

- New feature to use the SAT files as jinja2 templates (#37)
- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.23.0

## [1.22.11] - 2024-02-20

### Bug Fixes

- Bug with manta panicking while creating a cluster if image creation fails
- Error when getting CFS session logs using a CFS session which does not exists
- Update mesa library

### Miscellaneous Tasks

- Release manta version 1.22.11

## [1.22.10] - 2024-02-17

### Bug Fixes

- Command update node fails if user is not restricted to any HSM groups
- Fix error parsing cli opn 'ansible-verbosity' to 'apply image' subcommand

### Miscellaneous Tasks

- Release manta version 1.22.10

## [1.22.9] - 2024-02-16

### Bug Fixes

- Show_config function breaks if the list of HSM groups the user has access to is empty

### Miscellaneous Tasks

- Release manta version 1.22.9

## [1.22.8] - 2024-02-15

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.22.8

## [1.22.7] - 2024-02-13

### Bug Fixes

- Fix stdout messages
- Manta crashes:wa when CFS configuration layer had no commit id

### Miscellaneous Tasks

- Release manta version 1.22.7

## [1.22.6] - 2024-02-13

### Bug Fixes

- Manta crashes:wa when CFS configuration layer had no commit id

### Miscellaneous Tasks

- Release manta version 1.22.6

## [1.22.5] - 2024-02-13

### Bug Fixes

- Manta crashes:wa when CFS configuration layer had no commit id

### Miscellaneous Tasks

- Release manta version 1.22.5

## [1.22.4] - 2024-02-13

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.22.4

## [1.22.3] - 2024-02-10

### Bug Fixes

- Mesa crate

### Miscellaneous Tasks

- Release manta version 1.22.3

## [1.22.2] - 2024-02-10

### Bug Fixes

- Apply cluster command failing if session_template section in SAT file was in old format

### Miscellaneous Tasks

- Release manta version 1.22.2

## [1.22.1] - 2024-02-10

### Miscellaneous Tasks

- Release manta version 1.22.1

### Refactor

- Clean code and stdout messages

## [1.22.0] - 2024-02-09

### Bug Fixes

- Delete data was not filtering BOS sessiontemplate properly
- Create bos sessiontemplate from SAT file

### Features

- Apply image and apply cluster subcommands now manages IMS jobs through recipes and
- Apply configuration, apply image and apply cluster subcommands now
- Update manta version

### Miscellaneous Tasks

- Release manta version 1.22.0

### Refactor

- Clean gitea code since it is moved to mesa
- Clean code
- Clean code
- Clean code
- Move code related to import data from SAT file to its own
- Add tests to import images in SAT file
- Create module for SAT code

## [1.21.3] - 2024-01-30

### Bug Fixes

- Get configuration command ignoring configurations related to CFS
- Print statement
- Show error if apply cluster failt creating a configuration
- Bos sessiontemplate filter by list of xnames

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.21.3

## [1.21.2] - 2024-01-29

### Bug Fixes

- Format errors when deleting an image which does not exists

### Miscellaneous Tasks

- Release manta version 1.21.2

## [1.21.1] - 2024-01-29

### Bug Fixes

- Error when deleting an image based on a db recod but the artifact does not exists

### Miscellaneous Tasks

- Release manta version 1.21.1

## [1.21.0] - 2024-01-29

### Features

- Add new param to apply cluster to avoid nodes from rebooting

### Miscellaneous Tasks

- Release manta version 1.21.0

## [1.20.35] - 2024-01-28

### Bug Fixes

- Update mesa version in cargo.toml

### Miscellaneous Tasks

- Release manta version 1.20.35

## [1.20.34] - 2024-01-28

### Bug Fixes

- Rename 'force' param in 'delete' command to 'yes'
- Rename aliases for command 'apply configuration'
- Improve user validation to check access to a HSM group

### Fixes

- Migrate backup and migrate restore (#11)

### Miscellaneous Tasks

- Release manta version 1.20.34

### Refactor

- Code checkif user has access to HSM groups and members
- Fix some log messages

## [1.20.33] - 2024-01-27

### Bug Fixes

- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Github actions publishing mac m1
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add macos build to releases
- Add releases for other OS
- Add releases for other OS
- Remove windows as a target

### Features

- Add openssl vendor feature to git2

### Miscellaneous Tasks

- Release manta version 1.20.7
- Release manta version 1.20.8
- Release manta version 1.20.9
- Release manta version 1.20.10
- Release manta version 1.20.11
- Release manta version 1.20.12
- Release manta version 1.20.13
- Release manta version 1.20.14
- Release manta version 1.20.15
- Release manta version 1.20.16
- Release manta version 1.20.17
- Release manta version 1.20.18
- Release manta version 1.20.19
- Release manta version 1.20.20
- Release manta version 1.20.21
- Release manta version 1.20.22
- Release manta version 1.20.23
- Release manta version 1.20.24
- Release manta version 1.20.25
- Release manta version 1.20.26
- Release manta version 1.20.27
- Release manta version 1.20.28
- Release manta version 1.20.29
- Release manta version 1.20.30
- Release manta version 1.20.31
- Release manta version 1.20.32
- Release manta version 1.20.33

## [1.20.6] - 2024-01-26

### Features

- Sort hsm available list in 'config show' command
- Add new target for mac, the idea is to have a new binary in github releases for mac users

### Miscellaneous Tasks

- Release manta version 1.20.6

## [1.20.5] - 2024-01-24

### Bug Fixes

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.20.5

## [1.20.4] - 2024-01-24

### Bug Fixes

- Update mesa version to fix a bug

### Miscellaneous Tasks

- Release manta version 1.20.4

## [1.20.3] - 2024-01-24

### Bug Fixes

- Bug in 'apply cluster' subcommand where it was filtering wrong images

### Miscellaneous Tasks

- Release manta version 1.20.3

## [1.20.2] - 2024-01-24

### Bug Fixes

- Add migrate subcommand
- Merge migration functionality

### Co-authored-by

- Masber <masber@hotmail.com>

### Miscellaneous Tasks

- Release manta version 1.20.1
- Release manta version 1.20.2

### Refactor

- Apply clippy suggestions

## [1.20.0] - 2024-01-22

### Bug Fixes

- Merge cluster migration branch
- Cli build code fix

### Miscellaneous Tasks

- Release manta version 1.20.0

## [1.19.3] - 2024-01-21

### Miscellaneous Tasks

- Release manta version 1.19.3

### Refactor

- Clean code

## [1.19.2] - 2024-01-21

### Bug Fixes

- Rollback apply hw so it unpins all nodes in target hsm
- Fix issues related to add hw and remove hw subcommands

### Miscellaneous Tasks

- Release manta version 1.19.2

## [1.19.1] - 2024-01-19

### Bug Fixes

- Fix bug when creating clusters using sat file

### Features

- Add and remove nodes from HSM group
- Add functionality to add or remove nodes to/from an HSM group
- Add new mesa version

### Miscellaneous Tasks

- Release manta version 1.19.1

### Refactor

- Clean code related to subcommand 'apply hw'
- Clean code
- Add apply hw-configuration cli help message
- Clippy fixes
- Clippy fixes
- Clippy fixes
- Change apply/add/remove hw components to/from HSM group to

## [1.19.0] - 2024-01-15

### Features

- Apply hw partially working with first stage migrating hw components from target hsm group to parent, pending the other direction (migrating from parent to target hsm group)
- Apply hw partially working with first stage migrating hw components from target hsm group to parent, pending the other direction (migrating from parent to target hsm group)
- Initial code for apply hw subcommand keeping target hsm members

### Miscellaneous Tasks

- Release manta version 1.19.0

## [1.18.0] - 2024-01-11

### Bug Fixes

- Disable tests which need to connect to csm apis becuase they are not accessible from github test environment

### Miscellaneous Tasks

- Release manta version 1.18.0

## [1.17.0] - 2024-01-11

### Bug Fixes

- Fix bug passing params to "remove hw" subcommand

### Miscellaneous Tasks

- Release manta version 1.17.0

### Refactor

- Normalize add, remove and apply hw component subcommands
- Clean code
- Get mesa from repo

## [1.16.2] - 2024-01-10

### Bug Fixes

- Apply session and update mesa library

### Miscellaneous Tasks

- Release manta version 1.16.2

## [1.16.1] - 2024-01-10

### Bug Fixes

- Clean obsolete "use" statements

### Features

- Remove hw components from a target hsm groups and node scores calculated based on scarcity across target and parent hsm groups
- Apply and remove working with simple examples, not fully tested but in good condition
- Add new hw components to a cluster

### Miscellaneous Tasks

- Release manta version 1.16.1

### Refactor

- Clean code
- Update README
- Refactor code

## [1.16.0] - 2024-01-04

### Bug Fixes

- Disable build script because it was breaking cli module load from

### Features

- Start migrating hw components features from clstr crate
- Get hw cluster now accepts a new 'pattern' output

### Miscellaneous Tasks

- Release manta version 1.16.0

### Refactor

- : add clippy suggestions

## [1.15.0] - 2024-01-01

### Bug Fixes

- Get template not filtering by most recent value
- Replace std sleep for tokio sleep

### Features

- Add cluster power management

### Miscellaneous Tasks

- Release manta version 1.15.0

### Refactor

- Cargo fmt
- Use new manta utility functions
- Adapt to new mesa code
- Adapt to new mesa code
- Adopt mesa changes
- Move functions to print table data from mesa to manta
- Adapt to mesa code
- Housekeeping around HSM module
- Adapt to mesa code
- Adapt to new mesa code
- Adapt to new mesa code
- Adapt to new mesa code
- Swap to mesa library

## [1.14.0] - 2023-12-25

### Features

- Get nodes now shows CFS configuration related to image id used to

### Miscellaneous Tasks

- Release manta version 1.14.0

### Refactor

- Clean code
- Adapt code to new mesa
- Clean code

## [1.13.5] - 2023-12-22

### Miscellaneous Tasks

- Release manta version 1.13.5

### Refactor

- Update mesa version

## [1.13.4] - 2023-12-22

### Bug Fixes

- Import

### Miscellaneous Tasks

- Release manta version 1.13.4

### Refactor

- Update mesa version

## [1.13.3] - 2023-12-21

### Features

- Update mesa version

### Miscellaneous Tasks

- Release manta version 1.13.3

### Refactor

- High refactoring
- Cfs configuration structs
- Rename method name to get multiple CFS components
- Hoursekeeping around node methods

## [1.13.2] - 2023-12-11

### Miscellaneous Tasks

- Release manta version 1.13.2

### Refactor

- Clean code and update mesa version
- Cargo fmt

## [1.13.1] - 2023-12-10

### Miscellaneous Tasks

- Release manta version 1.13.1

### Refactor

- Refactor code to new mesa method signatures

## [1.13.0] - 2023-12-10

### Features

- Add new flag 'force' to  delete subcommand to make it script
- Delete subcommand sumamry shows more information

### Miscellaneous Tasks

- Release manta version 1.13.0

### Refactor

- Code in delete and get iamge subcommands

## [1.12.12] - 2023-12-08

### Miscellaneous Tasks

- Release manta version 1.12.12

### Refactor

- Adapt code to new mesa cfs config code structure

## [1.12.11] - 2023-12-08

### Miscellaneous Tasks

- Release manta version 1.12.11

### Refactor

- Adapt code to new mesa cfs config code structure

## [1.12.10] - 2023-12-08

### Bug Fixes

- Fix bug with get configuration subcommand

### Miscellaneous Tasks

- Release manta version 1.12.10

## [1.12.9] - 2023-12-07

### Bug Fixes

- Panic when trying to connect to CFS session (ansible) container
- Simplify the collection of the HSM group description data.

### Feature

- Migrate/backup first commit (partial)
- Migrate/backup ignore JetBrains stuff
- Migrate/backup download all files of a bos session template
- Migrate/backup fix count of artifacts in download info
- Migrate/backup add support to produce a file with the list of xnames belonging to the HSM groups in the BOS session template.
- Migrate/backup cleanup
- Migrate/backup more cleanup
- Migrate/restore load backed files into memory

### Features

- Node power status methods blocks the runtime
- Each section in sat files processed individually

### Miscellaneous Tasks

- Release manta version 1.12.8
- Release manta version 1.12.9

### Refactor

- Clean code
- Configurations section in SAT file processed independently
- Images section in SAT file processed independently
- Session_templates section in SAT file processed independently

## [1.12.7] - 2023-11-20

### Bug Fixes

- Bug when cluster creation won't realised if a CFS session failed
- Rename CFS session and configuration table headers
- Fix hsm available list send to methods

### Miscellaneous Tasks

- Release manta version 1.12.7

## [1.12.6] - 2023-11-16

### Bug Fixes

- Bug getting cluster with nodes being configured

### Miscellaneous Tasks

- Release manta version 1.12.6

## [1.12.5] - 2023-11-16

### Bug Fixes

- Fix bug getting hsm group from cli param

### Miscellaneous Tasks

- Release manta version 1.12.5

## [1.12.4] - 2023-11-16

### Bug Fixes

- Fix 'get cluster status' sub command

### Miscellaneous Tasks

- Release manta version 1.12.4

### Refactor

- Fix 'get cluster' command help message typo
- Add deprecated message in 'get nodes' subcommand
- Fix 'get hsm' cli help message

## [1.12.3] - 2023-11-16

### Documentation

- Fix README

### Miscellaneous Tasks

- Release manta version 1.12.3

## [1.12.2] - 2023-11-16

### Documentation

- Update README

### Miscellaneous Tasks

- Release manta version 1.12.2

## [1.12.1] - 2023-11-16

### Features

- Duplicate get nodes subcommand into get cluster
- Get cluster status

### Miscellaneous Tasks

- Release manta version 1.12.1

## [1.12.0] - 2023-11-14

### Features

- Add new sub command apply configuration to create CFS

### Miscellaneous Tasks

- Release manta version 1.12.0

## [1.11.0] - 2023-11-13

### Bug Fixes

- Bugs with subcommands ignoring of failing the restrictions

### Features

- Adopt subcommands to work with a group of target HSM group
- New function to validate requested hsm group
- New function to validate requested hsm group members
- Update mesa version
- Get configuration subcommand output to json format

### Miscellaneous Tasks

- Release manta version 1.11.0

### Refactor

- Clean code and adapt to new mesa version
- Clean code

## [1.10.6] - 2023-11-10

### Features

- Enable logs while building a cluster
- Git-clone CFS session logs integrated to 'watch-log' command

### Miscellaneous Tasks

- Release manta version 1.10.6

### Refactor

- [**breaking**] Layer-id param removed when fetching ansible logs (this
- Code related to cfs session logs

## [1.10.5] - 2023-11-07

### Bug Fixes

- Fix bug where app did not read socks 5 information

### Miscellaneous Tasks

- Release manta version 1.10.5

## [1.10.4] - 2023-11-07

### Documentation

- Update README with instructions on how to create releases and commit messages best practices for CHANGELOG.md

### Miscellaneous Tasks

- Release manta version 1.10.4

## [1.10.3] - 2023-11-01

### Bug Fixes

- Update manta version
- Add CHANGELOG integration with cargo-release
- Add git cliff configuration to support multiline git commits

### Features

- Add subcommand to change log level

### Miscellaneous Tasks

- Release manta version 1.10.3

## [1.10.2] - 2023-10-31

### Miscellaneous Tasks

- Release manta version 1.10.2

## [1.10.1] - 2023-10-26

### Miscellaneous Tasks

- Release manta version 1.10.1

## [1.10.0] - 2023-10-26

### Miscellaneous Tasks

- Release manta version 1.10.0

## [1.9.15] - 2023-10-26

### Miscellaneous Tasks

- Release manta version 1.9.15

## [1.9.14] - 2023-10-24

### Miscellaneous Tasks

- Release manta version 1.9.14

## [1.9.13] - 2023-10-24

### Miscellaneous Tasks

- Release manta version 1.9.13

## [1.9.12] - 2023-10-21

### Miscellaneous Tasks

- Release manta version 1.9.12

## [1.9.11] - 2023-10-21

### Miscellaneous Tasks

- Release manta version 1.9.11
- Release manta version 1.9.11

## [1.9.10] - 2023-10-12

### Miscellaneous Tasks

- Release manta version 1.9.10

## [1.9.9] - 2023-10-12

### Miscellaneous Tasks

- Release manta version 1.9.9

## [1.9.8] - 2023-10-12

### Miscellaneous Tasks

- Release manta version 1.9.8

## [1.9.7] - 2023-10-11

### Miscellaneous Tasks

- Release manta version 1.9.7

## [1.9.6] - 2023-10-07

### Miscellaneous Tasks

- Release manta version 1.9.6

## [1.9.5] - 2023-10-05

### Miscellaneous Tasks

- Release manta version 0.7.0
- Release manta version 0.7.1
- Release manta version 0.7.2
- Release manta version 0.8.0
- Release manta version 0.8.1
- Release manta version 1.9.5

## [1.9.4] - 2023-10-04

### Miscellaneous Tasks

- Release manta version 1.9.4

## [1.9.3] - 2023-10-03

### Miscellaneous Tasks

- Release manta version 1.9.3

## [1.9.2] - 2023-09-30

### Miscellaneous Tasks

- Release manta version 1.9.2

## [1.9.1] - 2023-09-30

### Miscellaneous Tasks

- Release manta version 1.7.0
- Release manta version 1.8.0
- Release manta version 1.9.0
- Release manta version 1.9.1

## [1.6.0] - 2023-09-13

### Miscellaneous Tasks

- Release manta version 1.6.0

## [1.5.0] - 2023-09-12

### Miscellaneous Tasks

- Release manta version 1.5.0

## [1.4.0] - 2023-09-01

### Miscellaneous Tasks

- Release manta version 0.7.0
- Release manta version 0.7.1
- Release manta version 0.7.2
- Release manta version 0.8.0
- Release manta version 0.8.1
- Release manta version 0.8.2
- Release manta version 0.8.3
- Release manta version 0.8.4
- Release manta version 0.8.5
- Release manta version 0.8.6
- Release manta version 0.8.7
- Release manta version 0.8.8
- Release manta version 0.8.9
- Release manta version 1.0.0
- Release manta version 1.0.1
- Release manta version 1.0.2
- Release manta version 1.0.3
- Release manta version 1.0.4
- Release manta version 1.1.0
- Release manta version 1.1.1
- Release manta version 1.1.2
- Release manta version 1.2.0
- Release manta version 1.2.1
- Release manta version 1.2.2
- Release manta version 1.3.0
- Release manta version 1.3.1
- Release manta version 1.3.2
- Release manta version 1.3.3
- Release manta version 0.7.0
- Release manta version 0.7.1
- Release manta version 0.7.2
- Release manta version 0.8.0
- Release manta version 0.8.1
- Release manta version 1.4.0

## [0.6.30] - 2023-08-30

### Miscellaneous Tasks

- Release manta version 0.6.30

## [0.6.29] - 2023-07-18

### Miscellaneous Tasks

- Release manta version 0.6.29

## [0.6.28] - 2023-07-05

### Miscellaneous Tasks

- Release manta version 0.6.28

## [0.6.27] - 2023-07-05

### Miscellaneous Tasks

- Release manta version 0.6.27

## [0.6.26] - 2023-06-29

### Miscellaneous Tasks

- Release manta version 0.6.26

## [0.6.25] - 2023-06-28

### Miscellaneous Tasks

- Release manta version 0.6.25

## [0.6.24] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.24

## [0.6.23] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.23

## [0.6.22] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.22

## [0.6.21] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.21

## [0.6.20] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.20

## [0.6.19] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.19

## [0.6.18] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.18

## [0.6.17] - 2023-06-27

### Miscellaneous Tasks

- Release manta version 0.6.17

## [0.6.16] - 2023-06-26

### Miscellaneous Tasks

- Release manta version 0.6.16

## [0.6.15] - 2023-06-25

### Co-authored-by

- Manuel Sopena Ballesteros <msopena@cscs.ch>

### Miscellaneous Tasks

- Release manta version 0.6.15

## [0.6.13] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.13

## [0.6.12] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.12

## [0.6.11] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.11

## [0.6.10] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.10

## [0.6.9] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.9

## [0.6.8] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.8

## [0.6.7] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.7

## [0.6.6] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.6

## [0.6.5] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.5

## [0.6.4] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.4

## [0.6.3] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.3

## [0.6.2] - 2023-06-22

### Miscellaneous Tasks

- Release manta version 0.6.2
- Release manta version 0.6.2

## [0.6.1] - 2023-06-21

### Miscellaneous Tasks

- Release manta version 0.6.1

## [0.6.0] - 2023-06-21

### Miscellaneous Tasks

- Release manta version 0.6.0

## [0.5.1] - 2023-06-21

### Miscellaneous Tasks

- Release manta version 0.5.1

<!-- generated by git-cliff -->
