# -*- mode: dockerfile -*-

# You can override this `--build-arg BASE_IMAGE=...` to use different
# version of Rust
ARG BASE_IMAGE=rust:1.59

ARG RUNTIME_IMAGE=debian:buster-slim

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

# Add our source code.
ADD . .

# Build our application.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
     cargo build --package spa-server --release

FROM ${RUNTIME_IMAGE}

RUN mkdir /data
COPY --from=builder ./config.release.conf ./config.conf
COPY --from=builder ./target/release/spa-server .

CMD ["./spa-server"]
