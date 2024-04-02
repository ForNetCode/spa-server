ARG BASE_IMAGE=ghcr.io/fornetcode/spa-server
ARG VERSION=1.2.6

FROM ${BASE_IMAGE}:${VERSION} as Source


FROM panubo/s3fs:1.84
COPY --from=Source /test/config.conf /config/config.conf
COPY --from=Source /usr/bin/spa-server /usr/bin/spa-server

COPY entry.sh /entry.sh

ENTRYPOINT ["/entry.sh"]
