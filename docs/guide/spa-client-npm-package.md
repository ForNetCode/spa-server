# NPM Package
## Overview
the npm package wraps spa-client command line by [napi-rs](https://github.com/napi-rs/napi-rs),
like [swc](https://github.com/swc-project/swc).
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


## Operating Systems

|                  | node12 | node14 | node16 |
| ---------------- |--------|--------|--------|
| Windows x64      | ✓      | ✓      | ✓      |
| Windows x32      | ✓      | ✓      | ✓      |
| Windows arm64    | x      | x      | x      |
| macOS x64        | ✓      | ✓      | ✓      |
| macOS arm64      | ✓      | ✓      | ✓      |
| Linux x64 gnu    | x      | x      | ✓      |
| Linux x64 musl   | ✓      | ✓      | ✓      |
| Linux arm gnu    | ✓      | ✓      | ✓      |
| Linux arm64 gnu  | ✓      | ✓      | ✓      |
| Linux arm64 musl | ✓      | ✓      | ✓      |
| Android arm64    | ✓      | ✓      | ✓      |
| Android armv7    | ✓      | ✓      | ✓      |
| FreeBSD x64      | ✓      | ✓      | ✓      |

Windows arm64 blocked by: https://github.com/briansmith/ring/issues/1167

Linux x64 gnu(node12,node14) blocked by : `Error: /build/jsclient/spa-client.linux-x64-gnu.node: cannot allocate memory in static TLS block`

But I test on my OpenSUSE Linux(x64 gnu) with nodeV14.17.1, it runs successfully.


## Source Code
```shell
git clone --recursive https://github.com/timzaak/spa-server
cd jsclient && yarn install && yarn build
```
### Command Line
You can install `spa-client` commandline tool. the doc is [here](https://timzaak.github.io/spa-server/guide/spa-client-command-line.html#source-code) 
