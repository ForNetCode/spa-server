# SPA-SERVER
This project is to create a static web server which make deploy and manage multiple single page applications easy and cost less.

## Feature
- Built with Hyper and Warp.
- SSL with Rustls.
- Memory cache、client cache and compression(gzip).
- SPA version control, you can regress or release new version with one HTTP api command.
- Hot reload support(Mac and Linux).
- CORS
- http redirect to https or both serving at the same time.

## Run Server
You can get all config description in file: [`config.release.conf`](./config.release.conf). If you want to change the server config file path, 
please set environment variable `SPA_CONFIG=${config_path}`.

```shell
git clone git@github.com:timzaak/spa-server.git
cd spa-server
git submodule init && git submodule update
cp config.release.conf config.conf # please remember to change `file_dir` in config.conf
RUST_LOG=info cargo run --bin spa-server 
```

You can build docker image by `docker build . -t=?`, and push it to your private docker repo. There no plan to release it to docker hub.

## How To Use
Before running server up, please read the config.release.conf file firstly. It's easy to understand.

After the server up. Copy your spa files to the directory where the admin server told, all the admin server api is in the [doc](./doc/Admin_Server_API.md).

```shell
scp $SPA_DIRECTORY user@ip:$(curl "http://$ADMIN_SERVER/upload/path?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN")
```

Request the admin server to change SPA version.
```shell
curl "http://$ADMIN_SERVER/update_version?domain=$DOMAIN&version=$VERSION" -H "Authorization: Bearer $TOKEN"
```

That's all!

## Roadmap
please ref [doc](./doc/Roadmap.md).

There's no feature plan about v1.3.x, if you have any idea, feel free to open issue.

## Why use self maintained warp
[#171 Add reply::file(path) helper](https://github.com/seanmonstar/warp/issues/171)

This project uses lots of private api at warp/src/filters/fs.rs.

## Project Origin
More details are described at [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) in Chinese.