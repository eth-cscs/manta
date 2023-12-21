# Changelog

All notable changes to this project will be documented in this file.

## [1.13.3] - 2023-12-21

### Features

- Update mesa version

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
