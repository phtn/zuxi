# Zuxi

A Rust + GPUI Markdown and PDF viewer. Zuxi scans a folder, lists supported files in
a sidebar, and previews the active document. Reusable tokens, themes, and primitives
live in `crates/design_system`; application state and document rendering live in
`crates/app`.

## Run

The repository pins the Rust toolchain and GPUI revision it needs. With no folder
argument, Zuxi scans the current directory:

```sh
cargo run -p app
```

Pass a folder to browse a different document library:

```sh
cargo run -p app -- /path/to/documents
```

Markdown files are parsed in-process. PDF pages are rasterized into a temporary,
automatically cleaned cache with Poppler. On macOS, install it with:

```sh
brew install poppler
```

The PDF preview renders the first 24 pages to keep selection responsive and reports
when a document has additional pages. Use the Reload button after adding or changing
files in the selected folder.

To inspect the primitives without the application:

```sh
cargo run -p design_system --example showcase
```

For automatic rebuilds, install `cargo-watch` once and use the repository alias:

```sh
cargo install cargo-watch
cargo dev
```

## Validate

```sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

See `PATTERNS.md` before introducing new stateful views or components, and `NOTES.md`
before changing the GPUI dependency.

## Learn the codebase

Start with [DEVLOG.md](DEVLOG.md) for a guided architecture map, the GPUI
concepts used by Zuxi, a record of important changes and debugging lessons,
known limitations, and a template for logging future work.
