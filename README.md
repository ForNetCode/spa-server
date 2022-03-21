## SPA-SERVER
This project is to create a static web server which make deploy and manage multiple single page applications easy and cost less.

More details are described at [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) in Chinese.

### Run Server
You can get all config description in file: [`config.release.conf`](./config.release.conf). If you want to change the server config file path, 
please set environment variable `SPA_CONFIG=${config_path}`.

```shell
git clone git@github.com:timzaak/spa-server.git
git submodule init && git submodule update
cp config.release.conf config.conf # please remember to change `file_dir` in config.conf
RUST_LOG=info cargo run --bin spa-server 
```

You can build docker image by `docker build . -t=?`, and push it to your private docker repo. There no plan to release it to docker hub.

### How To Use
Before running the server up, please read the config.release.conf file firstly. It's simple now.

After the server up. Copy your spa files to the directory where the admin server told, all the admin server api is in the [doc](./doc/Admin_Server_API.md):

```shell

scp $SPA_DIRECTORY user@ip:$(curl "http://$ADMIN_SERVER/upload/path?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN")

```

Request the admin server to change spa version.
```shell
curl "http://$ADMIN_SERVER/update_version?domain=$DOMAIN&version=$VERSION" -H "Authorization: Bearer $TOKEN"
```

That's all!

### Roadmap 
#### before release
- [x] very simple http1 spa server
- [x] very simple admin server(http api)
- [x] ssl(including wildcard domain ssl)
- [x] docker release
- [x] simple usage doc

#### version 1.0.x
- [x] 80 and 443 both support
- [ ] ~~compression~~ done @ v1.2.0.
- [ ] ~~multiple tls support~~ the feature may do not need.
- [x] cache file(cache all files in memory without LRU)

#### version 1.1.x
- [x] more doc(how to update static files)
- [x] rewrite Dockerfile to reduce docker image size
- [x] cache improve(big file ignore config option and if-range header support)
- [x] header`cache-control` for client cache
- [ ] ~~header `etag` for client cache~~ [warp #462](https://github.com/seanmonstar/warp/issues/462)
- [x] 80 redirect to 443 config option
- [x] compression for js/icon/json/css/html (only support gzip algo, only compress cached files, and ~~will occur error when client don't support gzip~~(fix @ v1.2))

#### version 1.2.x
- [x] more log for debug and trace
- [x] basic CORS
- [x] compress regression support(if client don't send accept-encoding header(including gzip), will send back data from file instead of cache) 
- [ ] different config(cors/cache strategy/https and so on) for different domain
- [ ] hot reload web static server(use SO_REUSEPORT *nix api, so it may be wrong with Windows).


### Version Choice
If you only want a static web server without any cache/ssl/CORS feature support(serving for cdn), please use version v1.0.x, it's simple and efficient,
you are free to open any issue about this version. Otherwise, please use the most recent version.

### why use self maintained warp
[#171 Add reply::file(path) helper](https://github.com/seanmonstar/warp/issues/171)

This project uses lots of private api at warp/src/filters/fs.rs.
