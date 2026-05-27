# Contributing to OxideMC

## Prerequisites

- Rust 1.85+ (edition 2024)
- Cargo

## Setup

```bash
git clone https://github.com/ezraclintoc/OxideMC.git
cd OxideMC
cargo build
```

Run the TUI:

```bash
cargo run -p oxidemc-tui
```

Run the web UI:

```bash
cargo run -p oxidemc-webui
```

## Project layout

See [docs/structure.md](docs/structure.md) for how the workspace and config files are organized.

## Making changes

- Keep UI code out of `oxidemc-core`. Core must be usable headlessly.
- New server type support goes in `oxidemc-core`.
- Open an issue before starting large changes so we can align on approach.

## Pull requests

- One logical change per PR.
- Make sure `cargo clippy` and `cargo test` pass before opening.
- Describe *why* the change is needed, not just what it does.
