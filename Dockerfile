# -*- mode: dockerfile -*-
#
# An example Dockerfile showing how to build a Rust executable using this
# image, and deploy it with a tiny Alpine Linux container.

# You can override this `--build-arg BASE_IMAGE=...` to use different
# version of Rust or OpenSSL.
ARG BASE_IMAGE=rust:1.59

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

# Add our source code.
ADD . .

# Build our application.
RUN cargo build --release

FROM rust:1.59

RUN mkdir /data
COPY --from=builder ./config.release.conf ./config.conf
COPY --from=builder ./target/release/spa-server .


CMD ["./spa-server"]

