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

# Record VHS
vhs-record:
    docker run \
        --rm \
        -v $PWD:/vhs \
        --entrypoint /bin/sh \
        ghcr.io/charmbracelet/vhs \
        -c "apt update && apt install vim -y && vhs doc/demo.tape"