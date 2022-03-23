## Docker Build
Use [docker/buildkit](https://github.com/moby/buildkit) to speed up building process.

### Shell
```bash
VERSION=1.2.1

DOCKER_BUILDKIT=1 docker build . -t="timzaak/spa-server:$(VERSION)" && \
docker push timzaak/spa-server:$(VERSION)
```