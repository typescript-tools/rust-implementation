from rust:1.69.0@sha256:9d78a0a4235f3b63f4e8303f53248a146693fc825c15d0831d1e072e474aefdf as build-image

# create a dummy project
RUN apt-get update && \
    apt-get install --yes musl-tools curl llvm clang && \
    rustup target add x86_64-unknown-linux-musl
WORKDIR /rust-implementation

# copy over manifests
COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml

# copy over project source
COPY ./templates ./templates
COPY ./src ./src

# build for release
RUN rm -f ./target/x86_64-unknown-linux-musl/release/deps/monorepo* && \
    cargo build --release --target x86_64-unknown-linux-musl

from alpine:3.17.3@sha256:124c7d2707904eea7431fffe91522a01e5a861a624ee31d03372cc1d138a3126
COPY --from=build-image /rust-implementation/target/x86_64-unknown-linux-musl/release/monorepo /usr/bin/monorepo
