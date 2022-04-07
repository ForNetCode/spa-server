# NPM Package
## Overview
the npm package wraps spa-client command line by [napi-rs](https://github.com/napi-rs/napi-rs),
like [swc](https://github.com/swc-project/swc).
So both have same user experience and same api.

There has an example project for npm package users: 
[js-app-example](https://github.com/timzaak/spa-server/tree/master/example/js-app-example).

## Install in new project
there is more info at [getting started](./getting-started.md#run-spa-client-in-npm-package) 

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
You can install `spa-client` commandline tool if the npm package does not support your OS platform. read [doc](./spa-client-command-line#source-code). 
