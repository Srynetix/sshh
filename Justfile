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

# Build app in Docker
build-in-docker:
    docker run \
        --name sshh-builder \
        -v $PWD:/app \
        rust:1.81-slim-bullseye \
        bash -c "cd /app && cargo build --release"

    mkdir -p ./target/release
    docker cp sshh-builder:/app/target/release/sshh ./target/release/sshh
    docker rm --force sshh-builder