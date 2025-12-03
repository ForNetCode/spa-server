# SPA-SERVER

[![GitHub Release](https://img.shields.io/github/release/ForNetCode/spa-server?color=brightgreen)](https://github.com/ForNetCode/spa-server/releases)
[![Build status](https://github.com/ForNetCode/spa-server/actions/workflows/spa-server-ci.yml/badge.svg)](https://github.com/ForNetCode/spa-server/actions/workflows/spa-server-ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-green)](LICENSE)
[![Maintenance](https://img.shields.io/badge/Maintained%3F-yes-green.svg)](https://github.com/ForNetCode/spa-server/graphs/commit-activity)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](https://github.com/ForNetCode/spa-server/pulls)

专为静态页面提供全链路优化的托管服务。

[ENGLISH README](./README.md)

## 特性

- 基于 Salvo 构建。
- SPA 版本管理， 仅需要一个 http 请求就能实现版本回滚和更新。
- 支持 Docker 镜像(压缩后大小:~26M)
- 提供 命令行/npm包 客户端，一行命令部署
- 每个域名可拥有独立的配置
- 提供JS SDK、命令行客户端与服务器进行交互。

## 文档

中文 README 目前仅提供一些简易`快速使用`指引，更多内容可参考英文文档，
中文版会在后续有精力的时候做，其网站托管在 [GitHub Pages](https://fornetcode.github.io/spa-server)，

## 服务跑起来

配置文件说明: [`config.release.conf`](./config.release.conf).

可以通过修改环境变量 `SPA_CONFIG=${config_path}`， 来更改配置文件地址。

### 通过源码

```shell
git clone git@github.com:fornetcode/spa-server.git
cd spa-server
cp config.release.toml config.toml # please remember to change `file_dir` in config.conf
cargo run --bin spa-server 
```

### 通过镜像

```shell
docker run -d -p 80  -v $HOST_VOLUME:/data -v $CONFIG:/config.conf ghcr.io/fornetcode/spa-server:latest
```

## 如何部署静态文件

如果你想集成到JS项目中，请参阅：[example/js-app-example](example/tmp/README.md)。

## 项目规划

请参阅 [项目规划](docs/develop/roadmap.md).

## 项目起源

请跳转至 [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) 浏览。
