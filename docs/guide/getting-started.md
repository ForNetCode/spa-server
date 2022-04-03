# Getting Started
This section will help you bring spa-server up, and upload your static web files to it.

## Run spa-server by docker
`uploading file` feature needs spa-server open admin-server. So we should create a config file first.

```bash
$ echo '
port = 8080
addr = "0.0.0.0"
file_dir = "/data"

admin_config {
  port = 9000
  addr = "0.0.0.0"  
  token = "token"
}
' > config.conf

$ docker run -it -p 8080 -p 9000 -v $(pwd)/config.conf:/config.conf \
timzaak/spa-server:latest
```

## Run spa-client by docker
::: tip
Assume your static files is in the path: `/path/build`, and your domain is `self.noti.link:8080`
PS: the domain `self.noti.link` is routed to 127.0.0.1, you can check by `ping self.noti.link`
:::
spa-client config support environment variables and file, for simple, we just use environment variables to inject config.

```shell
$ docker run --rm -it -v /path/build:/build \
 -e SPA_SERVER_ADDRESS='http://127.0.0.1:9000' \
 -e SPA_SERVER_AUTH_TOKEN='token' \
 timzaak/spa-client:lastest \
 spa-client upload /build self.noti.link:8080 && \
 spa-client release self.noti.link:8080
```
By now, your single page application is in serving at `http://self.noti.link:8080`.

## What's More
`spa-client` provide a npm package, you can integrate it easily with your project. check out the example: [js-app-example](https://github.com/timzaak/spa-server/blob/master/example/js-app-example/README.md).



