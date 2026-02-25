# ym - YAML Manipulator CLI Tool

A fast, focused Unix-philosophy CLI tool for searching and patching YAML files.

## Overview

**ym** addresses the critical gap in DevOps tooling: engineers managing Kubernetes configs and Hiera hierarchies need to search YAML by full key path (not just grep), and patch multiple values programmatically. Neither `yq` nor `grep` fill this need perfectly.

## Installation

```bash
cargo build --release
./target/release/ym --version
```

## Usage

- use **grep** to look for a specific key:

    ```bash
    $ # ym grep [OPTIONS] <PATTERN> [FILES]...
    $ ym grep -R 'database\..*\.password' tests/data/
    tests/data/app-config.yaml:database.primary.password: super_secret_123
    tests/data/app-config.yaml:database.replica.password: super_secret_123
    tests/data/config-dev.yaml:database.primary.password: dev_password
    tests/data/config-prod.yaml:database.primary.password: prod_secret_xyz789
    tests/data/config-prod.yaml:database.replica.password: prod_secret_xyz789
    ```
- use **set** and **unset** to edit keys:

    ```bash
    $ ym set tests/data/app-config.yaml database.primary.port=5433 app.version=2.0.0
    $ ym unset tests/data/app-config.yaml database.primary.port
    ```
- use **cp** to copy a value from one key to another:

    ```bash
    $ # Copy value to a different file with different key
    $ ym cp tests/data/config-prod.yaml:database.primary.password tests/data/app-config.yaml:database.secondary.password

    $ # Copy value from source.key to destination.key (same file, destination file is optional and defaults to source file)
    $ ym cp tests/data/app-config.yaml:database.primary.password database.replica.password

    $ # Copy value to a different file (with same key, destination key is optional and defaults to source key)
    $ ym cp tests/data/app-config.yaml:app.name tests/data/config-prod.yaml:
    ```

- use **mv** to move a value from one key to another (copies then deletes the source):

    ```bash
    $ # Move value to a different file with different key
    $ ym mv tests/data/config-prod.yaml:database.primary.password tests/data/app-config.yaml:database.secondary.password

    $ # Move value from source.key to destination.key (same file, destination file is optional and defaults to source file)
    $ ym mv tests/data/app-config.yaml:database.primary.password database.replica.password

    $ # Move value to a different file (with same key, destination key is optional and defaults to source key)
    $ ym mv tests/data/app-config.yaml:app.name tests/data/config-prod.yaml:
    ```
