ARG BASE_IMAGE=timzaak/spa-server
ARG VERSION=1.2.5

FROM ${BASE_IMAGE}:${VERSION} as Source


FROM panubo/s3fs:1.84
COPY --from=Source ./config.conf .
COPY --from=Source ./spa-server .

COPY entry.sh /entry.sh

ENTRYPOINT ["/entry.sh"]
