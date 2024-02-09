# Changelog

All notable changes to this project will be documented in this file.

## [1.22.0] - 2024-02-09

### Bug Fixes

- Delete data was not filtering BOS sessiontemplate properly
- Create bos sessiontemplate from SAT file

### Features

- Apply image and apply cluster subcommands now manages IMS jobs through recipes and
- Apply configuration, apply image and apply cluster subcommands now
- Update manta version

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

- Add releases for other OS
- Add releases for other OS
- Remove windows as a target

### Features

- Add openssl vendor feature to git2

### Miscellaneous Tasks

- Release manta version 1.20.29
- Release manta version 1.20.30
- Release manta version 1.20.31
- Release manta version 1.20.32
- Release manta version 1.20.33

## [1.20.28] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.28

## [1.20.27] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.27

## [1.20.26] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.26

## [1.20.25] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.25

## [1.20.24] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.24

## [1.20.23] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.23

## [1.20.22] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.22

## [1.20.21] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.21

## [1.20.20] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.20

## [1.20.19] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.19

## [1.20.18] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.18

## [1.20.17] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.17

## [1.20.16] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.16

## [1.20.15] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.15

## [1.20.14] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.14

## [1.20.13] - 2024-01-26

### Bug Fixes

- Add macos build to releases

### Miscellaneous Tasks

- Release manta version 1.20.13

## [1.20.12] - 2024-01-26

### Bug Fixes

- Github actions publishing mac m1

### Miscellaneous Tasks

- Release manta version 1.20.12

## [1.20.11] - 2024-01-26

### Bug Fixes

- Github actions publishing mac m1

### Miscellaneous Tasks

- Release manta version 1.20.11

## [1.20.10] - 2024-01-26

### Bug Fixes

- Github actions publishing mac m1

### Miscellaneous Tasks

- Release manta version 1.20.10

## [1.20.9] - 2024-01-26

### Bug Fixes

- Github actions publishing mac m1

### Miscellaneous Tasks

- Release manta version 1.20.9

## [1.20.8] - 2024-01-26

### Bug Fixes

- Github actions publishing mac m1

### Miscellaneous Tasks

- Release manta version 1.20.8

## [1.20.7] - 2024-01-26

### Bug Fixes

- Github actions publishing mac m1

### Miscellaneous Tasks

- Release manta version 1.20.7

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

- Merge migration functionality

### Co-authored-by

- Masber <masber@hotmail.com>

### Miscellaneous Tasks

- Release manta version 1.20.2

### Refactor

- Apply clippy suggestions

## [1.20.1] - 2024-01-23

### Bug Fixes

- Add migrate subcommand

### Miscellaneous Tasks

- Release manta version 1.20.1

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
