# MANTA

Another CLI tool for [Alps](https://www.cscs.ch/science/computer-science-hpc/2021/cscs-hewlett-packard-enterprise-and-nvidia-announce-worlds-most-powerful-ai-capable-supercomputer).

Manta is a frontend cli to interact with Shasta, it uses [mesa](https://crates.io/crates/mesa) for all Shasta interaction.

User guide can be found here https://eth-cscs.github.io/manta/

## Deployment

### Prerequisites

Install build dependencies

```shell
$ cargo install cargo-release cargo-dist git-cliff
```

### Build container image

This repo contains a Dockerfile to build a Container with manta cli.

```
docker build -t manta .
```

#### Run

```
$ docker run -it --network=host -v ~:/root/ manta --help
```

### Build from sources

Install Rust toolchain https://www.rust-lang.org/tools/install

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Install cross to be able to complile on different platforms

```shell
cargo install cross
```

Generate binary (cross compilation)

```shell
scripts/build
```

or

```shell
rustup target add x86_64-unknown-linux-gnu
cargo build --target=x86_64-unknown-linux-gnu
```

### Development

#### Prerequisites

Install 'cargo dist' and 'cargo release'

```
cargo install cargo-dist
cargo install cargo-release
```

Configure cargo-dist. Accept default options and only target linux assets

```
cargo dist init -t $(uname -m)-unknown-$(uname -s | tr '[:upper:]' '[:lower:]')-gnu
```

Then remove the assets for macos and windows

Make sure a github workflow is created in `.github/workflows/release.yml`

#### Deployment

This project is already integrated with github actions through 'cargo release' and 'git cliff'

> git cliff will parse your commits and update the CHANGELOG.md file automatically as long as your commits follows [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/#specification) and [git cliff extra commit types](https://github.com/eth-cscs/manta/blob/main/cliff.toml#L52-L65)

```
cargo release <bump level> --execute
```

> chose your [bump level](https://github.com/crate-ci/cargo-release/blob/master/docs/reference.md#bump-level) accordingly

If everything went well, then binary should be located in `manta/target/x86_64-unknown-linux-gnu/release/manta`

### Profiling

#### Enable capabilities

```bash
sudo sysctl -w kernel.perf_event_paranoid=-1
```

#### Install perf

```bash
sudo apt-get install linux-tools-common linux-tools-generic linux-tools-`uname -r`
```

#### Grant access to kernel address map

```bash
sudo sh -c " echo 0 > /proc/sys/kernel/kptr_restrict"
```

#### Create perf data

```bash
perf stat -ad -r 100 target/release/manta get session
```

#### Identify bottlenecks and get hotspots for those events


```bash
perf record -g --call-graph=dwarf -F max target/release/manta get session
```

#### Convert perf data file to a format firefox profiles understands

```bash
perf script -F +pid > manta.perf
```

Go to https://profiler.firefox.com/ and open manta.perf file


#### DHAT mem alloction profiling

> https://docs.rs/dhat/latest/dhat/
> lto in Cargo.toml needs to be disabled

##### Run

```bash
cargo run -r --features dhat-heap -- get session
```

##### View results (dhat-heap.json file)

https://nnethercote.github.io/dh_view/dh_view.html
