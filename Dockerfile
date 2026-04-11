ARG JOBS=1
ARG BUILD_TYPE=release

FROM node AS web
WORKDIR /web
COPY web .
RUN yarn
RUN yarn build

FROM rust:1.94.1-bookworm AS compiler
ARG JOBS
ARG BUILD_TYPE
RUN echo "Using $JOBS jobs"
RUN echo "Using build type: $BUILD_TYPE"
RUN apt-get update && apt-get install -y build-essential libtool libreadline-dev zlib1g-dev flex bison libseccomp-dev \
                      libssl-dev clang pkg-config libpq-dev cmake postgresql-client protobuf-compiler \
                      libprotobuf-dev libcurl4-openssl-dev openssl lsof libicu-dev
RUN export PROTOC_VERSION=22.2 && curl -fsSL "https://github.com/protocolbuffers/protobuf/releases/download/v${PROTOC_VERSION}/protoc-${PROTOC_VERSION}-linux-$(uname -m | sed 's/aarch64/aarch_64/g').zip" -o "protoc.zip" \
        && unzip -q protoc.zip -d protoc \
        && mv protoc/bin/protoc /usr/local/bin/protoc \
        && mv protoc/include/google /usr/local/include/google \
        && rm -rf protoc.zip protoc
WORKDIR /neond
COPY . .
COPY --from=web /web/dist /neond/web/dist
RUN rustup target add aarch64-unknown-linux-gnu
RUN rustup target add x86_64-unknown-linux-gnu
RUN CARGO_BUILD_JOBS=$JOBS BUILD_TYPE=$BUILD_TYPE make -C neon -j $JOBS -s
RUN if [ "$BUILD_TYPE" = "release" ]; then \
        CARGO_BUILD_JOBS=$JOBS BUILD_TYPE=$BUILD_TYPE cargo build --jobs $JOBS --release; \
    else \
        CARGO_BUILD_JOBS=$JOBS BUILD_TYPE=$BUILD_TYPE cargo build --jobs $JOBS; \
    fi

FROM debian:bookworm-slim
ARG BUILD_TYPE
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 libpq5 libreadline8 libseccomp2 libcurl4 libicu72 zlib1g liblz4-1 libzstd1 libxml2 libkrb5-3 && rm -rf /var/lib/apt/lists/*
RUN groupadd --system --gid 1000 neond && useradd --system --uid 1000 --gid 1000 --create-home --home-dir /home/neond --shell /bin/bash neond
COPY --from=compiler /neond/target/${BUILD_TYPE}/neond /usr/local/bin/neond
COPY --from=compiler /neond/neon/target/${BUILD_TYPE}/safekeeper /usr/local/share/neon/bin/safekeeper
COPY --from=compiler /neond/neon/target/${BUILD_TYPE}/pageserver /usr/local/share/neon/bin/pageserver
COPY --from=compiler /neond/neon/target/${BUILD_TYPE}/compute_ctl /usr/local/share/neon/bin/compute_ctl
COPY --from=compiler /neond/neon/target/${BUILD_TYPE}/storage_broker /usr/local/share/neon/bin/storage_broker
COPY --from=compiler /neond/neon/target/${BUILD_TYPE}/storage_controller /usr/local/share/neon/bin/storage_controller
COPY --from=compiler /neond/neon/pg_install /usr/local/share/neon/pg_install
ENV NEON_BINARIES_DIR=/usr/local/share/neon/bin
ENV PG_INSTALL_DIR=/usr/local/share/neon/pg_install
RUN mkdir -p /neond && chown -R neond:neond /neond /usr/local/share/neon
WORKDIR /neond
USER neond
ENTRYPOINT ["/usr/local/bin/neond"]