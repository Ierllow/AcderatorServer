# AcderatorServer

Rust backend for Acderator. It provides authentication, user data, and score APIs backed by a MySQL-compatible database.

## Setup

Install Rust, then run:

```bash
bash scripts/setup.sh
```

Enable the debug console with:

```bash
bash scripts/setup.sh --debug-ui
```

Setup logic lives in `scripts/dev.rs`; the shell scripts are thin wrappers. Run `bash scripts/setup.sh --help` for available options.

## Debug console

- `/debug`: API request inspector
- `/debug/master`: Master data browser and editor

## Development

Project settings are defined in `src/common/config.rs`.

```bash
bash scripts/cleanup_unused_uses.sh
```

## License

[MIT](LICENSE)
