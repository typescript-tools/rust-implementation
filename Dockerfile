# rust:1.56.1
from rust@sha256:dd7167fc31b49284971b42f9b227bcac2fe3b8c2709259ec64dab7a05b5b07b0 as build-image

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
RUN rm -f ./target/x86_64-unknown-linux-musl/release/deps/rust_typescript_tools* && \
    cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=build-image /rust-implementation/target/x86_64-unknown-linux-musl/release/rust_typescript_tools /usr/bin/typescript-tools
WORKDIR /workdir
ENTRYPOINT ["/usr/bin/typescript-tools"]
CMD [""]
