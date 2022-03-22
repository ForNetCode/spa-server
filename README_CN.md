# SPA-SERVER
本项目是用来创建一个托管静态web页面的服务，目标是使单页应用部署简便、开销少。

[ENGLISH README](./README.md)
## 特性
- 基于 Hyper 和 warp 构建。
- SSL 基于 Rustls.
- 服务器端缓存、客户端缓存（Cache-Content）、Gzip压缩.
- SPA 版本管理， 仅需要一个 http 请求就能实现版本回滚和更新。
- 支持热更新(Mac and Linux).
- 支持 CORS 跨域
- http/https 同时服务（http 也可用返回 redirect https）.
- 支持 Docker 镜像 

## 服务跑起来

配置文件说明: [`config.release.conf`](./config.release.conf). 

可以通过修改环境变量 `SPA_CONFIG=${config_path}`， 来更改配置文件地址。
### 通过源码
```shell
git clone git@github.com:timzaak/spa-server.git
cd spa-server
git submodule init && git submodule update
cp config.release.conf config.conf # please remember to change `file_dir` in config.conf
RUST_LOG=info cargo run --bin spa-server 
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

## 项目规划
请参阅 [项目规划](./doc/Roadmap.md).

目前没有任何关于 v1.3 版本的功能规划，如果您有何项目，可随时开 issue 或 WX 联系我： evenst 

## 为何自行维护 warp 版本
[#171 Add reply::file(path) helper](https://github.com/seanmonstar/warp/issues/171)

本项目用了很多 warp/src/filters/fs.rs 的私有API。

## 项目起源
请跳转至 [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) 浏览。