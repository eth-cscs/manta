# MANTA

Another CLI tool for [Alps](https://www.cscs.ch/science/computer-science-hpc/2021/cscs-hewlett-packard-enterprise-and-nvidia-announce-worlds-most-powerful-ai-capable-supercomputer).

Manta is a frontend cli to interact with CSM and OCHAMI.

## Deployment

### Prerequisites

Install build dependencies

```shell
$ cargo install cargo-release cargo-dist git-cliff
```

### Clone repo

```bash
git clone https://github.com/eth-cscs/manta && cd manta
git checkout 1.5
```

### Build container image

This repo contains a Dockerfile to build a Container with manta cli.

```
docker build -t manta .
```

#### Copy configuration file

```bash
mkdir -p ~/.config/manta
cat > ~/.config/manta/config.toml <<EOF
log = "info"

site = "ochami"
parent_hsm_group = "nodes_free"
audit_file = "/tmp/manta_audit.log"

[sites]

[sites.ochami]
backend = "ochami"
shasta_base_url = "https://foobar.openchami.cluster:8443"
root_ca_cert_file = "ochami_root_cert.pem"
EOF
```

#### Start the `ochami` services from the [deployment recipe quickstart](https://github.com/OpenCHAMI/deployment-recipes/tree/main/quickstart).

> [!NOTE]
> Make sure to set the `ACCESS_TOKEN` environment variable and create a CA certificate in the same directory as the config file. This can be done using the convienience functions from the the OpenCHAMI deployment recipe repository.
>
> To set the `ACCESS_TOKEN` environment variable and create/renew the CA certificate (assuming you have cloned the deployment recipe quickstart):
> ```bash
> # collection of useful functions
> ochami_deployment_recipe_quickstart=path/to/quickstart
> source $ochami_deployment_recipe_quickstart/bash_functions.sh
>
> # set environment variable then create the cert
> export ACCESS_TOKEN=$(gen_access_token)
> get_ca_cert > $HOME/.config/manta/ochami_root_cert.pem
> ```

#### Run the CLI with one of the two options mentioned above to confirm that `manta` is working.

```bash
docker run -it --network=host -v $HOME:/root/ -e ACCESS_TOKEN=$ACCESS_TOKEN manta manta get redfish-endpoints
```

> [!NOTE]
> Some commands will not work yet with OpenCHAMI services and will sometimes show a message indicating no implementation for the backend.
>
> ```bash
> docker run -it --rm --network=host -v $HOME:/root/ -e ACCESS_TOKEN=$ACCESS_TOKEN manta:latest manta get sessions
> INFO  | Get CFS sessions for HSM groups: Some([])
> ERROR | Failed to get CFS sessions. Reason:
> ERROR - Message: Get and filter sessions command not implemented for this backend
> exit status 1
> ```
>
> Some of the other commands may fail simply due to CSM services not included with OpenCHAMI if only using the OpenCHAMI deployment recipes:
>
> ```bash
> docker run -it --rm --network=host -v $HOME:/root/ -e ACCESS_TOKEN=$ACCESS_TOKEN manta:latest manta get images
> INFO  | Get IMS images 'all available'
>
> thread 'main' panicked at src/cli/commands/get_images.rs:23:6:
> called `Result::unwrap()` on an `Err` value: NetError(reqwest::Error { kind: Status(503), url: Url { scheme: "https", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("foobar.openchami.cluster") ), port: Some(8443), path: "/ims/v3/images", query: None, fragment: None } })
> note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
> exit status 101
> ```

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

> choose your [bump level](https://github.com/crate-ci/cargo-release/blob/master/docs/reference.md#bump-level) accordingly

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
