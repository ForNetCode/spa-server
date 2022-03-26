# SPA-SERVER
专为静态页面提供全链路优化的托管服务。

[ENGLISH README](./README_EN.md)
## 特性
- 基于 Hyper 和 warp 构建。
- SSL 基于 Rustls。
- 服务器端缓存、客户端缓存（Cache-Content）、Gzip压缩。
- SPA 版本管理， 仅需要一个 http 请求就能实现版本回滚和更新。
- 支持热更新(Mac and Linux)。
- 支持 CORS 跨域
- http/https 同时服务（http 也可返回 redirect https）。
- 支持 Docker 镜像(压缩后大小:32M)

## 服务跑起来

配置文件说明: [`config.release.conf`](./config.release.conf). 

可以通过修改环境变量 `SPA_CONFIG=${config_path}`， 来更改配置文件地址。

### 通过源码
```shell
git clone git@github.com:timzaak/spa-server.git
cd spa-server
git submodule init && git submodule update
cp config.release.conf config.conf # please remember to change `file_dir` in config.conf
cargo run --bin spa-server 
```

### 通过镜像
```shell
docker run -d -p 80 -p 443 -v $HOST_VOLUME:/data -v $CONFIG:/config.conf timzaak/spa-server:latest
```

## 如何部署静态文件

当服务跑起来后，可将 SPA 文件夹复制到 admin server 指定文件夹, （[api doc](./doc/Admin_Server_API.md)）.

```shell
scp $SPA_DIRECTORY user@ip:$(curl "http://$ADMIN_SERVER/upload/path?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN")
```
等文件传输完毕， 发起 HTTP 请求更新 SPA 版本。
```shell
curl "http://$ADMIN_SERVER/update_version?domain=$DOMAIN&version=$VERSION" -H "Authorization: Bearer $TOKEN"
```

至此，部署完毕！

## 使用场景

### 单独使用
根据自己的需求配置好 `cache`, 可获得极大的性能提升。 若需要热加载功能（ssl证书更替）/文件版本管理，请开启 `admin server`。
### 搭配 Nginx 使用
请在默认配置的基础上，请对 `cache.compression, cache.client_cache, cors` 保持默认，相关配置所带来的功能可设定在Nginx端，预防本服务和Nginx出现冲突。
### 高可用
目前项目没有做高可用的适配，需要注意以下几点：
1. 文件最好放在 S3/NFS 等可以共享到所有spa-server实例的文件系统中，否则 `Last-Modified` 不一致会对客户端缓存有一定影响。
2. 所有控制请求，需要对每个实例都发起一遍。

## 项目规划
请参阅 [项目规划](./doc/Roadmap.md).

目前没有任何关于 v1.3 版本的功能规划，如果您有何项目，可随时开 issue 或 WX 联系我： evenst 

## 为何自行维护 warp 版本
[#171 Add reply::file(path) helper](https://github.com/seanmonstar/warp/issues/171)

本项目用了很多 warp/src/filters/fs.rs 的私有API。

## 项目起源
请跳转至 [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) 浏览。