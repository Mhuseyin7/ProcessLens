# Contributing

Run `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace` before opening a pull request. Keep OS-specific behavior behind platform provider traits, and do not log command lines or environment values unnecessarily.
