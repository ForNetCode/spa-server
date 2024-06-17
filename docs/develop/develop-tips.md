# Develop Tips

## Docker Build

Use [moby/buildkit](https://github.com/moby/buildkit) to speed up building process.

```bash
VERSION=1.2.1

DOCKER_BUILDKIT=1 docker build . -t="ghcr.io/fornetcode/spa-server:$VERSION"

```

## SSL self sign

ref: https://docs.rancher.cn/docs/rancher2/installation/resources/advanced/self-signed-ssl/_index/