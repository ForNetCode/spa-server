# Change Log

### Version 2.4.1
- spa-server: `cors` value bool to array string.

### Version 2.4.0

- improve: extract client and server common entity
- conf: **break change** remove hocon config
- ci: add js client build ci, improve release cd
- improve: clean deps, update claps
- feat: add openTelemetry trace

### Version 2.3.0

- feat: support toml config format, deprecated hocon config format.
- feat: support host alias. add config `http.external_port`, `https.external_port`
- conf: **break change** `https.http_redirect_to_https` move to `http.redirect_https`, and value is bool.
- improve: improve change_status response text style (release JS SDK 2.3.0)

### Version 2.2.4

- feat: add cert query API (no doc, no client SDK support)
- improve: add check when upload of multiple/single domain
- ci: improve GitHub Action speed
- feat: add revoke version API (release JS SDK 2.2.4)

### Version 2.2.3

- fix: sub_path '' => '/', like GitHub pages
- fix: redirect with no querystring
- ci: support ACME pebble integration test
- deps: update server deps

### Version 2.2.2

- [x] config: (break change) `http_redirect_to_https` convert from bool to u32.
- [x] chore: change api domain response type.
- [x] fix: fix cors check for normal http request.
- [x] fix: update version of one SPA one domain would cause deadlock.
- [x] feat: support index.html alias "","/" for multiple SPA one domain.
- [x] improve: JS SDK keep same logic with command line, and improve error information. release JS SDK 2.2.2
- [x] improve: more integration tests.

### Version 2.2.1

- [x] fix: command client upload interrupt, get error file key.
- [x] fix: admin server report os:9 error when upload existed file.
- [x] ci: add ci for spa-server, and GitHub action could run it
- [x] fix: command client info incorrect handle domain query string
- [x] fix: fix .SPA-Multiple would write sub dir more than once

### Version 2.2.0

- [x] fix: admin can handle http/https init/hot reload error.
- [x] fix: http redirect https, now only support 443 port.
- [x] config: (break change) `port, addr` => `http.port, http.addr`
- [x] support https with let's encrypt
- [x] cicd: fix docker run failure, and add spa-server docker test image cd.
- [x] improve: more integration tests.

### Version 2.1.1

- [x] fix: cache discard correctly
- [x] feat: server could start should serve old version
- [x] improve: .SPA-Multiple file create when upload

### Version 2.1.0

- [x] fix: upload file api is not correct.
- [x] fix: command line compile successfully.
- [x] feat: support serving multiple spa in one domain.
- [x] chore: js client sdk add node.js version restriction.
- [x] chore: it's now used in production environment!

### Version 2.0.0(client:2.0.0)

- [x] move project from timzaak to ForNet
- [x] upgrade warp and other dependencies
- [x] upgrade vitePress to 1.0.1
- [x] change js sdk implementation from Rust to typescript
- [x] doc: rewrite to support new doc
- [x] cicd: remove client docker, server s3 docker.

### Version v1.2.6(client:v0.1.4)

- [x] chore(break change): change server docker config and binary location
- [x] feat: add cron job to delete deprecated version
- [x] feat(with client): delete deprecated version to save storage
- [x] feat: Support S3 by docker(backward: release docker image: timzaak/spa-server:1.2.5-s3)
- [x] deps: bump hocon 0.9, fix size unit config parse
- [x] fix(build): disable generating new tag when build spa-client(js) success
- [x] doc: add algolia search, thanks for algolia company!

### Version 1.2.5(client v0.1.3)

- [x] build: add docker image cache for (spa-client|spa-server)-docker-cd.yml to speed cd process
- [x] doc: use VitePress to rebuild docs, ready to get the world known it
- [x] build: add CD for doc release
- [x] feat: support multiple config for different domain (break change for config file)
- [x] feat: support multiple ssl
- [ ] ~~fix: disable put online domain which does not have correct ssl in server when https opened.~~(need to confirm if
  it's a bug?)
- [x] fix: fix wrong check when release new domain
- [x] fix(js-client): npm package error

### Version 1.2.4(client:v0.1.1)

- [x] release commandline of spa-client for mac/ios/linux (by GitHub Actions), put them with GitHub release page
- [x] fix possible bugs about uploading and spa-client(-js)
- [x] build: release docker image by GitHubActions
- [x] build: add docker image for spa-client
- [x] doc: how to use spa-client image
- [x] improve: add debug log for spa-server request

### version 1.2.3(client:v0.1.0)

- [x] admin server export http api to accept files to local file system
- [x] add client to sync local files to admin server（retry support）
- [ ] ~~release server/client to
  crate~~ [crate needs dep version, need replace warp firstly](https://github.com/rust-lang/cargo/issues/1565)
- [x] doc about how to use with shell client

#### add js plugin

- [x] add js wrapper for spa-client
- [x] and example/test frontend repo
- [x] doc about how to use with js client
- [x] release js wrapper to npm.org

### version 1.2.2

- [x] cache File `Range` Header support
- [ ] ~~drop self maintained `Warp`(copy out needed code from Warp)~~ (so much code from warp/fs, I give up after try,
  will wait Warp release proper version)
- [x] `HEAD` request support or drop(support, don't need to do anything)

### version 1.2.1

- [x] more log for debug and trace
- [x] basic CORS
- [x] compress regression support(~~if client don't send accept-encoding header(including gzip), will send back data
  from file instead of cache~~ improved by v1.2.3)
- [x] hot reload web static server(use SO_REUSEPORT *nix api, so it may be wrong with Windows).
- [ ] ~~different config(cors/cache strategy/https and so on) for different domain.~~ (if this is needed?)

### version 1.1.x

- [x] more doc(how to update static files)
- [x] rewrite Dockerfile to reduce docker image size
- [x] cache improve(big file ignore config option and if-range header support)
- [x] header`cache-control` for client cache
- [ ] ~~header `etag` for client cache~~ [warp #462](https://github.com/seanmonstar/warp/issues/462)
- [x] 80 redirect to 443 config option
- [x] compression for js/icon/json/css/html (only support gzip algo, only compress cached files, and ~~will occur error
  when client don't support gzip~~(fix @ v1.2))

### version 1.0.x

- [x] 80 and 443 both support
- [ ] ~~compression~~ done @ v1.2.0.
- [ ] ~~multiple tls support~~ the feature may do not need.
- [x] cache file(cache all files in memory without LRU)

### before release

- [x] very simple http1 spa server
- [x] very simple admin server(http api)
- [x] ssl(including wildcard domain ssl)
- [x] docker release
- [x] simple usage doc
