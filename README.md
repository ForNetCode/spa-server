# SPA-SERVER
It is to provide a static web http server with cache and hot reload.

[中文说明](./README_CN.md)

## Feature
- Built with Hyper and Warp, fast and small!
- SSL with Rustls.
- Memory cache, client cache and compression(gzip).
- Static web version control, you can regress or release new version easily.
- Hot reload support(Mac and Linux).
- CORS support.
- Http auto redirect to https.
- Docker support(compressed size: 32M), and support S3 as storage by S3FS.
- Provide command line/npm package to deploy spa.
- Multiple configs for different domain.

## Document
There is a nice [document](https://fornetcode.github.io/spa-server/) powered by VitePress and GitHub Pages,
you can quickly start spa-server with this [guide](https://fornetcode.github.io/spa-server/guide/getting-started.html). 

## Project Origin
More details are described at [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) in Chinese.