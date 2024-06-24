# SPA-SERVER
[![Build status](https://github.com/ForNetCode/spa-server/actions/workflows/spa-server-ci.yml/badge.svg)](https://github.com/ForNetCode/spa-server/actions/workflows/spa-server-ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green)](LICENSE)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://github.com/ForNetCode/spa-server/graphs/commit-activity)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/ForNetCode/spa-server/pulls)


It provides a static web http server with cache and hot reload.

[中文说明](./README_CN.md)

## Feature
- Built with Warp, fast and small!
- SSL with Rustls.
- Memory cache, client cache and compression(gzip).
- Static web version control, you can regress or release new version easily.
- Hot reload support(Mac and Linux).
- CORS support.
- Http auto redirect to https.
- Docker support(compressed size: ~26M)
- Provide command line/npm package to deploy spa.
- Multiple configs for different domain.
- support Let's Encrypt
- provide JS SDK and command line client to interact with Server

## Document
There is a nice [document](https://fornetcode.github.io/spa-server/) powered by VitePress and GitHub Pages,
you can quickly start spa-server with this [guide](https://fornetcode.github.io/spa-server/guide/getting-started.html). 

## Project Origin
More details are described at [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) in Chinese.