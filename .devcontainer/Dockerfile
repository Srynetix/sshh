FROM mcr.microsoft.com/devcontainers/rust:1-1-bullseye

RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall cargo-edit -y && cargo binstall cargo-dist -y