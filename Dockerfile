# rust:1.56.1
from rust@sha256:697241493f376f6adec75ab5005836008a51d83ce1916aba79c3ba7b98aeeb6a as build-image

# create a dummy project
RUN apt-get update && \
    apt-get install --yes musl-tools curl llvm clang && \
    rustup target add x86_64-unknown-linux-musl && \
    USER=root cargo new --bin rust-implementation
WORKDIR /rust-implementation

# copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# cache build dependencies
RUN cargo build --release --target x86_64-unknown-linux-musl && \
    rm -r src/

# copy over project source
COPY ./src ./src

# build for release
RUN rm -f ./target/x86_64-unknown-linux-musl/release/deps/monorepo* && \
    cargo build --release --target x86_64-unknown-linux-musl

# alpine:3.15.0
from alpine@sha256:21a3deaa0d32a8057914f36584b5288d2e5ecc984380bc0118285c70fa8c9300
COPY --from=build-image /rust-implementation/target/x86_64-unknown-linux-musl/release/monorepo /usr/bin/monorepo
