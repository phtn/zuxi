# Project Notes

## GPUI dependency

GPUI is pinned to Zed commit
`06b6160d46ae8a9074cd367ed64f742b47beca64`, fetched from the upstream `main`
branch on 2026-07-29.

Upgrade GPUI only in a dedicated, reviewed change. Update the workspace dependency
revision and this note together, then run the full formatting, check, Clippy, and
test suite before merging.

This GPUI revision requires Rust 1.95.0. The repository pins that toolchain in
`rust-toolchain.toml` so local development and CI use the same compiler.
