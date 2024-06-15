# Download and install

To install manta, go to the Github [releases section](https://github.com/eth-cscs/manta/releases) and download the `manta-installer.sh` script for the most recent version:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/eth-cscs/manta/releases/download/v1.37.0/manta-installer.sh | sh
```

If you try to run manta, you should get an error because the configuration file is missing.

```bash
$ manta --help
Error processing config.toml file. Reason:
configuration file "/home/msopena/.config/manta/config.toml" not found
```

So far so good, please proceed to the next section ([Configuration](configuration.md)) to create the manta configuration file
