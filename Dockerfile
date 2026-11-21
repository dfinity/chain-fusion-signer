#
# Reproducible Builds
#

FROM --platform=linux/amd64 ubuntu:24.04 AS base
ENV TZ=UTC
# Install basic tools
RUN DEBIAN_FRONTEND=noninteractive apt update && apt install -y \
    curl \
    ca-certificates \
    build-essential \
    pkg-config \
    libssl-dev \
    llvm-dev \
    liblmdb-dev \
    clang \
    cmake \
    jq \
    xxd \
    && rm -rf /var/lib/apt/lists/*


# Install tools && warm up the build cache
FROM base AS builder
SHELL ["bash", "-c"]
# Tools installed by `scripts/setup` with the "curl" method, such as didc, go to `$HOME/.local/bin`.
# We can't reference `$HOME` here so we hardcode `/root`.
ENV PATH="/root/.local/bin:${PATH}"
# Install Rust
COPY ./rust-toolchain.toml .
ENV RUSTUP_HOME=/opt/rustup \
    CARGO_HOME=/cargo \
    PATH=/cargo/bin:$PATH
COPY dev-tools.json dev-tools.json
COPY scripts/setup scripts/setup-cargo-binstall scripts/setup-rust scripts/
RUN scripts/setup rust
RUN scripts/setup cargo-binstall
RUN scripts/setup candid-extractor
RUN scripts/setup ic-wasm
RUN scripts/setup didc
# Optional: Pre-build dependencies
COPY Cargo.lock .
COPY Cargo.toml .
COPY src/example_backend/Cargo.toml src/example_backend/Cargo.toml
COPY src/signer/api/Cargo.toml src/signer/api/Cargo.toml
COPY src/signer/canister/Cargo.toml src/signer/canister/Cargo.toml
COPY src/test_proxy/Cargo.toml src/test_proxy/Cargo.toml
RUN    mkdir -p src/signer/canister/src \
    && touch    src/signer/canister/src/lib.rs \
    && mkdir -p src/signer/api/src \
    && touch    src/signer/api/src/lib.rs \
    && mkdir -p src/example_backend/src \
    && touch    src/example_backend/src/lib.rs \
    && mkdir -p src/test_proxy/src \
    && touch    src/test_proxy/src/lib.rs \
    && cargo build --locked --target wasm32-unknown-unknown \
    && rm -rf src


# Builds canister: example_backend
FROM builder AS build-example_backend
COPY src src
COPY scripts/build.example_backend.sh scripts/
RUN touch src/*/src/*.rs
RUN scripts/build.example_backend.sh
RUN cp out/example_backend.wasm.gz /example_backend.wasm.gz
RUN sha256sum /example_backend.wasm.gz

FROM scratch AS example_backend
COPY --from=build-example_backend /example_backend.wasm.gz /

# Builds canister: signer
FROM builder AS build-signer
COPY src src
COPY scripts/build.signer.sh scripts/build.signer.args.sh scripts/build.signer.report.sh scripts/
COPY target/commit target/tags target/
RUN touch src/*/src/*.rs
RUN DFX_NETWORK=ic scripts/build.signer.sh

FROM scratch AS signer
COPY --from=build-signer out/ /
