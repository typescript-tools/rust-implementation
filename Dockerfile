from rust:1.65.0@sha256:6d44ed87fe759752c89d1f68596f84a23493d3d3395ed843d3a1c104866e5d9e as build-image

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

from alpine:3.17.0@sha256:8914eb54f968791faf6a8638949e480fef81e697984fba772b3976835194c6d4
COPY --from=build-image /rust-implementation/target/x86_64-unknown-linux-musl/release/monorepo /usr/bin/monorepo
