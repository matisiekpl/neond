ARG JOBS=1

FROM node AS web
WORKDIR /web
COPY web .
RUN yarn
RUN yarn build

FROM rust:1.67 AS compiler
ARG JOBS
RUN apt-get update && apt install -y build-essential libtool libreadline-dev zlib1g-dev flex bison libseccomp-dev \
                      libssl-dev clang pkg-config libpq-dev cmake postgresql-client protobuf-compiler \
                      libprotobuf-dev libcurl4-openssl-dev openssl lsof libicu-dev
RUN export PROTOC_VERSION=22.2 && curl -fsSL "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-$(uname -m | sed 's/aarch64/aarch_64/g').zip" -o "protoc.zip" \
        && unzip -q protoc.zip -d protoc \
        && mv protoc/bin/protoc /usr/local/bin/protoc \
        && mv protoc/include/google /usr/local/include/google \
        && rm -rf protoc.zip protoc
WORKDIR /neon
COPY . .
COPY --from=web /web/dist /neon/web/dist
RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup target add x86_64-unknown-linux-gnu
RUN CARGO_BUILD_JOBS=$JOBS BUILD_TYPE=release make -C neon -j $JOBS -s
RUN ARCH=$(uname -m) && \
    if [ "$ARCH" = "x86_64" ]; then \
        RUSTFLAGS="-C link-arg=-Wl,--no-relax -C code-model=medium" cargo build --release --jobs $JOBS; \
    else \
        cargo build --release --jobs $JOBS; \
    fi

FROM alpine
COPY --from=compiler /neon/target/release/neond /usr/local/bin/neond
WORKDIR /neon
ENTRYPOINT ["/usr/local/bin/neond"]