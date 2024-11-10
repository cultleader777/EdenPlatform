# Development

## Setup

It is assumed that you're using NixOS to develop Eden platform. No other operating system will ever be supported.

Open these ports for virtual machines so they could take care of caching of nix store and docker containers

NixOS `configuration.nix`:

```
    networking.firewall.allowedTCPPorts = [
      12777 # docker containers
      12778 # nix store cache
    ];

    # docker required
    virtualisation.docker.enable = true;
    virtualisation.containerd.enable = true;
    # libvirtd required for running tests
    virtualisation.libvirtd.enable = true;
```

Nix shell described in `shell.nix` file. Use `direnv allow` to automatically evaluate shell environments.

## Running tests

### Cargo

Once you're inside `shell.nix` custom shell environment make sure you can run cargo tests.
First run of tests might take time because it is pulling docker images of postgres and clickhouse because these are used in tests.

```
cargo test
```

### Running a test smoke environment

Go into environment directory
```
cd test-envs/envs/single-dc
```
Compile the project
```
make compile
```
Setup the environment
```
sudo -E make full-provision
```
You can login to machines like so in other terminal
```
make login_server-a
```
Once `full-provision` step succeeds try running integration tests
```
make integration-tests
```

## Local release

To locally build and release eden platform executable for production run
```
cargo build --release && cp target/release/epl ~/bin
```

We assume ~/bin is in path.

## Testing

There are these ways to test code when working with Eden platform

### Source compilation error tests

In src/tests/ directory where most tests use function `assert_platform_validation_error`.

There is common data added not to bootstrap datacenters, can be removed with

```
        common::assert_platform_validation_error_wcustom_data_wargs(
            common::TestArgs {
                add_default_global_flags: false,
                add_default_data: false,
            },
```

Sources are parsed and all eden platform checks are run up until first returned error.

This is preferred way to test because it runs the fastest

### Integration tests with simulated/real test environments

Under `test-envs/` directory exists test environments. This is useful to find out if codegen actually works and infrastructure builds.

Integration tests for every project are automatically generated from codegen, but custom ones can be added in environments `integration-tests/src/manual.rs` file.

Usually for testing features `single-dc` environment is enough which runs libvirt virtual machines for testing.

To run this environment:
```
cd test-envs/envs/single-dc
sudo -E make full-provision-with-tests
```

Teardown after finish:
```
sudo -E make teardown
```

These are useful when testing cloud specific datacenter implementations.

### Configuration simulation tests

This is used after first testing the environment with integration tests and freezing the configuration.

These kind of tests run all checks, expect they pass and generate configuration files for all the environments which then can be frozen to know for a fact nothing broke and not to run expensive integration tests.

Examples of such tests can be found in `src/tests/networking/simulation/tests.rs`
