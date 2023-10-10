_all:
    @just --list

# Format code
fmt:
    cargo fmt

# Lint code
lint:
    cargo clippy

# Build code
build:
    cargo build --release

# Run executable
run:
    cargo run --release