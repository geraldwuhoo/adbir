ARG ICONS=no-icons

# BEGIN RUST BUILD

# chef
FROM docker.io/library/rust:1.79.0 AS chef
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

# END RUST BUILD


# BEGIN OPTIONAL ICON

# get dashboard-icons
FROM docker.io/curlimages/curl:8.9.0 AS dashboard-icons

WORKDIR /out
# hadolint ignore=DL4006
RUN curl -o- -L "https://github.com/walkxcode/dashboard-icons/archive/refs/heads/main.tar.gz" | tar xzf - -C /out && \
    mv -v /out/dashboard-icons-main /out/icons

# dummy icons (none)
FROM docker.io/library/alpine:3.20.1 AS no-icons
RUN mkdir -vp /out/icons/png /out/icons/svg

# Create selectable intermediate stage based on desired icons
# hadolint ignore=DL3006
FROM ${ICONS} AS icons

# END OPTIONAL ICON


# darkhttpd webserver to generate and host the files
FROM docker.io/library/alpine:3.20.1

ENV UID 1000
ENV GID 1000

RUN apk add -U --no-cache darkhttpd=1.16-r0

WORKDIR /public
RUN chown ${UID}:${GID} /public
COPY entrypoint.sh /entrypoint.sh
COPY --from=builder --chown=${UID}:${GID} /usr/src/target/x86_64-unknown-linux-musl/release/adbir /usr/bin/adbir
COPY --from=icons --chown=${UID}:${GID} /out/icons/png /public/icons/png
COPY --from=icons --chown=${UID}:${GID} /out/icons/svg /public/icons/svg

USER ${UID}:${GID}

EXPOSE 8080

ENTRYPOINT ["/bin/sh", "/entrypoint.sh"]
