# NPM Package
## Overview
the npm package wraps spa-client command line by [napi-rs](https://github.com/napi-rs/napi-rs),
like [SWC](https://github.com/swc-project/swc).
So both have same user experience and same api.

There has an example project for npm package users: 
[js-app-example](https://github.com/timzaak/spa-server/tree/master/example/js-app-example).

## Install in new project
1. Install spa-client npm package.
```shell
npm install spa-client --save-dev
```
2. add config for spa-client in the [.env](https://github.com/timzaak/spa-server/blob/master/example/js-app-example/.env) file


3. Add script to package.json (need `dotenv`).

```json
{
  "script":{
      "upload": "dotenv .env.prod spa-client upload ./build www.example.com",
      "release":"dotenv .env.prod spa-client release www.example.com"
  }
}
```

If you don't want to use `dotenv`, just like this, the config file is like [client_config_env.conf](./spa-client-command-line#config-reference)

```json
{
  "script":{
      "upload": "spa-client -c $CONFIG_PATH upload ./build www.example.com ",
      "release": "spa-client -c $CONFIG_PATH release www.example.com"
  }
}
```


## Support Platform
It now supports six popular architecture, feel free to open issue if it does not work at your platform.

- aarch64-apple-darwin
- x86_64-apple-darwin
- x86_64-pc-windows-msvc
- i686-pc-windows-msvc
- x86_64-unknown-linux-gnu
- x86_64-unknown-linux-musl

## Source Code
```shell
git clone --recursive https://github.com/timzaak/spa-server
cd jsclient && yarn install && yarn build:release
```