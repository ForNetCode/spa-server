# SPA-SERVER
It's just for static web deployment with server side  cache, client side cache and hot reload.   

[中文 README](./README.md)

## Feature
- Built with Hyper and Warp.
- SSL with Rustls.
- Memory cache、client cache and compression(gzip).
- SPA version control, you can regress or release new version with one HTTP api command.
- Hot reload support(Mac and Linux).
- CORS
- http redirect to https or both serving at the same time.
- Docker support(compressed size: 32M)

## Run Server
You can get all config description in file: [`config.release.conf`](./config.release.conf). If you want to change the server config file path, 
please set environment variable `SPA_CONFIG=${config_path}`.
### Run By Code
```shell
git clone git@github.com:timzaak/spa-server.git
cd spa-server
git submodule init && git submodule update
cp config.release.conf config.conf # please remember to change `file_dir` in config.conf
cargo run --bin spa-server 
```
### Run Docker
```shell
docker run -d -p 80 -p 443 -v $HOST_VOLUME:/data -v $CONFIG:/config.conf timzaak/spa-server:latest
```

## How To Upload SPA Files
Before running server up, please read the config.release.conf file firstly. It's easy to understand.

After the server up. Copy your spa files to the directory where the admin server told, all the admin server api is in the [Admin_Server_API.md](./doc/Admin_Server_API.md).

```shell
scp $SPA_DIRECTORY user@ip:$(curl "http://$ADMIN_SERVER/upload/path?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN")
```

Request the admin server to change SPA version.
```shell
curl "http://$ADMIN_SERVER/update_version?domain=$DOMAIN&version=$VERSION" -H "Authorization: Bearer $TOKEN"
```

That's all!

## Roadmap
please ref [Roadmap.md](./doc/Roadmap.md).

If you have any idea, feel free to open issue.

## Why use self maintained warp
[#171 Add reply::file(path) helper](https://github.com/seanmonstar/warp/issues/171)

This project uses lots of private api at warp/src/filters/fs.rs.

## Project Origin
More details are described at [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) in Chinese.