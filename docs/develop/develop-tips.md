# Develop Tips
## Docker Build
Use [moby/buildkit](https://github.com/moby/buildkit) to speed up building process.

```bash
VERSION=1.2.1

DOCKER_BUILDKIT=1 docker build . -t="timzaak/spa-server:$VERSION"

```