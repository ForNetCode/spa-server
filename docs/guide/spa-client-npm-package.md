# NPM Package

## Install in a new project

```shell
npm install --save-dev spa-client
```

Use with `.env` and vars, we would run the following command:

```shell
npm install --save-dev dotenv-cli cross-var spa-client
```

the `pachage.json` would like:

```json
{
  "scripts": {
    "upload": "dotenv cross-var spa-client upload ./build %DOMAIN%",
    "release": "dotenv cross-var spa-client release %DOMAIN%",
    "deploy": "npm run build && npm run upload && npm run release"
  }
}

```

the `.env` would be like:

```angular2html
# all config start with `SPA` for spa-client
SPA_SERVER_ADDRESS=http://127.0.0.1:9000

# spa server token
SPA_SERVER_AUTH_TOKEN=token

# default is 3
SPA_UPLOAD_PARALLEL=3

#DOMAIN=www.example.com/a
DOMAIN=www.example.com
```

There has more info at [getting started](./getting-started.md#run-spa-client-in-npm-package)

There id an example project for npm package users, you can view the package.json:
[js-app-example](https://github.com/fornetcode/spa-server/tree/master/example/js-app-example).

## Build Source Code

```shell
git clone --recursive https://github.com/fornetcode/spa-server
cd jsclient && npm install && npm build
```
