## SPA-SERVER
This project is to create a static web server which make deploy and manage multiple single page applications easy and cost less.

More details are described at [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) in Chinese.

### Run Server
You can get all config description in file: [`config.release.conf`](./config.release.conf). If you want to change the server config file path, 
please set environment variable `SPA_CONFIG=${config_path}`.

```shell
git clone git@github.com:timzaak/spa-server.git
git submodule init && git submodule update
# please remember to change `file_dir` in config.conf
cp config.release.conf config.conf
RUST_LOG=info cargo run --bin spa-server 
```

You can build docker image by `docker build . -t=?`, and push it to your private docker repo. There no plan to release it to docker hub.

### How To Use
Before running the server up, please read the config.release.conf file firstly. It's simple now.

After the server up. Copy your spa files to the directory where the admin server told, all the admin server api is in the [wiki](https://github.com/timzaak/spa-server/wiki/Admin-Server-API):

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

#### version 0.x
- [x] 80 and 443 both support
- [x] ~~compression~~ it can be done by frontend pack tool.
- [x] ~~multiple tls support~~ the feature may do not need.
- [x] cache file(cache all files in memory without LRU)

#### version 1.x
- [x] more doc(how to update static files)
- [x] rewrite Dockerfile to reduce docker image size
- [x] cache improve(big file ignore config option and if-range header support)
- [ ] more log for debug and trace
- [ ] refactor for test
- [ ] domain visit count/data analysis
- [ ] header `etag` `cache-control` `expires` `age` for client cache

### why use self maintained warp
[#171 Add reply::file(path) helper](https://github.com/seanmonstar/warp/issues/171)

This project used lots of api in war/fs.rs which is private, will change back until this issue can be solved.
