# Getting Started

This section will help you bring spa-server up, and upload your static web files to it.

## Run spa-server by docker

`uploading file` feature needs spa-server open admin-server. So we should create a config file first.

```bash
$ echo '
http {
  port = 8080
  addr = "0.0.0.0"
}
file_dir = "/data"

admin_config {
  port = 9000
  addr = "0.0.0.0"  
  token = "token"
}
' > config.conf

$ docker run -it -p 8080 -p 9000 -v $(pwd)/config.conf:/config.conf \
ghcr.io/fornetcode/spa-server:latest
```

## Run spa-client in npm project

1. Install spa-client npm package.

```shell
npm install spa-client dotenv --save-dev
```

2. add config for spa-client in the `.env` file

```
# all config start with `SPA` for spa-client
SPA_SERVER_ADDRESS=http://127.0.0.1:9000

SPA_SERVER_AUTH_TOKEN=token

# upload file parallel number, optional, default is 3
SPA_UPLOAD_PARALLEL=3
```

3. Add script to package.json (need `dotenv`). `www.example.com` is the domain of your website, and `./build` is
   directory of static web files.

```json
{
  "script": {
    "upload": "dotenv .env spa-client upload ./build www.example.com",
    "release": "dotenv .env spa-client release www.example.com"
  }
}
```

we also support serving multiple SPA in one domain:

```json
{
  "script": {
    "upload1": "dotenv .env spa-client upload ./build www.example.com/a",
    "release1": "dotenv .env spa-client release www.example.com/a",
    "upload2": "dotenv .env spa-client upload ./build www.example.com/b",
    "release2": "dotenv .env spa-client release www.example.com/b"
  }
}
```

## What's More

- a React example for
  spa-client: [js-app-example](https://github.com/fornetcode/spa-server/blob/master/example/js-app-example/README.md).
- spa-server [configuration](./spa-server-configuration.md) and its admin-server [http api](./spa-server-api.md)
- spa-client [npm package](./spa-client-npm-package.md)



