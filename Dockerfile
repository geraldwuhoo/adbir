# chef
FROM docker.io/library/rust:1.77.2 AS chef
RUN rustup target add x86_64-unknown-linux-musl && \
    apt-get update && \
    apt-get install -y --no-install-recommends musl-tools=1.2.3-1 musl-dev=1.2.3-1 && \
    rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef
WORKDIR /usr/src

# planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Builder
FROM chef AS builder
COPY --from=planner /usr/src/recipe.json recipe.json

RUN cargo chef cook --release --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl --bin adbir

# darkhttpd webserver to generate and host the files
FROM docker.io/library/alpine:3.19.1

ENV UID 1000
ENV GID 1000

RUN apk add -U --no-cache darkhttpd=1.14-r1

WORKDIR /public
RUN chown ${UID}:${GID} /public
COPY entrypoint.sh /entrypoint.sh
COPY --from=builder --chown=${UID}:${GID} /usr/src/target/x86_64-unknown-linux-musl/release/adbir /usr/bin/adbir

USER ${UID}:${GID}

EXPOSE 8080

ENTRYPOINT ["/bin/sh", "/entrypoint.sh"]
