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

## Run spa-client in npm project
1. Install spa-client npm package.
```shell
npm install spa-client dotenv --save-dev
```
2. add config for spa-client in the `.env` file
```dotenv
# all config start with `SPA` for spa-client
SPA_SERVER_ADDRESS=http://127.0.0.1:9000

SPA_SERVER_AUTH_TOKEN=token

# upload file parallel number, optional, default is 3
SPA_UPLOAD_PARALLEL=3
```

3. Add script to package.json (need `dotenv`). `www.example.com` is the domain of your website, and `./build` is directory of static web files.

```json
{
  "script":{
      "upload": "dotenv .env spa-client upload ./build www.example.com",
      "release":"dotenv .env spa-client release www.example.com"
  }
}
```


## Run spa-client by docker

spa-client config support environment variables and file, for simple, we use environment variables to inject config.

```shell
$ docker run --rm -it -v /path/build:/build \
 -e SPA_SERVER_ADDRESS='http://127.0.0.1:9000' \
 -e SPA_SERVER_AUTH_TOKEN='token' \
 timzaak/spa-client:lastest \
 spa-client upload ./build www.example.com && \
 spa-client release www.example.com
```
By now, your single page application is in serving at `http://www.example.com:8080`.(please add this dns record to your host)

## What's More
- a React example for spa-client: [js-app-example](https://github.com/timzaak/spa-server/blob/master/example/js-app-example/README.md).
- spa-server [configuration](./spa-server-configuration.md) and its admin-server [http api](./spa-server-api.md)
- spa-client [commands](./spa-client-command-line.md) and [npm package](./spa-client-npm-package.md)



