## SPA-SERVER
This project is to create a static web server which make deploy and manage multiple single page applications easy and cost less.

More details are described at [SPA 发布辅助工具](https://github.com/timzaak/blog/issues/80) in Chinese.


### Run Server
You can get config description in file: `config.release.conf`. If you want to change the server config file path, 
please set environment variable `SPA_CONFIG=${config_path}`.

```shell
git clone git@github.com:timzaak/spa-server.git
git submodule init && git submodule update
RUST_LOG=info cargo run --bin spa-server 
```

You can build docker image by `docker build . -t=?`, and push it to your private docker repo. There no plan to release it to docker hub.

#### Admin Server
Admin server provide http api to control static web files version upgrade, be careful to it access config, and it's disabled by default.
```shell
ADMIN_SERVER='127.0.0.1:9000' 
TOKEN='token'

# get all domains status
curl "http://$ADMIN_SERVER/status" -H "Authorization: Bearer $TOKEN"
# return json: [{"domain":"www.example.com","current_version":1,"versions":[1]}]

# get specific domain status
DOMAIN='www.example.com'
curl "http://$ADMIN_SERVER/status?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN"
# return json: {"domain":"www.example.com","current_version":1,"versions":[1]} or status code:404

# get the domain upload file path, it can be used with `rsync/scp` to upload web static files to the server.
curl "http://$ADMIN_SERVER/upload/path?domain=$DOMAIN" -H "Authorization: Bearer $TOKEN"
# return string: /$FILE_PATH/$DOMAIN/$NEW_VERSION ,like /data/www.example.com/2

# update the domain version. please be attention:
# *it will use the newest version after server restart*
# *it will use the newest version after server restart*
# *it will use the newest version after server restart*
VERSION=2
curl "http://$ADMIN_SERVER/update_version?domain=$DOMAIN&version=$VERSION" -H "Authorization: Bearer $TOKEN"
# return status code: 200(update version success) or 404(can not find files, please make sure you have upload files to correct place)
```

### why use self maintained warp
[#171 Add reply::file(path) helper](https://github.com/seanmonstar/warp/issues/171)

Will change back until this issue can be solved

### Roadmap 
#### before release
- [x] very simple http1 spa server
- [x] very simple admin server(http api)
- [x] single tls (support http://cookcode.cc/selfsign self sign, others does not test now)
- [x] docker release
- [x] simple usage doc

#### version 1.x
- [x] 80 and 443 both support
- [ ] compression
- [ ] http2
~~- [ ] multiple tls support~~ the feature may do not need.
- [ ] domain visit count/data analysis
- [ ] memory fs
