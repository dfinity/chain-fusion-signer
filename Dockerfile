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


# Gets dfx version
#
# Note: This can be done in the builder but is slow because unrelated changes to dfx.json can cause a rebuild.
FROM base AS tool_versions
SHELL ["bash", "-c"]
RUN mkdir -p config
COPY dfx.json dfx.json
RUN jq -r .dfx dfx.json > config/dfx_version


# Install tools && warm up the build cache
FROM base AS builder
SHELL ["bash", "-c"]
# Install dfx
# Note: dfx is installed in `$HOME/.local/share/dfx/bin` but we can't reference `$HOME` here so we hardcode `/root`.
COPY --from=tool_versions /config/*_version config/
ENV PATH="/root/.local/share/dfx/bin:/root/.local/bin:${PATH}"
RUN DFXVM_INIT_YES=true DFX_VERSION="$(cat config/dfx_version)" sh -c "$(curl -fsSL https://sdk.dfinity.org/install.sh)" && dfx --version
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
RUN    mkdir -p src/signer/canister/src \
    && touch    src/signer/canister/src/lib.rs \
    && mkdir -p src/signer/api/src \
    && touch    src/signer/api/src/lib.rs \
    && mkdir -p src/example_backend/src \
    && touch    src/example_backend/src/lib.rs \
    && cargo build --locked --target wasm32-unknown-unknown \
    && rm -rf src


# Builds canister: example_backend
FROM builder AS build-example_backend
COPY src src
COPY dfx.json dfx.json
COPY canister_ids.json canister_ids.json
RUN touch src/*/src/*.rs
RUN dfx build --ic example_backend
RUN cp .dfx/ic/canisters/example_backend/example_backend.wasm /example_backend.wasm.gz
RUN sha256sum /example_backend.wasm.gz

FROM scratch AS example_backend
COPY --from=build-example_backend /example_backend.wasm.gz /

# Builds canister: signer
FROM builder AS build-signer
COPY src src
COPY dfx.json dfx.json
COPY canister_ids.json canister_ids.json
COPY scripts/build.signer.sh scripts/build.signer.args.sh scripts/build.signer.report.sh scripts/
COPY target/commit target/tags target/
RUN touch src/*/src/*.rs
RUN dfx build --ic signer

FROM scratch AS signer
COPY --from=build-signer out/ /
