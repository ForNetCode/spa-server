# -*- mode: dockerfile -*-

# Dockerfile for spa-server

# You can override this `--build-arg BASE_IMAGE=...` to use different
# version of Rust
ARG BASE_IMAGE=rust:alpine

ARG RUNTIME_IMAGE=alpine

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

# Add our source code.
ADD . .

RUN apk add --no-cache musl-dev

# Build our application.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
     cargo build --package spa-server --release

FROM ${RUNTIME_IMAGE}
RUN mkdir /data
RUN apk add --no-cache tini
COPY --from=builder ./config.release.toml /config/config.toml
COPY --from=builder ./target/release/spa-server /usr/bin/

ENTRYPOINT ["/sbin/tini", "--"]

CMD ["spa-server"]
