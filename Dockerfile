from rust:1.67.0@sha256:f17725f98801cad04c856a05f93a9c403025f2c18c0a73e45a64c5f9720b1c58 as build-image

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

from alpine:3.17.1@sha256:f271e74b17ced29b915d351685fd4644785c6d1559dd1f2d4189a5e851ef753a
COPY --from=build-image /rust-implementation/target/x86_64-unknown-linux-musl/release/monorepo /usr/bin/monorepo
