## Docker Build
Use [moby/buildkit](https://github.com/moby/buildkit) to speed up building process.

### Shell
```bash
VERSION=1.2.1

DOCKER_BUILDKIT=1 docker build . -t="timzaak/spa-server:$VERSION" && \
docker push timzaak/spa-server:$VERSION
```

you can also use `make` to release
```shell
make docker-release VERSION=1.2.3
```